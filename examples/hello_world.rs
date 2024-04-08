use aws_lambda_runtime_proxy::Proxy;

#[tokio::main]
async fn main() {
  let proxy = Proxy::default().spawn().await;
  // equals to:
  // let proxy = Proxy::default()
  //   .port(3000)
  //   .command(Proxy::default_command())
  //   .spawn()
  //   .await;

  proxy.server.passthrough().await;
}
