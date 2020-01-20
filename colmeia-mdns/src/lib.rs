use async_std::net::UdpSocket as AsyncUdpSocket;
use futures::stream;
use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use trust_dns_proto::op::Message;
use trust_dns_proto::rr::RData::TXT;

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

/// Only Unix has this setup
#[cfg(unix)]
fn reuse_port(socket: &Socket) -> io::Result<()> {
    socket.set_reuse_port(true)
}

#[cfg(windows)]
fn reuse_port(_socket: &Socket) -> io::Result<()> {
    Ok(())
}

pub fn create_shared_socket() -> Result<UdpSocket, io::Error> {
    let socket = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    socket.set_reuse_address(true)?;
    reuse_port(&socket)?;
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

type MessageStream = (Message, SocketAddr);

async fn read_dns_message(
    socket: Arc<AsyncUdpSocket>,
) -> Option<(Option<MessageStream>, Arc<AsyncUdpSocket>)> {
    let mut buff = [0; 512];
    if let Ok((bytes, peer)) = socket.recv_from(&mut buff).await {
        if let Ok(message) = Message::from_vec(&buff[..bytes]) {
            log::debug!("MDNS message received");
            return Some((Some((message, peer)), socket));
        }
    }
    Some((None, socket))
}

fn select_location_response(
    dat_url: &crypto::DatLocalDiscoverUrl,
    response: Option<MessageStream>,
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
    pub fn new(
        socket: UdpSocket,
        dat_url: crypto::DatLocalDiscoverUrl,
        duration: Duration,
    ) -> Self {
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
                let bytes_writen = socket.send_to(&packet, *MULTICAST_DESTINATION).await;
                log::debug!("MDNS query sent");
                Some((bytes_writen, (socket, packet)))
            }
        })
        .throttle(duration);

        let listen_stream = stream::unfold(socket, read_dns_message)
            .filter_map(move |message| select_location_response(&dat_url, message));

        Locator {
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

fn annouce_dat(
    dat_url: &crypto::DatLocalDiscoverUrl,
    response: Option<MessageStream>,
) -> Option<SocketAddr> {
    None
}

pub struct Announcer {
    listen_stream: Pin<Box<dyn futures::Stream<Item = SocketAddr> + Send>>,
}

impl Announcer {
    pub fn new(socket: UdpSocket, dat_url: crypto::DatLocalDiscoverUrl) -> Self {
        Announcer::shared_socket(Arc::new(socket.into()), dat_url)
    }

    fn shared_socket(socket: Arc<AsyncUdpSocket>, dat_url: crypto::DatLocalDiscoverUrl) -> Self {
        use async_std::stream::StreamExt as AsyncStreamExt;

        let listen_stream = stream::unfold(socket.clone(), read_dns_message)
            .filter_map(move |message| annouce_dat(&dat_url, message));

        Announcer {
            listen_stream: Box::pin(listen_stream),
        }
    }
}

impl futures::Stream for Announcer {
    type Item = SocketAddr;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;

        self.listen_stream.poll_next_unpin(cx)
    }
}
