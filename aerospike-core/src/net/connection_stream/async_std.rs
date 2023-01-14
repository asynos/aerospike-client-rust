use std::{pin::Pin, task::{Poll, Context}};
use aerospike_rt::net::TcpStream;
use futures::{AsyncRead, AsyncWrite};
use aerospike_rt::async_std::net::Shutdown;

#[derive(Debug)]
pub enum ConnectionStream {
    Tcp(TcpStream),
}

impl ConnectionStream {
    pub fn shutdown(&mut self, how: Shutdown) -> Result<(), std::io::Error> {
        match self {
            Self::Tcp(stream) => stream.shutdown(how)
        }
    }
}

impl AsyncRead for ConnectionStream {
    fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            Self::Tcp(stream) => Pin::new(stream).poll_read(cx, buf)
        }
    }
}

impl AsyncWrite for ConnectionStream {
    fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            match self.get_mut() {
                Self::Tcp(stream) => Pin::new(stream).poll_write(cx, buf)
            }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Tcp(stream) => Pin::new(stream).poll_flush(cx)
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Tcp(stream) => Pin::new(stream).poll_close(cx)
        }
    }
}
