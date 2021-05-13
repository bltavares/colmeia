use anyhow::Context;
use async_std::{
    net::{TcpListener, TcpStream},
    prelude::FutureExt,
    sync::RwLock,
    task,
};
use colmeia_hyperdrive::Hyperdrive;
use colmeia_hyperswarm_dht::Config;
use ed25519_dalek::PublicKey;
use futures::{Future, Stream, StreamExt};
use hypercore_protocol::ProtocolBuilder;
use std::{collections::HashSet, net::SocketAddr, sync::Arc, time::Duration};

pub struct Hyperstack<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
{
    key: PublicKey,
    hyperdrive: Arc<RwLock<Hyperdrive<Storage>>>,
    connected_peers: Arc<RwLock<HashSet<SocketAddr>>>,
    listen_address: SocketAddr,
    discovery: Option<Box<dyn Stream<Item = (Vec<u8>, SocketAddr)> + Unpin + Send>>,
}

impl Hyperstack<random_access_disk::RandomAccessDisk> {
    // pub fn in_disk<P: AsRef<std::path::PathBuf>>(
    //     key: HashUrl,
    //     listen_address: SocketAddr,
    //     metadata: P,
    //     content: P,
    // ) -> anyhow::Result<Self> {
    //     let peers = Arc::new(RwLock::new(SegQueue::new()));
    //     let hyperdrive = hyperdrive::in_disk(key.public_key().clone(), metadata, content)?;
    //     Ok(Self {
    //         key,
    //         listen_address,
    //         peers,
    //         hyperdrive: Arc::new(RwLock::new(hyperdrive)),
    //         discovery: Box::new(stream::empty()),
    //     })
    // }
}

impl Hyperstack<random_access_memory::RandomAccessMemory> {
    pub async fn in_memory(key: PublicKey, listen_address: SocketAddr) -> anyhow::Result<Self> {
        let hyperdrive = colmeia_hyperdrive::in_memmory(key).await?;
        Ok(Self {
            key,
            listen_address,
            connected_peers: Arc::new(RwLock::new(HashSet::new())),
            hyperdrive: Arc::new(RwLock::new(hyperdrive)),
            discovery: None,
        })
    }
}

// TODO add_topic and remove_topic
// TODO handle multiple feeds
impl<Storage> Hyperstack<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    pub async fn lan(&self) -> anyhow::Result<impl Stream<Item = (Vec<u8>, SocketAddr)>> {
        let mut mdns = colmeia_hyperswarm_mdns::MdnsDiscovery::new();
        mdns.with_announcer(self.listen_address.port())
            .with_locator(Duration::from_secs(60)); // TODO: config
        mdns.add_topic(&hypercore_protocol::discovery_key(self.key.as_bytes()))
            .await?;
        Ok(mdns)
    }

    pub async fn dht(&self) -> anyhow::Result<impl Stream<Item = (Vec<u8>, SocketAddr)>> {
        let mut dht = colmeia_hyperswarm_dht::DHTDiscovery::default();
        dht.with_announcer(self.listen_address.port(), Duration::from_secs(60))
            .await
            .with_locator(Duration::from_secs(60)) // TODO: config
            .await;
        dht.add_topic(&hypercore_protocol::discovery_key(self.key.as_bytes()))
            .await?;
        Ok(dht)
    }

    pub fn with_discovery(
        &mut self,
        mechanisms: impl Stream<Item = (Vec<u8>, SocketAddr)> + Unpin + 'static + Send,
    ) -> &mut Self {
        self.discovery = Some(Box::new(mechanisms));
        self
    }

    pub fn hyperdrive(&self) -> Arc<RwLock<Hyperdrive<Storage>>> {
        self.hyperdrive.clone()
    }

    // TODO Move this method into a HypercoreExt trait?
    // TODO Allow access to the hypercore, so we can spanw the sync and keep using the hyperdrives
    pub fn replicate(&mut self) -> impl Future<Output = ()> + 'static {
        let discovery_driver = self.hyperdrive.clone();
        let discovery_connected_peers = self.connected_peers.clone();
        let discovery = self.discovery.take();

        let listen_address = self.listen_address;
        let listen_driver = self.hyperdrive.clone();
        let listen_connected_peers = self.connected_peers.clone();

        async move {
            let discovery = match discovery {
                Some(mut discovery) => task::spawn(async move {
                    while let Some((_, peer)) = discovery.next().await {
                        if !discovery_connected_peers.read().await.contains(&peer) {
                            let driver = discovery_driver.clone();
                            let connected_peers = discovery_connected_peers.clone();

                            task::spawn(async move {
                                if let Ok(tcp_stream) = TcpStream::connect(peer).await {
                                    connected_peers.write().await.insert(peer);
                                    let client = ProtocolBuilder::new(true).connect(tcp_stream);
                                    colmeia_hyperdrive::replicate_hyperdrive(
                                        client,
                                        driver.clone(),
                                    )
                                    .await;
                                    connected_peers.write().await.remove(&peer);
                                }
                            });
                        }
                    }
                }),
                None => task::spawn(async {}),
            };

            let listening = task::spawn(async move {
                let listener = TcpListener::bind(listen_address)
                    .await
                    .context("could not bind server to the address");

                if let Ok(listener) = listener {
                    loop {
                        if let Ok((tcp_stream, remote_addrs)) = listener.accept().await {
                            let driver = listen_driver.clone();
                            let connected_peers = listen_connected_peers.clone();
                            if !connected_peers.read().await.contains(&remote_addrs) {
                                task::spawn(async move {
                                    log::debug!("Received connection from {:?}", remote_addrs);
                                    connected_peers.write().await.insert(remote_addrs);
                                    let client = ProtocolBuilder::new(false).connect(tcp_stream);
                                    colmeia_hyperdrive::replicate_hyperdrive(
                                        client,
                                        driver.clone(),
                                    )
                                    .await;
                                    connected_peers.write().await.remove(&remote_addrs);
                                });
                            }
                        }
                    }
                } else {
                    log::error!("Error when creating listener {:?}", listener);
                }
            });

            listening.join(discovery).await;
        }
    }
}
