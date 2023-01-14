#[cfg(not(any(feature = "tokio-rustls", feature = "tokio-native-tls")))]
compile_error!("Please select a tls implementation from ['tokio-rustls', 'tokio-native-tls']");

#[cfg(feature = "tokio-rustls")]
extern crate tokio_rustls;

#[cfg(feature = "tokio-native-tls")]
extern crate tokio_native_tls;

mod tls_connector;
mod tls_stream;

pub use tls_connector::{TlsConnectError, TlsConnector};
pub use tls_stream::TlsStream;
#[cfg(feature = "tokio-rustls")]
pub use tokio_rustls::*;

#[cfg(feature = "tokio-native-tls")]
pub use tokio_native_tls::*;
