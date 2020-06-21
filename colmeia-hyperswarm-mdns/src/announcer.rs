use anyhow::Context;
use async_std::net::UdpSocket as AsyncUdpSocket;
use async_std::stream::StreamExt;
use async_std::task;
use futures::stream;
use futures::Stream;
use std::net::{Ipv4Addr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use trust_dns_proto::op::{Message, MessageType};
use trust_dns_proto::rr::{
    rdata::{SRV, TXT},
    Name, RData, Record, RecordType,
};
use trust_dns_proto::serialize::binary::{BinEncodable, BinEncoder};

pub fn packet(
    hyperswarm_domain: Name,
    port: u16,
    self_identifier: String,
) -> anyhow::Result<Vec<u8>> {
    let mut srv_query = Record::with(hyperswarm_domain.clone(), RecordType::SRV, 0);
    srv_query.set_rdata(RData::SRV(SRV::new(
        0,
        0,
        port,
        crate::UNSPECIFIED_NAME.clone(),
    )));

    let mut txt_query = Record::with(hyperswarm_domain, RecordType::TXT, 0);
    txt_query.set_rdata(RData::TXT(TXT::new(vec![self_identifier])));

    let mut a_query = Record::with(crate::HYPERSWARM_REFERER.clone(), RecordType::A, 0);
    a_query.set_rdata(RData::A(Ipv4Addr::UNSPECIFIED));

    let mut message = Message::new();
    message
        .set_id(0)
        .set_message_type(MessageType::Response)
        .set_authoritative(false)
        .add_answer(srv_query)
        .add_answer(txt_query)
        .add_answer(a_query);

    let mut buffer = Vec::with_capacity(75);
    let mut encoder = BinEncoder::new(&mut buffer);
    message.emit(&mut encoder).context("could not encode")?;
    Ok(buffer)
}

async fn respond(
    hyperswarm_domain: Name,
    socket: &AsyncUdpSocket,
    port: u16,
    self_identifier: String,
) -> anyhow::Result<()> {
    let mdns_packet_bytes = packet(hyperswarm_domain, port, self_identifier)?;

    socket
        .send_to(&mdns_packet_bytes, *crate::socket::MULTICAST_DESTINATION)
        .await
        .context("Could not send bytes")?;

    Ok(())
}

async fn wait_broadcast(socket: &AsyncUdpSocket) -> anyhow::Result<(Vec<u8>, Ipv4Addr)> {
    let mut buffer = [0; 512];
    let (read_size, origin_ip) = socket.recv_from(&mut buffer).await?;

    if let SocketAddr::V4(origin_ip) = origin_ip {
        let packet_data: Vec<_> = buffer.iter().take(read_size).cloned().collect();
        return Ok((packet_data, *origin_ip.ip()));
    }

    Err(anyhow::anyhow!(
        "packet response is not ipv4, {:?}",
        origin_ip
    ))
}

fn is_same_hash_questions(packet: Vec<u8>, hyperswarm_domain: &Name) -> Option<Message> {
    let dns_message = Message::from_vec(&packet).ok()?;
    if dns_message.query_count() != 1 {
        return None;
    }

    let query = dns_message.queries().first()?;
    if query.query_type() != RecordType::SRV {
        return None;
    }

    if query.name() != hyperswarm_domain {
        return None;
    }

    Some(dns_message)
}

pub struct Announcer {
    stream: Pin<Box<dyn Stream<Item = Ipv4Addr> + Send>>,
}

impl Announcer {
    pub fn new(socket: std::net::UdpSocket, hash: &[u8], port: u16) -> anyhow::Result<Self> {
        Announcer::with_identifier(socket, hash, port, crate::self_id())
    }

    pub fn with_identifier(
        socket: std::net::UdpSocket,
        hash: &[u8],
        port: u16,
        self_identifier: String,
    ) -> anyhow::Result<Self> {
        let hyperswarm_domain = crate::hash_as_domain_name(hash)?;
        let socket = Arc::from(AsyncUdpSocket::from(socket));
        let response_stream = stream::unfold(
            (
                socket.clone(),
                hyperswarm_domain.clone(),
                port,
                self_identifier,
            ),
            |(socket, hyperswarm_domain, port, self_identifier)| async move {
                let result = respond(
                    hyperswarm_domain.clone(),
                    &socket,
                    port,
                    self_identifier.clone(),
                )
                .await;

                if let Err(e) = result {
                    log::warn!("Could not respond to query {:?}", e);
                }

                Some(((), (socket, hyperswarm_domain, port, self_identifier)))
            },
        );

        let listen_stream = stream::unfold(socket, |socket| async {
            let response = wait_broadcast(&socket).await;
            Some((response, socket))
        })
        .filter_map(|item| item.ok()) // TODO filter if item is a good packet
        .filter_map(move |(packet, origin_ip)| {
            is_same_hash_questions(packet, &hyperswarm_domain).map(|_| origin_ip)
        })
        .zip(response_stream)
        .map(|(origin_ip, _)| origin_ip);

        Ok(Announcer {
            stream: Box::pin(listen_stream),
        })
    }
}

impl futures::Stream for Announcer {
    type Item = Ipv4Addr;
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;
        self.stream.poll_next_unpin(cx)
    }
}
