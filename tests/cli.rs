use std::net::{TcpListener, UdpSocket};
use std::process::{Command, Output};

fn ports(args: &[&str]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_ports"))
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("failed to run ports");
    assert!(
        output.status.success(),
        "ports {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// Find the output row for `port`, split into whitespace-separated columns.
fn row_for(stdout: &[u8], protocol: &str, port: u16) -> Option<Vec<String>> {
    let port = port.to_string();
    String::from_utf8_lossy(stdout).lines().find_map(|line| {
        let columns: Vec<String> = line.split_whitespace().map(str::to_owned).collect();
        (columns.get(1).map(String::as_str) == Some(protocol) && columns.get(2) == Some(&port))
            .then_some(columns)
    })
}

#[test]
fn help_and_version() {
    let help = ports(&["--help"]);
    assert!(String::from_utf8_lossy(&help.stdout).contains("--mine"));

    let version = ports(&["--version"]);
    assert!(String::from_utf8_lossy(&version.stdout).contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn lists_own_tcp_listener() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let output = ports(&["--tcp", "--ipv4", "--mine"]);
    let row = row_for(&output.stdout, "TCP", port).unwrap_or_else(|| {
        panic!(
            "port {port} not listed:\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    });
    assert_eq!(row[0], "127.0.0.1");
    assert_eq!(row[5], std::process::id().to_string());
}

#[test]
fn lists_own_udp_socket() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = socket.local_addr().unwrap().port();

    let output = ports(&["--udp", "--mine"]);
    let row = row_for(&output.stdout, "UDP", port).unwrap_or_else(|| {
        panic!(
            "port {port} not listed:\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    });
    assert_eq!(row[5], std::process::id().to_string());
}

#[test]
fn protocol_filter_excludes_other_protocol() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let output = ports(&["--udp"]);
    assert!(row_for(&output.stdout, "TCP", port).is_none());
}
