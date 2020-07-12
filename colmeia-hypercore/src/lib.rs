use anyhow::Context;
use ed25519_dalek::PublicKey;
use futures::io::{AsyncRead, AsyncWrite};
use hypercore_protocol as proto;
use std::{
    io,
    pin::Pin,
    sync::{Arc, RwLock},
};

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

pub struct HypercoreService {
    stream: Pin<Box<dyn futures::stream::Stream<Item = proto::Message>>>,
}

impl HypercoreService {
    pub fn new<C>(
        client: proto::Protocol<C, C>,
        observer: impl ChannelObserver<Err = Box<dyn std::error::Error + Send + Sync>> + 'static + Send,
    ) -> Self
    where
        C: AsyncRead + AsyncWrite + Send + Unpin + Clone + 'static,
    {
        use futures::StreamExt;

        let channel_stream = futures::stream::unfold(client, |mut client| async {
            let event = client.loop_next().await;
            match event {
                Ok(proto::Event::Channel(channel)) => {
                    return Some((Some(channel), client));
                }
                Ok(_) => Some((None, client)),
                Err(_) => None,
            }
        })
        .filter_map(|i| async move { i });

        let message_stream =
            channel_stream.flat_map(|channel| channel.map(|message| (channel, message)));

        let stream = message_stream.filter_map(|(channel, message)| async move {
            dbg!(&message);
            match message {
                proto::Message::Open(open) => observer
                    .on_open(&mut channel, &open)
                    .await
                    .expect("could not open"),
                _ => {}
            };
            Some(message)
        });

        Self {
            stream: Box::pin(stream),
        }
    }
}

impl futures::stream::Stream for HypercoreService {
    type Item = proto::Message;
    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;
        self.stream.poll_next_unpin(cx)
    }
}
