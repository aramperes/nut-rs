use crate::parser::UpsdName;
use anyhow::Context;
use core::convert::TryInto;
use nut_client::blocking::Connection;

/// Lists each UPS on the upsd server, one per line.
pub fn list_devices(server: UpsdName, with_description: bool, debug: bool) -> anyhow::Result<()> {
    let mut conn = connect(server, debug)?;

    for (name, description) in conn.list_ups()? {
        if with_description {
            println!("{}: {}", name, description);
        } else {
            println!("{}", name);
        }
    }

    Ok(())
}

pub fn print_variable(server: UpsdName, variable: &str, debug: bool) -> anyhow::Result<()> {
    let ups_name = server
        .upsname
        .with_context(|| "ups name must be specified: <upsname>[@<hostname>[:<port>]]")?;
    let mut conn = connect(server, debug)?;

    let variable = conn.get_var(ups_name, variable)?;
    println!("{}", variable.value());

    Ok(())
}

pub fn list_variables(server: UpsdName, debug: bool) -> anyhow::Result<()> {
    let ups_name = server
        .upsname
        .with_context(|| "ups name must be specified: <upsname>[@<hostname>[:<port>]]")?;
    let mut conn = connect(server, debug)?;

    for var in conn.list_vars(ups_name)? {
        println!("{}", var);
    }

    Ok(())
}

fn connect(server: UpsdName, debug: bool) -> anyhow::Result<Connection> {
    let host = server.try_into()?;
    let config = nut_client::ConfigBuilder::new()
        .with_host(host)
        .with_debug(debug)
        .build();
    Connection::new(config).with_context(|| format!("Failed to connect to upsd: {}", server))
}
