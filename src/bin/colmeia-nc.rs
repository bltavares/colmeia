use async_std::net::TcpStream;
use colmeia_hypercore_utils::{parse, UrlResolution};
use hypercore_protocol as proto;
use std::net::SocketAddr;

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
    let dat_key = parse(&key).expect("invalid dat argument");
    let dat_key = match dat_key {
        UrlResolution::HashUrl(result) => result,
        _ => panic!("invalid hash key"),
    };
    async_std::task::block_on(async {
        let tcp_stream = TcpStream::connect(address)
            .await
            .expect("could not open address");
        let mut stream = proto::ProtocolBuilder::initiator().connect(tcp_stream);

        while let Ok(event) = stream.loop_next().await {
            dbg!(&event);
            match event {
                proto::Event::Handshake(_) => {
                    stream
                        .open(dat_key.public_key().to_bytes().to_vec())
                        .await
                        .expect("could not start the stream");
                }
                proto::Event::DiscoveryKey(_) => {}
                proto::Event::Channel(mut channel) => {
                    let msg = proto::schema::Close {
                        discovery_key: Some(dat_key.discovery_key().to_vec()),
                    };
                    channel
                        .close(msg)
                        .await
                        .expect("could not close the channel");
                }
                proto::Event::Close(_) => {}
            };
        }
    });
}
