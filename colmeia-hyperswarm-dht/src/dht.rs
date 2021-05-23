use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};

use async_broadcast::{broadcast, InactiveReceiver, Sender};
use async_std::{net::UdpSocket, sync::RwLock, task};
use futures::{FutureExt, StreamExt};
use hyperswarm_dht::{DhtConfig, HyperDht, HyperDhtEvent, QueryOpts};

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
    // let bootstrap_result = swarm.next().await;
    // log::debug!("Bootstrap result {:?}", bootstrap_result);
    Ok(swarm)
}

const BUFFER_SIZE: usize = 1;
#[derive(Debug, Clone)]
pub enum Outbound {
    Lookup(QueryOpts),
    Announce(QueryOpts),
    UnAnnounce(QueryOpts),
}

#[derive(Debug, Clone)]
pub enum Inbound {
    Lookup(hyperswarm_dht::Lookup),
    Announce {
        peers: Vec<hyperswarm_dht::Peers>,
        topic: hyperswarm_dht::IdBytes,
    },
    UnAnnounce {
        peers: Vec<hyperswarm_dht::Peers>,
        topic: hyperswarm_dht::IdBytes,
    },
}

#[derive(Debug, Clone)]
pub struct BroadcastChannel {
    pub send: Sender<Outbound>,
    pub receive: InactiveReceiver<Inbound>,
}

impl BroadcastChannel {
    pub async fn listen(config: &Config) -> io::Result<Self> {
        let dht = dht(config).await?;
        let swarm = Arc::new(RwLock::new(dht));

        let (outbound_sender, mut outbound_receiver) = broadcast::<Outbound>(BUFFER_SIZE);
        let _outbound = {
            let swarm = swarm.clone();
            task::spawn(async move {
                loop {
                    if let Ok(outbound) = outbound_receiver.recv().await {
                        match outbound {
                            Outbound::Lookup(query) => {
                                swarm.write().await.lookup(query);
                            }
                            Outbound::Announce(query) => {
                                swarm.write().await.announce(query);
                            }
                            Outbound::UnAnnounce(query) => {
                                swarm.write().await.unannounce(query);
                            }
                        }
                    }
                }
            })
        };

        let (receiver_sender, receiver_receiver) = broadcast::<Inbound>(BUFFER_SIZE);
        let _inbound = {
            task::spawn(async move {
                loop {
                    let event = swarm.write().await.next().now_or_never();
                    if let Some(Some(event)) = event {
                        match event {
                            HyperDhtEvent::Bootstrapped { stats } => {
                                log::debug!("Swarm bootstrapped {:?}", stats)
                            }
                            HyperDhtEvent::AnnounceResult { peers, topic, .. } => {
                                let result = receiver_sender
                                    .broadcast(Inbound::Announce { peers, topic })
                                    .await;
                                log::trace!("AnnounceResult: {:?}", result);
                            }
                            HyperDhtEvent::LookupResult { lookup, .. } => {
                                let result =
                                    receiver_sender.broadcast(Inbound::Lookup(lookup)).await;
                                log::trace!("LookupResult: {:?}", result);
                            }
                            HyperDhtEvent::UnAnnounceResult { peers, topic, .. } => {
                                let result = receiver_sender
                                    .broadcast(Inbound::UnAnnounce { peers, topic })
                                    .await;
                                log::trace!("UnAnnounceResult: {:?}", result);
                            }
                            e => {
                                log::debug!("unexpected event received: {:?}", e);
                            }
                        }
                    }
                }
            })
        };

        Ok(Self {
            send: outbound_sender,
            receive: receiver_receiver.deactivate(),
        })
    }
}
