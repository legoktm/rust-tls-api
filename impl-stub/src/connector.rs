use tls_api::AsyncSocket;
use tls_api::BoxFuture;
use tls_api::ImplInfo;

use void::Void;

use crate::Error;

/// Non-instantiatable.
pub struct TlsConnectorBuilder(Void);
/// Non-instantiatable.
pub struct TlsConnector(Void);

impl tls_api::TlsConnectorBuilder for TlsConnectorBuilder {
    type Connector = TlsConnector;

    type Underlying = Void;

    fn underlying_mut(&mut self) -> &mut Void {
        &mut self.0
    }

    fn set_alpn_protocols(&mut self, _protocols: &[&[u8]]) -> tls_api::Result<()> {
        Err(tls_api::Error::new(Error))
    }

    fn set_verify_hostname(&mut self, _verify: bool) -> tls_api::Result<()> {
        Err(tls_api::Error::new(Error))
    }

    fn add_root_certificate(&mut self, _cert: &[u8]) -> tls_api::Result<()> {
        Err(tls_api::Error::new(Error))
    }

    fn build(self) -> tls_api::Result<TlsConnector> {
        Err(tls_api::Error::new(Error))
    }
}

impl tls_api::TlsConnector for TlsConnector {
    type Builder = TlsConnectorBuilder;

    const IMPLEMENTED: bool = false;
    const SUPPORTS_ALPN: bool = false;

    type Underlying = Void;

    fn underlying_mut(&mut self) -> &mut Self::Underlying {
        &mut self.0
    }

    fn info() -> ImplInfo {
        crate::info()
    }

    fn builder() -> tls_api::Result<TlsConnectorBuilder> {
        Err(tls_api::Error::new(Error))
    }

    fn connect_with_socket<'a, S>(
        &'a self,
        _domain: &'a str,
        _stream: S,
    ) -> BoxFuture<'a, tls_api::Result<tls_api::TlsStreamWithSocket<S>>>
    where
        S: AsyncSocket,
    {
        BoxFuture::new(async { Err(tls_api::Error::new(Error)) })
    }
}
