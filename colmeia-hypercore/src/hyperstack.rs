use crate::{
    hyperdrive::{self, Hyperdrive},
    sync_hyperdrive,
};
use anyhow::Context;
use async_std::{
    net::{TcpListener, TcpStream},
    prelude::FutureExt,
    sync::RwLock,
    task,
};
use colmeia_hypercore_utils::HashUrl;
use futures::{stream, Stream, StreamExt};
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
    key: HashUrl,
    hyperdrive: Arc<RwLock<Hyperdrive<Storage>>>,
    connected_peers: Arc<RwLock<HashSet<SocketAddr>>>,
    listen_address: SocketAddr,
    discovery: Box<dyn Stream<Item = SocketAddr> + Unpin + Send>,
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
    pub async fn in_memory(key: HashUrl, listen_address: SocketAddr) -> anyhow::Result<Self> {
        let hyperdrive = hyperdrive::in_memmory(*key.public_key()).await?;
        Ok(Self {
            key,
            listen_address,
            connected_peers: Arc::new(RwLock::new(HashSet::new())),
            hyperdrive: Arc::new(RwLock::new(hyperdrive)),
            discovery: Box::new(stream::empty()),
        })
    }
}

impl<Storage> Hyperstack<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    pub fn lan(&self) -> impl Stream<Item = SocketAddr> {
        let mut mdns =
            colmeia_hyperswarm_mdns::MdnsDiscovery::new(self.key.discovery_key().to_vec());
        mdns.with_announcer(self.listen_address.port())
            .with_locator(Duration::from_secs(60));
        mdns
    }

    pub fn with_discovery(
        &mut self,
        mechanisms: impl Stream<Item = SocketAddr> + Unpin + 'static + Send,
    ) -> &mut Self {
        self.discovery = Box::new(mechanisms);
        self
    }

    // TODO Move this method into a HypercoreExt trait?
    // TODO Allow access to the hypercore, so we can spanw the sync and keep using the hyperdrives
    pub async fn sync(self) {
        let driver = self.hyperdrive.clone();
        let connected_peers = self.connected_peers.clone();

        let mut discovery = self.discovery;
        let discovery = task::spawn(async move {
            while let Some(peer) = discovery.next().await {
                if !connected_peers.read().await.contains(&peer) {
                    let driver = driver.clone();
                    let connected_peers = connected_peers.clone();

                    task::spawn(async move {
                        if let Ok(tcp_stream) = TcpStream::connect(peer).await {
                            connected_peers.write().await.insert(peer);
                            let client = ProtocolBuilder::new(true).connect(tcp_stream);
                            sync_hyperdrive(client, driver.clone()).await;
                            connected_peers.write().await.remove(&peer);
                        }
                    });
                }
            }
        });

        let listen_address = self.listen_address;
        let driver = self.hyperdrive.clone();
        let connected_peers = self.connected_peers.clone();

        let listening = task::spawn(async move {
            let listener = TcpListener::bind(listen_address)
                .await
                .context("could not bind server to the address");

            if let Ok(listener) = listener {
                loop {
                    if let Ok((tcp_stream, remote_addrs)) = listener.accept().await {
                        let driver = driver.clone();
                        let connected_peers = connected_peers.clone();
                        if !connected_peers.read().await.contains(&remote_addrs) {
                            task::spawn(async move {
                                log::debug!("Received connection from {:?}", remote_addrs);
                                connected_peers.write().await.insert(remote_addrs);
                                let client = ProtocolBuilder::new(false).connect(tcp_stream);
                                sync_hyperdrive(client, driver.clone()).await;
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
