# cadre: _configure apps_

<p align="center">
<a href="https://crates.io/crates/cadre">
<kbd><img src="https://i.imgur.com/LOsl3Gm.png" width="640"></kbd><br>
<strong>cadre</strong>
</a>
</p>

High-performance, minimal remote configuration service that's extremely simple to set up and use.

It's just an in-memory JSON configuration service, updated from a human-readable web interface and persisted to a file on disk. You can access the configuration from a JSON web API. Plus, `cadre` is really easy to set up: just run a single binary that contains the entire application.

The web server is written in Rust and can easily support over 200,000 HTTP/2 requests per second running on a consumer MacBook Pro, tested using [vegeta](https://github.com/tsenart/vegeta).
