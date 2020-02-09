use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::FutureExt;
use async_std::stream::StreamExt;
use async_std::{stream, task};
use crossbeam_queue::SegQueue;
use futures::Stream;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use colmeia_dat1_core::HashUrl;

mod hypercore;
mod hyperdrive;
mod schema;

pub use crate::hypercore::*;
pub use crate::hyperdrive::*;

enum PeerState {
    Discovered(SocketAddr),
    Connected(TcpStream),
    Peered(DatService),
}

pub struct Dat<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = failure::Error>
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
{
    key: HashUrl,
    hyperdrive: Arc<RwLock<Hyperdrive<Storage>>>,
    listen_address: SocketAddr,
    peers: Arc<RwLock<SegQueue<PeerState>>>,
    discovery: Box<dyn Stream<Item = SocketAddr> + Unpin + Send>,
}

impl Dat<random_access_disk::RandomAccessDisk> {
    pub fn in_disk<P: AsRef<std::path::PathBuf>>(
        key: HashUrl,
        listen_address: SocketAddr,
        metadata: P,
        content: P,
    ) -> Self {
        let peers = Arc::new(RwLock::new(SegQueue::new()));
        let hyperdrive = hyperdrive::in_disk(key.public_key().clone(), metadata, content);
        Self {
            key,
            listen_address,
            peers,
            hyperdrive: Arc::new(RwLock::new(hyperdrive)),
            discovery: Box::new(stream::empty()),
        }
    }
}

impl Dat<random_access_memory::RandomAccessMemory> {
    pub fn in_memory(key: HashUrl, listen_address: SocketAddr) -> Self {
        let peers = Arc::new(RwLock::new(SegQueue::new()));
        let hyperdrive = hyperdrive::in_memmory(key.public_key().clone());
        Self {
            key,
            listen_address,
            peers,
            hyperdrive: Arc::new(RwLock::new(hyperdrive)),
            discovery: Box::new(stream::empty()),
        }
    }
}

impl<Storage> Dat<Storage>
where
    Storage:
        random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send + Sync,
{
    pub fn lan(&self) -> impl Stream<Item = SocketAddr> {
        let mut mdns = colmeia_dat1_mdns::Mdns::new(self.key.clone());
        mdns.with_announcer(self.listen_address.port())
            .with_location(Duration::from_secs(60));
        mdns
    }

    pub fn with_discovery(
        &mut self,
        mechanisms: Vec<impl Stream<Item = SocketAddr> + Unpin + 'static + Send>,
    ) -> &mut Self {
        let out: Box<dyn Stream<Item = SocketAddr> + Unpin + Send> = mechanisms
            .into_iter()
            .fold(Box::new(stream::empty()), |init, item| {
                Box::new(init.merge(item))
            });

        self.discovery = out;
        self
    }

    pub async fn sync(self) {
        let mut discovery = self.discovery;
        let peers = self.peers.clone();
        let discovery = task::spawn(async move {
            while let Some(peer) = discovery.next().await {
                peers.write().unwrap().push(PeerState::Discovered(peer));
            }
        });

        let listen_address = self.listen_address;
        let peers = self.peers.clone();
        let listening = task::spawn(async move {
            let listener = TcpListener::bind(listen_address)
                .await
                .expect("could not bind server to the address");

            loop {
                if let Ok((stream, remote_addrs)) = listener.accept().await {
                    log::debug!("Received connection from {:?}", remote_addrs);
                    peers.write().unwrap().push(PeerState::Connected(stream))
                }
            }
        });

        let peers = self.peers.clone();
        let key = self.key;
        let hyperdrive = self.hyperdrive;
        let connection = task::spawn(async move {
            loop {
                let connection = peers.write().unwrap().pop();
                match connection {
                    Ok(PeerState::Discovered(socket)) => {
                        let stream = TcpStream::connect(socket).await;
                        if let Ok(stream) = stream {
                            peers.write().unwrap().push(PeerState::Connected(stream));
                        }
                    }
                    Ok(PeerState::Connected(stream)) => {
                        let client_initialization = new_client(key.clone(), stream).await;
                        let client = handshake(client_initialization).await;
                        if let Some(client) = client {
                            let peered = PeeredHyperdrive::new(hyperdrive.clone());
                            let driver = DatService::new(client, peered);
                            peers.write().unwrap().push(PeerState::Peered(driver));
                        }
                    }
                    Ok(PeerState::Peered(mut service)) => {
                        if let Ok(Some(message)) =
                            async_std::future::timeout(Duration::from_secs(5), service.next()).await
                        {
                            log::debug!("Dat Message handled {:?}", message);
                            peers.write().unwrap().push(PeerState::Peered(service));
                        }
                    }
                    Err(e) => {
                        log::debug!("queue empty: {:?}", e);
                        task::sleep(Duration::from_secs(1)).await
                    }
                }
            }
        });

        listening.join(discovery).join(connection).await;
    }
}
