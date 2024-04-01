use hyper::{
  body::Body,
  client::conn::http1::{self, SendRequest},
};
use hyper_util::rt::TokioIo;
use std::ops::{Deref, DerefMut};
use tokio::net::TcpStream;

pub struct LambdaRuntimeApiClient<ReqBody>(SendRequest<ReqBody>);

impl<ReqBody> Deref for LambdaRuntimeApiClient<ReqBody> {
  type Target = SendRequest<ReqBody>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
impl<ReqBody> DerefMut for LambdaRuntimeApiClient<ReqBody> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<ReqBody: Body + Send + 'static> LambdaRuntimeApiClient<ReqBody> {
  /// Create a new client and connect to the runtime API.
  pub async fn start() -> Self
  where
    ReqBody::Data: Send,
    ReqBody::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
  {
    let address =
      std::env::var("AWS_LAMBDA_RUNTIME_API").expect("Missing AWS_LAMBDA_RUNTIME_API env var");

    let stream = TcpStream::connect(address)
      .await
      .expect("Failed to connect to runtime API");
    let io = TokioIo::new(stream);
    let (sender, conn) = http1::handshake(io)
      .await
      .expect("Failed to handshake with runtime API");

    // Spawn a task to poll the connection, driving the HTTP state
    tokio::task::spawn(async move {
      if let Err(err) = conn.await {
        println!("Connection failed: {:?}", err);
      }
    });

    Self(sender)
  }
}
