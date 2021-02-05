use anyhow::Context;
use async_std::sync::RwLock;
use std::sync::Arc;

pub struct Hyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    pub metadata: Arc<RwLock<hypercore::Feed<Storage>>>,
    pub content: Option<Arc<RwLock<hypercore::Feed<Storage>>>>,
    content_storage: Option<hypercore::Storage<Storage>>,
}

impl<Storage> Hyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    pub async fn initialize_content_feed(
        &mut self,
        public_key: hypercore::PublicKey,
    ) -> anyhow::Result<Arc<RwLock<hypercore::Feed<Storage>>>> {
        if let Some(storage) = self.content_storage.take() {
            let feed = hypercore::Feed::builder(public_key, storage)
                .build()
                .await
                .context("Could not start hypercore feed")?;

            self.content = Some(Arc::new(RwLock::new(feed)));
        }

        Ok(self.content.as_ref().context("No content to use")?.clone())
    }
}

pub async fn in_memmory(
    public_key: hypercore::PublicKey,
) -> anyhow::Result<Hyperdrive<random_access_memory::RandomAccessMemory>> {
    let metadata = hypercore::Feed::builder(
        public_key,
        hypercore::Storage::new_memory()
            .await
            .context("could not page feed memory")?,
    )
    .build()
    .await
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
