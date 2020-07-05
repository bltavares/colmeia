use anyhow::Context;
use async_std::{stream::StreamExt, task};
use futures::{stream, stream::StreamExt as FStreamExt, FutureExt, Stream};
use trust_dns_proto::op::{Message, MessageType};
use trust_dns_proto::rr::{
    rdata::{SRV, TXT},
    Name, RData, Record, RecordType,
};
use trust_dns_proto::serialize::binary::{BinEncodable, BinEncoder};

use std::io;
use std::sync::Arc;
use std::{net::Ipv4Addr, pin::Pin};

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
    interface: multicast_socket::Interface,
    socket: Arc<multicast_socket::MulticastSocket>,
    port: u16,
    self_identifier: String,
) -> anyhow::Result<()> {
    let mdns_packet_bytes = packet(hyperswarm_domain, port, self_identifier)?;

    task::spawn(async move { socket.send(&mdns_packet_bytes, &interface) })
        .await
        .context("Could not send bytes")?;

    Ok(())
}

async fn wait_broadcast(
    socket: Arc<multicast_socket::MulticastSocket>,
) -> io::Result<multicast_socket::Message> {
    task::spawn(async move { socket.receive() }).await
}

fn is_same_hash_questions(packet: &[u8], hyperswarm_domain: &Name) -> Option<Message> {
    let dns_message = Message::from_vec(packet).ok()?;
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

type ListenContext = (
    multicast_socket::Message,
    Arc<multicast_socket::MulticastSocket>,
);

pub struct Announcer {
    stream: Pin<Box<dyn Stream<Item = Ipv4Addr> + Send>>,
}

impl Announcer {
    pub fn new(
        socket: multicast_socket::MulticastSocket,
        hash: &[u8],
        port: u16,
    ) -> anyhow::Result<Self> {
        Announcer::with_identifier(socket, hash, port, crate::self_id())
    }

    pub fn with_identifier(
        socket: multicast_socket::MulticastSocket,
        hash: &[u8],
        port: u16,
        self_identifier: String,
    ) -> anyhow::Result<Self> {
        let hyperswarm_domain = Arc::from(crate::hash_as_domain_name(hash)?);
        let socket = Arc::from(socket);

        let listener = stream::unfold(socket, |socket| async {
            let response = wait_broadcast(socket.clone()).await;
            Some((response.map(|msg| (msg, socket.clone())), socket))
        });

        let listen_stream = StreamExt::filter_map(listener, |i| i.ok());

        let select_packets_fn = {
            let name = hyperswarm_domain.clone();
            move |(message, _): &ListenContext| {
                is_same_hash_questions(&message.data, &name).is_some()
            }
        };
        let listen_stream = StreamExt::filter(listen_stream, select_packets_fn);

        let response_fn = {
            let name = hyperswarm_domain.clone();
            move |(message, socket): ListenContext| {
                let result = respond(
                    (*name).clone(),
                    message.interface,
                    socket,
                    port,
                    self_identifier.clone(),
                );
                let origin_address = message.origin_address;
                result.map(move |i| (i, origin_address))
            }
        };
        let listen_stream = StreamExt::map(listen_stream, response_fn);
        let listen_stream = FStreamExt::then(listen_stream, |result| async {
            let (result, origin_address) = result.await;
            if let Err(e) = result {
                log::warn!("Could not send response back {}", e);
            }
            *origin_address.ip()
        });

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
