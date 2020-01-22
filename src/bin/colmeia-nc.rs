use async_std::net::TcpStream;
use async_std::stream::StreamExt;
use colmeia_proto::DatReader;
use std::net::SocketAddr;

fn address() -> SocketAddr {
  let args: Vec<String> = std::env::args().skip(1).collect();
  let input = args
    .first()
    .expect("must have dat server:port name as argument");
  input.parse().expect("invalid ip:port as input")
}

fn main() {
  env_logger::init();

  let socket = address();

  async_std::task::block_on(async {
    let mut client = DatReader::new(
      TcpStream::connect(socket)
        .await
        .expect("could not open socket"),
    );

    while let Some(message) = client.next().await {
      println!("{:?}", message);
    }
  });
}
