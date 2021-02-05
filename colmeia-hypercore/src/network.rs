use anyhow::Context;
use async_std::sync::RwLock;
use futures::{SinkExt, StreamExt};
use hypercore_protocol as proto;
use std::io;
use std::sync::Arc;

#[derive(Debug)]
pub enum Emit {
    OnData,
}

pub struct PeeredFeed<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
{
    pub channel: proto::Channel,
    pub feed: Arc<RwLock<hypercore::Feed<Storage>>>,
    pub remote_length: usize,
}

impl<Storage> PeeredFeed<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
{
    async fn on_open(&mut self) -> io::Result<()> {
        let status = proto::schema::Status {
            downloading: Some(true),
            uploading: Some(false), // TODO what to do here
        };
        self.channel.status(status).await?;

        let want = proto::schema::Want {
            start: 0,
            length: None, // must be in sizes of 8192 bytes
        };
        self.channel.want(want).await?;
        Ok(())
    }

    async fn on_data(&mut self, message: hypercore_protocol::schema::Data) -> anyhow::Result<()> {
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

    async fn on_have(&mut self, message: hypercore_protocol::schema::Have) -> anyhow::Result<()> {
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
        };
        // .context("invalid bitfield");

        // SEND REQUEST
        // TODO loop on length
        // Use missing data from feed
        for index in 0..=message.start {
            let request = proto::schema::Request {
                index,
                ..Default::default()
            };
            self.channel.request(request).await?;
        }
        Ok(())
    }

    async fn on_want(&mut self, message: hypercore_protocol::schema::Want) -> anyhow::Result<()> {
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
            self.channel.have(have).await?;
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
        self.channel.have(have).await?;
        Ok(())
    }

    pub async fn replicate(
        &mut self,
        mut rx: impl futures::Sink<Emit> + Unpin + Send + 'static,
    ) -> anyhow::Result<()> {
        while let Some(message) = self.channel.next().await {
            match message {
                proto::Message::Open(_) => {
                    self.on_open().await?;
                }
                proto::Message::Data(message) => {
                    self.on_data(message).await?;
                    // Let hypercore know we have data
                    // So it can operate once again and try to open content feed
                    rx.send(Emit::OnData)
                        .await
                        .map_err(|_| anyhow::anyhow!("failed to emit ondata"))?;
                }
                proto::Message::Have(message) => {
                    self.on_have(message).await?;
                }
                proto::Message::Want(message) => {
                    self.on_want(message).await?;
                }
                event => {
                    log::debug!("received event {:?}", event);
                }
            };
        }
        Ok(())
    }
}
