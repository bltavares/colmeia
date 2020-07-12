use std::sync::{Arc, RwLock};

pub struct PeeredHypercore<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    feed: Arc<RwLock<hypercore::Feed<Storage>>>,
    channel: u64,
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
    pub fn new(channel: u64, feed: Arc<RwLock<hypercore::Feed<Storage>>>) -> Self {
        Self {
            feed,
            channel,
            // handshake: SimpleDatHandshake::default(),
            // remote_bitfield: hypercore::bitfield::Bitfield::default(),
            remote_length: 0,
        }
    }
}