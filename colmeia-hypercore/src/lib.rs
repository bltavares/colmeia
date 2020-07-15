use anyhow::Context;
use async_std::{future::IntoFuture, stream::IntoStream, sync::RwLock, task};
use ed25519_dalek::PublicKey;
use futures::{
    io::{AsyncRead, AsyncWrite},
    StreamExt,
};
use std::{
    collections::HashMap,
    io,
    pin::Pin,
    sync::{Arc, Mutex},
};

use hypercore_protocol as proto;

mod network;
mod observer;

pub use network::hyperdrive::PeeredHyperdrive;
pub use observer::{EventObserver, stream};

pub struct Hyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    pub(crate) metadata: Arc<RwLock<hypercore::Feed<Storage>>>,
    pub(crate) content: Option<Arc<RwLock<hypercore::Feed<Storage>>>>,
    content_storage: Option<hypercore::Storage<Storage>>,
}

pub async fn in_memmory(
    public_key: PublicKey,
) -> anyhow::Result<Hyperdrive<random_access_memory::RandomAccessMemory>> {
    let metadata = hypercore::Feed::builder(
        public_key,
        hypercore::Storage::new_memory()
            .await
            .context("could not page feed memory")?,
    )
    .build()
    .context("Could not start feed")?;

    let content_storage = hypercore::Storage::new_memory()
        .await
        .context("could not initialize the content storage")?;

    Ok(Hyperdrive {
        content_storage: Some(content_storage),
        content: None,
        metadata: Arc::new(RwLock::new(metadata)),
    })
}
