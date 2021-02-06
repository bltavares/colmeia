use async_std::{sync::RwLock, task};
use futures::{channel::mpsc, FutureExt, SinkExt, Stream, StreamExt};
use hyperswarm_dht::{HyperDht, QueryOpts};
use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
    io,
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use crate::dht::{dht, Config};

pub struct Announcer {
    swarm: Arc<RwLock<HyperDht>>,
    topics: Arc<RwLock<HashSet<Vec<u8>>>>,
    _listener_task: task::JoinHandle<()>,
    receiver: mpsc::UnboundedReceiver<(Vec<u8>, SocketAddr)>,
}

// TODO impl Drop unnanaounce
impl Announcer {
    pub async fn new(config: &Config) -> io::Result<Self> {
        let swarm = dht(config).await?;
        let swarm = Arc::new(RwLock::new(swarm));
        let topics: Arc<RwLock<HashSet<Vec<u8>>>> = Default::default();

        let (mut sender, receiver) = mpsc::unbounded();

        let _listener_task = {
            let swarm = swarm.clone();
            let topics = topics.clone();
            task::spawn(async move {
                let mut timer = std::time::Instant::now();

                loop {
                    if let Some(Some(event)) = swarm.write().await.next().now_or_never() {
                        match event {
                            hyperswarm_dht::HyperDhtEvent::AnnounceResult {
                                peers,
                                topic,
                                query_id,
                            } => {
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
                                log::debug!("announced {:?}", (peers, topic, query_id));
                            }
                            hyperswarm_dht::HyperDhtEvent::UnAnnounceResult {
                                peers,
                                topic,
                                query_id,
                            } => {
                                log::debug!("un-announced {:?}", (peers, topic, query_id));
                            }
                            e => {
                                log::debug!("another event {:?}", e);
                            }
                        }
                    }

                    // TODO config
                    if timer.elapsed() > Duration::from_secs(10) {
                        log::debug!("Broadcasting new announcement of current topics");
                        for topic in topics.read().await.iter() {
                            if let Ok(query) = QueryOpts::try_from(&topic[..]) {
                                // TODO config
                                let query = query.port(1000);
                                swarm.write().await.announce(query);
                            }
                        }
                        timer = Instant::now();
                    }
                }
            })
        };

        Ok(Self {
            topics,
            swarm,
            _listener_task,
            receiver,
        })
    }

    pub async fn add_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        let query: QueryOpts = topic.try_into()?;
        let query = query.port(1000); // TODO: port
        dbg!(self.swarm.write().await.announce(query)); // TODO: spawn await on io pool
        self.topics.write().await.insert(topic.to_vec());
        Ok(())
    }

    pub async fn remove_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        let query: QueryOpts = topic.try_into()?;
        let query = query.port(1000); // TODO: port
        self.swarm.write().await.unannounce(query); // TODO: spawn await on io pool
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
