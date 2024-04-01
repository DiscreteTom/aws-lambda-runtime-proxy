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
  /// Create the handler command from the `argv[1..]`.
  /// For example if the command of the current process is `proxy node --help`
  /// then the handler command will be `node --help`.
  /// You can modify the handler command and pass it to [`Self::command`].
  /// # Examples
  /// ```
  /// use lambda_runtime_proxy::Proxy;
  /// use std::process::Stdio;
  ///
  /// #[tokio::main]
  /// async fn main {
  ///   let command = Proxy::default_command()
  ///     // override environment variables for the handler process
  ///     .env("KEY", "VALUE")
  ///     // pipe the stdout and stderr of the handler process
  ///     .stdout(Stdio::piped())
  ///     .stderr(Stdio::piped())
  ///   Proxy::default()
  ///     .command(command)
  ///     .spawn().await;
  /// }
  /// ```
  pub fn default_command() -> Command {
    let mut cmd = Command::new(std::env::args().nth(1).expect("Missing handler command"));
    cmd.args(std::env::args().skip(2));
    cmd
  }

  /// Set the port of the proxy server.
  /// If not set, the port will be read from the environment variable `AWS_LAMBDA_RUNTIME_PROXY_PORT`,
  /// or default to 3000.
  pub fn port(mut self, port: u16) -> Self {
    self.port = Some(port);
    self
  }

  /// Set the command of the handler process.
  /// If not set, the command will be created using [`Self::default_command`].
  pub fn command(mut self, cmd: Command) -> Self {
    self.command = Some(cmd);
    self
  }

  /// Spawn the proxy server, lambda runtime api client and the handler process.
  /// The handler process will be spawned with the environment variable `AWS_LAMBDA_RUNTIME_API`
  /// set to the address of the proxy server.
  pub async fn spawn(self) -> RunningProxy {
    let port = self
      .port
      .or_else(|| {
        std::env::var("AWS_LAMBDA_RUNTIME_PROXY_PORT")
          .ok()
          .and_then(|s| s.parse().ok())
      })
      .unwrap_or(3000);

    let mut command = self.command.unwrap_or_else(|| Self::default_command());
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
