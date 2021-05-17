use async_std::{prelude::StreamExt, task};
use colmeia_hyperstack::utils::PublicKeyExt;
use colmeia_hyperswarm_dht::{announcer::Announcer, dht::Config, locator::Locator};
use std::{env, time::Duration};

// 7e5998407b3d9dbb94db21ff50ad6f1b1d2c79e476fbaf9856c342eb4382e7f5
// pra
// b62ef6792a0e2f2c5be328593532415d80c0f52c // 32 bytes
// Derivacao igual ao dat1
fn main() {
    log::debug!("Starting up");
    let hyper_hash = env::args()
        .nth(1)
        .expect("provide a hyper hash as argument");

    let bootstrap_servers = env::args()
        .nth(2)
        .map(|x| x.split(',').map(str::to_owned).collect())
        .unwrap_or_else(|| {
            hyperswarm_dht::DEFAULT_BOOTSTRAP
                .iter()
                .map(|&x| x.to_owned())
                .collect()
        });

    let duration = env::args()
        .nth(3)
        .unwrap_or_else(|| "10".into())
        .parse::<u64>()
        .expect("Could not parse duration into a number");
    let duration = Duration::from_secs(duration);

    env_logger::init();

    let key = hyper_hash.parse_from_hash().expect("could not parse hash");

    let config = Config {
        bootstrap_servers,
        ..Default::default()
    };

    let port = 2323;

    task::block_on(async move {
        log::info!("Starting up");

        let topic = hypercore_protocol::discovery_key(key.as_bytes());
        let announcer = Announcer::listen(&config, duration, port)
            .await
            .expect("could not start announcer");

        let mut locator = Locator::listen(&config, Duration::from_secs(1))
            .await
            .expect("Could not start locator");

        log::info!("Adding topic");
        announcer
            .add_topic(&topic)
            .await
            .expect("could not write topic");

        log::info!("Finding topic");
        locator
            .add_topic(&topic)
            .await
            .expect("Could not exec find topic");

        let result = locator.next().await;
        log::debug!("Found peer: {:?}", result);

        log::info!("Remove topic");
        announcer
            .remove_topic(&topic)
            .await
            .expect("could not remove topic");

        log::info!("Sleep");
        task::sleep(Duration::from_secs(60)).await;

        log::info!("Done");
    });
}
