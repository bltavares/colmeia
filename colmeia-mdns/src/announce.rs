use async_std::net::UdpSocket as AsyncUdpSocket;
use futures::stream::{self, StreamExt};
use std::net::{SocketAddr, UdpSocket};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use trust_dns_proto::op::MessageType;

use crate::crypto;
use crate::socket;

fn dat_origin_request(
  dat_url: &crypto::DatLocalDiscoverUrl,
  item: Option<socket::MessageStream>,
) -> Option<SocketAddr> {
  let item = item?;
  let query = item.0.queries().first()?;
  if item.0.message_type() == MessageType::Query
    && query.name().to_lowercase().to_ascii() == dat_url.0
  {
    let origin = item.1;
    log::debug!("MDNS query originated {:#}", origin);
    return Some(origin);
  }
  None
}

fn packet(dat_url: &str) -> Vec<u8> {
  use std::str::FromStr;
  use trust_dns_proto::op::{Message, Query};
  use trust_dns_proto::rr::rdata::TXT;
  use trust_dns_proto::rr::{Name, RData, Record, RecordType};
  use trust_dns_proto::serialize::binary::{BinEncodable, BinEncoder};

  let name = Name::from_str(dat_url).expect("invalid DNS name");
  let query = Query::query(name, RecordType::TXT);

  let name = Name::from_str(dat_url).expect("invalid DNS name");
  let txt = TXT::new(vec![
    "token=asfdasfasf".into(), // TODO: Pass random value
    "peers=AAAAAAzS".into(),   // TODO: Pass port
  ]);
  let record = Record::from_rdata(name, 0, RData::TXT(txt));
  let mut message = Message::new();
  message
    .set_id(0)
    .set_message_type(MessageType::Response)
    .set_authoritative(true)
    .add_query(query)
    .add_answer(record);
  let mut buffer = Vec::with_capacity(196); // Size of the packet
  let mut encoder = BinEncoder::new(&mut buffer);
  message.emit(&mut encoder).expect("could not encode");
  buffer
}

pub struct Announcer {
  listen_stream: Pin<Box<dyn futures::Stream<Item = ()> + Send>>,
}

impl Announcer {
  pub fn new(socket: UdpSocket, dat_url: crypto::DatLocalDiscoverUrl) -> Self {
    Announcer::shared_socket(Arc::new(socket.into()), dat_url)
  }

  fn shared_socket(socket: Arc<AsyncUdpSocket>, dat_url: crypto::DatLocalDiscoverUrl) -> Self {
    let packet = packet(&dat_url.0);

    let response_stream = stream::unfold((socket.clone(), packet), |(socket, packet)| {
      async {
        let bytes_writen = socket
          .send_to(&packet, *socket::MULTICAST_DESTINATION)
          .await
          .ok()?;
        log::debug!("MDNS response sent: {:?} bytes", bytes_writen);
        Some((bytes_writen, (socket, packet)))
      }
    });

    let listen_stream = stream::unfold(socket, socket::read_dns_message)
      .filter_map(move |message| futures::future::ready(dat_origin_request(&dat_url, message)))
      .zip(response_stream)
      .map(|(_, _)| ());

    Announcer {
      listen_stream: Box::pin(listen_stream),
    }
  }
}

impl futures::Stream for Announcer {
  type Item = ();

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
    self.listen_stream.poll_next_unpin(cx)
  }
}
