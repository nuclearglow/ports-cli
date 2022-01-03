mod ports;
mod processes;

fn main() {
    let ports = ports::get_open_ports();

    for port in ports {
        println!(
            "IP {} Port {} {:?}",
            port.address, port.port, port.process_info
        )
    }
}
