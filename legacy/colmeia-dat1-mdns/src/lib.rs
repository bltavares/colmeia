use futures::Stream;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use colmeia_dat1_core as core;

pub mod announce;
pub mod locate;
pub mod socket;

pub struct Mdns {
    dat_url: core::HashUrl,
    announce: Option<announce::Announcer>,
    locate: Option<locate::Locator>,
}

impl Mdns {
    pub fn new(dat_url: core::HashUrl) -> Self {
        Self {
            dat_url,
            announce: None,
            locate: None,
        }
    }

    pub fn with_location(&mut self, duration: Duration) -> &mut Self {
        self.locate = crate::socket::create_shared()
            .map_err(anyhow::Error::from)
            .and_then(|socket| locate::Locator::new(socket, self.dat_url.clone(), duration))
            .map_err(|err| log::debug!("Location err: {:?}", err))
            .ok();
        self
    }

    pub fn with_announcer(&mut self, port: u16) -> &mut Self {
        self.announce = crate::socket::create_shared()
            .map_err(anyhow::Error::from)
            .and_then(|socket| announce::Announcer::new(socket, self.dat_url.clone(), port))
            .map_err(|err| log::debug!("Announcer err: {:?}", err))
            .ok();
        self
    }
}

impl Stream for Mdns {
    type Item = SocketAddr;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;

        if let Some(ref mut announcer) = &mut self.announce {
            let _ = announcer.poll_next_unpin(cx);
        };

        if let Some(ref mut locate) = &mut self.locate {
            return locate.poll_next_unpin(cx);
        }

        Poll::Pending
    }
}
