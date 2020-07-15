use anyhow::Context;
use async_std::{prelude::FutureExt, sync::RwLock, task};
use async_trait::async_trait;
use futures::{
    io::{AsyncRead, AsyncWrite},
    StreamExt,
};
use hypercore_protocol as proto;
use std::{pin::Pin, sync::Arc, time::Duration};

// TODO macro?

#[async_trait]
pub trait MessageObserver {
    type Err: std::fmt::Debug;

    async fn on_start(&mut self, _client: &mut proto::Channel) -> Result<(), Self::Err> {
        log::debug!("Starting");
        Ok(())
    }

    async fn on_finish(&mut self, _client: &mut proto::Channel) {
        log::debug!("on_finish");
    }

    async fn on_open(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Open,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_open {:?}", message);
        Ok(())
    }

    async fn on_options(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Options,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_options {:?}", message);
        Ok(())
    }

    async fn on_status(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Status,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_status {:?}", message);
        Ok(())
    }

    async fn on_have(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Have,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_have {:?}", message);
        Ok(())
    }

    async fn on_unhave(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Unhave,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_unhave {:?}", message);
        Ok(())
    }

    async fn on_want(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Want,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_want {:?}", message);
        Ok(())
    }

    async fn on_unwant(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Unwant,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_unwant {:?}", message);
        Ok(())
    }

    async fn on_request(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Request,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_request {:?}", message);
        Ok(())
    }

    async fn on_cancel(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Cancel,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_cancel {:?}", message);
        Ok(())
    }

    async fn on_data(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Data,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_data {:?}", message);
        Ok(())
    }

    async fn on_close(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Close,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_close {:?}", message);
        Ok(())
    }

    async fn on_extension(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::ExtensionMessage,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_extension {:?}", message);
        Ok(())
    }
}

pub fn sync_channel(
    mut channel: proto::Channel,
    mut observer: impl MessageObserver + Send + 'static,
) -> (
    crossbeam_channel::Receiver<proto::Message>,
    task::JoinHandle<anyhow::Result<()>>,
) {
    let (sx, rx) = crossbeam_channel::unbounded();
    let job = task::spawn(async move {
        if let Err(_) = observer.on_start(&mut channel).await {
            anyhow::bail!("failed on start");
        };

        while let Some(message) = channel.next().await {
            let result = match dbg!(message) {
                proto::Message::Open(message) => observer.on_open(&mut channel, &message).await,
                proto::Message::Options(message) => {
                    observer.on_options(&mut channel, &message).await
                }
                proto::Message::Status(message) => observer.on_status(&mut channel, &message).await,
                proto::Message::Have(message) => observer.on_have(&mut channel, &message).await,
                proto::Message::Unhave(message) => observer.on_unhave(&mut channel, &message).await,
                proto::Message::Want(message) => observer.on_want(&mut channel, &message).await,
                proto::Message::Unwant(message) => observer.on_unwant(&mut channel, &message).await,
                proto::Message::Request(message) => {
                    observer.on_request(&mut channel, &message).await
                }
                proto::Message::Cancel(message) => observer.on_cancel(&mut channel, &message).await,
                proto::Message::Data(message) => observer.on_data(&mut channel, &message).await,
                proto::Message::Close(message) => observer.on_close(&mut channel, &message).await,
                proto::Message::Extension(message) => {
                    observer.on_extension(&mut channel, &message).await
                }
            };

            if let Err(e) = result {
                log::error!("Failed when dealing with event loop {:?}", e);
                anyhow::bail!("Failed when dealing with an event");
            };
        }

        observer.on_finish(&mut channel).await;
        Ok(())
    });
    (rx, job)
}

#[async_trait]
pub trait EventObserver<S>
where
    S: AsyncRead + AsyncWrite + Send + Unpin + Clone + 'static,
{
    type Err: std::fmt::Debug + Send;

    async fn on_start(&mut self, _client: &mut proto::Protocol<S, S>) -> Result<(), Self::Err> {
        log::debug!("Starting");
        Ok(())
    }

    async fn on_finish(&mut self, _client: &mut proto::Protocol<S, S>) {
        log::debug!("on_finish");
    }

    async fn on_handshake(
        &mut self,
        _client: &mut proto::Protocol<S, S>,
        message: &[u8],
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_handshake {:?}", message);
        Ok(())
    }

    async fn on_discovery_key(
        &mut self,
        _client: &mut proto::Protocol<S, S>,
        message: &[u8],
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_discovery_key {:?}", message);
        Ok(())
    }

    async fn on_close(
        &mut self,
        _client: &mut proto::Protocol<S, S>,
        message: &[u8],
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_close {:?}", message);
        Ok(())
    }

    async fn on_channel(
        &mut self,
        _client: &mut proto::Protocol<S, S>,
        message: proto::Channel,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message on_channel {:?}", message);
        Ok(())
    }
}

pub fn stream<C>(
    mut client: proto::Protocol<C, C>,
    mut observer: impl EventObserver<C> + Send + 'static,
) -> (
    crossbeam_channel::Receiver<proto::Event>,
    task::JoinHandle<anyhow::Result<()>>,
)
where
    C: AsyncRead + AsyncWrite + Send + Unpin + Clone + 'static,
{
    let (sx, rx) = crossbeam_channel::unbounded();
    let job = task::spawn(async move {
        if let Err(_) = observer.on_start(&mut client).await {
            anyhow::bail!("Failed to start stream");
        }

        while let Ok(event) = client.loop_next().await {
            let result = match dbg!(&event) {
                proto::Event::Handshake(message) => {
                    observer.on_handshake(&mut client, &message).await
                }
                proto::Event::DiscoveryKey(message) => {
                    observer.on_discovery_key(&mut client, &message).await
                }
                proto::Event::Channel(message) => Ok(()),
                proto::Event::Close(ref message) => observer.on_close(&mut client, &message).await,
            };

            if let proto::Event::Channel(message) = event {
                observer.on_channel(&mut client, message).await;
            } else {
                sx.send(event);
            };

            if let Err(e) = result {
                anyhow::bail!("handling event event {:?} has failed", e);
            };
        }
        observer.on_finish(&mut client).await;
        Ok(())
    });

    (rx, job)
}
