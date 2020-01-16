use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::time::Duration;

use async_std::net::UdpSocket as AsyncUdpSocket;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use trust_dns_proto::op::Message;

// const PORT: u16 = 7645;
const IPV4_MULTICAST: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
const MULTICAST_DESTINATION: SocketAddr = SocketAddr::new(IPV4_MULTICAST.into(), 5353);
const DAT_MULTICAST: SocketAddr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5353);

/// On Windows, unlike all Unix variants, it is improper to bind to the multicast address
///
/// see https://msdn.microsoft.com/en-us/library/windows/desktop/ms737550(v=vs.85).aspx
#[cfg(windows)]
fn bind_multicast(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
  let addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), addr.port());
  socket.bind(&socket2::SockAddr::from(addr))
}

/// On unixes we bind to the multicast address, which causes multicast packets to be filtered
#[cfg(unix)]
fn bind_multicast(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
  socket.bind(&socket2::SockAddr::from(*addr))
}

fn create_socket() -> Result<UdpSocket, io::Error> {
  let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
  socket.set_read_timeout(Some(Duration::from_millis(100)))?;
  socket.set_reuse_address(true)?;
  socket.set_reuse_port(true)?;
  socket.join_multicast_v4(&IPV4_MULTICAST, &Ipv4Addr::UNSPECIFIED)?;
  bind_multicast(&socket, &DAT_MULTICAST)?;
  Ok(socket.into_udp_socket())
}

async fn go(socket: UdpSocket, packet: &[u8]) -> Result<(), io::Error> {
  let async_socket = async_std::net::UdpSocket::from(socket);
  async_socket.send_to(&packet, MULTICAST_DESTINATION).await?;
  use trust_dns_proto::op::Message;

  let mut buff = [0; 512];
  loop {
    if let Ok((_bytes, peer)) = async_socket.recv_from(&mut buff).await {
      dbg!(peer);
      dbg!(_bytes);
      if let Ok(message) = Message::from_vec(&buff[.._bytes]) {
        dbg!(message);
      }
    }
  }
}

async fn go_stream(
  socket: UdpSocket,
  packet: &[u8],
) -> Result<impl futures::Stream<Item = Message>, io::Error> {
  let async_socket = async_std::net::UdpSocket::from(socket);
  async_socket.send_to(&packet, MULTICAST_DESTINATION).await?;

  Ok(futures::stream::unfold(async_socket, |socket| {
    async {
      let mut buff = [0; 512];

      loop {
        if let Ok((_bytes, peer)) = socket.recv_from(&mut buff).await {
          dbg!(peer);
          dbg!(_bytes);
          if let Ok(message) = Message::from_vec(&buff[.._bytes]) {
            return Some((message, socket));
          }
        }
      }
    }
  }))
}

struct Locator {
  socket: AsyncUdpSocket,
  packet: Vec<u8>,
}

use core::pin::Pin;
use core::task::{Context, Poll};
impl futures::Stream for Locator {
  type Item = std::io::Result<Message>;

  // Trying to copy from the impl from async_std::net::Incoming
  // Can't run poll
  fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    let query_future = self.socket.send_to(&self.packet, MULTICAST_DESTINATION);
    pin_utils::pin_mut!(query_future);
    // no method named `poll` found for type `std::pin::Pin<&mut impl core::future::future::Future>` in the current scope
    // method not found in `std::pin::Pin<&mut impl core::future::future::Future>`
    // let query_response = futures::ready!(query_future.poll(cx))?;

    Poll::Pending
  }
}

fn name() -> String {
  let args: Vec<String> = std::env::args().skip(1).collect();
  let dat_name = args.first().expect("must have dat name");
  colmeia::dat_url_mdns_discovery_name(&dat_name).expect("invalid dat url")
}

fn packet(dat_url: &str) -> Vec<u8> {
  use std::str::FromStr;
  use trust_dns_client::serialize::binary::{BinEncodable, BinEncoder};
  use trust_dns_proto::op::{Message, MessageType, Query};
  use trust_dns_proto::rr::{DNSClass, Name, RecordType};

  let name = Name::from_str(dat_url).expect("invalid DNS name");
  let mut query = Query::query(name, RecordType::TXT);
  query.set_query_class(DNSClass::IN);
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

fn main() {
  let socket = create_socket().expect("socket creation failed");
  let name = name();
  let packet = packet(&name);
  // This works fine, but I can't make it return the peer informations as a stream
  // Implementation needs to pass a callback?
  // async_std::task::block_on(go(socket, &packet)).expect("error spawning");

  // This does not work...
  // Something about Unpin
  // help: the following implementations were found:
  // <std::future::GenFuture<T> as std::marker::Unpin>
  // use async_std::stream::StreamExt;
  // async_std::task::block_on(async {
  //   let bound_socket = go_stream(socket, &packet)
  //     .await
  //     .expect("Could not bind socket");
  //   while let Some(message) = bound_socket.next().await {
  //     dbg!("message received");
  //   }
  // });
}
