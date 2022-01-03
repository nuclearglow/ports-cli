use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use std::cmp::Ordering;
use std::net;

use crate::processes::{get_process_info, ProcessInfo};
use crate::Args;

#[derive(Debug)]
pub struct PortInfo {
    pub address: net::IpAddr,
    pub port: u16,
    pub protocol: String,
    pub process_info: Vec<ProcessInfo>,
}
impl From<netstat2::SocketInfo> for PortInfo {
    fn from(socket_info: netstat2::SocketInfo) -> Self {
        let protocol = match socket_info.protocol_socket_info {
            ProtocolSocketInfo::Tcp(_) => "TCP",
            ProtocolSocketInfo::Udp(_) => "UDP",
        };
        Self {
            address: socket_info.local_addr(),
            port: socket_info.local_port(),
            protocol: protocol.to_string(),
            process_info: get_process_info(socket_info.associated_pids),
        }
    }
}

/// get the PortInfo list:
/// - get socket info according to flags --ipv6 and --udp
/// - filter open tcp ports
/// - assemble PortInfo
/// - filter according to flag --mine
/// - sort by port number ascending
pub fn get_open_ports(args: Args) -> Vec<PortInfo> {
    let af_flags = match args.ipv6 {
        true => AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        false => AddressFamilyFlags::IPV4,
    };

    let proto_flags = match args.udp {
        true => ProtocolFlags::TCP | ProtocolFlags::UDP,
        false => ProtocolFlags::TCP,
    };

    let sockets = get_sockets_info(af_flags, proto_flags).unwrap_or_default();

    let mut ports: Vec<PortInfo> = sockets
        .into_iter()
        .filter(|socket_info| match &socket_info.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => tcp.state == TcpState::Listen,
            ProtocolSocketInfo::Udp(_) => true,
        })
        .map(|socket_info| PortInfo::from(socket_info))
        .filter(|port_info| match args.mine {
            true => port_info
                .process_info
                .iter()
                .any(|process| process.is_current_user),
            false => true,
        })
        .collect();

    ports.sort_by(|a, b| {
        if a.port < b.port {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    ports
}
