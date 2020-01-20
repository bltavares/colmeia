use async_std::net::UdpSocket as AsyncUdpSocket;
use futures::stream;
use std::net::{SocketAddr, UdpSocket};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use trust_dns_proto::op::Message;
use trust_dns_proto::rr::RData::TXT;

use crate::crypto;
use crate::socket;

fn packet(dat_url: &str) -> Vec<u8> {
  use std::str::FromStr;
  use trust_dns_proto::op::{MessageType, Query};
  use trust_dns_proto::rr::{Name, RecordType};
  use trust_dns_proto::serialize::binary::{BinEncodable, BinEncoder};

  let name = Name::from_str(dat_url).expect("invalid DNS name");
  let query = Query::query(name, RecordType::TXT);
  let mut message = Message::new();
  message
    .set_id(0)
    .set_message_type(MessageType::Query)
    .set_authoritative(false)
    .add_query(query);
  let mut buffer = Vec::with_capacity(68); // Size of the packet
  let mut encoder = BinEncoder::new(&mut buffer);
  message.emit(&mut encoder).expect("could not encode");
  buffer
}

fn select_location_response(
  dat_url: &crypto::DatLocalDiscoverUrl,
  response: Option<socket::MessageStream>,
) -> Option<SocketAddr> {
  let response = response?;
  let answer = response.0.answers().first()?;
  if answer.name().to_lowercase().to_ascii() != dat_url.0 {
    return None;
  }
  if let TXT(rdata) = answer.rdata() {
    let peer_record = rdata
      .txt_data()
      .iter()
      .flat_map(|payload| std::str::from_utf8(payload))
      .find(|x| x.starts_with("peers="))?;
    let peer_info = peer_record.split_terminator('=').nth(1)?;
    let peer_info = base64::decode(peer_info).ok()?;
    let port = u16::from_be_bytes([*peer_info.get(4)?, *peer_info.get(5)?]);
    let origin = response.1;

    return Some(SocketAddr::new(origin.ip(), port));
  }
  None
}

pub struct Locator {
  query_stream: Pin<Box<dyn futures::Stream<Item = std::io::Result<usize>> + Send>>,
  listen_stream: Pin<Box<dyn futures::Stream<Item = SocketAddr> + Send>>,
}

#[must_use = "streams do nothing unless polled"]
impl Locator {
  pub fn new(socket: UdpSocket, dat_url: crypto::DatLocalDiscoverUrl, duration: Duration) -> Self {
    Self::shared_socket(Arc::new(socket.into()), dat_url, duration)
  }

  fn shared_socket(
    socket: Arc<AsyncUdpSocket>,
    dat_url: crypto::DatLocalDiscoverUrl,
    duration: Duration,
  ) -> Self {
    use async_std::stream::StreamExt as AsyncStreamExt;

    let socket_handle = socket.clone();
    let packet = packet(&dat_url.0);

    let query_stream = stream::unfold((socket_handle, packet), |(socket, packet)| {
      async {
        let bytes_writen = socket
          .send_to(&packet, *socket::MULTICAST_DESTINATION)
          .await;
        log::debug!("MDNS query sent");
        Some((bytes_writen, (socket, packet)))
      }
    })
    .throttle(duration);

    let listen_stream = stream::unfold(socket, socket::read_dns_message)
      .filter_map(move |message| select_location_response(&dat_url, message));

    Self {
      query_stream: Box::pin(query_stream),
      listen_stream: Box::pin(listen_stream),
    }
  }
}

impl futures::Stream for Locator {
  type Item = SocketAddr;

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
    use futures::stream::StreamExt;

    drop(self.query_stream.poll_next_unpin(cx));
    self.listen_stream.poll_next_unpin(cx)
  }
}
