use hyper::{
  body::Incoming, client::conn::http1::SendRequest, server::conn::http1, service::service_fn,
};
use hyper_util::rt::TokioIo;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
  net::{TcpListener, TcpStream},
  sync::Mutex,
};

pub async fn start_proxy(port: u16) {
  let client = start_lambda_runtime_api_client().await;
  let server = create_http_server(port).await;

  // client and server are both ready
  // TODO: spawn child process here

  start_proxy_requests(client, server).await
}

async fn start_lambda_runtime_api_client() -> Arc<Mutex<SendRequest<Incoming>>> {
  let address =
    std::env::var("AWS_LAMBDA_RUNTIME_API").expect("Missing AWS_LAMBDA_RUNTIME_API env var");
  let stream = TcpStream::connect(address)
    .await
    .expect("Failed to connect to runtime API");
  let io = TokioIo::new(stream);
  let (sender, _) = hyper::client::conn::http1::handshake(io)
    .await
    .expect("Failed to handshake with runtime API");
  Arc::new(Mutex::new(sender))
}

async fn create_http_server(port: u16) -> TcpListener {
  let addr = SocketAddr::from(([127, 0, 0, 1], port));

  TcpListener::bind(addr)
    .await
    .expect("Failed to bind for proxy server")
}

async fn start_proxy_requests(client: Arc<Mutex<SendRequest<Incoming>>>, server: TcpListener) {
  // handle runtime api requests
  loop {
    let (stream, _) = server.accept().await.expect("Failed to accept connection");
    let io = TokioIo::new(stream);
    let client = client.clone();

    tokio::task::spawn(async move {
      if let Err(err) = http1::Builder::new()
        .serve_connection(
          io,
          service_fn(|req| async { client.lock().await.send_request(req).await }),
        )
        .await
      {
        println!("Error serving connection: {:?}", err);
      }
    });
  }
}
