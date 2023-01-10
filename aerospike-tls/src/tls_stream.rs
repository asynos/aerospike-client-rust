use std::{fmt::Debug, pin::Pin};

use aerospike_rt::{io::AsyncRead, io::AsyncWrite, net::TcpStream};
use std::task::{Context, Poll};

pub enum TlsStream {
    #[cfg(feature = "tokio-rustls")]
    Rustls(tokio_rustls::client::TlsStream<TcpStream>),
    #[cfg(feature = "tokio-native-tls")]
    NativeTls(tokio_native_tls::TlsStream<TcpStream>),
}

impl Debug for TlsStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "tokio-rustls")]
            Self::Rustls(_) => f.debug_tuple("Rustls").finish(),
            #[cfg(feature = "tokio-native-tls")]
            Self::NativeTls(_) => f.debug_tuple("NativeTls").finish(),
        }
    }
}

impl AsyncRead for TlsStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut aerospike_rt::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(feature = "tokio-rustls")]
            Self::Rustls(stream) => Pin::new(stream).poll_read(cx, buf),
            #[cfg(feature = "tokio-native-tls")]
            Self::NativeTls(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for TlsStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            #[cfg(feature = "tokio-rustls")]
            Self::Rustls(stream) => Pin::new(stream).poll_write(cx, buf),
            #[cfg(feature = "tokio-native-tls")]
            Self::NativeTls(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            #[cfg(feature = "tokio-rustls")]
            Self::Rustls(stream) => Pin::new(stream).poll_flush(cx),
            #[cfg(feature = "tokio-native-tls")]
            Self::NativeTls(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            #[cfg(feature = "tokio-rustls")]
            Self::Rustls(stream) => Pin::new(stream).poll_shutdown(cx),
            #[cfg(feature = "tokio-native-tls")]
            Self::NativeTls(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}
