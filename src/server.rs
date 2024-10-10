use crate::LambdaRuntimeApiClient;
use anyhow::Result;
use hyper::{
  body::{Body, Incoming},
  server::conn::http1,
  service::service_fn,
  Request, Response,
};
use hyper_util::rt::TokioIo;
use std::{future::Future, net::SocketAddr};
use tokio::{io, net::TcpListener};
use tracing::error;

/// A mock server for the Lambda Runtime API.
/// Use [`Self::bind`] to create a new server, and [`Self::serve`] to start serving requests.
///
/// If you want to handle each connection manually, use [`Self::handle_next`].
/// If you want to forward requests to the real Lambda Runtime API, use [`Self::passthrough`].
pub struct MockLambdaRuntimeApiServer(TcpListener);

impl MockLambdaRuntimeApiServer {
  /// Create a new server bound to the provided port.
  pub async fn bind(port: u16) -> io::Result<Self> {
    Ok(Self(
      TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).await?,
    ))
  }

  /// Handle the next incoming connection with the provided processor.
  pub async fn handle_next<ResBody, Fut>(
    &self,
    processor: impl Fn(Request<Incoming>) -> Fut + Send + Sync + 'static,
  ) where
    ResBody: hyper::body::Body + Send + 'static,
    <ResBody as Body>::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    Fut: Future<Output = Result<Response<ResBody>>> + Send,
    <ResBody as Body>::Data: Send,
  {
    let (stream, _) = self.0.accept().await.expect("Failed to accept connection");
    let io = TokioIo::new(stream);

    // in lambda's execution environment there is usually only one connection
    // but we can't rely on that, so spawn a task for each connection
    tokio::spawn(async move {
      if let Err(err) = http1::Builder::new()
        .serve_connection(io, service_fn(|req| async { processor(req).await }))
        .await
      {
        error!("Error serving connection: {:?}", err);
      }
    });
  }

  /// Block the current thread and handle connections with the processor in a loop.
  pub async fn serve<ResBody, Fut>(
    &self,
    processor: impl Fn(Request<Incoming>) -> Fut + Send + Sync + Clone + 'static,
  ) where
    Fut: Future<Output = Result<Response<ResBody>>> + Send,
    ResBody: hyper::body::Body + Send + 'static,
    <ResBody as Body>::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    <ResBody as Body>::Data: Send,
  {
    loop {
      self.handle_next(processor.clone()).await
    }
  }

  /// Block the current thread and handle connections in a loop,
  /// forwarding requests to a new [`LambdaRuntimeApiClient`], and responding with the client's response.
  pub async fn passthrough(&self) {
    self
      .serve(|req| async {
        // tested and it looks like creating the client every time is faster
        // than locking an Arc<Mutex<LambdaRuntimeApiClient>> and reuse it.
        // creating a new client and sending the request usually cost < 1ms.
        LambdaRuntimeApiClient::new().await?.forward(req).await
      })
      .await
  }
}
