# Benchmark

## Deploy

```bash
# run in the root of the project
RUSTFLAGS="-C link-arg=-s" cargo build --release --target x86_64-unknown-linux-musl --example hello_world
mkdir -p benchmark/layer
cp target/x86_64-unknown-linux-musl/release/examples/hello_world benchmark/layer/
cp benchmark/scripts/entry.sh benchmark/layer/

cd benchmark
sam build
sam deploy # maybe add '-g' for the first time
cd ..
```

In one line

```bash
RUSTFLAGS="-C link-arg=-s" cargo build --release --target x86_64-unknown-linux-musl --example hello_world && mkdir -p benchmark/layer && cp target/x86_64-unknown-linux-musl/release/examples/hello_world benchmark/layer/ && cp benchmark/scripts/entry.sh benchmark/layer/ && cd benchmark && sam build && sam deploy && cd ..
```

## Test

The SAM will deploy the stack with an API. Test it with [`plow`](https://github.com/six-ddc/plow):

```bash
# e.g. disable the layer, test 1000 requests
plow -n 1000 https://abcdefgh.execute-api.us-east-1.amazonaws.com/Prod/disabled
# e.g. enable the layer, test 1000 requests
plow -n 1000 https://abcdefgh.execute-api.us-east-1.amazonaws.com/Prod/enabled
```

## Result

> [!NOTE]
> The result is tested on an EC2 instance in the same region as the deployed stack. The latency data is the http request latency, not the lambda handler duration. The actual lambda handler duration is about 2ms.
> We should only focus on the latency difference between the two tests, instead of the absolute value. For the actual lambda handler duration, checkout lambda metrics in CloudWatch.

Conclusions: the proxy will introduce less than 2ms latency, which is acceptable for most use cases.

### Without Proxy

```
Summary:
  Elapsed      20.2s
  Count         1000
    2xx         1000
  RPS         49.322
  Reads    0.026MB/s
  Writes   0.006MB/s

Statistics    Min       Mean     StdDev      Max
  Latency   12.611ms  20.257ms  12.035ms  319.727ms
  RPS        30.95     49.24      4.83      53.01

Latency Percentile:
  P50         P75       P90       P95       P99       P99.9     P99.99
  19.065ms  21.145ms  23.156ms  25.251ms  45.889ms  319.727ms  319.727ms

Latency Histogram:
  18.288ms   575  57.50%
  20.565ms   256  25.60%
  24.004ms   139  13.90%
  29.619ms    25   2.50%
  54.705ms     4   0.40%
  180.863ms    1   0.10%
```

### With Proxy

```
Summary:
  Elapsed      21.2s
  Count         1000
    2xx         1000
  RPS         46.986
  Reads    0.025MB/s
  Writes   0.006MB/s

Statistics    Min       Mean     StdDev      Max
  Latency   13.993ms  21.257ms  12.978ms  399.597ms
  RPS        27.94     46.99      4.84      51.01

Latency Percentile:
  P50        P75       P90       P95      P99      P99.9     P99.99
  20.08ms  21.799ms  24.251ms  26.71ms  47.84ms  399.597ms  399.597ms

Latency Histogram:
  19.676ms  652  65.20%
  22.426ms  257  25.70%
  26.177ms   72   7.20%
  38.603ms   16   1.60%
  54.277ms    3   0.30%
```
