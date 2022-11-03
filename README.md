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

- `aws:<NAME>`: A JSON secret stored in
  [AWS Secrets Manager](https://aws.amazon.com/secrets-manager/). For
  performance, secrets are cached by the server for up to a minute.

All resolution of fields within templates is recursive. Additionally, you can
optionally specify a _default template_, which is merged with the selected
template whenever a configuration is requested.

## Deployment

Run the `cargo install cadre` and use the `cadre` command. We also offer a
Docker image at `ghcr.io/modal-labs/cadre`, which is automatically built from
the Dockerfile in this repository on each release.

To perform a new release, check that you have write access to this GitHub
repository and the `cadre` crate on crates.io, then just run a single command:
[`cargo release`](https://github.com/crate-ci/cargo-release).

## Authors

This library is created by the team behind [Modal](https://modal.com/).

- Luis Capelo ([@luiscape](https://twitter.com/luiscape)) – [Modal](https://modal.com/)
- Eric Zhang ([@ekzhang1](https://twitter.com/ekzhang1)) – [Modal](https://modal.com/)
