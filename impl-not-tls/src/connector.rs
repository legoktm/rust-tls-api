use tls_api::AsyncSocket;
use tls_api::BoxFuture;
use tls_api::ImplInfo;
use tls_api::TlsStreamWithSocket;

pub struct TlsConnectorBuilder(pub ());

impl tls_api::TlsConnectorBuilder for TlsConnectorBuilder {
    type Connector = TlsConnector;
    type Underlying = ();

    fn underlying_mut(&mut self) -> &mut Self::Underlying {
        &mut self.0
    }

    fn set_alpn_protocols(&mut self, protocols: &[&[u8]]) -> tls_api::Result<()> {
        let _ = protocols;
        Err(crate::Error::Alpn.into())
    }

    fn set_verify_hostname(&mut self, verify: bool) -> tls_api::Result<()> {
        let _ = verify;
        Ok(())
    }

    fn add_root_certificate(&mut self, cert: &[u8]) -> tls_api::Result<()> {
        let _ = cert;
        Ok(())
    }

    fn build(self) -> tls_api::Result<Self::Connector> {
        Ok(TlsConnector(self.0))
    }
}

pub struct TlsConnector(pub ());

impl tls_api::TlsConnector for TlsConnector {
    type Builder = TlsConnectorBuilder;

    const IMPLEMENTED: bool = false;
    const SUPPORTS_ALPN: bool = false;

    type Underlying = ();

    fn underlying_mut(&mut self) -> &mut Self::Underlying {
        &mut self.0
    }

    fn info() -> ImplInfo {
        crate::info()
    }

    fn builder() -> tls_api::Result<TlsConnectorBuilder> {
        Ok(TlsConnectorBuilder(()))
    }

    fn connect_with_socket<'a, S>(
        &'a self,
        domain: &'a str,
        stream: S,
    ) -> BoxFuture<'a, tls_api::Result<TlsStreamWithSocket<S>>>
    where
        S: AsyncSocket,
    {
        let _ = domain;
        BoxFuture::new(async {
            Ok(tls_api::TlsStreamWithSocket::new(crate::stream::TlsStream(
                stream,
            )))
        })
    }
}
