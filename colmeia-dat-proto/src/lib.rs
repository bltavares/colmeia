use async_std::net::TcpStream;
use async_std::stream::StreamExt;
pub use async_trait::async_trait;
use colmeia_dat_core::DatUrlResolution;
use futures::io::{AsyncWriteExt, BufReader, BufWriter};
use futures::stream;
pub use protobuf::Message;
use rand::Rng;
pub use simple_message_channels::{Message as ChannelMessage, Reader, Writer};
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

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
}

impl Client {
    pub fn reader(&mut self) -> &mut Reader<BufReader<socket::CloneableStream>> {
        &mut self.reader
    }

    pub fn writer(&mut self) -> &mut Writer<BufWriter<socket::CloneableStream>> {
        &mut self.writer
    }
}

// TODO macro?
// TODO async-trait?
#[async_trait]
pub trait DatObserver {
    async fn on_start(&mut self, _client: &mut Client) -> Option<()> {
        log::debug!("Starting");
        Some(())
    }

    async fn on_stop(&mut self, _client: &mut Client) -> Option<()> {
        log::debug!("Starting");
        Some(())
    }

    async fn on_feed(&mut self, _client: &mut Client, message: &proto::Feed) -> Option<()> {
        log::debug!("Received message {:?}", message);
        Some(())
    }

    async fn on_handshake(
        &mut self,
        _client: &mut Client,
        message: &proto::Handshake,
    ) -> Option<()> {
        log::debug!("Received message {:?}", message);
        Some(())
    }

    async fn on_info(&mut self, _client: &mut Client, message: &proto::Info) -> Option<()> {
        log::debug!("Received message {:?}", message);
        Some(())
    }

    async fn on_have(&mut self, _client: &mut Client, message: &proto::Have) -> Option<()> {
        log::debug!("Received message {:?}", message);
        Some(())
    }

    async fn on_unhave(&mut self, _client: &mut Client, message: &proto::Unhave) -> Option<()> {
        log::debug!("Received message {:?}", message);
        Some(())
    }

    async fn on_want(&mut self, _client: &mut Client, message: &proto::Want) -> Option<()> {
        log::debug!("Received message {:?}", message);
        Some(())
    }

    async fn on_unwant(&mut self, _client: &mut Client, message: &proto::Unwant) -> Option<()> {
        log::debug!("Received message {:?}", message);
        Some(())
    }

    async fn on_request(&mut self, _client: &mut Client, message: &proto::Request) -> Option<()> {
        log::debug!("Received message {:?}", message);
        Some(())
    }

    async fn on_cancel(&mut self, _client: &mut Client, message: &proto::Cancel) -> Option<()> {
        log::debug!("Received message {:?}", message);
        Some(())
    }

