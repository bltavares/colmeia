use async_std::{prelude::StreamExt, task};
use colmeia_dat1_core::{parse, DatUrlResolution};
use colmeia_hyperswarm_mdns::MdnsDiscovery;
use std::env;
use std::time::Duration;

// 7e5998407b3d9dbb94db21ff50ad6f1b1d2c79e476fbaf9856c342eb4382e7f5
// pra
// b62ef6792a0e2f2c5be328593532415d80c0f52c // 32 bytes
// Derivacao igual ao dat1
fn main() {
    let hyper_hash = env::args()
        .nth(1)
        .expect("provide a hyper hash as argument");

    let port = env::args()
        .nth(2)
        .expect("requires a port as second argument")
        .parse::<u16>()
        .expect("Could not parse port as number");

    let duration = env::args()
        .nth(3)
        .unwrap_or("10".into())
        .parse::<u64>()
        .expect("Could not parse duration into a number");

    env_logger::init();

    let parse_result = parse(&hyper_hash).expect("could not parse data");

    if let DatUrlResolution::HashUrl(hash) = parse_result {
        task::block_on(async move {
            let mut mdns = MdnsDiscovery::new(hash.discovery_key().to_vec());
            mdns.with_announcer(port)
                .with_locator(Duration::from_secs(duration));

            while let Some(packet) = mdns.next().await {
                println!("Found peer on {:?}", packet);
            }
        });
    } else {
        println!("this is a regular DNS")
    }
}
