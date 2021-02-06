use std::{io, net::SocketAddr};

use async_std::net::UdpSocket;
use hyperswarm_dht::{DhtConfig, HyperDht};

pub struct Config {
    pub bootstrap_servers: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            bootstrap_servers: hyperswarm_dht::DEFAULT_BOOTSTRAP
                .iter()
                .map(|&z| z.to_owned())
                .collect(),
        }
    }
}

pub(crate) async fn dht(config: &Config) -> io::Result<HyperDht> {
    // TODO config
    let bind_address: SocketAddr = ([0, 0, 0, 0], 0).into();
    let socket = UdpSocket::bind(bind_address).await?;
    let dht_config = DhtConfig::default()
        .set_socket(socket)
        .set_bootstrap_nodes(&config.bootstrap_servers)
        .adaptive() // Becomes non-ephemeral after 30m
        .ephemeral(); // ephemeral defaults to true in discovery but defaults to false in dht
    let swarm = HyperDht::with_config(dht_config).await?;
    Ok(swarm)
}
