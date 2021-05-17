use crate::hyperdrive::Hyperdrive;
use async_std::{sync::RwLock, task};
use colmeia_hypercore::{Emit, PeeredFeed};
use futures::{
    io::{AsyncRead, AsyncWrite},
    FutureExt, StreamExt,
};
use hypercore_protocol as proto;
use std::sync::Arc;

#[derive(Debug)]
enum HyperdriveEvents {
    Client(std::io::Result<proto::Event>),
    Metadata(Emit),
    Content(Emit),
}

enum JobHolder {
    Sender(futures::channel::mpsc::UnboundedSender<Emit>),
    Job(async_std::task::JoinHandle<anyhow::Result<()>>),
}

pub async fn replicate_hyperdrive<C, Storage>(
    mut client: proto::Protocol<C, C>,
    hyperdrive: Arc<RwLock<Hyperdrive<Storage>>>,
) where
    C: AsyncRead + AsyncWrite + Send + Unpin + Clone + 'static,
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
{
    let (metadata_sender, mut metadata_receiver) = futures::channel::mpsc::unbounded();
    let mut metadata_job = Some(JobHolder::Sender(metadata_sender));

    let (content_sender, mut content_receiver) = futures::channel::mpsc::unbounded();
    let mut content_job = Some(JobHolder::Sender(content_sender));

    loop {
        let (event, _, _) = futures::future::select_all(vec![
            client.loop_next().map(HyperdriveEvents::Client).boxed(),
            metadata_receiver
                .select_next_some()
                .map(HyperdriveEvents::Metadata)
                .boxed(),
            content_receiver
                .select_next_some()
                .map(HyperdriveEvents::Content)
                .boxed(),
        ])
        .await;

        match event {
            HyperdriveEvents::Client(Ok(proto::Event::DiscoveryKey(message))) => {
                if let Some(JobHolder::Sender(_)) = &metadata_job {
                    let drive = hyperdrive.read().await;
                    let metadata = drive.metadata.read().await;
                    let public_key_for_metadata = metadata.public_key().as_bytes();
                    let metadata_discovery_key =
                        hypercore_protocol::discovery_key(public_key_for_metadata);
                    if message == metadata_discovery_key.as_slice()
                        && client.open(public_key_for_metadata.to_vec()).await.is_err()
                    {
                        log::error!("failed to open metadata data exchange");
                        break;
                    };
                }
            }
            HyperdriveEvents::Client(Ok(proto::Event::Channel(channel))) => {
                // TODO don't rely on channel event order - check the keys to see if they match
                if let Some(JobHolder::Sender(sender)) = metadata_job.take() {
                    log::debug!("initializing metadata feed");
                    let feed = hyperdrive.read().await.metadata.clone();
                    metadata_job = Some(JobHolder::Job(task::spawn(async {
                        let mut peer = PeeredFeed {
                            channel,
                            feed,
                            remote_length: 0,
                        };
                        peer.replicate(sender).await
                    })));
                    continue;
                }
                if let Some(JobHolder::Sender(sender)) = content_job.take() {
                    log::debug!("initializing content feed");
                    let feed = hyperdrive.read().await.content.clone();
                    if let Some(feed) = feed {
                        content_job = Some(JobHolder::Job(task::spawn(async {
                            let mut peer = PeeredFeed {
                                channel,
                                feed,
                                remote_length: 0,
                            };
                            peer.replicate(sender).await
                        })));
                    }
                    continue;
                }
            }
            HyperdriveEvents::Client(Err(_)) => {
                break;
            }
            // TODO listen to metadata ondata event to initialize content feed
            HyperdriveEvents::Metadata(_) => {
                if let Some(JobHolder::Sender(_)) = &content_job {
                    // Initialize the content feed if we have no job started
                    let initial_metadata = {
                        let driver = hyperdrive.read().await;
                        let mut metadata = driver.metadata.write().await;
                        metadata.get(0).await
                    };
                    if let Ok(Some(initial_metadata)) = initial_metadata {
                        let content: crate::schema::Index =
                            match protobuf::Message::parse_from_bytes(&initial_metadata) {
                                Ok(e) => e,
                                _ => continue,
                            };

                        let public_key =
                            match hypercore::PublicKey::from_bytes(content.get_content()) {
                                Ok(e) => e,
                                _ => {
                                    log::error!(
                                        "feed content first entry is not a valid public key",
                                    );
                                    break;
                                }
                            };

                        if hyperdrive
                            .write()
                            .await
                            .initialize_content_feed(public_key)
                            .await
                            .is_ok()
                            && client.open(public_key.as_bytes().to_vec()).await.is_err()
                        {
                            log::error!("failed to open metadata data exchange");
                            break;
                        };
                    }
                }
            }
            _ => {}
        };
    }
}
