///! # rupsc
///! A demo program to display UPS variables.
///! This a Rust clone of [upsc](https://github.com/networkupstools/nut/blob/master/clients/upsc.c).
///!
///! P.S.: pronounced "r-oopsie".
mod cmd;
mod parser;

use crate::parser::UpsdName;
use anyhow::Context;
use clap::{App, Arg};
use core::convert::TryInto;

fn main() -> anyhow::Result<()> {
    let args = App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            Arg::with_name("list")
                .short("l")
                .conflicts_with_all(&["list-full", "clients"])
                .takes_value(false)
                .help("Lists each UPS on <hostname>, one per line."),
        )
        .arg(
            Arg::with_name("list-full")
                .short("L")
                .conflicts_with_all(&["list", "clients"])
                .takes_value(false)
                .help("Lists each UPS followed by its description (from ups.conf)."),
        )
        .arg(
            Arg::with_name("clients")
                .short("c")
                .conflicts_with_all(&["list", "list-full"])
                .takes_value(false)
                .help("Lists each client connected on <upsname>, one per line."),
        )
        .arg(
            Arg::with_name("upsd-server")
                .required(false)
                .value_name("[upsname][@<hostname>[:<port>]]")
                .help("upsd server (with optional upsname, if applicable)."),
        )
        .arg(
            Arg::with_name("variable")
                .required(false)
                .value_name("variable")
                .help("Optional, display this variable only."),
        )
        .get_matches();

    let server: parser::UpsdName = args.value_of("upsd-server").map_or_else(
        || Ok(UpsdName::default()),
        |s| s.try_into().with_context(|| "Invalid upsd server name"),
    )?;

    if args.is_present("list") {
        return cmd::list_devices(server, false);
    }

    if args.is_present("list-full") {
        return cmd::list_devices(server, true);
    }

    if args.is_present("clients") {
        todo!("listing clients")
    }

    // Fallback: prints one variable (or all of them)
    if let Some(variable) = args.value_of("variable") {
        cmd::print_variable(server, variable)
    } else {
        cmd::list_variables(server)
    }
}
