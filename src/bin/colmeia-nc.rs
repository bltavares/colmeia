use async_std::net::TcpStream;
use hypercore_protocol as proto;
use std::net::SocketAddr;

use colmeia_hypercore::PublicKeyExt;

fn name() -> String {
    let args: Vec<String> = std::env::args().skip(1).collect();
    args.first().expect("must have dat name as argument").into()
}

fn address() -> SocketAddr {
    let args: Vec<String> = std::env::args().skip(2).collect();
    let input = args
        .first()
        .expect("must have dat server:port name as argument");
    input.parse().expect("invalid ip:port as input")
}
fn main() {
    // 1. receive ip:port as argument (eg: hashs and 192.168.137.1:58906)
    // 2. try to connect and hadhsake (with hypercore-protocol) (eg: 7e5998407b3d9dbb94db21ff50ad6f1b1d2c79e476fbaf9856c342eb4382e7f5)
    // 3. Finalize with all working
    env_logger::init();

    let key = name();
    let address = address();
    let key = key.parse_from_hash().expect("invalid dat argument");

    async_std::task::block_on(async {
        let tcp_stream = TcpStream::connect(address)
            .await
            .expect("could not open address");
        let mut stream = proto::ProtocolBuilder::initiator().connect(tcp_stream);

        while let Ok(event) = stream.loop_next().await {
            match event {
                proto::Event::Handshake(_) => {
                    stream
                        .open(key.as_bytes().to_vec())
                        .await
                        .expect("could not start the stream");
                }
                proto::Event::DiscoveryKey(_) | proto::Event::Close(_) => {}
                proto::Event::Channel(mut channel) => {
                    let msg = proto::schema::Close {
                        discovery_key: Some(hypercore_protocol::discovery_key(key.as_bytes())),
                    };
                    channel
                        .close(msg)
                        .await
                        .expect("could not close the channel");
                }
            };
        }
    });
}
