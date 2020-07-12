use anyhow::Context;
use std::sync::{Arc, RwLock};

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

impl<Storage> Hyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    pub fn readable_content_feed(
        &mut self,
        public_key: hypercore::PublicKey,
    ) -> anyhow::Result<Arc<RwLock<hypercore::Feed<Storage>>>> {
        if let Some(storage) = self.content_storage.take() {
            let feed = hypercore::Feed::builder(public_key, storage)
                .build()
                .context("Could not start hypercore feed")?;

            self.content = Some(Arc::new(RwLock::new(feed)));
        }

        Ok(self.content.as_ref().context("No content to use")?.clone())
    }
}

pub fn in_memmory(
    public_key: hypercore::PublicKey,
) -> anyhow::Result<Hyperdrive<random_access_memory::RandomAccessMemory>> {
    let metadata = hypercore::Feed::builder(
        public_key,
        hypercore::Storage::new_memory().context("could not page feed memory")?,
    )
    .build()
    .context("Could not start feed")?;

    let content_storage =
        hypercore::Storage::new_memory().context("could not initialize the content storage")?;

    Ok(Hyperdrive {
        content_storage: Some(content_storage),
        content: None,
        metadata: Arc::new(RwLock::new(metadata)),
    })
}

pub fn in_disk<P: AsRef<std::path::PathBuf>>(
    public_key: hypercore::PublicKey,
    metadata: P,
    content: P,
) -> anyhow::Result<Hyperdrive<random_access_disk::RandomAccessDisk>> {
    let metadata = hypercore::Feed::builder(
        public_key,
        hypercore::Storage::new_disk(metadata.as_ref()).context("could not page feed memory")?,
    )
    .build()
    .context("Could not start feed")?;

    let content_storage = hypercore::Storage::new_disk(content.as_ref())
        .context("could not initialize the content storage")?;

    Ok(Hyperdrive {
        content_storage: Some(content_storage),
        content: None,
        metadata: Arc::new(RwLock::new(metadata)),
    })
}

// Content is another feed, with the pubkey derived from the metadata key
// https://github.com/mafintosh/hyperdrive/blob/e182fcbe6636a9c18fe4366b6ec3ce7c2f7f12a0/index.js#L955-L968
// const HYPERDRIVE: &[u8; 8] = b"hyperdri";
// fn derive_content_key(public_key: &hypercore::PublicKey) -> hypercore::PublicKey {
// What to do if blake2b and dalek don't have the KDF function?
// Only needed for creating metadata, as it needs the private key
// Readers only use what is provided on initialization.
// }
