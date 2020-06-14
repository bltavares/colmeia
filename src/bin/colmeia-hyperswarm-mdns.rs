use async_std::stream::StreamExt;
use async_std::task;
use colmeia_dat1_core::{parse, DatUrlResolution};
use colmeia_hyperswarm_mdns::Locator;
use std::time::Duration;

// 7e5998407b3d9dbb94db21ff50ad6f1b1d2c79e476fbaf9856c342eb4382e7f5
// pra
// b62ef6792a0e2f2c5be328593532415d80c0f52c // 32 bytes
// Derivacao igual ao dat1
fn main() {
    env_logger::init();

    task::block_on(async {
        let parse_result =
            parse("7e5998407b3d9dbb94db21ff50ad6f1b1d2c79e476fbaf9856c342eb4382e7f5")
                .expect("could not parse data");

        if let DatUrlResolution::HashUrl(hash) = parse_result {
            let mut locator = Locator::new(hash.discovery_key(), Duration::from_secs(10))
                .expect("could not initiate our locator");

            while let Some(response) = locator.next().await {
                println!("Peer found: {:?}", response);
            }
        } else {
            println!("this is a regular DNS")
        }
    });
}
