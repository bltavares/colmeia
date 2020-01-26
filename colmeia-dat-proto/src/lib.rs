use async_std::net::TcpStream;
use colmeia_dat_core::DatUrlResolution;
use futures::FutureExt;
use protobuf::Message;
use rand::Rng;
use simple_message_channels::{Cipher, Message as ChannelMessage, Reader, Writer};

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};

mod schema;
mod socket;
use crate::schema as proto;

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
    reader: Reader<socket::CloneableStream>,
    writer: Writer<socket::CloneableStream>,
    handshake_writer: Pin<Box<dyn Future<Output = std::io::Result<()>>>>,
    step: u8,
}

impl Client {
    // TODO error handling
    pub fn new(key: &str, address: SocketAddr) -> impl Future<Output = Self> + '_ {
        let dat_key = colmeia_dat_core::parse(&key).expect("invalid dat argument");

        let dat_key = match dat_key {
            DatUrlResolution::HashUrl(result) => result,
            _ => panic!("invalid hash key"),
        };

        async move {
            let reader = socket::CloneableStream(Arc::new(
                TcpStream::connect(address)
                    .await
                    .expect("could not open socket"),
            ));
            let writer = reader.clone();

            let cipher = Arc::new(RwLock::new(Cipher::new(
                dat_key.public_key().as_bytes().to_vec(),
            )));

            let client_reader = Reader::encrypted(reader, cipher.clone(), |message, cypher| {
                if let Ok(DatMessage::Feed(payload)) = message.parse() {
                    if payload.has_nonce() {
                        cypher.initialize(payload.get_nonce());
                    }
                }
            });
            let mut handshake_writer = Writer::encrypted(writer.clone(), cipher.clone());
            let handshake_writer = async move {
                let id: [u8; 32] = rand::thread_rng().gen(); // TODO use id to avoid self connects
                let mut handshake = proto::Handshake::new();
                handshake.set_live(true);
                handshake.set_ack(false);
                handshake.set_id(id.to_vec());

                handshake_writer
                    .send(ChannelMessage::new(
                        0,
                        1,
                        handshake
                            .write_to_bytes()
                            .expect("invalid handshake crated"),
                    ))
                    .await
            }
            .boxed();

            let client_writer = Writer::encrypted(writer, cipher);

            Self {
                reader: client_reader,
                writer: client_writer,
                handshake_writer,
                step: 0,
            }
        }
    }

    pub fn writer(&mut self) -> &mut Writer<socket::CloneableStream> {
        &mut self.writer
    }
}

impl futures::Stream for Client {
    type Item = std::io::Result<ChannelMessage>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;

        if self.step == 0 {
            let response = futures::ready!(self.reader.poll_next_unpin(cx));
            self.step = 1;
            return Poll::Ready(response);
        }

        if self.step == 1 {
            let response = futures::ready!(self.reader.poll_next_unpin(cx));
            self.step = 2;
            return Poll::Ready(response);
        }

        if self.step == 2 {
            log::debug!("sending handshake back");
            futures::ready!(self.handshake_writer.poll_unpin(cx))?;
            self.step = 3;
        }

        self.reader.poll_next_unpin(cx)
    }
}
