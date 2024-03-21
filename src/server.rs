use crate::LambdaRuntimeApiClient;
use hyper::{body::Incoming, server::conn::http1, service::service_fn, Request, Response};
use hyper_util::rt::TokioIo;
use std::{future::Future, net::SocketAddr};
use tokio::{net::TcpListener, sync::Mutex};

pub struct MockLambdaRuntimeApiServer(TcpListener);

impl MockLambdaRuntimeApiServer {
  pub(crate) async fn bind(port: u16) -> Self {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    Self(
      TcpListener::bind(addr)
        .await
        .expect("Failed to bind for proxy server"),
    )
  }

  pub async fn handle_next<F>(&self, processor: impl Fn(Request<Incoming>) -> F)
  where
    F: Future<Output = hyper::Result<Response<Incoming>>>,
  {
    let (stream, _) = self.0.accept().await.expect("Failed to accept connection");
    let io = TokioIo::new(stream);

    if let Err(err) = http1::Builder::new()
      .serve_connection(io, service_fn(|req| async { processor(req).await }))
      .await
    {
      println!("Error serving connection: {:?}", err);
    }
  }

  /// Block the current thread and handle requests in a loop.
  pub async fn serve<F>(&self, processor: impl Fn(Request<Incoming>) -> F + Clone)
  where
    F: Future<Output = hyper::Result<Response<Incoming>>>,
  {
    loop {
      self.handle_next(processor.clone()).await
    }
  }

  /// Block the current thread and handle requests in a loop,
  /// forwarding them to the provided client, and responding with the client's response.
  pub async fn passthrough(&self, client: LambdaRuntimeApiClient) {
    let client = Mutex::new(client);
    self
      .serve(|req| async { client.lock().await.send_request(req).await })
      .await
  }
}
