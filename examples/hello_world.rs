use aws_lambda_runtime_proxy::Proxy;

#[tokio::main]
async fn main() {
  // create the proxy server and the handler process using the default configuration
  let proxy = Proxy::default().spawn().await.unwrap();
  // forward all requests to the real Lambda Runtime API
  proxy.server.passthrough().await;
}
