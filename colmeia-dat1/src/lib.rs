use async_std::stream::StreamExt;
use async_std::{stream, task};
use chashmap::CHashMap;
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
    Discovered,
    Disconnected,
}

pub struct Dat {
    key: HashUrl,
    discovered_peers: Arc<RwLock<CHashMap<SocketAddr, PeerState>>>,
    discovery: Box<dyn Stream<Item = SocketAddr> + Unpin>,
}

impl Dat {
    pub fn readonly(key: HashUrl) -> Self {
        let discovered_peers = Arc::new(RwLock::new(CHashMap::new()));
        Self {
            key,
            discovered_peers,
            discovery: Box::new(stream::empty()),
        }
    }

    pub fn lan(&self, address: SocketAddr) -> impl Stream<Item = SocketAddr> {
        let mut mdns = colmeia_dat1_mdns::Mdns::new(self.key.clone());
        mdns.with_announcer(address.port())
            .with_location(Duration::from_secs(60));
        mdns
    }

    pub fn with_discovery(
        &mut self,
        mechanisms: Vec<impl Stream<Item = SocketAddr> + Unpin + 'static>,
    ) {
        let out: Box<dyn Stream<Item = SocketAddr> + Unpin> = mechanisms
            .into_iter()
            .fold(Box::new(stream::empty()), |init, item| {
                Box::new(init.merge(item))
            });

        self.discovery = out;
    }
}
