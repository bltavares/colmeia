use anyhow::Context as context;
use async_std::net::TcpStream;
use async_std::stream::StreamExt;
use futures::io::{AsyncWriteExt, BufReader, BufWriter};
use futures::stream;
use rand::Rng;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};
use std::time::Duration;

pub use async_trait::async_trait;
pub use protobuf::Message;
pub use simple_message_channels::{Message as ChannelMessage, Reader, Writer};

use colmeia_dat1_core::HashUrl;

mod cipher;
pub mod schema;
mod socket;
use crate::cipher::Cipher;
pub use crate::schema as proto;

#[non_exhaustive]
#[derive(Debug)]
pub enum DatMessage {
    Feed(proto::Feed),
    Handshake(proto::Handshake),
    Info(proto::Info),
    Have(proto::Have),
    Unhave(proto::Unhave),
    Want(proto::Want),
    Unwant(proto::Unwant),
    Request(proto::Request),
    Cancel(proto::Cancel),
    Data(proto::Data),
}

type ParseResult = Result<DatMessage, protobuf::ProtobufError>;

pub trait MessageExt {
    fn parse(&self) -> ParseResult;
}

impl MessageExt for ChannelMessage {
    fn parse(&self) -> ParseResult {
        match self.typ {
            0 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Feed),
            1 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Handshake),
            2 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Info),
            3 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Have),
            4 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Unhave),
            5 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Want),
            6 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Unwant),
            7 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Request),
            8 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Cancel),
            9 => protobuf::parse_from_bytes(&self.message).map(DatMessage::Data),
            _ => panic!("Uknonw message"), // TODO proper error handling
        }
    }
}

pub struct Client {
    reader: Reader<BufReader<socket::CloneableStream>>,
    writer: Writer<BufWriter<socket::CloneableStream>>,
    pub(crate) writer_socket: socket::CloneableStream,
    first_message: Option<proto::Feed>,
}

impl Client {
    pub fn reader(&mut self) -> &mut Reader<BufReader<socket::CloneableStream>> {
        &mut self.reader
    }

    pub fn writer(&mut self) -> &mut Writer<BufWriter<socket::CloneableStream>> {
        &mut self.writer
    }

    pub async fn have(&mut self, channel: u64, message: &proto::Have) -> anyhow::Result<()> {
        self.writer()
            .send(ChannelMessage::new(
                channel,
                3,
                message.write_to_bytes().context("not a valid have")?,
            ))
            .await
            .context("could not write have")
    }

    pub async fn want(&mut self, channel: u64, message: &proto::Want) -> anyhow::Result<()> {
        self.writer()
            .send(ChannelMessage::new(
                channel,
                5,
                message.write_to_bytes().context("not a valid want")?,
            ))
            .await
            .context("could not write want")
    }

    pub async fn request(&mut self, channel: u64, message: &proto::Request) -> anyhow::Result<()> {
        self.writer()
            .send(ChannelMessage::new(
                channel,
                7,
                message.write_to_bytes().context("not a valid request")?,
            ))
            .await
            .context("could not write request")
    }
}

// TODO macro?
#[async_trait]
pub trait DatProtocolEvents {
    type Err;
    async fn on_start(&mut self, _client: &mut Client) -> Result<(), Self::Err> {
        log::debug!("Starting");
        Ok(())
    }

    async fn on_finish(&mut self, _client: &mut Client) {
        log::debug!("on_finish");
    }

