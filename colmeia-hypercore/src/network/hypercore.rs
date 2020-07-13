use async_std::sync::RwLock;
use futures::stream::StreamExt;
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
    channel: proto::Channel,
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
    pub fn new(channel: proto::Channel, feed: Arc<RwLock<hypercore::Feed<Storage>>>) -> Self {
        Self {
            feed,
            channel,
            // handshake: SimpleDatHandshake::default(),
            // remote_bitfield: hypercore::bitfield::Bitfield::default(),
            remote_length: 0,
        }
    }

    pub async fn loop_next(&mut self) -> Option<proto::Message> {
        self.channel.next().await
    }
}
