use async_std::net::TcpStream;
use async_std::stream::StreamExt;
use colmeia_dat_proto::*;
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
  dat_key: Option<Vec<u8>>,
}

impl SimpleDatClient {
  pub fn new() -> Self {
    Self { dat_key: None }
  }
}

#[async_trait]
impl DatObserver for SimpleDatClient {
  async fn on_feed(&mut self, client: &mut Client, message: &proto::Feed) -> Option<()> {
    if let None = self.dat_key {
      self.dat_key = Some(message.get_discoveryKey().to_vec());
      eprintln!("{:?}", hex::encode(message.get_discoveryKey().to_vec()));
      client
        .writer()
        .send(ChannelMessage::new(
          1,
          0,
          message.write_to_bytes().expect("invalid dat message"),
        ))
        .await
        .expect("could not write message back");

      return Some(());
    } else {
      return None;
    }
  }
}

fn main() {
  env_logger::init();

  let address = address();
  let key = name();
  async_std::task::block_on(async {
    let tcp_stream = TcpStream::connect(address)
      .await
      .expect("could not open address");
    let client_initialization = new_client(&key, tcp_stream).await;
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
