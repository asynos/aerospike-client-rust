use std::{pin::Pin, task::{Poll, Context}};
use aerospike_rt::net::TcpStream;
use aerospike_rt::io::{AsyncRead, AsyncWrite};

#[cfg(feature = "tls")]
use aerospike_tls::TlsStream;

#[derive(Debug)]
pub enum ConnectionStream {
    Tcp(TcpStream),
    #[cfg(feature = "tls")]
    Tls(TlsStream),
}

impl AsyncRead for ConnectionStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut aerospike_rt::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(feature = "tls")]
            Self::Tls(stream) => Pin::new(stream).poll_read(cx, buf),
            Self::Tcp(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for ConnectionStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            #[cfg(feature = "tls")]
            Self::Tls(stream) => Pin::new(stream).poll_write(cx, buf),
            Self::Tcp(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            #[cfg(feature = "tls")]
            Self::Tls(stream) => Pin::new(stream).poll_flush(cx),
            Self::Tcp(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            #[cfg(feature = "tls")]
            Self::Tls(stream) => Pin::new(stream).poll_shutdown(cx),
            Self::Tcp(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}
