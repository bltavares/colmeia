use async_std::{prelude::StreamExt, task};
use colmeia_hyperstack::utils::PublicKeyExt;
use colmeia_hyperswarm_dht::{dht::Config, DHTDiscovery};
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

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let key = hyper_hash.parse_from_hash().expect("could not parse hash");

    let config = Config {
        bootstrap_servers,
        ..Default::default()
    };

    task::block_on(async move {
        log::info!("Starting up");

        let topic = hypercore_protocol::discovery_key(key.as_bytes());

        let swarm = DHTDiscovery::listen(config).await;
        let mut swarm = swarm.expect("Could not bind socket");
        swarm
            // .with_announcer(2323, duration)
            .with_locator(duration);

        log::info!("Adding topic");
        swarm
            .add_topic(&topic)
            .await
            .expect("could not write topic");

        log::info!("Finding topic");
        let result = swarm.next().await;
        log::info!("Found peer: {:?}", result);

        log::info!("Remove topic");
        swarm
            .remove_topic(&topic)
            .await
            .expect("could not remove topic");

        log::info!("Sleep");
        task::sleep(Duration::from_secs(60)).await;

        log::info!("Done");
    });
}
