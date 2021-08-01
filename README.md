# nut-client

[![crates.io](https://img.shields.io/crates/v/nut-client.svg)](https://crates.io/crates/nut-client)
[![Documentation](https://docs.rs/nut-client/badge.svg)](https://docs.rs/nut-client)
[![MIT licensed](https://img.shields.io/crates/l/nut-client.svg)](./LICENSE)
[![CI](https://github.com/aramperes/nut-client-rs/workflows/CI/badge.svg)](https://github.com/aramperes/nut-client-rs/actions?query=workflow%3ACI)

A [Network UPS Tools](https://github.com/networkupstools/nut) (NUT) client library for Rust.

- Connect to `upsd`/`nut-server` using TCP
- Login with username and password
- List UPS devices
- List variables for a UPS device
- Connect securely with SSL (optional feature)
- Supports blocking and async (Tokio)

## Getting Started

You'll need a running instance of the NUT daemon (`upsd`, version >= 2.6.4) and
a [compatible UPS device](https://networkupstools.org/stable-hcl.html)
to use this library:

1. [Install NUT](https://networkupstools.org/docs/user-manual.chunked/ar01s05.html)
2. [Configure and launch upsd](https://networkupstools.org/docs/user-manual.chunked/ar01s06.html)

Verify that your UPS is connected using the built-in `upsc` tool:

```bash
upsc myupsname@localhost ups.status
```

## Example

The [rupsc](https://github.com/aramperes/nut-client-rs/tree/master/rupsc)
CLI is written using this library, and is a clone of NUT's
built-in [upsc](https://networkupstools.org/docs/man/upsc.html) tool.

Below is a sample program using this library (`cargo run --example blocking`).

You can also run the async version of this code using
`cargo run --example async --features async-rt` (source: `nut-client/examples/async.rs`).

```rust
// nut-client/examples/blocking.rs

use std::env;

use nut_client::blocking::Connection;
use nut_client::{Auth, ConfigBuilder};
use std::convert::TryInto;

fn main() -> nut_client::Result<()> {
    let host = env::var("NUT_HOST").unwrap_or_else(|_| "localhost".into());
    let port = env::var("NUT_PORT")
        .ok()
        .map(|s| s.parse::<u16>().ok())
        .flatten()
        .unwrap_or(3493);

    let username = env::var("NUT_USER").ok();
    let password = env::var("NUT_PASSWORD").ok();
    let auth = username.map(|username| Auth::new(username, password));

    let config = ConfigBuilder::new()
        .with_host((host, port).try_into().unwrap_or_default())
        .with_auth(auth)
        .with_debug(false) // Turn this on for debugging network chatter
        .build();

    let mut conn = Connection::new(&config)?;

    // Print a list of all UPS devices
    println!("Connected UPS devices:");
    for (name, description) in conn.list_ups()? {
        println!("\t- Name: {}", name);
        println!("\t  Description: {}", description);

        // List UPS variables (key = val)
        println!("\t  Variables:");
        for var in conn.list_vars(&name)? {
            println!("\t\t- {}", var);
        }
    }

    Ok(())
}
```

## SSL

You can turn on SSL support by adding `.with_ssl(true)` in the `ConfigBuilder`. This requires the `ssl` feature, which
uses `rustls` under the hood.

Note that, by default, `.with_ssl(true)` will enable **strict** verification. This means it will verify the server
certificate's DNS entries, check for revocation, and verify the chain using the local root trust. You must also ensure
that the connection hostname is a valid DNS name (e.g. `localhost`, not `127.0.0.1`).

If the server is using a self-signed certificate, and you'd like to ignore the strict validation, you can add
`.with_insecure_ssl(true)` along with `.with_ssl(true)`.

## Async (Tokio)

The `nut-client` library supports async network requests. This requires the `async` feature, which uses Tokio v1 under
the hood.

For SSL support, you must use the `async-ssl` feature as well.
