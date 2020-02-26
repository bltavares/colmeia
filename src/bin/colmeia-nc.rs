use anyhow::Context;
use async_std::net::TcpStream;
use async_std::stream::StreamExt;
use colmeia_dat1_proto::*;
use std::collections::HashMap;
use std::net::SocketAddr;

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

pub struct SimpleDatClient {
    handshake: SimpleDatHandshake,
    dat_keys: HashMap<u64, Vec<u8>>,
}

impl Default for SimpleDatClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleDatClient {
    pub fn new() -> Self {
        Self {
            dat_keys: HashMap::new(),
            handshake: SimpleDatHandshake::new(),
        }
    }
}

#[async_trait]
impl DatProtocolEvents for SimpleDatClient {
    type Err = anyhow::Error;

    async fn on_feed(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Feed,
    ) -> Result<(), Self::Err> {
        if !self.dat_keys.contains_key(&channel) {
            client
                .writer()
                .send(ChannelMessage::new(
                    channel,
                    0,
                    message.write_to_bytes().expect("invalid feed message"),
                ))
                .await
                .context("could not send feed")?;
        }
        self.handshake.on_feed(client, channel, message).await?;
        self.dat_keys
            .insert(channel, message.get_discoveryKey().to_vec());
        Ok(())
    }

    async fn on_handshake(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Handshake,
    ) -> Result<(), Self::Err> {
        self.handshake
            .on_handshake(client, channel, message)
            .await?;
        Ok(())
    }

    async fn on_finish(&mut self, _client: &mut Client) {
        for (channel, key) in &self.dat_keys {
            eprintln!("collected info {:?} dat://{:?}", channel, hex::encode(&key));
        }
    }
}

fn main() {
    env_logger::init();

    let address = address();
    let key = name();
    let dat_key = colmeia_dat1_core::parse(&key).expect("invalid dat argument");
    let dat_key = match dat_key {
        colmeia_dat1_core::DatUrlResolution::HashUrl(result) => result,
        _ => panic!("invalid hash key"),
    };
    async_std::task::block_on(async {
        let tcp_stream = TcpStream::connect(address)
            .await
            .expect("could not open address");
        let client_initialization = new_client(dat_key, tcp_stream).await;
        let client = handshake(client_initialization)
            .await
            .expect("could not handshake");

        let observer = SimpleDatClient::new();
        let mut service = DatService::new(client, observer);

        while let Some(message) = service.next().await {
            if let DatMessage::Feed(message) = message.parse().unwrap() {
                eprintln!(
                    "Received message {:?}",
                    hex::encode(message.get_discoveryKey())
                );
            }
        }
    });
}