    async fn on_feed(
        &mut self,
        _client: &mut Client,
        channel: u64,
        message: &proto::Feed,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}: {:?}", channel, message);
        Ok(())
    }

    async fn on_handshake(
        &mut self,
        _client: &mut Client,
        channel: u64,
        message: &proto::Handshake,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}: {:?}", channel, message);
        Ok(())
    }

    async fn on_info(
        &mut self,
        _client: &mut Client,
        channel: u64,
        message: &proto::Info,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}: {:?}", channel, message);
        Ok(())
    }

    async fn on_have(
        &mut self,
        _client: &mut Client,
        channel: u64,
        message: &proto::Have,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}: {:?}", channel, message);
        Ok(())
    }

    async fn on_unhave(
        &mut self,
        _client: &mut Client,
        channel: u64,
        message: &proto::Unhave,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}: {:?}", channel, message);
        Ok(())
    }

    async fn on_want(
        &mut self,
        _client: &mut Client,
        channel: u64,
        message: &proto::Want,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}: {:?}", channel, message);
        Ok(())
    }

    async fn on_unwant(
        &mut self,
        _client: &mut Client,
        channel: u64,
        message: &proto::Unwant,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}: {:?}", channel, message);
        Ok(())
    }

    async fn on_request(
        &mut self,
        _client: &mut Client,
        channel: u64,
        message: &proto::Request,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}: {:?}", channel, message);
        Ok(())
    }

    async fn on_cancel(
        &mut self,
        _client: &mut Client,
        channel: u64,
        message: &proto::Cancel,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}: {:?}", channel, message);
        Ok(())
    }

    async fn on_data(
        &mut self,
        _client: &mut Client,
        channel: u64,
        message: &proto::Data,
    ) -> Result<(), Self::Err> {
        log::debug!("Received message {:?}: {:?}", channel, message);
        Ok(())
    }
}

// async-trait?
pub async fn ping(client: &mut Client) -> std::io::Result<usize> {
    client.writer_socket.write(&[0u8]).await
}

pub struct ClientInitialization {
    bare_reader: Reader<socket::CloneableStream>,
    bare_writer: Writer<socket::CloneableStream>,
    upgradable_reader: socket::CloneableStream,
    upgradable_writer: socket::CloneableStream,
    dat_key: HashUrl,
    writer_socket: socket::CloneableStream,
}

impl ClientInitialization {
    pub fn dat_key(&self) -> &HashUrl {
        &self.dat_key
    }
}

pub async fn new_client(dat_key: HashUrl, tcp_stream: TcpStream) -> ClientInitialization {
    let socket = Arc::new(tcp_stream);

    let reader_cipher = Arc::new(RwLock::new(Cipher::new(
        dat_key.public_key().as_bytes().to_vec(),
    )));
    let reader_socket = socket::CloneableStream {
        socket: socket.clone(),
        cipher: reader_cipher,
        buffer: None,
    };
    let upgradable_reader = reader_socket.clone();
    let bare_reader = Reader::new(reader_socket);

    let writer_cipher = Arc::new(RwLock::new(Cipher::new(
        dat_key.public_key().as_bytes().to_vec(),
    )));
    let writer_socket = socket::CloneableStream {
        socket,
        cipher: writer_cipher,
        buffer: None,
    };
    let upgradable_writer = writer_socket.clone();
    let bare_writer = Writer::new(writer_socket.clone());

    ClientInitialization {
        bare_reader,
        bare_writer,
        upgradable_reader,
        upgradable_writer,
        dat_key,
        writer_socket,
    }
}

pub async fn handshake(mut init: ClientInitialization) -> anyhow::Result<Client> {
    log::debug!("Bulding nonce to start connection");
    let nonce: [u8; 24] = rand::thread_rng().gen();
    let nonce = nonce.to_vec();
    let mut feed = proto::Feed::new();
    feed.set_discoveryKey(init.dat_key.discovery_key().to_vec());
    feed.set_nonce(nonce);
    async_std::io::timeout(
        Duration::from_secs(1),
        init.bare_writer.send(ChannelMessage::new(
            0,
            0,
            feed.write_to_bytes().context("invalid feed message")?,
        )),
    )
    .await
    .context("could not send initial feed")?;

    log::debug!("Sent a nonce, upgrading write socket");
    init.upgradable_writer.upgrade(feed.get_nonce())?;
    let writer = Writer::new(BufWriter::new(init.upgradable_writer));

    log::debug!("Preparing to read feed nonce");
    let received_feed =
        async_std::future::timeout(Duration::from_secs(1), init.bare_reader.next()).await?;
    let received_feed = match received_feed {
        Some(received_feed) => received_feed,
        _ => return Err(anyhow::anyhow!("Timed out on initial feed")),
    };
    let parsed_feed = received_feed?.parse()?;
    let payload = match parsed_feed {
        DatMessage::Feed(payload) => payload,
        _ => return Err(anyhow::anyhow!("Invalid initial message")),
    };

    log::debug!("Dat feed received {:?}", payload);
    if payload.get_discoveryKey() != init.dat_key.discovery_key() && !payload.has_nonce() {
        return Err(anyhow::anyhow!(
            "Wrong discovery key and no nonce on first message"
        ));
    }
    log::debug!("Feed received, upgrading read socket");
    init.upgradable_reader.upgrade(payload.get_nonce())?;
    let reader = Reader::new(BufReader::new(init.upgradable_reader));

    log::debug!("Handshake finished");

    Ok(Client {
        reader,
        writer,
        writer_socket: init.writer_socket,
        first_message: Some(payload),
    })
}

