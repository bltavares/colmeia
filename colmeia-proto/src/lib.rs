use async_std::io::ReadExt;
use async_std::net::TcpStream;
use async_std::stream::StreamExt;
use futures::stream;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
struct Header {
    length: u64,
    channel: u16,
    msg_type: u8,
}

#[derive(Debug)]
pub struct Message {
    header: Header,
    body: Vec<u8>,
}

impl Message {
    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

pub struct DatReader {
    listener: Pin<Box<dyn stream::Stream<Item = Message> + Send>>,
}

async fn read_varint(
    mut socket: impl stream::Stream<Item = std::io::Result<u8>> + Unpin,
) -> Option<u64> {
    let mut buffer = Vec::with_capacity(10);
    let mut decoded = 0;
    while let Some(byte) = socket.next().await {
        let byte = byte.ok()?;
        buffer.push(byte);
        if (byte & 0b10000000) == 0 {
            break;
        }
    }
    let _ = varinteger::decode(&buffer, &mut decoded);
    Some(decoded)
}

impl DatReader {
    pub fn new(socket: TcpStream) -> Self {
        let listener = stream::unfold(socket.bytes(), |mut socket| {
            async {
                let length = read_varint(&mut socket).await?;

                // PING
                if length == 0 {
                    return Some((None, socket));
                }

                let msg_chan_type = read_varint(&mut socket).await?;
                log::debug!("Length to read {:?}", length);
                log::debug!("Encoded channel-type {:?}", msg_chan_type);

                let channel = (msg_chan_type >> 4) as u16;
                let msg_type = (msg_chan_type & 0b1111) as u8;

                log::debug!("Msg channel {:?}", channel);
                log::debug!("Msg type {:?}", msg_type);

                let mut body = Vec::with_capacity(length as usize);
                for _ in 0..length {
                    if let Some(byte) = socket.next().await {
                        body.push(byte.ok()?);
                    }
                }

                let message = Message {
                    header: Header {
                        length,
                        channel,
                        msg_type,
                    },
                    body,
                };

                Some((Some(message), socket))
            }
        })
        .filter_map(|m| m);

        Self {
            listener: Box::pin(listener),
        }
    }
}

impl stream::Stream for DatReader {
    type Item = Message;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;

        self.listener.poll_next_unpin(cx)
    }
}
