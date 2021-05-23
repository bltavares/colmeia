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

use crate::dht::{BroadcastChannel, Outbound};

pub struct Announcer {
    channel: BroadcastChannel,
    topics: Arc<RwLock<HashSet<Vec<u8>>>>,
    port: u16,
    receiver: mpsc::UnboundedReceiver<(Vec<u8>, SocketAddr)>,
}

// TODO impl Drop unnanaounce (async drop)
impl Announcer {
    pub fn listen_to_channel(channel: BroadcastChannel, duration: Duration, port: u16) -> Self {
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
                            crate::dht::Inbound::Announce { peers, topic } => {
                                let topic = topic.to_vec();
                                for peer in peers
                                    .iter()
                                    .flat_map(|z| z.peers.iter().copied())
                                    .chain(peers.iter().flat_map(|z| z.local_peers.iter().copied()))
                                {
                                    let result = sender.send((topic.clone(), peer)).await;
                                    if let Err(e) = result {
                                        log::warn!("Could not send peer: {:?}", e);
                                    }
                                }
                                log::debug!("announced {:?}", (peers, topic));
                            }
                            crate::dht::Inbound::UnAnnounce { peers, topic } => {
                                log::debug!("un-announced {:?}", (peers, topic));
                            }
                            e => {
                                log::debug!("another event {:?}", e);
                            }
                        }
                    }

                    if timer.elapsed() > duration {
                        log::debug!("Broadcasting new announcement of current topics");
                        for topic in topics.read().await.iter() {
                            if let Ok(query) = QueryOpts::try_from(&topic[..]) {
                                let query = query.port(u32::from(port));
                                let result = send.broadcast(Outbound::Announce(query)).await;
                                log::trace!("broadcast {:?}", result);
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
            port,
            receiver,
        }
    }

    /// # Errors
    ///
    /// Will return `Err` it failed to register the topics due to concurrent writes
    pub async fn add_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        let query: QueryOpts = topic.try_into()?;
        let query = query.port(u32::from(self.port));
        self.channel
            .send
            .broadcast(Outbound::Announce(query))
            .await?;
        self.topics.write().await.insert(topic.to_vec());
        Ok(())
    }

    /// # Errors
    ///
    /// Will return `Err` it failed to remove the topics due to concurrent writes
    pub async fn remove_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        let query: QueryOpts = topic.try_into()?;
        let query = query.port(u32::from(self.port));
        self.channel
            .send
            .broadcast(Outbound::UnAnnounce(query))
            .await?;
        self.topics.write().await.remove(topic);
        Ok(())
    }
}

impl Stream for Announcer {
    type Item = (Vec<u8>, SocketAddr);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.receiver.poll_next_unpin(cx)
    }
}
