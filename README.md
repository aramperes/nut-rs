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

## Getting Started

You'll need a running instance of the NUT daemon (`upsd`) and
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

Here is an example use of this library (`cargo run --example blocking`):

```rust
use std::env;
use std::net::ToSocketAddrs;

use nut_client::blocking::Connection;
use nut_client::{Auth, ConfigBuilder, Host};

fn main() -> nut_client::Result<()> {
    let addr = env::var("NUT_ADDR")
        .unwrap_or_else(|_| "localhost:3493".into())
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    let username = env::var("NUT_USER").ok();
    let password = env::var("NUT_PASSWORD").ok();
    let auth = username.map(|username| Auth::new(username, password));

    let config = ConfigBuilder::new()
        .with_host(Host::Tcp(addr))
        .with_auth(auth)
        .with_debug(false) // Turn this on for debugging network chatter
        .build();

    let mut conn = Connection::new(config)?;

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
