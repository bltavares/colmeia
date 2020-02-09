use colmeia_dat1_mdns::Mdns;
use futures::stream::StreamExt;
use std::time::Duration;

fn name() -> String {
    let args: Vec<String> = std::env::args().skip(1).collect();
    args.first().expect("must have dat name as argument").into()
}

fn main() {
    env_logger::init();

    let name = name();

    async_std::task::block_on(async {
        let dat_key = colmeia_dat1_core::parse(&name).expect("invalid dat argument");
        let dat_key = match dat_key {
            colmeia_dat1_core::DatUrlResolution::HashUrl(result) => result,
            _ => panic!("invalid hash key"),
        };
        let mut mdns = Mdns::new(dat_key);
        mdns.with_announcer(3282)
            .with_location(Duration::from_secs(10));

        while let Some(peer) = mdns.next().await {
            println!("Peer found {:?}", peer);
        }
    })
}
