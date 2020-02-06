use colmeia_dat_proto::*;

use crate::hypercore::PeeredHypercore;

pub struct PeeredHyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send,
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

// TODO make it generic if possible
impl PeeredHyperdrive<random_access_memory::RandomAccessMemory> {
    pub fn new(public_key: hypercore::PublicKey) -> Self {
        let metadata = PeeredHypercore::new(0, public_key);
        Self {
            metadata,
            content: None,
            delay_feed_content: None,
        }
    }

    pub fn initialize_content_feed(&mut self, public_key: hypercore::PublicKey) {
        if self.content.is_some() {
            panic!("double initialization of content")
        }

        self.content = Some(PeeredHypercore::new(1, public_key));
    }
}

#[async_trait]
impl<Storage> DatProtocolEvents for PeeredHyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send,
{
    async fn on_finish(&mut self, _client: &mut Client) {
        println!("Metadata audit: {:?}", self.metadata.audit());
        println!("Metadata len: {:?}", self.metadata.len());
        if let Some(ref mut content) = self.content {
            println!("Conent audit: {:?}", content.audit());
        }
    }
    async fn on_feed(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Feed,
    ) -> Option<()> {
        if channel == 0 {
            self.metadata.on_feed(client, channel, message).await?;
        } else if channel == 1 {
            self.delay_feed_content = Some(message.clone());
            eprintln!("Ignoring feed {:?} @ {:?}", message, channel);
        } else {
            // TOO MANY FEEDS
            return None;
        }

        Some(())
    }

    async fn on_handshake(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Handshake,
    ) -> Option<()> {
        if channel > 1 {
            // TODO implement content handshake
            return None;
        }

        self.metadata.on_handshake(client, channel, message).await?;

        Some(())
    }

    async fn on_have(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Have,
    ) -> Option<()> {
        if channel > 1 {
            // TODO implement content handshake
            return None;
        }
        self.metadata.on_have(client, channel, message).await?;
        Some(())
    }

    async fn on_data(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Data,
    ) -> Option<()> {
        if channel > 1 {
            // TODO implement content handshake
            return None;
        }
        self.metadata.on_data(client, channel, message).await?;
        Some(())
    }
}
