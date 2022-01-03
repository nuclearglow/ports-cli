#[macro_use]
extern crate prettytable;
use prettytable::{format, Table};

mod ports;
mod processes;

fn main() {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![l->"IP", l->"Port", l->"Command"]);

    let ports = ports::get_open_ports();
    for port in ports {
        table.add_row(row![
            l->port.address,
            l->port.port,
            l->
                port.process_info
                    .iter()
                    .map(|info| info.name.to_owned())
                    .collect::<Vec<String>>()
                    .join(" "),

        ]);
    }
    println!("{}", table);
}
