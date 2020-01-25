use futures::Stream;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

pub mod announce;
pub mod crypto;
pub mod locate;
pub mod socket;

pub struct Mdns {
  dat_url: crypto::DatLocalDiscoverUrl,
  announce: Option<announce::Announcer>,
  locate: Option<locate::Locator>,
}

impl Mdns {
  pub fn new(dat_url: &str) -> Result<Self, parse_dat_url::Error> {
    let url_name = crypto::dat_url_mdns_discovery_name(&dat_url)?;
    Ok(Self {
      dat_url: url_name,
      announce: None,
      locate: None,
    })
  }

  pub fn with_location(&mut self, duration: Duration) -> &mut Self {
    self.locate = Some(locate::Locator::new(
      crate::socket::create_shared().expect("socket creation failed"),
      self.dat_url.clone(),
      duration,
    ));
    self
  }

  pub fn with_announcer(&mut self, port: u16) -> &mut Self {
    self.announce = Some(announce::Announcer::new(
      socket::create_shared().expect("socket creation failed"),
      self.dat_url.clone(),
      port,
    ));
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
