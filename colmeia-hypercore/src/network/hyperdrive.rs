use super::hypercore::PeeredHypercore;
use crate::{observer::EventObserver, Hyperdrive};

use async_std::sync::RwLock;
use futures::io::{AsyncRead, AsyncWrite};
use hypercore_protocol as proto;
use std::sync::Arc;

pub struct PeeredHyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    metadata: Option<PeeredHypercore<Storage>>,
    hyperdrive: Arc<RwLock<Hyperdrive<Storage>>>,
    content: Option<PeeredHypercore<Storage>>,
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
        + Sync,
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
            let core = PeeredHypercore::new(channel, feed);
            self.metadata = Some(core);
            return Ok(());
        };
        if self.content.is_none() {
            log::debug!("initializing content feed");
            let feed = self.hyperdrive.read().await.metadata.clone();
            let core = PeeredHypercore::new(channel, feed);
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
            client.open(metadata.public_key().as_bytes().to_vec()).await?;
        }
        Ok(())
    }

    async fn loop_next(&mut self, _client: &mut proto::Protocol<S, S>) -> Result<(), Self::Err> {
        if let Some(ref mut metadata) = &mut self.metadata {
            dbg!(metadata.loop_next().await);
        }

        if let Some(ref mut content) = &mut self.content {
            dbg!(content.loop_next().await);
        }

        Ok(())
    }
}
