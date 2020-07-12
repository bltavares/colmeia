use super::hypercore::PeeredHypercore;
use crate::{observer::ChannelObserver, Hyperdrive};
use std::sync::{Arc, RwLock};

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
                // delay_feed_content: None,
            });
        }
        Ok(Self {
            metadata,
            hyperdrive,
            content: None,
            // delay_feed_content: None,
        })
    }
}

impl<Storage> ChannelObserver for PeeredHyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    type Err = Box<dyn std::error::Error + Send + Sync>;
}
