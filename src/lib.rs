mod client;
mod server;

pub use client::*;
pub use server::*;

use tokio::process::{Child, Command};

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

  pub async fn spawn(self) -> RunningProxy {
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
    let server = MockLambdaRuntimeApiServer::bind(port).await;

    // client and server are both ready, spawn the real handler process
    let runtime = command.spawn().expect("Failed to spawn handler process");

    RunningProxy {
      client,
      server,
      runtime,
    }
  }
}

pub struct RunningProxy {
  pub client: LambdaRuntimeApiClient,
  pub server: MockLambdaRuntimeApiServer,
  pub runtime: Child,
}
