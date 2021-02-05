use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use announcer::Announcer;
use futures::{Stream, StreamExt};
use locator::Locator;

pub mod announcer;
pub mod locator;

struct DHTDiscovery {
    announce: Option<Announcer>,
    locate: Option<Locator>,
}

impl DHTDiscovery {
    pub fn new() -> Self {
        Self {
            announce: None,
            locate: None,
        }
    }

    pub fn with_locator(&mut self, duration: Duration) -> &mut Self {
        self
    }

    pub fn with_announcer(&mut self, port: u16) -> &mut Self {
        self
    }

    pub async fn add_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        if let Some(announcer) = &self.announce {
            announcer.add_topic(topic).await?;
        }
        if let Some(locator) = &self.locate {
            locator.add_topic(topic).await?;
        }
        Ok(())
    }

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
            let _ = announcer.poll_next_unpin(cx);
        };

        if let Some(ref mut locate) = &mut self.locate {
            return locate.poll_next_unpin(cx);
        }

        Poll::Pending
    }
}

impl Default for DHTDiscovery {
    fn default() -> Self {
        Self::new()
    }
}
