use anyhow::Context;

use nut_client::blocking::Connection;
use nut_client::Config;

/// Lists each UPS on the upsd server, one per line.
pub fn list_devices(config: Config, with_description: bool) -> anyhow::Result<()> {
    let mut conn = connect(config)?;

    for (name, description) in conn.list_ups()? {
        if with_description {
            println!("{}: {}", name, description);
        } else {
            println!("{}", name);
        }
    }

    logout(conn)
}

pub fn print_variable(config: Config, ups_name: &str, variable: &str) -> anyhow::Result<()> {
    let mut conn = connect(config)?;

    let variable = conn.get_var(ups_name, variable)?;
    println!("{}", variable.value());

    logout(conn)
}

pub fn list_variables(config: Config, ups_name: &str) -> anyhow::Result<()> {
    let mut conn = connect(config)?;

    for var in conn.list_vars(ups_name)? {
        println!("{}", var);
    }

    logout(conn)
}

pub fn list_clients(config: Config, ups_name: &str) -> anyhow::Result<()> {
    let mut conn = connect(config)?;

    for client_ip in conn.list_clients(ups_name)? {
        println!("{}", client_ip);
    }

    logout(conn)
}

fn connect(config: Config) -> anyhow::Result<Connection> {
    Connection::new(&config).with_context(|| format!("Failed to connect to upsd: {:?}", &config))
}

fn logout(conn: Connection) -> anyhow::Result<()> {
    conn.close().with_context(|| "Failed to close gracefully")
}
