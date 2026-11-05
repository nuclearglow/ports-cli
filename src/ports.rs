use std::fmt;
use std::net::IpAddr;

use anyhow::{Context, Result};
use netstat2::{
    AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, SocketInfo, TcpState, get_sockets_info,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        })
    }
}

/// Which sockets to list. Selecting neither or both of a pair means "both".
#[derive(Debug, Clone, Copy, Default)]
pub struct Filter {
    pub tcp: bool,
    pub udp: bool,
    pub ipv4: bool,
    pub ipv6: bool,
}

impl Filter {
    fn protocol_flags(self) -> ProtocolFlags {
        match (self.tcp, self.udp) {
            (true, false) => ProtocolFlags::TCP,
            (false, true) => ProtocolFlags::UDP,
            _ => ProtocolFlags::TCP | ProtocolFlags::UDP,
        }
    }

    fn address_family_flags(self) -> AddressFamilyFlags {
        match (self.ipv4, self.ipv6) {
            (true, false) => AddressFamilyFlags::IPV4,
            (false, true) => AddressFamilyFlags::IPV6,
            _ => AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        }
    }
}

/// A local socket that is listening (TCP) or bound (UDP).
#[derive(Debug)]
pub struct Socket {
    pub protocol: Protocol,
    pub address: IpAddr,
    pub port: u16,
    pub pids: Vec<u32>,
    /// Owner of the socket, on platforms that report it (Linux).
    pub uid: Option<u32>,
}

impl Socket {
    fn from_listening(info: SocketInfo) -> Option<Self> {
        let protocol = match &info.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) if tcp.state == TcpState::Listen => Protocol::Tcp,
            ProtocolSocketInfo::Tcp(_) => return None,
            ProtocolSocketInfo::Udp(_) => Protocol::Udp,
        };
        Some(Self {
            protocol,
            address: info.local_addr(),
            port: info.local_port(),
            uid: socket_uid(&info),
            pids: info.associated_pids,
        })
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn socket_uid(info: &SocketInfo) -> Option<u32> {
    Some(info.uid)
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
fn socket_uid(_info: &SocketInfo) -> Option<u32> {
    None
}

/// List listening TCP sockets and bound UDP sockets matching `filter`.
pub fn listening_sockets(filter: Filter) -> Result<Vec<Socket>> {
    let sockets = get_sockets_info(filter.address_family_flags(), filter.protocol_flags())
        .context("failed to read socket information")?;
    Ok(sockets
        .into_iter()
        .filter_map(Socket::from_listening)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter(tcp: bool, udp: bool, ipv4: bool, ipv6: bool) -> Filter {
        Filter {
            tcp,
            udp,
            ipv4,
            ipv6,
        }
    }

    #[test]
    fn no_flags_selects_everything() {
        let f = Filter::default();
        assert_eq!(f.protocol_flags(), ProtocolFlags::TCP | ProtocolFlags::UDP);
        assert_eq!(
            f.address_family_flags(),
            AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6
        );
    }

    #[test]
    fn single_flags_narrow_selection() {
        assert_eq!(
            filter(true, false, false, false).protocol_flags(),
            ProtocolFlags::TCP
        );
        assert_eq!(
            filter(false, true, false, false).protocol_flags(),
            ProtocolFlags::UDP
        );
        assert_eq!(
            filter(false, false, true, false).address_family_flags(),
            AddressFamilyFlags::IPV4
        );
        assert_eq!(
            filter(false, false, false, true).address_family_flags(),
            AddressFamilyFlags::IPV6
        );
    }

    #[test]
    fn both_flags_select_both() {
        let f = filter(true, true, true, true);
        assert_eq!(f.protocol_flags(), ProtocolFlags::TCP | ProtocolFlags::UDP);
        assert_eq!(
            f.address_family_flags(),
            AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6
        );
    }
}
