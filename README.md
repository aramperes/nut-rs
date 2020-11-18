# nut-client

[![crates.io](https://img.shields.io/crates/v/nut-client.svg)](https://crates.io/crates/nut-client)
[![Documentation](https://docs.rs/nut-client/badge.svg)](https://docs.rs/nut-client)
[![MIT licensed](https://img.shields.io/crates/l/nut-client.svg)](./LICENSE)
[![CI](https://github.com/aramperes/nut-client-rs/workflows/CI/badge.svg)](https://github.com/aramperes/nut-client-rs/actions?query=workflow%3ACI)

A [Network UPS Tools](https://github.com/networkupstools/nut) (NUT) client library for Rust.

- Connect to `upsd`/`nut-server` using TCP
- Login with with username and password
- List UPS devices

## ⚠️ Safety Goggles Required ⚠️

Do not use this library with critical UPS devices. This library is in early development and I cannot
guarantee that it won't mess up your UPS configurations, and potentially cause catastrophic failure to your hardware.

Be careful and stay safe!

## Example

Check out the `examples` directory for more advanced examples.

```rust
use std::env;
use std::net::ToSocketAddrs;

use nut_client::{Auth, ConfigBuilder, Host};
use nut_client::blocking::Connection;

fn main() -> nut_client::Result<()> {
    // The TCP host:port for upsd/nut-server
    let addr = env::var("NUT_ADDR")
        .unwrap_or_else(|_| "localhost:3493".into())
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    // Username and password (optional)
    let username = env::var("NUT_USER").ok();
    let password = env::var("NUT_PASSWORD").ok();
    let auth = username.map(|username| Auth::new(username, password));

    // Build the config
    let config = ConfigBuilder::new()
        .with_host(Host::Tcp(addr))
        .with_auth(auth)
        .build();

    // Open a connection and login
    let mut conn = Connection::new(config)?;

    // Print a list of all UPS devices
    println!("Connected UPS devices:");
    for (id, description) in conn.list_ups()? {
        println!("\t- ID: {}", id);
        println!("\t  Description: {}", description);
    }

    Ok(())
}
```
