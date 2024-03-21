use hyper::{
  body::Incoming, client::conn::http1::SendRequest, server::conn::http1, service::service_fn,
  Request, Response,
};
use hyper_util::rt::TokioIo;
use std::{future::Future, net::SocketAddr, sync::Arc};
use tokio::{
  net::{TcpListener, TcpStream},
  process::{Child, Command},
  sync::Mutex,
};

#[derive(Default)]
pub struct Proxy {
  pub port: Option<u16>,
  pub command: Option<Command>,
}

impl Proxy {
  pub fn port(mut self, port: u16) -> Self {
    self.port = Some(port);
    self
  }
  pub fn command(mut self, cmd: Command) -> Self {
    self.command = Some(cmd);
    self
  }

  pub async fn start<F>(
    self,
    processor: impl Fn(Request<Incoming>, Arc<Mutex<SendRequest<Incoming>>>, Arc<Mutex<Child>>) -> F,
  ) where
    F: Future<Output = hyper::Result<Response<Incoming>>>,
  {
    let port = self
      .port
      .or_else(|| {
        std::env::var("AWS_LAMBDA_RUNTIME_PROXY_PORT")
          .ok()
          .and_then(|s| s.parse().ok())
      })
      .unwrap_or(3000);

    let mut command = self.command.unwrap_or_else(|| {
      let mut cmd = Command::new(std::env::args().nth(1).expect("Missing handler command"));
      cmd.args(std::env::args().skip(2));
      cmd
    });
    command.env("AWS_LAMBDA_RUNTIME_API", format!("127.0.0.1:{}", port));

    let client = start_lambda_runtime_api_client().await;
    let server = create_http_server(port).await;

    // client and server are both ready, spawn the real handler process
    let child = command.spawn().expect("Failed to spawn handler process");

    start_proxy_requests(client, server, child, processor).await
  }
}

async fn start_lambda_runtime_api_client() -> SendRequest<Incoming> {
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

async fn create_http_server(port: u16) -> TcpListener {
  let addr = SocketAddr::from(([127, 0, 0, 1], port));

  TcpListener::bind(addr)
    .await
    .expect("Failed to bind for proxy server")
}

async fn start_proxy_requests<F>(
  client: SendRequest<Incoming>,
  server: TcpListener,
  child: Child,
  processor: impl Fn(Request<Incoming>, Arc<Mutex<SendRequest<Incoming>>>, Arc<Mutex<Child>>) -> F,
) where
  F: Future<Output = hyper::Result<Response<Incoming>>>,
{
  // TODO： prevent Arc Mutex?
  let client = Arc::new(Mutex::new(client));
  let child = Arc::new(Mutex::new(child));

  // handle runtime api requests
  loop {
    let (stream, _) = server.accept().await.expect("Failed to accept connection");
    let io = TokioIo::new(stream);

    if let Err(err) = http1::Builder::new()
      .serve_connection(
        io,
        service_fn(|req| {
          let client = client.clone();
          let child = child.clone();
          async { processor(req, client, child).await }
        }),
      )
      .await
    {
      println!("Error serving connection: {:?}", err);
    }
  }
}
