use futures::{Stream, StreamExt};
use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

pub use announcer::Announcer;
pub use dht::Config;
pub use locator::Locator;

pub mod announcer;
pub mod dht;
pub mod locator;

pub struct DHTDiscovery {
    announce: Option<Announcer>,
    locate: Option<Locator>,
    config: Config,
}

// TODO Optmize to share swarm client across both announcer & locator
impl DHTDiscovery {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            announce: None,
            locate: None,
        }
    }

    pub async fn with_announcer(&mut self, port: u16, duration: Duration) -> &mut Self {
        let announcer = Announcer::listen(&self.config, duration, port).await;
        self.announce = announcer.ok();
        self
    }

    pub async fn with_locator(&mut self, duration: Duration) -> &mut Self {
        let locator = Locator::listen(&self.config, duration).await;
        self.locate = locator.ok();
        self
    }

    /// # Errors
    ///
    /// Will return `Err` it failed to register the topics due to concurrent writes
    pub async fn add_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        if let Some(announcer) = &self.announce {
            announcer.add_topic(topic).await?;
        }
        if let Some(locator) = &self.locate {
            locator.add_topic(topic).await?;
        }
        Ok(())
    }

    /// # Errors
    ///
    /// Will return `Err` it failed to remove the topics due to concurrent writes
    pub async fn remove_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        if let Some(announcer) = &self.announce {
            announcer.remove_topic(topic).await?;
        }
        if let Some(locator) = &self.locate {
            locator.remove_topic(topic).await?;
        }
        Ok(())
    }
}

impl Stream for DHTDiscovery {
    type Item = (Vec<u8>, SocketAddr);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if let Some(ref mut announcer) = &mut self.announce {
            let _announcer = announcer.poll_next_unpin(cx);
        };

        if let Some(ref mut locate) = &mut self.locate {
            return locate.poll_next_unpin(cx);
        }

        Poll::Pending
    }
}

impl Default for DHTDiscovery {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
