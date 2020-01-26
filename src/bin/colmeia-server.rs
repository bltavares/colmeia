use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::FutureExt;
use async_std::stream::StreamExt;
use colmeia_dat_proto::*;
use std::net::SocketAddr;
use std::time::Duration;

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
    let mut mdns = colmeia_dat_mdns::Mdns::new(&dat_url).expect("could not start mdns");
    mdns.with_announcer(address.port())
        .with_location(Duration::from_secs(10));
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
    let mut client = handshake(server_initialization)
        .await
        .expect("could not handshake");
    if let Some(Ok(message)) = client.reader().next().await {
        eprintln!("{:?}", message);
        eprintln!("{:?}", message.parse().expect("parsed message"));
    }
    ping(&mut client).await.expect("could not ping");
    ping(&mut client).await.expect("could not ping");
    if let Some(Ok(message)) = client.reader().next().await {
        eprintln!("{:?}", message);
        eprintln!("{:?}", message.parse().expect("parsed message"));
    }
}

/// Simple Server to test connectivity
fn main() {
    env_logger::init();

    let address = address();
    let key = name();

    async_std::task::block_on(async {
        let lan = async_std::task::spawn(lan(key.clone(), address.clone()));
        let serve = async_std::task::spawn(serve(key.clone(), address.clone()));

        lan.join(serve).await;
    });
}
