mod client;
mod server;

use std::env;

use anyhow::Result;
use tokio::process::{Child, Command};

pub use client::*;
pub use server::*;

/// This is the high level API for this crate,
/// which will create a [`MockLambdaRuntimeApiServer`] and a handler process.
///
/// You can also use [`MockLambdaRuntimeApiServer`] and [`LambdaRuntimeApiClient`] directly.
/// # Examples
/// ```
/// use aws_lambda_runtime_proxy::Proxy;
///
/// # async fn t1() {
/// // create the proxy server and the handler process using the default configuration
/// let proxy = Proxy::default().spawn().await.unwrap();
/// // forward all requests to the real Lambda Runtime API
/// proxy.server.passthrough().await;
/// # }
/// ```
#[derive(Default)]
pub struct Proxy {
  /// See [`Self::port`].
  pub port: Option<u16>,
  /// See [`Self::command`].
  pub command: Option<Command>,
}

impl Proxy {
  /// Create the handler command from the `argv[1..]`.
  /// For example if the command of the current process is `proxy node --help`
  /// then the handler command will be `node --help`.
  /// You can modify the handler command and pass it to [`Self::command`].
  /// # Examples
  /// ```
  /// use aws_lambda_runtime_proxy::Proxy;
  /// use std::process::Stdio;
  ///
  /// # async fn t1() {
  /// // retrieve the default handler command
  /// let mut command = Proxy::default_command().unwrap();
  ///
  /// // enhance the handler command
  /// command
  ///   // override environment variables for the handler process
  ///   .env("KEY", "VALUE")
  ///   // pipe the stdout and stderr of the handler process
  ///   .stdout(Stdio::piped())
  ///   .stderr(Stdio::piped());
  ///
  /// Proxy::default()
  ///   .command(command)
  ///   .spawn()
  ///   .await;
  /// # }
  /// ```
  pub fn default_command() -> Option<Command> {
    let mut cmd = Command::new(env::args().nth(1)?);
    cmd.args(env::args().skip(2));
    cmd.into()
  }

  /// Set the port of the proxy server.
  /// If not set, the port will be read from the environment variable `AWS_LAMBDA_RUNTIME_PROXY_PORT`,
  /// or default to `3000`.
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

  /// Spawn the proxy server and the handler process.
  /// The handler process will be spawned with the environment variable `AWS_LAMBDA_RUNTIME_API`
  /// set to the address of the proxy server.
  pub async fn spawn(self) -> Result<RunningProxy> {
    let port = self
      .port
      .or_else(|| {
        env::var("AWS_LAMBDA_RUNTIME_PROXY_PORT")
          .ok()
          .and_then(|s| s.parse().ok())
      })
      .unwrap_or(3000);

    let mut command = self
      .command
      .or_else(Self::default_command)
      .ok_or_else(|| anyhow::anyhow!("Handler command is not set."))?;
    command.env("AWS_LAMBDA_RUNTIME_API", format!("127.0.0.1:{}", port));

    let server = MockLambdaRuntimeApiServer::bind(port).await?;

    // server is ready, spawn the real handler process
    let handler = command.spawn()?;

    Ok(RunningProxy { server, handler })
  }
}

/// Created by [`Proxy::spawn`].
pub struct RunningProxy {
  pub server: MockLambdaRuntimeApiServer,
  /// The lambda handler process.
  pub handler: Child,
}
