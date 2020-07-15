use crate::Hyperdrive;
use anyhow::Context;
use async_std::{sync::RwLock, task};
use futures::{
    io::{AsyncRead, AsyncWrite},
    FutureExt, SinkExt, StreamExt,
};
use hypercore_protocol as proto;
use std::sync::Arc;

#[derive(Debug)]
pub enum Emit {
    OnData,
}

pub fn sync_channel<Storage>(
    mut channel: proto::Channel,
    feed: Arc<RwLock<hypercore::Feed<Storage>>>,
    mut rx: impl futures::Sink<Emit> + Unpin + Send + 'static,
) -> task::JoinHandle<anyhow::Result<()>>
where
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
{
    task::spawn(async move {
        let mut remote_length = 0;

        while let Some(message) = channel.next().await {
            match dbg!(message) {
                proto::Message::Open(_) => {
                    let status = proto::schema::Status {
                        downloading: Some(true),
                        uploading: Some(false), // TODO what to do here
                    };
                    channel.status(dbg!(status)).await?;

                    let want = proto::schema::Want {
                        start: 0,
                        length: None, // must be in sizes of 8192 bytes
                    };
                    channel.want(dbg!(want)).await?;
                }
                proto::Message::Data(message) => {
                    dbg!("received data");

                    let proof = hypercore::Proof {
                        index: message.index,
                        nodes: message
                            .nodes
                            .iter()
                            .map(|node| {
                                hypercore::Node::new(node.index, node.hash.to_vec(), node.size)
                            })
                            .collect(),
                        signature: message
                            .signature
                            .clone()
                            .and_then(|s| hypercore::Signature::from_bytes(&s).ok()),
                    };

                    feed.write()
                        .await
                        .put(message.index, message.value.as_deref(), proof)
                        .await
                        .context("could not write data to feed")?;

                    // Let hypercore know we have data
                    // So it can operate once again and try to open content feed
                    rx.send(Emit::OnData)
                        .await
                        .map_err(|_| anyhow::anyhow!("failed to emit ondata"))?;
                }
                proto::Message::Have(message) => {
                    dbg!("received have");
                    // TODO implement setting the length and request data on metadata
                    if let Some(ref bitfield) = message.bitfield {
                        let buf = bitfield_rle::decode(bitfield)
                            .map_err(|e| anyhow::anyhow!(e))
                            .context("could not decode bitfield")?;
                        let bits = buf.len() * 8;
                        // TODO
                        // Compare bitfields
                        // remoteAndNotLocal(this.feed.bitfield, buf, this.remoteBitfield.littleEndian, have.start)
                        // TODO fill
                        // self.remoteBitfield.fill(buf, message.get_start());
                        if bits > remote_length {
                            // TODO last
                            // self.remoteLength = self.remoteBitfield.last() + 1;
                        }
                    } else {
                        let mut start = message.start as usize;
                        let len = message.length.unwrap_or(1);
                        for _ in 0..len {
                            start += 1;
                            // TODO feed.bitfield is private
                            // self.remoteBitfield
                            // .set(start, !self.feed.bitfield.get(start));
                        }
                        if start > remote_length {
                            remote_length = start
                        }
                    };
                    // .context("invalid bitfield");

                    // SEND REQUEST
                    // TODO loop on length
                    // Use missing data from feed
                    for index in 0..=message.start {
                        let request = proto::schema::Request {
                            index,
                            ..Default::default()
                        };
                        dbg!(channel.request(dbg!(request)).await?);
                    }
                }
                proto::Message::Want(message) => {
                    dbg!("Received want");
                    // We only reploy to multiple of 8192 in terms of offsets and lengths for want messages
                    // since this is much easier for the bitfield, in terms of paging.
                    if (message.start & 8191 != 0) || (message.length.unwrap_or(0) & 8191 != 0) {
                        return Ok(());
                    }

                    let feed_length = feed.read().await.len() - 1;
                    if feed.write().await.has(feed_length) {
                        // Eagerly send the length of the feed to the otherside
                        // TODO: only send this if the remote is not wanting a region
                        // where this is contained in
                        let have = proto::schema::Have {
                            start: feed_length as u64,
                            ..Default::default()
                        };
                        channel.have(have).await?;
                    }
                    let rle = feed
                        .read()
                        .await
                        .bitfield()
                        .compress(message.start as usize, message.length.unwrap_or(0) as usize)?;
                    let have = proto::schema::Have {
                        start: message.start,
                        length: message.length,
                        bitfield: Some(rle),
                        ack: None,
                    };
                    channel.have(have).await?;
                }
                event => {
                    log::debug!("received event {:?}", event);
                }
            };
        }

        Ok(())
    })
}

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
pub fn sync_hyperdrive<C, Storage>(
    mut client: proto::Protocol<C, C>,
    hyperdrive: Hyperdrive<Storage>,
) -> task::JoinHandle<()>
where
    C: AsyncRead + AsyncWrite + Send + Unpin + Clone + 'static,
    Storage: random_access_storage::RandomAccess<Error = Box<dyn std::error::Error + Send + Sync>>
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
{
    let hyperdrive = Arc::new(RwLock::new(hyperdrive));
    task::spawn(async move {
        let (metadata_sx, mut metadata_rx) = futures::channel::mpsc::unbounded();
        let mut metadata_job = Some(JobHolder::Sender(metadata_sx));

        let (content_sx, mut content_rx) = futures::channel::mpsc::unbounded();
        let mut content_job = Some(JobHolder::Sender(content_sx));

        loop {
            dbg!("start loop");
            let (event, _, _) = futures::future::select_all(vec![
                client.loop_next().map(HyperdriveEvents::Client).boxed(),
                metadata_rx
                    .select_next_some()
                    .map(HyperdriveEvents::Metadata)
                    .boxed(),
                content_rx
                    .select_next_some()
                    .map(HyperdriveEvents::Content)
                    .boxed(),
            ])
            .await;

            match dbg!(event) {
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
                    // TODO don't rely on channel event oerder - check the keys to see if they match
                    if let Some(JobHolder::Sender(sender)) = metadata_job.take() {
                        log::debug!("initializing metadata feed");
                        let feed = hyperdrive.read().await.metadata.clone();
                        metadata_job = Some(JobHolder::Job(sync_channel(channel, feed, sender)));
                        continue;
                    }
                    if let Some(JobHolder::Sender(sender)) = content_job.take() {
                        log::debug!("initializing content feed");
                        let feed = hyperdrive.read().await.content.clone();
                        if let Some(feed) = feed {
                            content_job = Some(JobHolder::Job(sync_channel(channel, feed, sender)));
                        }
                        continue;
                    }
                }
                HyperdriveEvents::Client(Err(_)) => {
                    let driver = hyperdrive.read().await;
                    let mut metadata = driver.metadata.write().await;
                    log::debug!("Metadata audit: {:?}", metadata.audit().await);
                    log::debug!("Metadata len: {:?}", metadata.len());

                    if let Some(ref content) = driver.content {
                        let mut content = content.write().await;
                        log::debug!("Content audit: {:?}", content.audit().await);
                        log::debug!("Content len: {:?}", content.len());
                    };
                    break;
                }
                // TODO listen to metadata ondata evnt to initialize content feed
                HyperdriveEvents::Metadata(_) => {
                    if let Some(JobHolder::Sender(_)) = &content_job {
                        dbg!("metadata on data");

                        // Initialize the content feed if we have no job started
                        let initial_metadata = {
                            let driver = hyperdrive.read().await;
                            let mut metadata = driver.metadata.write().await;
                            metadata.get(0).await
                        };
                        if let Ok(Some(initial_metadata)) = initial_metadata {
                            let content = match protobuf::parse_from_bytes::<crate::schema::Index>(
                                &initial_metadata,
                            ) {
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
                                .initialize_content_feed(dbg!(public_key))
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
            dbg!("here");
        }
    })
}
