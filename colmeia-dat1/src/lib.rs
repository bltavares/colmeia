pub use colmeia_dat_proto::*;

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

    async fn on_finish(&mut self, _client: &mut Client) {
        println!("Metadata audit: {:?}", self.metadata.audit());
        if let Some(ref mut content) = self.content {
            println!("Conent audit: {:?}", content.audit());
        }
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
}

struct PeeredHypercore<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send,
{
    feed: hypercore::Feed<Storage>,
    channel: u64,
    handshake: SimpleDatHandshake,
    remoteBitfield: hypercore::bitfield::Bitfield,
    remoteLength: usize,
}

impl<Storage> std::ops::Deref for PeeredHypercore<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send,
{
    type Target = hypercore::Feed<Storage>;
    fn deref(&self) -> &Self::Target {
        &self.feed
    }
}

impl<Storage> std::ops::DerefMut for PeeredHypercore<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.feed
    }
}

impl PeeredHypercore<random_access_memory::RandomAccessMemory> {
    fn new(channel: u64, public_key: hypercore::PublicKey) -> Self {
        let feed = hypercore::Feed::builder(
            public_key,
            hypercore::Storage::new_memory().expect("could not page feed memory"),
        )
        .build()
        .expect("Could not start feed");

        Self {
            feed,
            channel,
            handshake: SimpleDatHandshake::default(),
            remoteBitfield: hypercore::bitfield::Bitfield::default(),
            remoteLength: 0,
        }
    }
}

#[async_trait]
impl<Storage> DatProtocolEvents for PeeredHypercore<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = failure::Error> + std::fmt::Debug + Send,
{
    async fn on_handshake(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Handshake,
    ) -> Option<()> {
        if channel != self.channel {
            // Wrong channel in use
            return None;
        }
        self.handshake
            .on_handshake(client, channel, message)
            .await?;

        eprintln!("handshaken, sending want");
        let mut message = proto::Want::new();
        message.set_start(0);
        message.set_length(0); // must be in sizes of 8192 bytes
        client.want(channel, &message).await?;

        Some(())
    }

    // https://github.com/mafintosh/hypercore/blob/84990baa477491478f79b968123d2233eebeba76/lib/replicate.js#L109-L123
    async fn on_want(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Want,
    ) -> Option<()> {
        // We only reploy to multipla of 8192 in terms of offsets and lengths for want messages
        // since this is much easier for the bitfield, in terms of paging.
        if (message.get_start() & 8191 != 0) || (message.get_length() & 8191 != 0) {
            return Some(());
        }

        let feed_length = self.feed.len() - 1;
        if self.feed.has(feed_length) {
            // Eagerly send the length of the feed to the otherside
            // TODO: only send this if the remote is not wanting a region
            // where this is contained in
            let mut have = proto::Have::new();
            have.set_start(feed_length as u64);
            client.have(channel, &have).await?;
        }
        // TODO expose bitfield
        // TODO compress bitfield;
        let rle = vec![]; // self
                          //     .feed
                          //     .bitfield
                          //     .compress(message.get_start(), message.get_length());
        let mut have = proto::Have::new();
        have.set_start(message.get_start());
        have.set_length(message.get_length());
        have.set_bitfield(rle);
        client.have(channel, &have).await?;
        Some(())
    }

    // https://github.com/mafintosh/hypercore/blob/84990baa477491478f79b968123d2233eebeba76/lib/replicate.js#L295-L343
    async fn on_have(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Have,
    ) -> Option<()> {
        // TODO implement setting the length and request data on metadata
        if message.has_bitfield() {
            let buf = bitfield_rle::decode(message.get_bitfield()).ok()?;
            let bits = buf.len() * 8;
            // TODO
            // Compare bitfields
            // remoteAndNotLocal(this.feed.bitfield, buf, this.remoteBitfield.littleEndian, have.start)
            // TODO fill
            // self.remoteBitfield.fill(buf, message.get_start());
            if bits > self.remoteLength {
                // TODO last
                // self.remoteLength = self.remoteBitfield.last() + 1;
            }
        } else {
            let mut start = message.get_start() as usize;
            let len = if message.has_length() {
                message.get_length()
            } else {
                1
            };
            for _ in 0..len {
                start += 1;
                // TODO feed.bitfield is private
                // self.remoteBitfield
                // .set(start, !self.feed.bitfield.get(start));
            }
            if start > self.remoteLength {
                self.remoteLength = start
            }
        }
        // .expect("invalid bitfield");

        // SEND WANT
        Some(())
    }
}
