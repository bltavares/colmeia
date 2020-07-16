use async_std::prelude::FutureExt;
use async_trait::async_trait;
use futures::{
    io::{AsyncRead, AsyncWrite},
    StreamExt,
};
use hypercore_protocol as proto;
use std::{pin::Pin, task, time::Duration};

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

pub struct MessageDriver {
    stream: Pin<Box<dyn futures::Stream<Item = Emit> + Send + 'static>>,
}

#[derive(Debug)]
pub enum Emit {
    Timeout,
    OperationError,
    Message(proto::Message),
}

impl MessageDriver {
    pub fn stream(
        channel: proto::Channel,
        observer: impl MessageObserver + Send + 'static,
    ) -> Self {
        let stream = futures::stream::unfold(
            (channel, observer, 0, true),
            |(mut channel, mut observer, error_count, first_loop)| async move {
                if first_loop {
                    // drop stream on initialization error
                    observer.on_start(&mut channel).await.ok()?;
                }

                if error_count > 1 {
                    observer.on_finish(&mut channel).await;
                    return None;
                }

                let message = match channel.next().timeout(Duration::from_micros(165)).await {
                    Ok(Some(e)) => e,
                    _ => return Some((Emit::Timeout, (channel, observer, error_count, false))),
                };

                let result = match dbg!(&message) {
                    proto::Message::Open(message) => observer.on_open(&mut channel, &message).await,
                    proto::Message::Options(message) => {
                        observer.on_options(&mut channel, &message).await
                    }
                    proto::Message::Status(message) => {
                        observer.on_status(&mut channel, &message).await
                    }
                    proto::Message::Have(message) => observer.on_have(&mut channel, &message).await,
                    proto::Message::Unhave(message) => {
                        observer.on_unhave(&mut channel, &message).await
                    }
                    proto::Message::Want(message) => observer.on_want(&mut channel, &message).await,
                    proto::Message::Unwant(message) => {
                        observer.on_unwant(&mut channel, &message).await
                    }
                    proto::Message::Request(message) => {
                        observer.on_request(&mut channel, &message).await
                    }
                    proto::Message::Cancel(message) => {
                        observer.on_cancel(&mut channel, &message).await
                    }
                    proto::Message::Data(message) => observer.on_data(&mut channel, &message).await,
                    proto::Message::Close(message) => {
                        observer.on_close(&mut channel, &message).await
                    }
                    proto::Message::Extension(message) => {
                        observer.on_extension(&mut channel, &message).await
                    }
                };

                if let Err(e) = result {
                    log::error!("Failed when dealing with event loop {:?}", e);
                    return Some((
                        Emit::OperationError,
                        (channel, observer, error_count + 1, false),
                    ));
                };

                Some((Emit::Message(message), (channel, observer, 0, false)))
            },
        )
        .fuse();

        Self {
            stream: Box::pin(stream),
        }
    }
}

impl futures::Stream for MessageDriver {
    type Item = Emit;
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }
}

#[async_trait]
pub trait EventObserver<S>
where
    S: AsyncRead + AsyncWrite + Send + Unpin + Clone + 'static,
{
    type Err: std::fmt::Debug;

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

    async fn tick(&mut self, _client: &mut proto::Protocol<S, S>) -> Result<(), Self::Err> {
        log::debug!("Tick");
        Ok(())
    }
}

pub struct EventDriver {
    stream: Pin<Box<dyn futures::Stream<Item = ()>>>,
}

impl EventDriver {
    pub fn stream<C>(
        client: proto::Protocol<C, C>,
        observer: impl EventObserver<C> + Send + 'static,
    ) -> Self
    where
        C: AsyncRead + AsyncWrite + Send + Unpin + Clone + 'static,
    {
        // TODO Convert to a stream instead of a future with a loop
        let stream = futures::stream::unfold(
            (client, observer, 0, true),
            |(mut client, mut observer, error_count, first_loop)| async move {
                if first_loop {
                    // drop stream on initialization error
                    observer.on_start(&mut client).await.ok()?;
                }

                if error_count > 0 {
                    observer.on_finish(&mut client).await;
                    log::debug!("Error count reached maximum penalty. Bailing.");
                    return None;
                }

                let error_count = match observer.tick(&mut client).await {
                    Ok(_) => 0,
                    Err(_) => error_count + 1,
                };

                // It seems like the channel only receives messages if loop_next is called
                // is that why it is so slow to receive messages?
                // Interrupt the loop frequently to allow progress on other channels to happen
                // Concern: finding the interrupt time correctly, as it could likely lead to data loss when moving the
                // control message from the internal buffer to close the channel
                let event = match client.handle_next().await {
                    Ok(Some(e)) => e,
                    Ok(None) => return Some(((), (client, observer, error_count, false))),
                    Err(_) => return Some(((), (client, observer, error_count + 1, false))),
                };

                dbg!("next");
                let result = match dbg!(event) {
                    proto::Event::Handshake(message) => {
                        observer.on_handshake(&mut client, &message).await
                    }
                    proto::Event::DiscoveryKey(message) => {
                        observer.on_discovery_key(&mut client, &message).await
                    }
                    proto::Event::Channel(message) => {
                        observer.on_channel(&mut client, message).await
                    }
                    proto::Event::Close(message) => observer.on_close(&mut client, &message).await,
                };

                if let Err(e) = result {
                    log::error!("Failed when dealing with event loop {:?}", e);
                    return Some(((), (client, observer, error_count + 1, false)));
                };

                Some(((), (client, observer, 0, false)))
            },
        )
        .fuse();

        Self {
            stream: Box::pin(stream),
        }
    }
}

impl futures::Stream for EventDriver {
    type Item = ();
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }
}
