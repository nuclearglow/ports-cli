//! A minimal CLI tool to list open ports and associated processes.

#[macro_use]
extern crate prettytable;

use clap::Parser;
use prettytable::{format, Cell, Row, Table};

mod constants;
mod ports;
mod processes;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(about, version)]
pub struct Args {
    /// show only ports opened by myself
    #[clap(short, long)]
    mine: bool,
    /// display UDP ports
    #[clap(long)]
    udp: bool,
    /// display IPv6 addresses with open ports
    #[clap(long)]
    ipv6: bool,
}

fn main() {
    let args = Args::parse();

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(
        row![l->"IP", c->"Protocol", l->"Port", l->"Owner", l->"Process", l->"PID", l->"Command"],
    );

    let sockets = ports::get_open_ports(args);
    for socket in sockets {
        let colorspec;

        // port opened by current user -> all yellow
        if socket.process_info.iter().any(|info| info.is_current_user) {
            colorspec = "FY"
        } else {
            colorspec = match socket.port {
                // The Well Known Ports are those from 0 through 1023 -> red
                0..=1023 => "Fr",
                // The Registered Ports are those from 1024 through 49151. -> Bright Green
                1024..=49151 => "FG",
                // The Dynamic and Private Ports are those from 49152 through 65535. -> Green
                _ => "Fg",
            }
        }

        table.add_row(Row::new(vec![
            Cell::new(&socket.address.to_string()).style_spec(colorspec),
            Cell::new(&socket.protocol).style_spec(&format!("c{:?}", colorspec)),
            Cell::new(&socket.port.to_string()).style_spec(colorspec),
            Cell::new(
                &socket
                    .process_info
                    .iter()
                    .map(|info| info.owner.name().to_os_string().into_string().unwrap())
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .style_spec(colorspec),
            Cell::new(
                &socket
                    .process_info
                    .iter()
                    .map(|info| info.name.to_owned())
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .style_spec(colorspec),
            Cell::new(
                &socket
                    .process_info
                    .iter()
                    .map(|info| format!("{}", info.pid))
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .style_spec(colorspec),
            Cell::new(
                &socket
                    .process_info
                    .iter()
                    .map(|info| match info.cmd.chars().count() {
                        0..=constants::COMMAND_DISPLAY_MAX_LENGTH => info.cmd.to_owned(),
                        _ => format!(
                            "{}...",
                            info.cmd[0..constants::COMMAND_DISPLAY_MAX_LENGTH].to_string()
                        ),
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .style_spec(colorspec),
        ]));
    }
    table.printstd();
}
