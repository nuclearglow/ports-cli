use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use std::cmp::Ordering;
use std::net;

use crate::processes::{get_process_info, ProcessInfo};

#[derive(Debug)]
pub struct PortInfo {
    pub address: net::IpAddr,
    pub port: u16,
    pub process_info: Vec<ProcessInfo>,
}
impl From<netstat2::SocketInfo> for PortInfo {
    fn from(socket_info: netstat2::SocketInfo) -> Self {
        Self {
            address: socket_info.local_addr(),
            port: socket_info.local_port(),
            process_info: get_process_info(socket_info.associated_pids),
        }
    }
}

/// get a list of SocketInfo for open tcp ports (LISTENING) or udp ports
pub fn get_open_ports() -> Vec<PortInfo> {
    // TODO: flags as params --tcp --udp --ipv4 --ipv6 ?
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
    let sockets = get_sockets_info(af_flags, proto_flags).unwrap_or_default();

    let mut ports: Vec<PortInfo> = sockets
        .into_iter()
        .filter(|socket_info| match &socket_info.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) => tcp.state == TcpState::Listen,
            ProtocolSocketInfo::Udp(_) => true,
        })
        .map(|socket_info| PortInfo::from(socket_info))
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
