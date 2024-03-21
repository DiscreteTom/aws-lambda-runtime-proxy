use hyper::{body::Incoming, client::conn::http1::SendRequest};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

pub type LambdaRuntimeApiClient = SendRequest<Incoming>;

pub async fn start_lambda_runtime_api_client() -> LambdaRuntimeApiClient {
  let address =
    std::env::var("AWS_LAMBDA_RUNTIME_API").expect("Missing AWS_LAMBDA_RUNTIME_API env var");
  let stream = TcpStream::connect(address)
    .await
    .expect("Failed to connect to runtime API");
  let io = TokioIo::new(stream);
  let (sender, conn) = hyper::client::conn::http1::handshake(io)
    .await
    .expect("Failed to handshake with runtime API");

  // Spawn a task to poll the connection, driving the HTTP state
  tokio::task::spawn(async move {
    if let Err(err) = conn.await {
      println!("Connection failed: {:?}", err);
    }
  });

  sender
}
