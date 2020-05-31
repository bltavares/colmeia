use async_std::net::UdpSocket as AsyncUdpSocket;
use async_std::stream::StreamExt;
use async_std::task;
use futures::stream;
use futures::Stream;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use trust_dns_proto::op::{Message, MessageType, Query};
use trust_dns_proto::rr::{Name, RecordType};
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

async fn broadcast(mdns_url: String, socket: Arc<AsyncUdpSocket>) {
    let mdns_packet_bytes = packet(&mdns_url);

    socket
        .send_to(&mdns_packet_bytes, *crate::socket::MULTICAST_DESTINATION)
        .await
        .expect("Could not send bytes");
}

async fn wait_response(socket: Arc<AsyncUdpSocket>) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = [0; 512];
    let read_size = socket.recv(&mut buffer).await?;
    let debug_vec: Vec<_> = buffer.iter().take(read_size).cloned().collect();
    Ok(debug_vec)
}

pub struct Locator {
    broadcast_stream: Pin<Box<dyn Stream<Item = ()>>>,
    response_stream: Pin<Box<dyn Stream<Item = Vec<u8>>>>,
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
                broadcast(packet.clone(), socket.clone()).await;
                Some(((), (packet, socket)))
            },
        )
        .throttle(announcement_window);

        let response_stream = stream::unfold(socket, |socket| async move {
            let response = wait_response(socket.clone()).await;
            Some((response, socket))
        })
        .filter_map(|item| item.ok());

        Locator {
            broadcast_stream: Box::pin(broadcast_stream),
            response_stream: Box::pin(response_stream),
        }
    }
}

impl futures::Stream for Locator {
    type Item = Vec<u8>;
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;

        drop(self.broadcast_stream.poll_next_unpin(cx));
        self.response_stream.poll_next_unpin(cx)
    }
}
