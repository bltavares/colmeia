use async_std::sync::RwLock;
use futures::Stream;
use std::{
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

pub struct Locator {
    topics: Arc<RwLock<Vec<u8>>>,
}

impl Locator {
    pub async fn add_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        Ok(())
    }

    pub async fn remove_topic(&self, topic: &[u8]) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Stream for Locator {
    type Item = (Vec<u8>, SocketAddr);

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}
