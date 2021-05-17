use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

use async_std::net::UdpSocket;
use hyperswarm_dht::{DhtConfig, HyperDht};

pub struct Config {
    pub bootstrap_servers: Vec<String>,
    pub bind_address: SocketAddr,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            bootstrap_servers: hyperswarm_dht::DEFAULT_BOOTSTRAP
                .iter()
                .map(|&z| z.to_owned())
                .collect(),
            bind_address: SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0).into(),
        }
    }
}

pub(crate) async fn dht(config: &Config) -> io::Result<HyperDht> {
    let socket = UdpSocket::bind(config.bind_address).await?;
    let dht_config = DhtConfig::default()
        .set_socket(socket)
        .set_bootstrap_nodes(&config.bootstrap_servers)
        .adaptive() // Becomes non-ephemeral after 30m
        .ephemeral(); // ephemeral defaults to true in discovery but defaults to false in dht
    let swarm = HyperDht::with_config(dht_config).await?;
    Ok(swarm)
}
