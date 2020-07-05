use multicast_socket::MulticastSocket;
use std::io;
use std::net::{Ipv4Addr, SocketAddrV4};

const MDNS_IP: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
lazy_static::lazy_static! {
  pub static ref MDNS_ADDRESS: SocketAddrV4 = SocketAddrV4::new(MDNS_IP, 5353);
}

pub(crate) fn create() -> io::Result<MulticastSocket> {
    MulticastSocket::all_interfaces(*MDNS_ADDRESS)
}
