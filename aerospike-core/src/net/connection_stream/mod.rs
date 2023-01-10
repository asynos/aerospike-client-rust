#[cfg(all(any(not(feature = "rt-async-std")), feature = "rt-tokio"))]
mod tokio;

#[cfg(all(any(feature = "rt-async-std"), not(feature = "rt-tokio")))]
mod async_std;

#[cfg(all(any(not(feature = "rt-async-std")), feature = "rt-tokio"))]
pub use self::tokio::ConnectionStream;

#[cfg(all(any(feature = "rt-async-std"), not(feature = "rt-tokio")))]
pub use self::async_std::ConnectionStream;