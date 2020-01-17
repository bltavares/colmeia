use async_std::net::UdpSocket as AsyncUdpSocket;
use futures::stream::{self, StreamExt};
use futures::FutureExt;
use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use trust_dns_proto::op::Message;

pub mod crypto;

const IPV4_MULTICAST: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
lazy_static::lazy_static! {
  static ref MULTICAST_DESTINATION: SocketAddr = SocketAddr::new(IpAddr::V4(IPV4_MULTICAST), 5353);
  static ref DAT_MULTICAST: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 5353);
}

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

pub fn create_shared_socket() -> Result<UdpSocket, io::Error> {
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.join_multicast_v4(&IPV4_MULTICAST, &Ipv4Addr::UNSPECIFIED)?;
    bind_multicast(&socket, &DAT_MULTICAST)?;
    Ok(socket.into_udp_socket())
}

fn packet(dat_url: &str) -> Vec<u8> {
    use std::str::FromStr;
    use trust_dns_proto::op::{MessageType, Query};
    use trust_dns_proto::rr::{DNSClass, Name, RecordType};
    use trust_dns_proto::serialize::binary::{BinEncodable, BinEncoder};

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

pub struct Locator {
    socket: Arc<AsyncUdpSocket>,
    query_stream: Pin<Box<dyn futures::Stream<Item = std::io::Result<usize>>>>,
}

#[must_use = "streams do nothing unless polled"]
impl Locator {
    pub fn new(
        socket: UdpSocket,
        dat_url: crypto::DatLocalDiscoverUrl,
        duration: Duration,
    ) -> Self {
        Self::shared_socket(socket.into(), dat_url, duration)
    }

    pub fn shared_socket(
        socket: AsyncUdpSocket,
        dat_url: crypto::DatLocalDiscoverUrl,
        duration: Duration,
    ) -> Self {
        use async_std::stream::StreamExt as AsyncStreamExt;

        let socket = Arc::new(socket);
        let socket_handle = socket.clone();
        let packet = packet(&dat_url.0);

        let query_stream = stream::unfold((socket_handle, packet), |(socket, packet)| {
            async move {
                let bytes_writen = socket.send_to(&packet, *MULTICAST_DESTINATION).await;
                Some((bytes_writen, (socket, packet)))
            }
        })
        .throttle(duration)
        .boxed();

        Locator {
            query_stream,
            socket,
        }
    }
}

impl futures::Stream for Locator {
    type Item = std::io::Result<(Message, SocketAddr)>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        drop(self.query_stream.poll_next_unpin(cx));

        let mut buff = [0; 512];
        let response = {
            let mut read_future = self.socket.recv_from(&mut buff).boxed();
            futures::ready!(read_future.poll_unpin(cx))
        };

        if let Ok((bytes, peer)) = response {
            if let Ok(message) = Message::from_vec(&buff[..bytes]) {
                return Poll::Ready(Some(Ok((message, peer))));
            }
        }

        Poll::Pending
    }
}
