use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::FutureExt;
use async_std::stream::StreamExt;
use colmeia_dat1_proto::*;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

pub struct SimpleDatServer {
    handshake: SimpleDatHandshake,
    dat_keys: HashMap<u64, Vec<u8>>,
}

impl Default for SimpleDatServer {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleDatServer {
    pub fn new() -> Self {
        Self {
            dat_keys: HashMap::new(),
            handshake: SimpleDatHandshake::new(),
        }
    }
}

#[async_trait]
impl DatProtocolEvents for SimpleDatServer {
    async fn on_feed(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Feed,
    ) -> Option<()> {
        if !self.dat_keys.contains_key(&channel) {
            client
                .writer()
                .send(ChannelMessage::new(
                    channel,
                    0,
                    message.write_to_bytes().expect("invalid feed message"),
                ))
                .await
                .ok()?;
        }
        self.handshake.on_feed(client, channel, message).await?;
        self.dat_keys
            .insert(channel, message.get_discoveryKey().to_vec());
        Some(())
    }

    async fn on_handshake(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Handshake,
    ) -> Option<()> {
        self.handshake
            .on_handshake(client, channel, message)
            .await?;
        Some(())
    }

    async fn on_finish(&mut self, _client: &mut Client) {
        for (channel, key) in &self.dat_keys {
            eprintln!("collected info {:?} dat://{:?}", channel, hex::encode(&key));
        }
    }
}

fn address() -> SocketAddr {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let input = args
        .first()
        .expect("must have dat server:port name as argument");
    input.parse().expect("invalid ip:port as input")
}

fn name() -> String {
    let args: Vec<String> = std::env::args().skip(2).collect();
    args.first().expect("must have dat name as argument").into()
}

async fn lan(dat_url: String, address: SocketAddr) {
    let mut mdns = colmeia_dat1_mdns::Mdns::new(&dat_url).expect("could not start mdns");
    mdns.with_announcer(address.port())
        .with_location(Duration::from_secs(60));
    while let Some(peer) = mdns.next().await {
        println!("Peer found {:?}", peer);
    }
}

async fn serve(dat_url: String, address: SocketAddr) {
    let listener = TcpListener::bind(address)
        .await
        .expect("could not bind server to the address");

    loop {
        // TODO Spawn new connections without awaitng. how?
        if let Ok((stream, remote_addrs)) = listener.accept().await {
            eprintln!("Received connection from {:?}", remote_addrs);
            handle_connection(&dat_url, stream).await;
        }
    }
}
async fn handle_connection(dat_url: &str, tcp_stream: TcpStream) {
    let server_initialization = new_client(&dat_url, tcp_stream).await;
    let client = handshake(server_initialization)
        .await
        .expect("could not handshake");

    let observer = SimpleDatServer::new();
    let mut service = DatService::new(client, observer);

    while let Some(message) = service.next().await {
        if let DatMessage::Feed(message) = message.parse().unwrap() {
            eprintln!(
                "Received message discovery {:?}",
                hex::encode(message.get_discoveryKey())
            );
        }
    }
}

/// Simple Server to test connectivity
fn main() {
    env_logger::init();

    let address = address();
    let key = name();

    async_std::task::block_on(async {
        let lan = async_std::task::spawn(lan(key.clone(), address));
        let serve = async_std::task::spawn(serve(key.clone(), address));

        lan.join(serve).await;
    });
}
