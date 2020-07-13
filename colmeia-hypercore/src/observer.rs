use async_trait::async_trait;
use futures::io::{AsyncRead, AsyncWrite};
use hypercore_protocol as proto;

// TODO macro?

#[async_trait]
pub trait MessageObserver {
    type Err;
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

    async fn loop_next(
        &mut self,
        _client: &mut proto::Protocol<S, S>,
    ) -> Result<(), Self::Err> {
        log::debug!("Looping");
        Ok(())
    }
}
