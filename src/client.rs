use anyhow::Result;
use http_body_util::{BodyExt, Full};
use hyper::{
  body::{Body, Bytes, Incoming},
  client::conn::http1::{self, SendRequest},
  Request, Response,
};
use hyper_util::rt::TokioIo;
use std::ops::{Deref, DerefMut};
use tokio::net::TcpStream;
use tracing::error;

/// An http client for the Lambda Runtime API.
pub struct LambdaRuntimeApiClient<ReqBody>(SendRequest<ReqBody>);

// TODO: use AsRef instead of Deref?
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
  pub async fn new() -> Result<Self>
  where
    ReqBody::Data: Send,
    ReqBody::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
  {
    let address = std::env::var("AWS_LAMBDA_RUNTIME_API")?; // TODO: cache the result?

    // TODO: re-use the connection?
    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let (sender, conn) = http1::handshake(io).await?;

    // Spawn a task to poll the connection, driving the HTTP state
    tokio::spawn(async move {
      if let Err(err) = conn.await {
        error!("Connection failed: {:?}", err);
      }
    });

    Ok(Self(sender))
  }
}

impl LambdaRuntimeApiClient<Incoming> {
  /// Send a request to the runtime API and return the response.
  pub async fn forward(req: Request<Incoming>) -> Result<Response<Full<Bytes>>> {
    // tested and it looks like we create the client every time is faster than lock a Arc<Mutex<>> and reuse it.
    // create a new client and send the request usually cost < 1ms.
    let res = LambdaRuntimeApiClient::new()
      .await?
      .send_request(req)
      .await?;
    let (parts, body) = res.into_parts();
    let bytes = body.collect().await?.to_bytes();
    Ok(Response::from_parts(parts, Full::new(bytes)))

    // TODO: why we can't just return `LambdaRuntimeApiClient::new().await.send_request(req).await`?
    // tested and it works but will add ~40ms latency when serving API GW event (maybe for all big event), why?
    // maybe because of the `Incoming` type? can we stream the body instead of buffering it?
  }
}
