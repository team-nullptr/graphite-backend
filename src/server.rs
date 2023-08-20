use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    pin::Pin,
    sync::Arc,
};

use axum::{routing::IntoMakeService, Router};
use futures_util::future::poll_fn;
use hyper::{
    server::{
        accept::Accept,
        conn::{AddrIncoming, Http},
    },
    Body, Request,
};
use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio::net::TcpListener;
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig},
    TlsAcceptor,
};

pub struct CertConfig {
    pub key: PathBuf,
    pub cert: PathBuf,
}

/// Starts a new server listening on giver address with given router.
pub async fn listen(addr: String, cert_config: CertConfig, mut router: IntoMakeService<Router>) {
    let protocol = Arc::new(Http::new());
    let rustls_config = build_rustls_server_config(cert_config.key, cert_config.cert);

    let acceptor = TlsAcceptor::from(rustls_config);
    let mut listener =
        AddrIncoming::from_listener(TcpListener::bind(&addr).await.unwrap()).unwrap();

    loop {
        let acceptor = acceptor.clone();
        let protocol = protocol.clone();

        let stream = poll_fn(|cx| Pin::new(&mut listener).poll_accept(cx))
            .await
            .unwrap()
            .unwrap();

        let svc = tower::MakeService::<_, Request<Body>>::make_service(&mut router, &stream);

        tokio::spawn(async move {
            if let Ok(stream) = acceptor.accept(stream).await {
                let _ = protocol.serve_connection(stream, svc.await.unwrap()).await;
            }
        });
    }
}

// Based on rustls example in axum repo.
// See https://github.com/tokio-rs/axum/tree/main/examples/tls-rustls
fn build_rustls_server_config(key: impl AsRef<Path>, cert: impl AsRef<Path>) -> Arc<ServerConfig> {
    let mut key_reader = BufReader::new(File::open(key).unwrap());
    let mut cert_reader = BufReader::new(File::open(cert).unwrap());

    let key = PrivateKey(pkcs8_private_keys(&mut key_reader).unwrap().remove(0));
    let certs = certs(&mut cert_reader)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();

    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("Invalid certificate or key");

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Arc::new(config)
}
