use std::env;
use std::net::ToSocketAddrs;

use nut_client::{Auth, ConfigBuilder, Host};
use nut_client::blocking::Connection;

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
        .build();

    let mut conn = Connection::new(config)?;

    // Print a list of all UPS devices
    println!("Connected UPS devices:");
    for (id, description) in conn.list_ups()? {
        println!("\t- ID: {}", id);
        println!("\t  Description: {}", description);
    }

    Ok(())
}