    async fn on_data(&mut self, _client: &mut Client, message: &proto::Data) -> Option<()> {
        log::debug!("Received message {:?}", message);
        Some(())
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
    dat_key: colmeia_dat_core::HashUrl,
    writer_socket: socket::CloneableStream,
}

impl ClientInitialization {
    pub fn dat_key(&self) -> &colmeia_dat_core::HashUrl {
        &self.dat_key
    }
}

pub async fn new_client(key: &str, tcp_stream: TcpStream) -> ClientInitialization {
    let dat_key = colmeia_dat_core::parse(&key).expect("invalid dat argument");

    let dat_key = match dat_key {
        DatUrlResolution::HashUrl(result) => result,
        _ => panic!("invalid hash key"),
    };

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
        socket: socket.clone(),
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

pub async fn handshake(mut init: ClientInitialization) -> Option<Client> {
    log::debug!("Bulding nonce to start connection");
    let nonce: [u8; 24] = rand::thread_rng().gen();
    let nonce = nonce.to_vec();
    let mut feed = proto::Feed::new();
    feed.set_discoveryKey(init.dat_key.discovery_key().to_vec());
    feed.set_nonce(nonce);
    init.bare_writer
        .send(ChannelMessage::new(
            0,
            0,
            feed.write_to_bytes().expect("invalid feed message"),
        ))
        .await
        .ok()?;

    log::debug!("Sent a nonce, upgrading write socket");
    init.upgradable_writer.upgrade(feed.get_nonce());
    let mut writer = Writer::new(BufWriter::new(init.upgradable_writer));

    log::debug!("Preparing to read feed nonce");
    let received_feed = init.bare_reader.next().await?.ok()?;
    let parsed_feed = received_feed.parse().ok()?;
    let payload = match parsed_feed {
        DatMessage::Feed(payload) => payload,
        _ => return None,
    };

    log::debug!("Dat feed received {:?}", payload);
    if payload.get_discoveryKey() != init.dat_key.discovery_key() && !payload.has_nonce() {
        return None;
    }
    log::debug!("Feed received, upgrading read socket");
    init.upgradable_reader.upgrade(payload.get_nonce());
    let mut reader = Reader::new(BufReader::new(init.upgradable_reader));

    log::debug!("Preparing to send encrypted handshake");
    let mut handshake = proto::Handshake::new();
    let id: [u8; 32] = rand::thread_rng().gen();
    let id = id.to_vec();
    handshake.set_id(id);
    handshake.set_live(true);
    handshake.set_ack(false);
    log::debug!("Dat handshake to send {:?}", &handshake);

    writer
        .send(ChannelMessage::new(
            0,
            1,
            handshake
                .write_to_bytes()
                .expect("invalid handshake message"),
        ))
        .await
        .ok()?;

    log::debug!("Preparing to read encrypted handshake");
    let handshake_received = reader.next().await?.ok()?;
    let handshake_parsed = handshake_received.parse().ok()?;
    let payload = match handshake_parsed {
        DatMessage::Handshake(payload) => payload,
        _ => return None,
    };

    if handshake.get_id() == payload.get_id() {
        // disconnect, we connect to ourselves
        return None;
    }

    log::debug!("Handshake finished");

    Some(Client {
        reader,
        writer,
        writer_socket: init.writer_socket,
    })
}

pub struct DatService {
    stream: Pin<Box<dyn stream::Stream<Item = ChannelMessage>>>,
}

impl DatService {
    pub fn new<O>(client: Client, observer: O) -> Self
    where
        O: DatObserver + Send + 'static,
    {
        let stream = stream::unfold(
            (client, observer, true),
            |(mut client, mut observer, first)| async move {
                if first {
                    observer.on_start(&mut client).await;
                };

                let response = client.reader().next().await?;
                match &response {
                    Err(err) => {
                        log::debug!("Error {:?}. stopping stream", err);
                        return None;
                    }
                    Ok(message) => match message.parse() {
                        Ok(DatMessage::Feed(m)) => observer.on_feed(&mut client, &m).await?,
                        Ok(DatMessage::Handshake(m)) => {
                            observer.on_handshake(&mut client, &m).await?
                        }
                        Ok(DatMessage::Info(m)) => observer.on_info(&mut client, &m).await?,
                        Ok(DatMessage::Have(m)) => observer.on_have(&mut client, &m).await?,
                        Ok(DatMessage::Unhave(m)) => observer.on_unhave(&mut client, &m).await?,
                        Ok(DatMessage::Want(m)) => observer.on_want(&mut client, &m).await?,
                        Ok(DatMessage::Unwant(m)) => observer.on_unwant(&mut client, &m).await?,
                        Ok(DatMessage::Request(m)) => observer.on_request(&mut client, &m).await?,
                        Ok(DatMessage::Cancel(m)) => observer.on_cancel(&mut client, &m).await?,
                        Ok(DatMessage::Data(m)) => observer.on_data(&mut client, &m).await?,
                        Err(err) => log::debug!("Dropping message {:?} err {:?}", message, err),
                    },
                };

                Some((response.unwrap(), (client, observer, false)))
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
