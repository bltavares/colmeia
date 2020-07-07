use anyhow::Context as ErrContext;
use futures::Stream;
use std::net::SocketAddr;
use std::pin::Pin;
use std::str::FromStr;
use std::task::{Context, Poll};
use std::time::Duration;
use trust_dns_proto::rr::Name;

static HYPERSWARM_DOMAIN: &str = ".hyperswarm.local";

pub mod announcer;
pub mod locator;
pub mod socket;

pub use announcer::Announcer;
pub use locator::Locator;

lazy_static::lazy_static! {
    static ref UNSPECIFIED_NAME: Name = Name::from_str("0.0.0.0").unwrap();
    static ref HYPERSWARM_REFERER: Name = Name::from_str("referrer.hyperswarm.local").unwrap();
}

/// {hash}.hyperswarm.local
fn domain_name(hash: &str) -> String {
    let mut domain = String::with_capacity(hash.len() + HYPERSWARM_DOMAIN.len());
    domain.push_str(hash);
    domain.push_str(HYPERSWARM_DOMAIN);
    domain
}

fn bytes_to_hash(discovery_key: &[u8]) -> String {
    hex::encode(discovery_key).chars().take(40).collect()
}

fn hash_to_domain(hash: &[u8]) -> String {
    let hash_as_str = bytes_to_hash(hash);
    domain_name(&hash_as_str)
}

fn hash_as_domain_name(hash: &[u8]) -> anyhow::Result<Name> {
    let hyperswarm_domain = crate::hash_to_domain(hash);
    log::debug!("hyperswarm domain: {:?}", hyperswarm_domain);
    Name::from_str(&hyperswarm_domain)
        .context("could not create hyperswarm dns name from provided hash")
}

type OwnedSelfId = [Box<[u8]>; 1];
type SelfId<'a> = &'a [Box<[u8]>];

pub fn self_id() -> String {
    use rand::Rng;
    let generated_id: [u8; 32] = rand::thread_rng().gen();
    format!("id={}", hex::encode(generated_id))
}

pub struct MdnsDiscovery {
    hypercore_topic: Vec<u8>,
    self_id: String,
    announce: Option<announcer::Announcer>,
    locate: Option<locator::Locator>,
}

impl MdnsDiscovery {
    pub fn new(hypercore_topic: Vec<u8>) -> Self {
        Self {
            hypercore_topic,
            self_id: self_id(),
            announce: None,
            locate: None,
        }
    }

    pub fn with_locator(&mut self, duration: Duration) -> &mut Self {
        self.locate = crate::socket::create()
            .map_err(anyhow::Error::from)
            .and_then(|socket| {
                locator::Locator::with_identifier(
                    socket,
                    &self.hypercore_topic,
                    self.self_id.clone(),
                    duration,
                )
            })
            .map_err(|err| log::debug!("Location err: {:?}", err))
            .ok();
        self
    }

    pub fn with_announcer(&mut self, port: u16) -> &mut Self {
        self.announce = crate::socket::create()
            .map_err(anyhow::Error::from)
            .and_then(|socket| {
                announcer::Announcer::with_identifier(
                    socket,
                    &self.hypercore_topic,
                    port,
                    self.self_id.clone(),
                )
            })
            .map_err(|err| log::debug!("Announcer err: {:?}", err))
            .ok();
        self
    }
}

impl Stream for MdnsDiscovery {
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
