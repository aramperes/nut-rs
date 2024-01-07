use std::convert::TryInto;
use std::env;

use rups::blocking::Connection;
use rups::{Auth, ConfigBuilder};

fn main() -> rups::Result<()> {
    let host = env::var("NUT_HOST").unwrap_or_else(|_| "localhost".into());
    let port = env::var("NUT_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
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

    // Get server information
    println!("NUT server:");
    println!("\tVersion: {}", conn.get_server_version()?);
    println!("\tNetwork Version: {}", conn.get_network_version()?);

    // Print a list of all UPS devices
    println!("Connected UPS devices:");
    for (name, description) in conn.list_ups()? {
        println!("\t- Name: {}", name);
        println!("\t  Description: {}", description);
        println!("\t  Number of logins: {}", conn.get_num_logins(&name)?);

        // Get list of mutable variables
        let mutable_vars = conn.list_mutable_vars(&name)?;

        // List UPS variables (key = val)
        println!("\t  Mutable Variables:");
        for var in mutable_vars.iter() {
            println!("\t\t- {}", var);
            println!("\t\t  {:?}", conn.get_var_type(&name, var.name())?);
        }

        // List UPS immutable properties (key = val)
        println!("\t  Immutable Properties:");
        for var in conn.list_vars(&name)? {
            if mutable_vars.iter().any(|v| v.name() == var.name()) {
                continue;
            }
            println!("\t\t- {}", var);
            println!("\t\t  {:?}", conn.get_var_type(&name, var.name())?);
        }

        // List UPS commands
        println!("\t  Commands:");
        for cmd in conn.list_commands(&name)? {
            let description = conn.get_command_description(&name, &cmd)?;
            println!("\t\t- {} ({})", cmd, description);
        }
    }

    // Gracefully shut down the connection using the `LOGOUT` command
    conn.close()
}
