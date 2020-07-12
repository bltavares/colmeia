use anyhow::Context;
use async_std::{future::IntoFuture, stream::IntoStream, task};
use ed25519_dalek::PublicKey;
use futures::{
    io::{AsyncRead, AsyncWrite},
    StreamExt,
};
use std::{
    collections::HashMap,
    io,
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
};

use hypercore_protocol as proto;

mod network;
mod observer;

pub use network::hyperdrive::PeeredHyperdrive;
use observer::ChannelObserver;

/// Returns a successfully handshaken stream
pub async fn initiate<S>(public_key: &PublicKey, tcp_stream: S) -> io::Result<proto::Protocol<S, S>>
where
    S: AsyncRead + AsyncWrite + Send + Unpin + Clone + 'static,
{
    let mut stream = proto::ProtocolBuilder::initiator().connect(tcp_stream);

    if let Ok(proto::Event::Handshake(_)) = stream.loop_next().await {
        stream.open(public_key.as_bytes().to_vec()).await?;
    }

    return Ok(stream);
}
pub struct Hyperdrive<Storage>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync,
{
    pub(crate) metadata: Arc<RwLock<hypercore::Feed<Storage>>>,
    pub(crate) content: Option<Arc<RwLock<hypercore::Feed<Storage>>>>,
    content_storage: Option<hypercore::Storage<Storage>>,
}

pub async fn in_memmory(
    public_key: PublicKey,
) -> anyhow::Result<Hyperdrive<random_access_memory::RandomAccessMemory>> {
    let metadata = hypercore::Feed::builder(
        public_key,
        hypercore::Storage::new_memory()
            .await
            .context("could not page feed memory")?,
    )
    .build()
    .context("Could not start feed")?;

    let content_storage = hypercore::Storage::new_memory()
        .await
        .context("could not initialize the content storage")?;

    Ok(Hyperdrive {
        content_storage: Some(content_storage),
        content: None,
        metadata: Arc::new(RwLock::new(metadata)),
    })
}
#[derive(Debug)]
pub enum Item {
    Event(proto::Event),
    Message(proto::Message),
}

pub struct HypercoreService {
    stream: Pin<Box<dyn futures::Stream<Item = Item>>>,
}

impl HypercoreService {
    pub fn stream<C>(client: proto::Protocol<C, C>) -> Self
    where
        C: AsyncRead + AsyncWrite + Send + Unpin + Clone + 'static,
    {
        let queue = Arc::new(async_std::sync::RwLock::new(
            futures::stream::FuturesUnordered::new(),
        ));

        let rw_queue = queue.clone();

        let event_loop =
            futures::stream::unfold((client, rw_queue), |(mut client, rw_queue)| async {
                match client.loop_next().await {
                    Ok(event) => {
                        dbg!(&event);
                        let event = match dbg!(event) {
                            proto::Event::Channel(channel) => {
                                rw_queue.read().await.push(channel.into_future());
                                None
                            }
                            event => Some(event),
                        };
                        Some((event, (client, rw_queue)))
                    }
                    Err(_) => None,
                }
            })
            .filter_map(|i| async { i })
            .map(Item::Event);

        let channel_loop = futures::stream::unfold(queue, |queue| async move {
            let result = queue.write().await.next().await;
            if let Some((Some(message), stream)) = result {
                queue.read().await.push(stream.into_future());
                Some((Some(message), queue))
            } else {
                Some((None, queue))
            }
        })
        .filter_map(|i| async { i })
        .map(Item::Message);

        // TODO why does merge does not work?!
        let stream = async_std::stream::StreamExt::merge(channel_loop, event_loop);

        HypercoreService {
            stream: Box::pin(stream),
        }
    }
}

impl futures::Stream for HypercoreService {
    type Item = Item;
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }
}
