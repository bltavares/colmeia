use async_std::io::ReadExt;
use async_std::net::TcpStream;
use async_std::stream::StreamExt;
use futures::stream;
use std::pin::Pin;
use std::task::{Context, Poll};

struct Header {
    length: usize,
    channel: u16,
    msg_type: u8,
}

struct Message {
    header: Header,
    body: Vec<u8>,
}

pub struct DatReader {
    listener: Pin<Box<dyn stream::Stream<Item = ()> + Send>>,
}

impl DatReader {
    pub fn new(socket: TcpStream) -> Self {
        let listener = stream::unfold(socket.bytes().filter_map(Result::ok), |mut socket| {
            async {
                let mut length = Vec::with_capacity(20);
                let mut length_decoded = 0;
                let mut msg_type = 0;

                while let Some(byte) = socket.next().await {
                    length.push(byte);
                    if (byte & 0xF0) == 0 {
                        break;
                    }
                }

                let remaining = varinteger::decode(&length, &mut length_decoded);
                varinteger::decode(&length[remaining..], &mut msg_type);

                log::debug!("Length to read {:?}", length_decoded);
                log::debug!("Msg type {:?}", msg_type);

                let mut body = Vec::with_capacity(length_decoded as usize);
                for _ in 0..length_decoded {
                    if let Some(byte) = socket.next().await {
                        body.push(byte);
                    }
                }

                dbg!(body);

                Some(((), socket))
            }
        });

        Self {
            listener: Box::pin(listener),
        }
    }
}

impl stream::Stream for DatReader {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        use futures::stream::StreamExt;

        self.listener.poll_next_unpin(cx)
    }
}
