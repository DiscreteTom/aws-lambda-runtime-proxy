# CHANGELOG

## v0.2.1

- Feat: add `LambdaRuntimeApiClient::forward`.

## v0.2.0

- **_Breaking Change_**: stricter trait bound for `MockLambdaRuntimeApiServer` methods.
- **_Breaking Change_**: `MockLambdaRuntimeApiServer::passthrough` will create a new client every time.
- **_Breaking Change_**: remove `RunningProxy::client`. `RunningProxy` and `Proxy::spawn` doesn't have generic params anymore.
- **_Breaking Change_**: rename `LambdaRuntimeApiClient::start` to `new`.
- Perf: optimize performance for `MockLambdaRuntimeApiServer::passthrough`.

## v0.1.1

Exclude unnecessary files from the published package.

## v0.1.0

The initial release.
