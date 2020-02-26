use anyhow::Context;
use std::sync::{Arc, RwLock};

use colmeia_dat1_proto::*;

use crate::hyperdrive::Hyperdrive;
use crate::network::hypercore::PeeredHypercore;

pub struct PeeredHyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    metadata: PeeredHypercore<Storage>,
    hyperdrive: Arc<RwLock<Hyperdrive<Storage>>>,
    content: Option<PeeredHypercore<Storage>>,
    delay_feed_content: Option<proto::Feed>,
}

impl<Storage> PeeredHyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    pub fn new(hyperdrive: Arc<RwLock<Hyperdrive<Storage>>>) -> anyhow::Result<Self> {
        let metadata = PeeredHypercore::new(
            0,
            hyperdrive
                .read()
                .map_err(|_| anyhow::anyhow!("Could not aquire metadata lock"))?
                .metadata
                .clone(),
        );
        let content = hyperdrive
            .read()
            .map_err(|_| anyhow::anyhow!("Could not aquire hyperdrive lock"))?
            .content
            .clone();
        if let Some(content) = content {
            return Ok(Self {
                metadata,
                hyperdrive,
                content: Some(PeeredHypercore::new(1, content)),
                delay_feed_content: None,
            });
        }
        Ok(Self {
            metadata,
            hyperdrive,
            content: None,
            delay_feed_content: None,
        })
    }

    pub fn initialize_content_feed(
        &mut self,
        public_key: hypercore::PublicKey,
    ) -> anyhow::Result<()> {
        if let None = self.content {
            let feed = self
                .hyperdrive
                .write()
                .map_err(|_| anyhow::anyhow!("Could not aquire hyperdrive lock"))?
                .readable_content_feed(public_key)?;
            self.content = Some(PeeredHypercore::new(1, feed));
        }
        Ok(())
    }
}

#[async_trait]
impl<Storage> DatProtocolEvents for PeeredHyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    type Err = anyhow::Error;

    async fn on_finish(&mut self, _client: &mut Client) {
        if let Ok(ref mut metadata) = self.metadata.write() {
            log::debug!("Metadata audit: {:?}", metadata.audit());
            log::debug!("Metadata len: {:?}", metadata.len());
        }

        if let Some(Ok(ref mut content)) = self.content.as_ref().map(|c| c.write()) {
            log::debug!("Content audit: {:?}", content.audit());
            log::debug!("Content len: {:?}", content.len());
        }
    }

    async fn on_feed(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Feed,
    ) -> Result<(), Self::Err> {
        match channel {
            0 => self.metadata.on_feed(client, channel, message).await?,
            1 => {
                if let Some(ref mut content) = self.content {
                    content.on_feed(client, channel, message).await?;
                } else {
                    self.delay_feed_content = Some(message.clone());
                }
            }
            _ => return Err(anyhow::anyhow!("Too many channels")),
        }

        Ok(())
    }

    async fn on_handshake(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Handshake,
    ) -> Result<(), Self::Err> {
        match channel {
            // Only the first feed sends a handshake on hyperdrive v9
            0 => self.metadata.on_handshake(client, channel, message).await?,
            _ => return Err(anyhow::anyhow!("Too many handshakes")),
        }

        Ok(())
    }

    async fn on_have(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Have,
    ) -> Result<(), Self::Err> {
        match channel {
            0 => self.metadata.on_have(client, channel, message).await?,
            1 => {
                if let Some(ref mut content) = self.content {
                    content.on_have(client, channel, message).await?;
                }
            }
            _ => return Err(anyhow::anyhow!("Too many channels")),
        }
        Ok(())
    }

    async fn on_data(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Data,
    ) -> Result<(), Self::Err> {
        match channel {
            0 => self.metadata.on_data(client, channel, message).await?,
            1 => {
                if let Some(ref mut content) = self.content {
                    content.on_data(client, channel, message).await?;
                }
            }
            _ => return Err(anyhow::anyhow!("Too many channels")),
        }

        if let None = self.content {
            let initial_metadata = self
                .metadata
                .write()
                .map_err(|_| anyhow::anyhow!("Could not aquire hyperdrive lock"))?
                .get(0);
            if let Ok(Some(initial_metadata)) = initial_metadata {
                let content: crate::schema::Index = protobuf::parse_from_bytes(&initial_metadata)?;
                let public_key = hypercore::PublicKey::from_bytes(content.get_content())
                    .context("invalid content key stored in metadata")?;
                self.initialize_content_feed(public_key)?;
                let delayed_message = self.delay_feed_content.take();
                if let Some(delayed_message) = delayed_message {
                    if let Some(ref mut content) = self.content {
                        content.on_feed(client, 1, &delayed_message).await?;
                        // Simulate a handshake to keep going
                        content
                            .on_handshake(client, 1, &proto::Handshake::default())
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }
}
