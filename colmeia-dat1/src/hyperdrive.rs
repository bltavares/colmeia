use std::sync::{Arc, RwLock};

use colmeia_dat1_proto::*;

use crate::hypercore::PeeredHypercore;

pub struct Hyperdrive<Storage>
where
    Storage:
        random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send + Sync,
{
    metadata: Arc<RwLock<hypercore::Feed<Storage>>>,
    content_storage: hypercore::Storage<Storage>,
}

pub fn in_memmory<Storage>(
    public_key: hypercore::PublicKey,
) -> Hyperdrive<
    impl random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send + Sync,
> {
    let metadata = hypercore::Feed::builder(
        public_key,
        hypercore::Storage::new_memory().expect("could not page feed memory"),
    )
    .build()
    .expect("Could not start feed");

    let content_storage =
        hypercore::Storage::new_memory().expect("could not initialize the content storage");

    Hyperdrive {
        content_storage,
        metadata: Arc::new(RwLock::new(metadata)),
    }
}

pub fn in_disk<P: AsRef<std::path::PathBuf>>(
    public_key: hypercore::PublicKey,
    metadata: P,
    content: P,
) -> Hyperdrive<
    impl random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send + Sync,
> {
    let metadata = hypercore::Feed::builder(
        public_key,
        hypercore::Storage::new_disk(metadata.as_ref()).expect("could not page feed memory"),
    )
    .build()
    .expect("Could not start feed");

    let content_storage = hypercore::Storage::new_disk(content.as_ref())
        .expect("could not initialize the content storage");

    Hyperdrive {
        content_storage,
        metadata: Arc::new(RwLock::new(metadata)),
    }
}

pub struct PeeredHyperdrive<Storage>
where
    Storage:
        random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send + Sync,
{
    metadata: PeeredHypercore<Storage>,
    content: Option<PeeredHypercore<Storage>>,
    delay_feed_content: Option<proto::Feed>,
}

// Content is another feed, with the pubkey derived from the metadata key
// https://github.com/mafintosh/hyperdrive/blob/e182fcbe6636a9c18fe4366b6ec3ce7c2f7f12a0/index.js#L955-L968
// const HYPERDRIVE: &[u8; 8] = b"hyperdri";
// fn derive_content_key(public_key: &hypercore::PublicKey) -> hypercore::PublicKey {
// What to do if blake2b and dalek don't have the KDF function?
// Only needed for creating metadata, as it needs the private key
// Readers only use what is provided on initialization.
// }

// // TODO make it generic if possible
impl PeeredHyperdrive<random_access_memory::RandomAccessMemory> {
    pub fn new(public_key: hypercore::PublicKey) -> Self {
        let feed = hypercore::Feed::builder(
            public_key,
            hypercore::Storage::new_memory().expect("could not page feed memory"),
        )
        .build()
        .expect("Could not start feed");
        let metadata = PeeredHypercore::new(0, Arc::new(RwLock::new(feed)));
        Self {
            metadata,
            content: None,
            delay_feed_content: None,
        }
    }

    pub fn initialize_content_feed(&mut self, public_key: hypercore::PublicKey) {
        if let None = self.content {
            let feed = hypercore::Feed::builder(
                public_key,
                hypercore::Storage::new_memory().expect("could not page feed memory"),
            )
            .build()
            .expect("Could not start feed");
            self.content = Some(PeeredHypercore::new(1, Arc::new(RwLock::new(feed))));
        }
    }
}

#[async_trait]
impl DatProtocolEvents for PeeredHyperdrive<random_access_memory::RandomAccessMemory> {
    async fn on_finish(&mut self, _client: &mut Client) {
        println!("Metadata audit: {:?}", self.metadata.audit());
        println!("Metadata len: {:?}", self.metadata.len());
        if let Some(ref mut content) = self.content {
            println!("Content audit: {:?}", content.audit());
            println!("Content len: {:?}", content.len());
        }
    }
    async fn on_feed(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Feed,
    ) -> Option<()> {
        match channel {
            0 => self.metadata.on_feed(client, channel, message).await?,
            1 => {
                if let Some(ref mut content) = self.content {
                    content.on_feed(client, channel, message).await;
                } else {
                    self.delay_feed_content = Some(message.clone());
                }
            }
            _ => return None,
        }

        Some(())
    }

    async fn on_handshake(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Handshake,
    ) -> Option<()> {
        match channel {
            // Only the first feed sends a handshake on hyperdrive v9
            0 => self.metadata.on_handshake(client, channel, message).await?,
            _ => return None,
        }

        Some(())
    }

    async fn on_have(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Have,
    ) -> Option<()> {
        match channel {
            0 => self.metadata.on_have(client, channel, message).await?,
            1 => {
                if let Some(ref mut content) = self.content {
                    content.on_have(client, channel, message).await;
                }
            }
            _ => return None,
        }
        Some(())
    }

    async fn on_data(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Data,
    ) -> Option<()> {
        match channel {
            0 => self.metadata.on_data(client, channel, message).await?,
            1 => {
                if let Some(ref mut content) = self.content {
                    content.on_data(client, channel, message).await;
                }
            }
            _ => return None,
        }

        if let None = self.content {
            if let Ok(Some(initial_metadata)) = self.metadata.get(0) {
                let content: crate::schema::Index =
                    protobuf::parse_from_bytes(&initial_metadata).ok()?;
                let public_key = hypercore::PublicKey::from_bytes(content.get_content())
                    .expect("invalid content key stored in metadata");
                self.initialize_content_feed(public_key);
                let delayed_message = self.delay_feed_content.take();
                if let Some(ref mut content) = self.content {
                    content
                        .on_feed(client, 1, &delayed_message.unwrap())
                        .await?;
                    // Simulate a handshake to keep going
                    content
                        .on_handshake(client, 1, &proto::Handshake::default())
                        .await?;
                }
            }
        }
        Some(())
    }
}
