pub use colmeia_dat_proto::*;

pub struct HypercoreV7Client<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send,
{
    metadata: hypercore::Feed<Storage>,
    content: Option<hypercore::Feed<Storage>>,
    handshake: SimpleDatHandshake,
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
impl HypercoreV7Client<random_access_memory::RandomAccessMemory> {
    pub fn new(public_key: hypercore::PublicKey) -> Self {
        let metadata = hypercore::Feed::builder(
            public_key,
            hypercore::Storage::new_memory().expect("could not page metadata memory"),
        )
        .build()
        .expect("Could not start metadata");
        Self {
            metadata,
            content: None,
            handshake: SimpleDatHandshake::new(),
        }
    }

    pub fn initialize_content_feed(&mut self, public_key: hypercore::PublicKey) {
        if self.content.is_some() {
            panic!("double initialization of metadata")
        }

        self.content = Some(
            hypercore::Feed::builder(
                public_key,
                hypercore::Storage::new_memory().expect("could not page content memory"),
            )
            .build()
            .expect("Could not start content"),
        )
    }
}

#[async_trait]
impl<Storage> DatObserver for HypercoreV7Client<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send,
{
    async fn on_feed(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Feed,
    ) -> Option<()> {
        if channel > 2 {
            return None;
        }

        self.handshake.on_feed(client, channel, message).await?;
        Some(())
    }

    async fn on_handshake(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Handshake,
    ) -> Option<()> {
        if channel > 2 {
            return None;
        }

        self.handshake
            .on_handshake(client, channel, message)
            .await?;

        if channel == 1 && self.content.is_none() {
            // TODO not read, must initalize content feed from metadata content
            return Some(());
        }

        eprintln!("handshaken, sending want");
        let mut message = proto::Want::new();
        message.set_start(0);
        message.set_length(0); // must be in sizes of 8192 bytes
        client
            .writer()
            .send(ChannelMessage::new(
                channel,
                5,
                message.write_to_bytes().expect("malformated want message"),
            ))
            .await
            .ok()?;
        Some(())
    }

    async fn on_have(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Have,
    ) -> Option<()> {
        // TODO implement setting the length and request data on metadata
        if message.has_bitfield() {
            eprintln!(
                "{:?} @ {:?}",
                channel,
                bitfield_rle::decode(message.get_bitfield())
            );
        } else {
            eprintln!("start")
        }
        // .expect("invalid bitfield");
        Some(())
    }

    async fn on_finish(&mut self, _client: &mut Client) {
        println!("Metadata audit: {:?}", self.metadata.audit());
        if let Some(ref mut content) = self.content {
            println!("Conent audit: {:?}", content.audit());
        }
    }
}
