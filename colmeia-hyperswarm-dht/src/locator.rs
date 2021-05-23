use async_std::{sync::RwLock, task};
use futures::{channel::mpsc, SinkExt, Stream, StreamExt};
use hyperswarm_dht::QueryOpts;
use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use crate::dht::{BroadcastChannel, Inbound, Outbound};

pub struct Locator {
    topics: Arc<RwLock<HashSet<Vec<u8>>>>,
    channel: BroadcastChannel,
    receiver: mpsc::UnboundedReceiver<(Vec<u8>, SocketAddr)>,
}

impl Locator {
    /// # Errors
    ///
    /// Will return `Err` if it failed to bind to the socket on config
    pub fn listen_to_channel(channel: BroadcastChannel, duration: Duration) -> Self {
        let topics: Arc<RwLock<HashSet<Vec<u8>>>> = Arc::default();

        let (mut sender, receiver) = mpsc::unbounded();

        let _job = {
            let topics = topics.clone();
            let mut messages = channel.receive.activate_cloned();
            let send = channel.send.clone();
            task::spawn(async move {
                let mut timer = std::time::Instant::now();

                loop {
                    if let Ok(event) = messages.try_recv() {
                        match event {
                            Inbound::Lookup(lookup) => {
                                let topic = lookup.topic.to_vec();
                                for peer in lookup.all_peers().copied() {
                                    let result = sender.send((topic.clone(), peer)).await;
                                    if let Err(e) = result {
                                        log::warn!("Could not send peer: {:?}", e);
                                    }
                                }
                                log::debug!("lookup {:?}", lookup);
                            }
                            e => {
                                log::debug!("another event {:?}", e);
                            }
                        }
                    }

                    if timer.elapsed() > duration {
                        log::debug!("Broadcasting new lookup of current topics");
                        for topic in topics.read().await.iter() {
                            if let Ok(query) = QueryOpts::try_from(&topic[..]) {
                                let result = send.broadcast(Outbound::Lookup(query)).await;
                                log::trace!("broadcast result {:?}", result);
                            }
                        }
                        timer = Instant::now();
                    }
                }
            })
        };

        Self {
            topics,
            channel,
            receiver,
        }
    }

    /// # Errors
    ///
    /// Will return `Err` it failed to register the topics due to concurrent writes
    pub async fn add_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        let query: QueryOpts = topic.try_into()?;
        self.channel.send.broadcast(Outbound::Lookup(query)).await?;
        self.topics.write().await.insert(topic.to_vec());
        Ok(())
    }

    /// # Errors
    ///
    /// Will return `Err` it failed to remove the topics due to concurrent writes
    pub async fn remove_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        self.topics.write().await.remove(topic);
        Ok(())
    }
}

impl Stream for Locator {
    type Item = (Vec<u8>, SocketAddr);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.receiver.poll_next_unpin(cx)
    }
}
