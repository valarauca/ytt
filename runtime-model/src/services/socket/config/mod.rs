use std::{
    pin::{Pin},
    future::{Future},
};
use tokio::{
    net::{TcpStream,TcpListener},
    io::{AsyncRead, AsyncWrite},
};
use serde::{Serialize,Deserialize};
use hyper_util::{
    rt::tokio::{TokioExecutor,TokioIo},
    server::conn::auto::{Builder},
    server::graceful::{GracefulShutdown},
};
use hyper::{
    service::{Service},
    body::{Body,Incoming,Buf},
};
use tracing::warn;
use http::{Request,Response};
use tokio_rustls::{TlsAcceptor};
use tokio_stream::{StreamExt, wrappers::TcpListenerStream};
use futures_util::{
    stream::FuturesUnordered,
    future::{FutureExt, Either},
};

use crate::adapters::maybe_async::{MaybeFuture,make_ready,make_boxed};

pub mod keepalive;
pub mod tcp;
use self::tcp::TcpListenerConfig;
pub mod http1;
use self::http1::{Http1};
pub mod http2;
use self::http2::{Http2};
pub mod tls;
use self::tls::{RusTLSServerConfig};

#[derive(Clone,Serialize,Deserialize,PartialEq,Debug)]
pub enum OnlyMode {
    #[serde(rename = "http_v1")]
    V1,
    #[serde(rename = "http_v2")]
    V2,
}

#[derive(Clone,Deserialize,PartialEq,Debug)]
pub struct HttpListener {
    pub socket: TcpListenerConfig,
    pub tls: Option<RusTLSServerConfig>,
    pub only: Option<OnlyMode>,
    pub http1: Option<Http1>,
    pub http2: Option<Http2>,
}
impl HttpListener {

    fn make_server(&self) -> (Builder<TokioExecutor>, Vec<Vec<u8>>) {
        // uses the existing runtime
        let mut builder = Builder::new(TokioExecutor::default());

        let mut alpn: Vec<Vec<u8>> = Vec::new();

        let (httpv1,httpv2) = match &self.only {
            &Option::None => (true,true),
            &Option::Some(OnlyMode::V1) => (true,false),
            &Option::Some(OnlyMode::V2) => (false,true),
        };

        if httpv1 {
            let mut http1 = builder.http1();
            Http1::set_options(&self.http1, &mut http1);
            debug_assert!(builder.is_http1_available());
            if self.tls.is_some() {
                alpn.push(Vec::from(b"http/1.1"));
            }
        }
        if httpv2 {
            let mut http2 = builder.http2();
            Http2::set_options(&self.http2, &mut http2);
            debug_assert!(builder.is_http2_available());
            if self.tls.is_some() {
                alpn.push(Vec::from(b"h2"));
            }
        }
        (builder,alpn)
    }

    fn make_socket(&self) -> anyhow::Result<TcpListener> {
        let socket = self.socket.bind_socket()?;
        Ok(TcpListener::from_std(socket)?)
    }

    fn make_tls_server(&self, alpn: Vec<Vec<u8>>) -> anyhow::Result<Option<TlsAcceptor>> {
        if let Some(tls_config) = &self.tls {
            return Ok(Some(tls_config.build(alpn)?));
        }
        Ok(None)
    }
}



type BoxedStream = Box<dyn BoxableStream>;
type StreamResult = Result<BoxedStream, std::io::Error>;
type IntoConnection = MaybeFuture<StreamResult>;
pub trait BoxableStream: AsyncRead + AsyncWrite + Unpin + Send + 'static { }
impl<T: AsyncRead + AsyncWrite + Unpin + Send + 'static> BoxableStream for T { }

pub type BoxedErr = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type ListenerFuture = Pin<Box<dyn Future<Output=Result<(),BoxedErr>> + Send + 'static>>;
fn pf<F>(f: F) -> ListenerFuture
where
    F: Future<Output=Result<(),BoxedErr>> + Send + 'static,
{ Box::pin(f) }

impl HttpListener {

    /// Construct a new listener
    pub fn new<S,B,G>(&self, service: S, mut graceful_stop: G) -> anyhow::Result<ListenerFuture>
    where
        G: Future<Output=()> + Unpin + Send + 'static,
        B: Body + Send + 'static,
        <B as Body>::Data: Buf + Send + 'static,
        <B as Body>::Error: Into<BoxedErr>,
        S: Service<Request<Incoming>,Response=Response<B>> + Clone + Send + 'static,
        <S as Service<Request<Incoming>>>::Future: Send + 'static,
        <S as Service<Request<Incoming>>>::Error: Into<BoxedErr>,
    {
        let (server,alpn) = self.make_server();
        let socket = self.make_socket()?;
        let stream = TcpListenerStream::new(socket);

        let boxer: Box<dyn Fn(Result<TcpStream,std::io::Error>) -> IntoConnection + Send> = match self.make_tls_server(alpn)? {
            None => Box::new(|result: Result<TcpStream,std::io::Error>| -> IntoConnection {
                make_ready(result.map(|stream| -> BoxedStream { Box::new(stream) }))
            }),
            Some(acceptor) => Box::new(move |result: Result<TcpStream,std::io::Error>| -> IntoConnection {
                let x = match result {
                    Err(e) => return make_ready(Err(e)),
                    Ok(x) => x,
                };
                let fut = acceptor.accept(x)
                    .map(|x| -> StreamResult {
                        match x {
                            Ok(x) => Ok(Box::new(x)),
                            Err(e) => Err(e),
                        }
                    });
                make_boxed(fut)
            }),
        };

        let mut stream = stream.then(boxer);

        Ok(Box::pin(async move {
            let mut pool = FuturesUnordered::new();
            let graceful = GracefulShutdown::new();

            loop {
                let pool_future = if pool.is_empty() {
                    std::future::pending().right_future()
                } else {
                    pool.next().left_future()
                };

                tokio::select! {
                    _ = pool_future => {},
                    conn = stream.next() => {
                        let conn = match conn {
                            None => break,
                            Some(Ok(conn)) => conn,
                            Some(Err(e)) => {
                                warn!("error occured accepting a connection: '{:?}'", e);
                                continue;
                            }
                        };
                        let conn = TokioIo::new(conn);
                        let worker = if server.is_http2_available() && server.is_http1_available() {
                            let c = server.serve_connection_with_upgrades(conn, service.clone()).into_owned();
                            let c = graceful.watch(c);
                            pf(c) 
                        } else {
                            let c = server.serve_connection(conn, service.clone()).into_owned();
                            let c = graceful.watch(c);
                            pf(c)
                        };
                        pool.push(worker);
                    },
                    _ = &mut graceful_stop => {
                        drop(stream);
                        break;
                    }
                };
            }
            graceful.shutdown().await;
            Ok(())
        }))
    }
}


#[derive(Clone,Deserialize,PartialEq,Debug)]
pub struct HttpServerConfig {
    pub path: String,
    pub router: String,
    pub config: HttpListener
}



