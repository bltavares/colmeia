use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::time::Duration;

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

// const PORT: u16 = 7645;
const IPV4_MULTICAST: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);

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
  // TODO: const
  let dat_multicast = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5353);

  let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
  socket.set_read_timeout(Some(Duration::from_millis(100)))?;
  socket.set_reuse_address(true)?;
  socket.set_reuse_port(true)?;
  socket.join_multicast_v4(&IPV4_MULTICAST, &Ipv4Addr::UNSPECIFIED)?;
  bind_multicast(&socket, &dat_multicast)?;

  Ok(socket.into_udp_socket())
}

async fn go(socket: &UdpSocket, packet: &[u8]) -> Result<(), io::Error> {
  socket.send_to(&packet, SocketAddr::new(IPV4_MULTICAST.into(), 5353))?;
  Ok(())
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
    // .set_query_count(1)
    // .set_answer_count(0)
    .set_authoritative(false)
    // .set_additional_count(0);
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
  async_std::task::block_on(go(&socket, &packet)).expect("error spawning");
}
