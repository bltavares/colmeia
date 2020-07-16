use super::hypercore::PeeredHypercore;
use crate::{
    observer::{EventObserver, MessageDriver},
    Emit, Hyperdrive,
};

use anyhow::Context;
use async_std::prelude::StreamExt;
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
    metadata: Option<MessageDriver>,
    hyperdrive: Arc<RwLock<Hyperdrive<Storage>>>,
    content: Option<MessageDriver>,
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
            let feed = self.hyperdrive.read().await.content.clone();
            if let Some(feed) = feed {
                let core = MessageDriver::stream(channel, PeeredHypercore::new(feed));
                self.content = Some(core);
            };
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

    async fn tick(&mut self, client: &mut proto::Protocol<S, S>) -> Result<(), Self::Err> {
        dbg!("tick");

        if let Some(ref mut metadata) = &mut self.metadata {
            dbg!("tick - metadata");
            if let Some(Emit::Message(proto::Message::Data(_))) = metadata.next().await {
                // initializes content feed when metadata has data
                if self.content.is_none() {
                    let initial_metadata = {
                        let driver = self.hyperdrive.read().await;
                        let mut metadata = driver.metadata.write().await;
                        metadata.get(0).await
                    };
                    if let Ok(Some(initial_metadata)) = initial_metadata {
                        let content: crate::schema::Index =
                            protobuf::parse_from_bytes(&initial_metadata)?;
                        let public_key = hypercore::PublicKey::from_bytes(content.get_content())
                            .context("invalid content key stored in metadata")?;
                        self.hyperdrive
                            .write()
                            .await
                            .initialize_content_feed(public_key)?;
                        client.open(public_key.as_bytes().to_vec()).await?;
                    }
                }
            };
        }

        if let Some(ref mut content) = &mut self.content {
            dbg!(content.next().await);
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
