use async_std::{sync::RwLock, task};
use futures::{channel::mpsc, FutureExt, SinkExt, Stream, StreamExt};
use hyperswarm_dht::QueryOpts;
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

pub struct Locator {
    topics: Arc<RwLock<HashSet<Vec<u8>>>>,
    _job: task::JoinHandle<()>,
    swarm: Arc<RwLock<hyperswarm_dht::HyperDht>>,
    receiver: mpsc::UnboundedReceiver<(Vec<u8>, SocketAddr)>,
}

impl Locator {
    pub async fn new(config: &Config) -> io::Result<Self> {
        let swarm = dht(config).await?;
        let swarm = Arc::new(RwLock::new(swarm));
        let topics: Arc<RwLock<HashSet<Vec<u8>>>> = Default::default();

        let (mut sender, receiver) = mpsc::unbounded();

        let _job = {
            let swarm = swarm.clone();
            let topics = topics.clone();
            task::spawn(async move {
                let mut timer = std::time::Instant::now();

                loop {
                    if let Some(Some(event)) = swarm.write().await.next().now_or_never() {
                        match event {
                            hyperswarm_dht::HyperDhtEvent::LookupResult { lookup, query_id } => {
                                let topic = lookup.topic.to_vec();
                                for peer in lookup.all_peers().copied() {
                                    let result = sender.send((topic.clone(), peer)).await;
                                    if let Err(e) = result {
                                        log::warn!("Could not send peer: {:?}", e);
                                    }
                                }

                                log::debug!("lookup {:?}", (lookup, query_id));
                            }
                            e => {
                                log::debug!("another event {:?}", e);
                            }
                        }
                    }

                    // TODO config
                    if timer.elapsed() > Duration::from_secs(10) {
                        log::debug!("Broadcasting new lookup of current topics");
                        for topic in topics.read().await.iter() {
                            if let Ok(query) = QueryOpts::try_from(&topic[..]) {
                                swarm.write().await.lookup(query);
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
            _job,
            receiver,
        })
    }

    pub async fn add_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        let query: QueryOpts = topic.try_into()?;
        self.swarm.write().await.lookup(query);
        self.topics.write().await.insert(topic.to_vec());
        Ok(())
    }

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
