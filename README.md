# cadre: _configure apps_

<p align="center">
<a href="https://crates.io/crates/cadre">
<kbd><img src="https://i.imgur.com/hIRb9PK.png" width="640"></kbd><br>
<strong>cadre</strong>
</a>
</p>

High-performance, minimal remote configuration service that's extremely simple
to set up and use.

Configuration is backed by S3, updated from a human-readable web interface and
persisted to a file on disk. Multiple environments are supported and accessible
from a JSON web API. Plus, `cadre` is really easy to set up: just run a single
binary that contains the entire application, passing in an S3 bucket name.

The web server is written in Rust and can easily support over 80,000 HTTP/2
requests per second running on a consumer MacBook Pro, tested using
[vegeta](https://github.com/tsenart/vegeta). It's also horizontally scalable.

## Template Syntax

Template values have keys that are prefixed with the `*` character. These values
are resolved at request-time.

- `aws:<NAME>`: A secret stored in
  [AWS Secrets Manager](https://aws.amazon.com/secrets-manager/). For
  performance, secrets are cached by the server for up to a minute.

All resolution of fields within templates is recursive.
