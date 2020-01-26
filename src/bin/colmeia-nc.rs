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
    let mut client = Client::new(&key, socket).await;
    while let Some(Ok(message)) = client.next().await {
      println!("{:?}", message.parse().expect("parsed message"));
    }
  });
}
