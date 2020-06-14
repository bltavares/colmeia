use anyhow::Context;
use async_std::net::UdpSocket as AsyncUdpSocket;
use async_std::stream::StreamExt;
use async_std::task;
use futures::{stream, Stream};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use trust_dns_proto::op::{Message, MessageType, Query};
use trust_dns_proto::rr::{Name, RData, Record, RecordType};
use trust_dns_proto::serialize::binary::{BinEncodable, BinEncoder};

pub fn packet(hyperswarm_domain: Name) -> anyhow::Result<Vec<u8>> {
    let query = Query::query(hyperswarm_domain, RecordType::SRV);
    let mut message = Message::new();
    message
        .set_id(0)
        .set_message_type(MessageType::Query)
        .set_authoritative(false)
        .add_query(query);

    let mut buffer = Vec::with_capacity(75);
    let mut encoder = BinEncoder::new(&mut buffer);
    message
        .emit(&mut encoder)
        .context("malformed mdns packet")?;
    Ok(buffer)
}

async fn broadcast(hyperswarm_domain: Name, socket: &AsyncUdpSocket) -> anyhow::Result<()> {
    let mdns_packet_bytes = packet(hyperswarm_domain)?;

    socket
        .send_to(&mdns_packet_bytes, *crate::socket::MULTICAST_DESTINATION)
        .await
        .context("could not send packet to multicast address")?;

    Ok(())
}

// Only works for ipv4 mdns
async fn wait_response(socket: &AsyncUdpSocket) -> Result<(Vec<u8>, Ipv4Addr), std::io::Error> {
    let mut buffer = [0; 512];
    let (read_size, origin_ip) = socket.recv_from(&mut buffer).await?;

    if let SocketAddr::V4(origin_ip) = origin_ip {
        let packet_data: Vec<_> = buffer.iter().take(read_size).cloned().collect();
        return Ok((packet_data, origin_ip.ip().clone()));
    }
    // TODO make this a generic error
    // rataria: using a random error code
    Err(std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "not ipv4").into())
}

pub struct Locator {
    broadcast_stream: Pin<Box<dyn Stream<Item = ()>>>,
    response_stream: Pin<Box<dyn Stream<Item = SocketAddr>>>,
}

impl Locator {
    pub fn new(hash: &[u8], announcement_window: Duration) -> anyhow::Result<Self> {
        let hyperswarm_domain = crate::hash_to_domain(hash);
        log::debug!("hyperswarm domain: {:?}", hyperswarm_domain);

        let hyperswarm_domain = Name::from_str(&hyperswarm_domain)
            .context("could not create hyperswarm dns name from provided hash")?;

        let socket = crate::socket::create_shared().context("could not allocate a new socket")?;
        log::debug!("allocated socket {:?}", socket);

        let socket = Arc::from(AsyncUdpSocket::from(socket));

        let broadcast_stream = stream::unfold(
            (hyperswarm_domain.clone(), socket.clone()),
            |(hyperswarm_domain, socket)| async move {
                let broadcast_result = broadcast(hyperswarm_domain.clone(), &socket).await;
                if let Err(problem) = broadcast_result {
                    log::warn!(
                        "failed to broadcast a packet. trying again later. {:?}",
                        problem
                    );
                }
                Some(((), (hyperswarm_domain, socket)))
            },
        )
        .throttle(announcement_window);

        let response_stream = stream::unfold(socket, |socket| async move {
            let response = wait_response(&socket).await;
            Some((response, socket))
        })
        .filter_map(|item| item.ok())
        .filter_map(move |(packet, origin_ip)| {
            select_ip_from_hyperswarm_mdns_response(packet, origin_ip, &hyperswarm_domain)
        });

        Ok(Locator {
            broadcast_stream: Box::pin(broadcast_stream),
            response_stream: Box::pin(response_stream),
        })
    }
}

lazy_static::lazy_static! {
    // TODO Generate one SELF_ID for each program execution
    static ref SELF_ID: [Box<[u8]>; 1] = [b"id=banana".to_vec().into_boxed_slice()];
    static ref UNSPECIFIED_NAME: Name = Name::from_str("0.0.0.0").unwrap();
}

fn select_ip_from_hyperswarm_mdns_response(
    packet: Vec<u8>,
    origin_ip: Ipv4Addr,
    hyperswarm_domain: &Name,
) -> Option<SocketAddr> {
    let dns_message = Message::from_vec(&packet).ok()?;
    if dns_message.answer_count() != 3 {
        return None;
    }

    let answers = dns_message.answers();
    // TODO be more explicit on this logic and early return
    let _id_different = answers.iter().find(|record: &&Record| {
        if let RData::TXT(txt_data) = record.rdata() {
            return txt_data.txt_data() != &*SELF_ID;
        }
        false
    })?;

    let srv_matches = answers
        .iter()
        .find(|record| record.name() == hyperswarm_domain)?;
    if let RData::SRV(srv_data) = srv_matches.rdata() {
        let port = srv_data.port();
        if srv_data.target() == &*UNSPECIFIED_NAME {
            return Some(SocketAddr::new(IpAddr::V4(origin_ip), port));
        } else {
            let target_ipv4 = srv_data.target().to_utf8().parse::<Ipv4Addr>().ok()?;
            return Some(SocketAddr::new(IpAddr::V4(target_ipv4), port));
        }
    }
    None
}

impl futures::Stream for Locator {
    type Item = SocketAddr;
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;

        drop(self.broadcast_stream.poll_next_unpin(cx));
        self.response_stream.poll_next_unpin(cx)
    }
}