pub struct DatService {
    stream: Pin<Box<dyn stream::Stream<Item = ChannelMessage> + Send>>,
}

async fn should_finish<O, S, R: std::fmt::Debug>(
    client: &mut Client,
    observer: &mut O,
    result: Result<S, R>,
) -> Result<S, R>
where
    O: DatProtocolEvents + Send + 'static,
{
    if let Err(e) = result.as_ref() {
        log::debug!("Errors: {:?}", e);
        observer.on_finish(client).await;
    }
    result
}

// TODO: Integrate timeout and pings
impl DatService {
    pub fn new<O>(client: Client, observer: O) -> Self
    where
        O: DatProtocolEvents + Send + 'static,
        <O as DatProtocolEvents>::Err: std::fmt::Debug + Send + Sync,
    {
        let stream = stream::unfold(
            (client, observer, true),
            |(mut client, mut observer, first)| async move {
                if first {
                    let result = observer.on_start(&mut client).await;
                    should_finish(&mut client, &mut observer, result)
                        .await
                        .ok()?;
                    if let Some(message) = client.first_message.take() {
                        let result = observer.on_feed(&mut client, 0, &message).await;
                        should_finish(&mut client, &mut observer, result)
                            .await
                            .ok()?;
                    }
                };

                log::debug!("READING from socket");
                // TODO timeout configurable
                let response =
                    async_std::future::timeout(Duration::from_secs(5), client.reader().next())
                        .await;

                let response = match response {
                    Ok(response) => response,
                    Err(err) => {
                        log::debug!(
                            "Connection timed out during read {:?}. stopping stream",
                            err
                        );
                        observer.on_finish(&mut client).await;
                        return None;
                    }
                };

                let response = should_finish(&mut client, &mut observer, response?).await;
                match &response {
                    Err(err) => {
                        log::debug!("Error {:?}. stopping stream", err);
                        observer.on_finish(&mut client).await;
                        return None;
                    }
                    Ok(message) => match message.parse() {
                        Ok(DatMessage::Feed(m)) => {
                            let result = observer.on_feed(&mut client, message.channel, &m).await;
                            should_finish(&mut client, &mut observer, result)
                                .await
                                .ok()?;
                        }
                        Ok(DatMessage::Handshake(m)) => {
                            let result = observer
                                .on_handshake(&mut client, message.channel, &m)
                                .await;
                            should_finish(&mut client, &mut observer, result)
                                .await
                                .ok()?;
                        }
                        Ok(DatMessage::Info(m)) => {
                            let result = observer.on_info(&mut client, message.channel, &m).await;
                            should_finish(&mut client, &mut observer, result)
                                .await
                                .ok()?;
                        }
                        Ok(DatMessage::Have(m)) => {
                            let result = observer.on_have(&mut client, message.channel, &m).await;
                            should_finish(&mut client, &mut observer, result)
                                .await
                                .ok()?;
                        }
                        Ok(DatMessage::Unhave(m)) => {
                            let result = observer.on_unhave(&mut client, message.channel, &m).await;
                            should_finish(&mut client, &mut observer, result)
                                .await
                                .ok()?;
                        }
                        Ok(DatMessage::Want(m)) => {
                            let result = observer.on_want(&mut client, message.channel, &m).await;
                            should_finish(&mut client, &mut observer, result)
                                .await
                                .ok()?;
                        }
                        Ok(DatMessage::Unwant(m)) => {
                            let result = observer.on_unwant(&mut client, message.channel, &m).await;
                            should_finish(&mut client, &mut observer, result)
                                .await
                                .ok()?;
                        }
                        Ok(DatMessage::Request(m)) => {
                            let result =
                                observer.on_request(&mut client, message.channel, &m).await;
                            should_finish(&mut client, &mut observer, result)
                                .await
                                .ok()?;
                        }
                        Ok(DatMessage::Cancel(m)) => {
                            let result = observer.on_cancel(&mut client, message.channel, &m).await;
                            should_finish(&mut client, &mut observer, result)
                                .await
                                .ok()?;
                        }
                        Ok(DatMessage::Data(m)) => {
                            let result = observer.on_data(&mut client, message.channel, &m).await;
                            should_finish(&mut client, &mut observer, result)
                                .await
                                .ok()?;
                        }
                        Err(err) => log::debug!("Dropping message {:?} err {:?}", message, err),
                    },
                };

                Some((response.ok()?, (client, observer, false)))
            },
        );

        Self {
            stream: Box::pin(stream),
        }
    }
}

