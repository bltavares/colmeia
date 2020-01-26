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

fn main() {
  env_logger::init();

  let socket = address();
  let key = name();
  async_std::task::block_on(async {
    let client_initialization = new_client(&key, socket).await;
    let mut client = handshake(client_initialization)
      .await
      .expect("could not handshake");
    if let Some(Ok(message)) = client.reader().next().await {
      eprintln!("{:?}", message.parse().expect("parsed message"));
    }
    ping(&mut client).await.expect("could not ping");
    ping(&mut client).await.expect("could not ping");
    if let Some(Ok(message)) = client.reader().next().await {
      eprintln!("{:?}", message.parse().expect("parsed message"));
    }
  });
}
