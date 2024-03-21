use hyper::{
  body::Incoming, client::conn::http1::SendRequest, server::conn::http1, service::service_fn,
  Request, Response,
};
use hyper_util::rt::TokioIo;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
  net::{TcpListener, TcpStream},
  sync::Mutex,
};

pub async fn start() {
  // start lambda runtime api client
  let address =
    std::env::var("AWS_LAMBDA_RUNTIME_API").expect("Missing AWS_LAMBDA_RUNTIME_API env var");
  let stream = TcpStream::connect(address)
    .await
    .expect("Failed to connect to runtime API");
  let io = TokioIo::new(stream);
  let (sender, _) = hyper::client::conn::http1::handshake(io)
    .await
    .expect("Failed to handshake with runtime API");
  let client = Arc::new(Mutex::new(sender));

  // start http server as the lambda runtime api proxy
  let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // TODO: customizable
  let listener = TcpListener::bind(addr)
    .await
    .expect("Failed to bind for proxy server");

  // proxy server is ready
  // TODO: spawn child process here

  // handle runtime api requests
  loop {
    let (stream, _) = listener
      .accept()
      .await
      .expect("Failed to accept connection");
    let io = TokioIo::new(stream);
    let client = client.clone();
    tokio::task::spawn(async move {
      if let Err(err) = http1::Builder::new()
        .serve_connection(
          io,
          service_fn(move |req| runtime_api_handler(client.clone(), req)),
        )
        .await
      {
        println!("Error serving connection: {:?}", err);
      }
    });
  }
}

async fn runtime_api_handler(
  client: Arc<Mutex<SendRequest<Incoming>>>,
  req: Request<Incoming>,
) -> Result<Response<Incoming>, hyper::Error> {
  client.lock().await.send_request(req).await
}
