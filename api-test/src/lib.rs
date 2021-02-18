//! Common implementation of tests for all TLS API implementations

#[macro_use]
extern crate log;

#[macro_use]
mod t;

use std::str;
use std::thread;

use tls_api::runtime::AsyncReadExt;
use tls_api::runtime::AsyncWriteExt;
use tls_api::TlsAcceptor;
use tls_api::TlsAcceptorBuilder;
use tls_api::TlsConnector;
use tls_api::TlsConnectorBuilder;
use tls_api::TlsStream;

use std::net::ToSocketAddrs;

use test_cert_gen::ServerKeys;

#[cfg(feature = "runtime-async-std")]
use async_std::net::TcpListener;
#[cfg(feature = "runtime-async-std")]
use async_std::net::TcpStream;
#[cfg(feature = "runtime-async-std")]
pub use async_std::task::block_on;
#[cfg(feature = "runtime-tokio")]
use std::future::Future;
#[cfg(feature = "runtime-tokio")]
use tokio::net::TcpListener;
#[cfg(feature = "runtime-tokio")]
use tokio::net::TcpStream;

#[cfg(feature = "runtime-tokio")]
pub fn block_on<F, T>(future: F) -> T
where
    F: Future<Output = T>,
{
    t!(tokio::runtime::Runtime::new()).block_on(future)
}

async fn test_google_impl<C: TlsConnector>() {
    drop(env_logger::try_init());

    // First up, resolve google.com
    let addr = t!("google.com:443".to_socket_addrs()).next().unwrap();

    let connector: C = C::builder().expect("builder").build().expect("build");
    let tcp_stream = t!(TcpStream::connect(addr).await);
    let mut tls_stream: TlsStream<_> = t!(connector.connect("google.com", tcp_stream).await);

    info!("handshake complete");

    t!(tls_stream.write_all(b"GET / HTTP/1.0\r\n\r\n").await);
    let mut result = vec![];
    t!(tls_stream.read_to_end(&mut result).await);

    println!("{}", String::from_utf8_lossy(&result));
    assert!(
        result.starts_with(b"HTTP/1.0"),
        "wrong result: {:?}",
        result
    );
    assert!(result.ends_with(b"</HTML>\r\n") || result.ends_with(b"</html>"));
}

pub fn test_google<C: TlsConnector>() {
    block_on(test_google_impl::<C>())
}

async fn connect_bad_hostname_impl<C: TlsConnector>() -> tls_api::Error {
    drop(env_logger::try_init());

    // First up, resolve google.com
    let addr = t!("google.com:443".to_socket_addrs()).next().unwrap();

    let connector: C = C::builder().expect("builder").build().expect("build");
    let tcp_stream = t!(TcpStream::connect(addr).await);
    connector
        .connect("goggle.com", tcp_stream)
        .await
        .unwrap_err()
}

pub fn connect_bad_hostname<C: TlsConnector>() -> tls_api::Error {
    block_on(connect_bad_hostname_impl::<C>())
}

async fn connect_bad_hostname_ignored_impl<C: TlsConnector>() {
    drop(env_logger::try_init());

    // First up, resolve google.com
    let addr = t!("google.com:443".to_socket_addrs()).next().unwrap();

    let tcp_stream = t!(TcpStream::connect(addr).await);

    let mut builder = C::builder().expect("builder");
    builder
        .set_verify_hostname(false)
        .expect("set_verify_hostname");
    let connector: C = builder.build().expect("build");
    t!(connector.connect("ignore", tcp_stream).await);
}

pub fn connect_bad_hostname_ignored<C: TlsConnector>() {
    block_on(connect_bad_hostname_ignored_impl::<C>())
}

fn new_acceptor<A, F>(acceptor: F) -> A::Builder
where
    A: TlsAcceptor,
    F: FnOnce(&ServerKeys) -> A::Builder,
{
    let keys = &test_cert_gen::keys().server;

    acceptor(keys)
}

fn new_connector_with_root_ca<C: TlsConnector>() -> C::Builder {
    let keys = test_cert_gen::keys();
    let root_ca = &keys.client.ca_der;

    let mut connector = C::builder().expect("connector builder");
    t!(connector.add_root_certificate(root_ca));
    connector
}

