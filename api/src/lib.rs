//! Implementation neutral TLS API.
//!
//! The idea is that code can be written without the knowledge of the TLS implementation used,
//! like this:
//!
//! ```
//! # { #![cfg(feature = "runtime-tokio")]
//! use tls_api::{TlsConnector, TlsConnectorBuilder};
//! use tokio::net::TcpStream;
//! # use tls_api::runtime::AsyncWriteExt;
//! # use tls_api::runtime::AsyncReadExt;
//!
//! async fn download_rust_lang_org<C: TlsConnector>() -> tls_api::Result<Vec<u8>> {
//!     let stream = TcpStream::connect(("rust-lang.org", 443)).await?;
//!     let mut  stream = C::builder()?.build()?.connect("rust-lang.org", stream).await?;
//!     stream.write_all(b"GET / HTTP/1.1\r\nHost: rust-lang.org\r\n\r\n").await?;
//!     let mut buf = Vec::new();
//!     stream.read_to_end(&mut buf).await?;
//!     Ok(buf)
//! }
//! # }
//! ```
//!
//! or the same code with dynamic connector:
//!
//! ```
//! # { #![cfg(feature = "runtime-tokio")]
//! use tls_api::TlsConnectorType;
//! use tokio::net::TcpStream;
//! # use tls_api::runtime::AsyncWriteExt;
//! # use tls_api::runtime::AsyncReadExt;
//!
//! async fn download_rust_lang_org(connector_type: &dyn TlsConnectorType) -> tls_api::Result<Vec<u8>> {
//!     let stream = TcpStream::connect(("rust-lang.org", 443)).await?;
//!     let mut  stream = connector_type.builder()?.build()?.connect("rust-lang.org", stream).await?;
//!     stream.write_all(b"GET / HTTP/1.1\r\nHost: rust-lang.org\r\n\r\n").await?;
//!     let mut buf = Vec::new();
//!     stream.read_to_end(&mut buf).await?;
//!     Ok(buf)
//! }
//! # }
//! ```
//!
//! And then this function can be executed with any API implementations. The implementations are:
//! * `tls-api-openssl`, wraps `openssl` crate
//! * `tls-api-rustls`, wraps `rustls` crate
//! * `tls-api-native-tls`, wraps `native-tls` crate
//! * `tls-api-security-framework`, wraps `security-framework` crate
//!
//! Have a look at working example invoking all implementation
//! [on GitHub](https://github.com/stepancheg/rust-tls-api/blob/master/examples/examples/download-rust-lang-org.rs#L66).
//!
//! There are also two fake implementations:
//! * `tls-api-stub` crate which returns an error on any operations, useful to check code compiles
//! * `tls-api-no-tls` fake implementation which returns plain sockets without TLS
//!
//! The API is provided to be compatible with both tokio and async-std.
//! Crate features:
//! * `runtime-tokio` enables the implementation over tokio
//! * `runtime-async-std` enables the implementation over async-std
//!
//! Currently the features are mutually exclusive.

#![deny(broken_intra_doc_links)]
#![deny(missing_docs)]

pub use acceptor::TlsAcceptor;
pub use acceptor::TlsAcceptorBuilder;
pub use acceptor_box::TlsAcceptorBox;
pub use acceptor_box::TlsAcceptorBuilderBox;
pub use acceptor_box::TlsAcceptorType;
pub use connector::TlsConnector;
pub use connector::TlsConnectorBuilder;
pub use connector_box::TlsConnectorBox;
pub use connector_box::TlsConnectorBuilderBox;
pub use connector_box::TlsConnectorType;
pub(crate) use error::CommonError;
pub use error::Error;
pub use error::Result;
pub use future::BoxFuture;
pub use info::ImplInfo;
pub use socket::AsyncSocket;
pub use socket_box::AsyncSocketBox;
pub use stream::TlsStream;
pub use stream_with_socket::TlsStreamWithSocket;

pub(crate) use crate::assert_kinds::assert_send;
pub(crate) use crate::assert_kinds::assert_send_value;
pub(crate) use crate::assert_kinds::assert_sync;

pub mod runtime;
pub mod spi;

mod acceptor;
mod acceptor_box;
mod assert_kinds;
mod connector;
mod connector_box;
mod error;
mod future;
mod info;
mod openssl;
mod socket;
mod socket_box;
mod stream;
mod stream_dyn;
mod stream_with_socket;

fn _assert_kinds() {
    fn connect_future_is_send<C, S>(c: &C, s: S)
    where
        C: TlsConnector,
        S: AsyncSocket,
    {
        let f = c.connect_with_socket("dom", s);
        assert_send_value(f);
    }

    fn accept_future_is_send<A, S>(a: &A, s: S)
    where
        A: TlsAcceptor,
        S: AsyncSocket,
    {
        let f = a.accept_with_socket(s);
        assert_send_value(f);
    }
}
