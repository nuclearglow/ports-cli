#[macro_use]
extern crate prettytable;
use prettytable::{format, Table};
use std::ffi::OsStr;

mod ports;
mod processes;

fn main() {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row![l->"IP", l->"Port", l->"Process", l->"Owner", l->"CLI"]);

    let sockets = ports::get_open_ports();
    for socket in sockets {
        table.add_row(row![
            l->socket.address,
            l->socket.port,
            l->
                socket.process_info
                    .iter()
                    .map(|info| info.name.to_owned())
                    .collect::<Vec<String>>()
                    .join("\n"),
            l->
                socket.process_info
                    .iter()
                    .map(|info| info.owner.name().to_os_string().into_string().unwrap())
                    .collect::<Vec<String>>()
                    .join("\n"),
            l->socket.process_info
                    .iter()
                    .map(|info| match info.cmd.chars().count() {
                        0..=40 => info.cmd.to_owned(),
                        _ => format!("{}...", info.cmd[0..40].to_string())
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
        ]);
    }
    println!("{}", table);
}
