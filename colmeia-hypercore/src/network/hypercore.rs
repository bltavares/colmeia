use crate::observer::MessageObserver;
use anyhow::Context;
use async_std::sync::RwLock;
use hypercore_protocol as proto;
use std::sync::Arc;

pub struct PeeredHypercore<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    feed: Arc<RwLock<hypercore::Feed<Storage>>>,
    // handshake: SimpleDatHandshake,
    // remote_bitfield: hypercore::bitfield::Bitfield,
    remote_length: usize,
}

impl<Storage> PeeredHypercore<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    pub fn new(feed: Arc<RwLock<hypercore::Feed<Storage>>>) -> Self {
        Self {
            feed,
            // handshake: SimpleDatHandshake::default(),
            // remote_bitfield: hypercore::bitfield::Bitfield::default(),
            remote_length: 0,
        }
    }
}

#[async_trait::async_trait]
impl<Storage> MessageObserver for PeeredHypercore<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    type Err = anyhow::Error;

    async fn on_open(
        &mut self,
        channel: &mut proto::Channel,
        message: &proto::schema::Open,
    ) -> Result<(), Self::Err> {
        dbg!(message);

        let status = proto::schema::Status {
            downloading: Some(true), // TODO what to do here?
            uploading: Some(true),   // TODO what to do here?
        };
        channel.status(status).await?;

        let want = proto::schema::Want {
            start: 0,
            length: None,
        };
        channel.want(dbg!(want)).await?;

        Ok(())
    }

    // https://github.com/mafintosh/hypercore/blob/84990baa477491478f79b968123d2233eebeba76/lib/replicate.js#L109-L123
    async fn on_want(
        &mut self,
        client: &mut proto::Channel,
        message: &proto::schema::Want,
    ) -> Result<(), Self::Err> {
        dbg!("Received want");
        // We only reploy to multiple of 8192 in terms of offsets and lengths for want messages
        // since this is much easier for the bitfield, in terms of paging.
        if (message.start & 8191 != 0) || (message.length.unwrap_or(0) & 8191 != 0) {
            return Ok(());
        }

        let feed_length = self.feed.read().await.len() - 1;
        if self.feed.write().await.has(feed_length) {
            // Eagerly send the length of the feed to the otherside
            // TODO: only send this if the remote is not wanting a region
            // where this is contained in
            let have = proto::schema::Have {
                start: feed_length as u64,
                ..Default::default()
            };
            client.have(have).await?;
        }
        let rle = self
            .feed
            .read()
            .await
            .bitfield()
            .compress(message.start as usize, message.length.unwrap_or(0) as usize)?;
        let have = proto::schema::Have {
            start: message.start,
            length: message.length,
            bitfield: Some(rle),
            ack: None,
        };
        client.have(have).await?;
        Ok(())
    }

    // https://github.com/mafintosh/hypercore/blob/84990baa477491478f79b968123d2233eebeba76/lib/replicate.js#L295-L343
    async fn on_have(
        &mut self,
        client: &mut proto::Channel,
        message: &proto::schema::Have,
    ) -> Result<(), Self::Err> {
        dbg!("received have");
        // TODO implement setting the length and request data on metadata
        if let Some(ref bitfield) = message.bitfield {
            let buf = bitfield_rle::decode(bitfield)
                .map_err(|e| anyhow::anyhow!(e))
                .context("could not decode bitfield")?;
            let bits = buf.len() * 8;
            // TODO
            // Compare bitfields
            // remoteAndNotLocal(this.feed.bitfield, buf, this.remoteBitfield.littleEndian, have.start)
            // TODO fill
            // self.remoteBitfield.fill(buf, message.get_start());
            if bits > self.remote_length {
                // TODO last
                // self.remoteLength = self.remoteBitfield.last() + 1;
            }
        } else {
            let mut start = message.start as usize;
            let len = message.length.unwrap_or(1);
            for _ in 0..len {
                start += 1;
                // TODO feed.bitfield is private
                // self.remoteBitfield
                // .set(start, !self.feed.bitfield.get(start));
            }
            if start > self.remote_length {
                self.remote_length = start
            }
        }
        // .context("invalid bitfield");

        // SEND REQUEST
        // TODO loop on length
        // Use missing data from feed
        for index in 0..=message.start {
            let request = proto::schema::Request {
                index,
                ..Default::default()
            };
            client.request(request).await?;
        }
        Ok(())
    }

    async fn on_data(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Data,
    ) -> Result<(), Self::Err> {
        dbg!("received data");

        let proof = hypercore::Proof {
            index: message.index,
            nodes: message
                .nodes
                .iter()
                .map(|node| hypercore::Node::new(node.index, node.hash.to_vec(), node.size))
                .collect(),
            signature: message
                .signature
                .clone()
                .and_then(|s| hypercore::Signature::from_bytes(&s).ok()),
        };

        self.feed
            .write()
            .await
            .put(message.index, message.value.as_deref(), proof)
            .await
            .context("could not write data to feed")?;
        Ok(())
    }
}