// `::1` is broken on travis-ci
// https://travis-ci.org/stepancheg/rust-tls-api/jobs/312681800
const BIND_HOST: &str = "127.0.0.1";

async fn server_impl<C, A, F>(acceptor: F)
where
    C: TlsConnector,
    A: TlsAcceptor,
    F: FnOnce(&ServerKeys) -> A::Builder,
{
    drop(env_logger::try_init());

    let acceptor = new_acceptor::<A, _>(acceptor);

    let acceptor: A = acceptor.build().expect("acceptor build");
    #[allow(unused_mut)]
    let mut listener = t!(TcpListener::bind((BIND_HOST, 0)).await);
    let port = listener.local_addr().expect("local_addr").port();

    let server_thread_name = format!("{}-server", thread::current().name().unwrap_or("test"));
    let j = thread::Builder::new()
        .name(server_thread_name)
        .spawn(move || {
            let future = async {
                let socket = t!(listener.accept().await).0;
                let mut socket = t!(acceptor.accept(socket).await);

                let mut buf = [0; 5];
                t!(socket.read_exact(&mut buf).await);
                assert_eq!(&buf, b"hello");

                t!(socket.write_all(b"world").await);
            };
            block_on(future);
        })
        .unwrap();

    let socket = t!(TcpStream::connect((BIND_HOST, port)).await);

    let connector: C::Builder = new_connector_with_root_ca::<C>();
    let connector: C = connector.build().expect("acceptor build");
    let mut socket = t!(connector.connect("localhost", socket).await);

    t!(socket.write_all(b"hello").await);
    let mut buf = vec![];
    t!(socket.read_to_end(&mut buf).await);
    assert_eq!(buf, b"world");

    j.join().expect("thread join");
}

pub fn server<C, A, F>(acceptor: F)
where
    C: TlsConnector,
    A: TlsAcceptor,
    F: FnOnce(&ServerKeys) -> A::Builder,
{
    block_on(server_impl::<C, A, F>(acceptor))
}

async fn alpn_impl<C, A, F>(acceptor: F)
where
    C: TlsConnector,
    A: TlsAcceptor,
    F: FnOnce(&ServerKeys) -> A::Builder,
{
    drop(env_logger::try_init());

    if !C::SUPPORTS_ALPN {
        debug!("connector does not support ALPN");
        return;
    }

    if !A::SUPPORTS_ALPN {
        debug!("acceptor does not support ALPN");
        return;
    }

    let mut acceptor: A::Builder = new_acceptor::<A, _>(acceptor);

    acceptor
        .set_alpn_protocols(&[b"abc", b"de", b"f"])
        .expect("set_alpn_protocols");

    let acceptor: A = t!(acceptor.build());

    #[allow(unused_mut)]
    let mut listener = t!(TcpListener::bind((BIND_HOST, 0)).await);
    let port = listener.local_addr().expect("local_addr").port();

    let j = thread::spawn(move || {
        let f = async {
            let socket = t!(listener.accept().await).0;
            let mut socket = t!(acceptor.accept(socket).await);

            assert_eq!(b"de", &socket.get_alpn_protocol().unwrap()[..]);

            let mut buf = [0; 5];
            t!(socket.read_exact(&mut buf).await);
            assert_eq!(&buf, b"hello");

            t!(socket.write_all(b"world").await);
        };
        block_on(f);
    });

    let socket = t!(TcpStream::connect((BIND_HOST, port)).await);

    let mut connector: C::Builder = new_connector_with_root_ca::<C>();

    connector
        .set_alpn_protocols(&[b"xyz", b"de", b"u"])
        .expect("set_alpn_protocols");

    let connector: C = connector.build().expect("acceptor build");
    let mut socket = t!(connector.connect("localhost", socket).await);

    assert_eq!(b"de", &socket.get_alpn_protocol().unwrap()[..]);

    t!(socket.write_all(b"hello").await);
    let mut buf = vec![];
    t!(socket.read_to_end(&mut buf).await);
    assert_eq!(buf, b"world");

    j.join().expect("thread join");
}

pub fn alpn<C, A, F>(acceptor: F)
where
    C: TlsConnector,
    A: TlsAcceptor,
    F: FnOnce(&ServerKeys) -> A::Builder,
{
    block_on(alpn_impl::<C, A, F>(acceptor))
}
