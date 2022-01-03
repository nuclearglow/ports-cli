#[macro_use]
extern crate prettytable;

use colored::*;
use prettytable::{color, format, Attr, Cell, Row, Table};

mod ports;
mod processes;

fn main() {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row![l->"IP", l->"Port", l->"Process", l->"Owner", l->"CLI"]);

    let sockets = ports::get_open_ports();
    for socket in sockets {
        let color;

        if socket.process_info.iter().any(|info| info.is_current_user) {
            color = color::YELLOW
        } else {
            color = match socket.port {
                0..=1023 => color::RED,
                1024..=49151 => color::BRIGHT_GREEN,
                _ => color::GREEN,
            }
        }

        table.add_row(Row::new(vec![
            Cell::new(&socket.address.to_string().green()).with_style(Attr::ForegroundColor(color)),
            Cell::new(&socket.port.to_string().red()).with_style(Attr::ForegroundColor(color)),
            Cell::new(
                &socket
                    .process_info
                    .iter()
                    .map(|info| info.name.to_owned())
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .with_style(Attr::ForegroundColor(color)),
            Cell::new(
                &socket
                    .process_info
                    .iter()
                    .map(|info| info.owner.name().to_os_string().into_string().unwrap())
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .with_style(Attr::ForegroundColor(color)),
            Cell::new(
                &socket
                    .process_info
                    .iter()
                    .map(|info| match info.cmd.chars().count() {
                        0..=40 => info.cmd.to_owned(),
                        _ => format!("{}...", info.cmd[0..40].to_string()),
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
            )
            .with_style(Attr::ForegroundColor(color)),
        ]));
    }
    table.printstd();
}
