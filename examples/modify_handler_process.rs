use aws_lambda_runtime_proxy::Proxy;
use std::process::Stdio;

#[tokio::main]
async fn main() {
  let mut handler = Proxy::default_command()
    // capture stdout
    .stdout(Stdio::piped())
    // override environment variables for the handler process
    .env("KEY", "VALUE")
    // pass additional arguments to the handler process
    .arg("--help")
    .spawn()
    .unwrap();

  // take the stdout
  let _stdout = handler.stdout.take().unwrap();
  // do something with the stdout

  // wait until the handler process exits
  handler.wait().await.unwrap();
}