impl stream::Stream for DatService {
    type Item = ChannelMessage;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;

        self.stream.poll_next_unpin(cx)
    }
}

pub struct SimpleDatHandshake {
    id: [u8; 32],
    feeds: HashMap<u64, proto::Feed>,
    handshakes: HashMap<u64, proto::Handshake>,
}

impl SimpleDatHandshake {
    pub fn new() -> Self {
        let id: [u8; 32] = rand::thread_rng().gen();
        Self {
            id,
            feeds: HashMap::new(),
            handshakes: HashMap::new(),
        }
    }

    pub fn handshakes(&self) -> &HashMap<u64, proto::Handshake> {
        &self.handshakes
    }

    pub fn feeds(&self) -> &HashMap<u64, proto::Feed> {
        &self.feeds
    }
}

impl Default for SimpleDatHandshake {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatProtocolEvents for SimpleDatHandshake {
    type Err = anyhow::Error;

    async fn on_feed(
        &mut self,
        client: &mut Client,
        channel: u64,
        message: &proto::Feed,
    ) -> Result<(), Self::Err> {
        log::debug!("Feed received {:?} {:?}", channel, message);

        // Initial channel is sent as part of the protocol negotiation.
        // We must initialize feeds from there on
        if channel > 0 {
            client
                .writer()
                .send(ChannelMessage::new(
                    channel,
                    0,
                    message
                        .write_to_bytes()
                        .context("unable to re-encode received feed")?,
                ))
                .await?;
        }

        self.feeds.insert(channel, message.clone());
        log::debug!("Preparing to send encrypted handshake");
        let mut handshake = proto::Handshake::new();
        handshake.set_id(self.id.to_vec());
        handshake.set_live(true);
        handshake.set_ack(false);
        log::debug!("Dat handshake to send {:?}", &handshake);

        client
            .writer()
            .send(ChannelMessage::new(
                channel,
                1,
                handshake
                    .write_to_bytes()
                    .context("invalid handshake message")?,
            ))
            .await
            .context("connection closed")?;

        Ok(())
    }

    async fn on_handshake(
        &mut self,
        _client: &mut Client,
        channel: u64,
        message: &proto::Handshake,
    ) -> Result<(), Self::Err> {
        log::debug!(
            "Preparing to read encrypted handshake, {:?} {:?}",
            channel,
            message
        );
        if message.get_id() == self.id {
            return Err(anyhow::anyhow!("Connected to ourselves. disconnect"));
        }
        self.handshakes.insert(channel, message.clone());
        Ok(())
    }
}
