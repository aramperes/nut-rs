use std::env;

use nut_client::tokio::Connection;
use nut_client::{Auth, ConfigBuilder};
use std::convert::TryInto;

#[tokio::main]
async fn main() -> nut_client::Result<()> {
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

    let mut conn = Connection::new(&config).await?;

    // Print a list of all UPS devices
    println!("Connected UPS devices:");
    for (name, description) in conn.list_ups().await? {
        println!("\t- Name: {}", name);
        println!("\t  Description: {}", description);

        // List UPS variables (key = val)
        println!("\t  Variables:");
        for var in conn.list_vars(&name).await? {
            println!("\t\t- {}", var);
        }
    }

    Ok(())
}
