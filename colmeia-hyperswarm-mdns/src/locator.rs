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

pub fn packet(hyperswarm_domain: &str) -> Vec<u8> {
    let name = Name::from_str(hyperswarm_domain).expect("could not write name");
    let query = Query::query(name, RecordType::SRV);
    let mut message = Message::new();
    message
        .set_id(0)
        .set_message_type(MessageType::Query)
        .set_authoritative(false)
        .add_query(query);

    let mut buffer = Vec::with_capacity(75);
    let mut encoder = BinEncoder::new(&mut buffer);
    message.emit(&mut encoder).expect("could not encode");
    buffer
}

async fn broadcast(mdns_url: String, socket: &AsyncUdpSocket) {
    let mdns_packet_bytes = packet(&mdns_url);

    socket
        .send_to(&mdns_packet_bytes, *crate::socket::MULTICAST_DESTINATION)
        .await
        .expect("Could not send bytes");
}

// Only works for ipv4 mdns
async fn wait_response(socket: &AsyncUdpSocket) -> Result<(Vec<u8>, Ipv4Addr), std::io::Error> {
    let mut buffer = [0; 512];
    let (read_size, origin_ip) = socket.recv_from(&mut buffer).await?;

    if let SocketAddr::V4(origin_ip) = origin_ip {
        let debug_vec: Vec<_> = buffer.iter().take(read_size).cloned().collect();
        return Ok((debug_vec, origin_ip.ip().clone()));
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
    pub fn new(hash: &[u8], announcement_window: Duration) -> Self {
        let mdns_url = crate::hash_to_domain(hash);
        println!("Parse result: {:?}", mdns_url);

        // Criar socket e enviar para Destination: 224.0.0.251
        let socket = crate::socket::create_shared().expect("Could not create socket");
        println!("Socket: {:?}", socket);

        let socket = Arc::from(AsyncUdpSocket::from(socket));

        let broadcast_stream = stream::unfold(
            (mdns_url.clone(), socket.clone()),
            |(packet, socket)| async move {
                broadcast(packet.clone(), &socket).await;
                Some(((), (packet, socket)))
            },
        )
        .throttle(announcement_window);

        let response_stream = stream::unfold(socket, |socket| async move {
            let response = wait_response(&socket).await;
            Some((response, socket))
        })
        .filter_map(|item| item.ok())
        .filter_map(move |(packet, origin_ip)| {
            select_ip_from_hyperswarm_mdns_response(packet, origin_ip, &mdns_url)
        });

        Locator {
            broadcast_stream: Box::pin(broadcast_stream),
            response_stream: Box::pin(response_stream),
        }
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
    hyperswarm_domain: &str,
) -> Option<SocketAddr> {
    // TODO: use a single Name for packet generation and packet comparison
    let name = Name::from_str(hyperswarm_domain).expect("could not write name");

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

    let srv_matches = answers.iter().find(|record| record.name() == &name)?;
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
