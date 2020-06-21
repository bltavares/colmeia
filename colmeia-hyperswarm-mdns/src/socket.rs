use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::Duration;

const IPV4_MULTICAST: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
lazy_static::lazy_static! {
  pub static ref MULTICAST_DESTINATION: SocketAddr = SocketAddr::new(IPV4_MULTICAST.into(), 5353);
  static ref DAT_MULTICAST: SocketAddr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 5353);
}

/// On Windows, unlike all Unix variants, it is improper to bind to the multicast address
///
/// see https://msdn.microsoft.com/en-us/library/windows/desktop/ms737550(v=vs.85).aspx
#[cfg(windows)]
fn bind_multicast(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
    let addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), addr.port());
    socket.bind(&socket2::SockAddr::from(addr))
}

/// On unixes we bind to the multicast address, which causes multicast packets to be filtered
#[cfg(unix)]
fn bind_multicast(socket: &Socket, addr: &SocketAddr) -> io::Result<()> {
    socket.bind(&socket2::SockAddr::from(*addr))
}

/// Only Unix has this setup
#[cfg(unix)]
fn reuse_port(socket: &Socket) -> io::Result<()> {
    socket.set_reuse_port(true)
}

#[cfg(windows)]
fn reuse_port(_socket: &Socket) -> io::Result<()> {
    Ok(())
}

pub fn create_shared() -> Result<UdpSocket, io::Error> {
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    socket.set_reuse_address(true)?;
    reuse_port(&socket)?;
    socket.join_multicast_v4(&IPV4_MULTICAST, &Ipv4Addr::UNSPECIFIED)?;
    bind_multicast(&socket, &DAT_MULTICAST)?;
    Ok(socket.into_udp_socket())
}
