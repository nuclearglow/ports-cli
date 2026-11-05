use std::net::{SocketAddr, TcpListener, UdpSocket};
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

/// All output rows for the socket `addr`, split into whitespace-separated columns.
///
/// Rows are matched on address *and* port: other processes may hold the same port
/// number on a different address, which would otherwise be picked up by mistake.
fn rows_for(stdout: &[u8], protocol: &str, addr: SocketAddr) -> Vec<Vec<String>> {
    let address = addr.ip().to_string();
    let port = addr.port().to_string();
    String::from_utf8_lossy(stdout)
        .lines()
        .map(|line| line.split_whitespace().map(str::to_owned).collect())
        .filter(|columns: &Vec<String>| {
            columns.len() > 5
                && columns[0] == address
                && columns[1] == protocol
                && columns[2] == port
        })
        .collect()
}

/// Assert that `addr` is listed as held by this test process.
///
/// Other rows for the same socket are allowed: a process spawned concurrently by
/// another test can briefly inherit the socket before it executes `ports`.
fn assert_listed_by_self(output: &Output, protocol: &str, addr: SocketAddr) {
    let pid = std::process::id().to_string();
    let rows = rows_for(&output.stdout, protocol, addr);
    assert!(
        rows.iter().any(|row| row[5] == pid),
        "{protocol} {addr} not listed for pid {pid}:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
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
    let addr = listener.local_addr().unwrap();

    let output = ports(&["--tcp", "--ipv4", "--mine"]);
    assert_listed_by_self(&output, "TCP", addr);
}

#[test]
fn lists_own_udp_socket() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let addr = socket.local_addr().unwrap();

    let output = ports(&["--udp", "--ipv4", "--mine"]);
    assert_listed_by_self(&output, "UDP", addr);
}

#[test]
fn protocol_filter_excludes_other_protocol() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let output = ports(&["--udp"]);
    assert!(rows_for(&output.stdout, "TCP", addr).is_empty());
}
