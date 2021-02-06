use async_std::{prelude::StreamExt, task};
use hyperswarm_dht::{DhtConfig, HyperDht};

fn main() {
    env_logger::init();

    task::block_on(async move {
        let config = DhtConfig::default().ephemeral().empty_bootstrap_nodes();
        let mut bootstrap = HyperDht::with_config(config)
            .await
            .expect("Could not start service");

        log::info!(
            "Bootstrap server running on: {}",
            bootstrap.local_addr().expect("could not get local address")
        );

        while let Some(event) = bootstrap.next().await {
            log::debug!("Event: {:?}", event);
        }
    });
}
