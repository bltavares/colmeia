use async_trait::async_trait;
use futures::{
    io::{AsyncRead, AsyncWrite},
    StreamExt,
};
use hypercore_protocol as proto;
use std::{pin::Pin, task};

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
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_options(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Options,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_status(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Status,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_have(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Have,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_unhave(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Unhave,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_want(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Want,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_unwant(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Unwant,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_request(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Request,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_cancel(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Cancel,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_data(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Data,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_close(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::Close,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_extension(
        &mut self,
        _client: &mut proto::Channel,
        message: &proto::schema::ExtensionMessage,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }
}

pub struct MessageDriver {
    stream: Pin<Box<dyn futures::Stream<Item = proto::Message> + Send + 'static>>,
}

impl MessageDriver {
    pub fn stream(
        channel: proto::Channel,
        observer: impl MessageObserver + Send + 'static,
    ) -> Self {
        // TODO Convert to a stream instead of a future with a loop
        let stream = futures::stream::unfold(
            (channel, observer),
            |(mut channel, mut observer)| async move {
                let event = match channel.next().await {
                    Some(e) => e,
                    None => return Some((None, (channel, observer))),
                };

                let result = match &event {
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
                    return None;
                };

                Some((Some(event), (channel, observer)))
            },
        )
        .filter_map(|i| async { i });

        Self {
            stream: Box::pin(stream),
        }
    }
}

impl futures::Stream for MessageDriver {
    type Item = proto::Message;
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

    async fn on_handshake(
        &mut self,
        _client: &mut proto::Protocol<S, S>,
        message: &[u8],
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_discovery_key(
        &mut self,
        _client: &mut proto::Protocol<S, S>,
        message: &[u8],
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_close(
        &mut self,
        _client: &mut proto::Protocol<S, S>,
        message: &[u8],
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn on_channel(
        &mut self,
        _client: &mut proto::Protocol<S, S>,
        message: proto::Channel,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}", message);
        Ok(())
    }

    async fn loop_next(&mut self, _client: &mut proto::Protocol<S, S>) -> Result<(), Self::Err> {
        log::debug!("Looping");
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
            (client, observer),
            |(mut client, mut observer)| async move {
                let event = match client.loop_next().await {
                    Ok(e) => e,
                    Err(_) => return None,
                };

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
                    return None;
                };

                let _ = match observer.loop_next(&mut client).await {
                    Ok(e) => e,
                    Err(_) => return None,
                };

                Some(((), (client, observer)))
            },
        );

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
