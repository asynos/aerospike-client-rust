use crate::tls_stream::TlsStream;
use aerospike_rt::net::TcpStream;
use std::fmt::Debug;
#[cfg(feature = "tokio-rustls")]
use tokio_rustls::rustls::ServerName;

#[derive(Clone)]
pub enum TlsConnector {
    #[cfg(feature = "tokio-rustls")]
    Rustls(tokio_rustls::TlsConnector),
    #[cfg(feature = "tokio-native-tls")]
    NativeTls(tokio_native_tls::TlsConnector),
}

impl Debug for TlsConnector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "tokio-rustls")]
            Self::Rustls(_) => f.debug_tuple("Rustls").finish(),
            #[cfg(feature = "tokio-native-tls")]
            Self::NativeTls(_) => f.debug_tuple("NativeTls").finish(),
        }
    }
}

pub enum TlsConnectError {
    IO(std::io::Error),
    InvalidDnsName,
    #[cfg(feature = "tokio-native-tls")]
    NativeTlsError(tokio_native_tls::native_tls::Error),
}

impl TlsConnector {
    #[cfg(feature = "tokio-rustls")]
    pub fn new_rustls(connector: tokio_rustls::TlsConnector) -> Self {
        Self::Rustls(connector)
    }

    #[cfg(feature = "tokio-native-tls")]
    pub fn new_native_tls(connector: tokio_native_tls::TlsConnector) -> Self {
        Self::NativeTls(connector)
    }

    pub async fn connect(
        &self,
        domain: &str,
        stream: TcpStream,
    ) -> Result<TlsStream, TlsConnectError> {
        match self {
            #[cfg(feature = "tokio-rustls")]
            Self::Rustls(connector) => {
                let domain =
                    ServerName::try_from(domain).map_err(|_| TlsConnectError::InvalidDnsName)?;
                connector
                    .connect(domain, stream)
                    .await
                    .map_err(|err| TlsConnectError::IO(err))
                    .map(|stream| TlsStream::Rustls(stream))
            }
            #[cfg(feature = "tokio-native-tls")]
            Self::NativeTls(connector) => connector
                .connect(domain, stream)
                .await
                .map_err(|err| TlsConnectError::NativeTlsError(err))
                .map(|stream| TlsStream::NativeTls(stream)),
        }
    }
}
