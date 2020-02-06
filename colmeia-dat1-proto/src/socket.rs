use async_std::net::TcpStream;
use futures::io::{AsyncRead, AsyncWrite};
use std::io::Result;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::cipher::SharedCipher;

#[derive(Clone)]
pub struct CloneableStream {
    pub(crate) socket: Arc<TcpStream>,
    pub(crate) cipher: SharedCipher,
    pub(crate) buffer: Option<Vec<u8>>,
}

impl CloneableStream {
    pub fn upgrade(&mut self, nonce: &[u8]) {
        self.cipher
            .write()
            .expect("could not aquire upgrade lock")
            .initialize(nonce);
    }
}

impl AsyncRead for CloneableStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context, buf: &mut [u8]) -> Poll<Result<usize>> {
        let content = futures::ready!(Pin::new(&mut &*self.socket).poll_read(cx, buf))?;
        if content == 0 {
            return Poll::Ready(Ok(content));
        }
        log::debug!("content to read {:?}", content);
        log::debug!("original content {:?}", &buf[..content]);

        self.cipher
            .write()
            .expect("could not acquire read encrypted lock")
            .try_apply(&mut buf[..content]);

        log::debug!("maybe encrypted content {:?}", &buf[..content]);
        Poll::Ready(Ok(content))
    }
}

impl AsyncWrite for CloneableStream {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context, buf: &[u8]) -> Poll<Result<usize>> {
        log::debug!("Sending information over the network");
        log::debug!("content to encrypt {:?}", buf);
        log::debug!("content to encrypt {:?}", self.buffer);

        if self.buffer.is_none() {
            log::debug!("content to encrypt {:?}", buf);
            let mut buffer = buf.to_vec();
            self.cipher
                .write()
                .expect("could not acquire write encrypted lock")
                .try_apply(&mut buffer);
            log::debug!("maybe encrypted {:?}", buffer);
            self.buffer = Some(buffer);
        }
        if let Some(output) = &self.buffer {
            log::debug!("buffered to send {:?}", output);
            let sent = futures::ready!(Pin::new(&mut &*self.socket).poll_write(cx, &output))?;
            if sent == 0 {
                self.buffer = None;
            } else {
                let remaining: Vec<u8> = output.iter().skip(sent).cloned().collect();
                if remaining.is_empty() {
                    self.buffer = None;
                } else {
                    self.buffer = Some(remaining);
                }
            }
            return Poll::Ready(Ok(sent));
        }
        Poll::Pending
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        Pin::new(&mut &*self.socket).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<()>> {
        Pin::new(&mut &*self.socket).poll_close(cx)
    }
}
