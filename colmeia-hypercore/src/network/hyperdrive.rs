use super::hypercore::PeeredHypercore;
use crate::{
    observer::{EventObserver, MessageDriver},
    Hyperdrive,
};

use async_std::prelude::{FutureExt, StreamExt};
use async_std::sync::RwLock;
use futures::io::{AsyncRead, AsyncWrite};
use hypercore_protocol as proto;
use std::{sync::Arc, time::Duration};

pub struct PeeredHyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    metadata: Option<MessageDriver>,
    hyperdrive: Arc<RwLock<Hyperdrive<Storage>>>,
    content: Option<MessageDriver>,
    // delay_feed_content: Option<proto::Feed>,
}

impl<Storage> PeeredHyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    pub fn new(hyperdrive: Arc<RwLock<Hyperdrive<Storage>>>) -> anyhow::Result<Self> {
        Ok(Self {
            metadata: None,
            hyperdrive,
            content: None,
            // delay_feed_content: None,
        })
    }
}

#[async_trait::async_trait]
impl<Storage, S> EventObserver<S> for PeeredHyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
    S: AsyncRead + AsyncWrite + Send + Unpin + Clone + 'static,
{
    type Err = anyhow::Error;

    async fn on_channel(
        &mut self,
        _client: &mut proto::Protocol<S, S>,
        channel: proto::Channel,
    ) -> Result<(), Self::Err> {
        if self.metadata.is_none() {
            log::debug!("initializing metadata feed");
            let feed = self.hyperdrive.read().await.metadata.clone();
            let core = MessageDriver::stream(channel, PeeredHypercore::new(feed));
            self.metadata = Some(core);
            return Ok(());
        };
        if self.content.is_none() {
            log::debug!("initializing content feed");
            let feed = self.hyperdrive.read().await.metadata.clone();
            let core = MessageDriver::stream(channel, PeeredHypercore::new(feed));
            self.metadata = Some(core);
            return Ok(());
        }

        anyhow::bail!("Initialized more than once")
    }

    async fn on_discovery_key(
        &mut self,
        client: &mut proto::Protocol<S, S>,
        message: &[u8],
    ) -> Result<(), Self::Err> {
        if self.metadata.is_none() {
            let drive = self.hyperdrive.read().await;
            let metadata = drive.metadata.read().await;
            let public_key_for_metadata = metadata.public_key().as_bytes();
            let metadata_discovery_key = hypercore_protocol::discovery_key(public_key_for_metadata);
            if message == metadata_discovery_key.as_slice() {
                client.open(public_key_for_metadata.to_vec()).await?;
            }
        }
        Ok(())
    }

    async fn tick(&mut self, _client: &mut proto::Protocol<S, S>) -> Result<(), Self::Err> {
        dbg!("tick");

        if let Some(ref mut metadata) = &mut self.metadata {
            dbg!("tick - metadata");
            dbg!(metadata.next().timeout(Duration::from_secs(1)).await?);
        }

        if let Some(ref mut content) = &mut self.content {
            dbg!("tick - content");
            dbg!(content.next().timeout(Duration::from_secs(1)).await?);
        }

        Ok(())
    }

    async fn on_finish(&mut self, _client: &mut proto::Protocol<S, S>) {
        let driver = self.hyperdrive.read().await;
        let mut metadata = driver.metadata.write().await;
        log::debug!("Metadata audit: {:?}", metadata.audit().await);
        log::debug!("Metadata len: {:?}", metadata.len());

        if let Some(ref content) = driver.content {
            let mut content = content.write().await;
            log::debug!("Content audit: {:?}", content.audit().await);
            log::debug!("Content len: {:?}", content.len());
        }
    }
}
