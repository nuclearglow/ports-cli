//! A minimal CLI tool to list open ports and associated processes.

use std::io::{self, Write};

use anyhow::Result;
use clap::Parser;

mod output;
mod ports;
mod processes;

use output::Entry;
use ports::{Filter, Socket};
use processes::{Processes, uid_from_raw};

const AFTER_HELP: &str = "\
Without filters, all listening TCP and bound UDP sockets on IPv4 and IPv6 are shown.

Colors: yellow = your own processes, red = well-known ports (0-1023),
green = registered ports (1024-49151), dark green = dynamic ports (49152-65535).
Set NO_COLOR to disable colors.

Processes of other users are only visible when running as root (e.g. with sudo).";

/// A minimal CLI tool to list open ports and associated processes.
#[derive(Parser, Debug)]
#[command(version, about, after_help = AFTER_HELP)]
struct Args {
    /// Show TCP sockets
    #[arg(short, long)]
    tcp: bool,
    /// Show UDP sockets
    #[arg(short, long)]
    udp: bool,
    /// Show IPv4 sockets
    #[arg(short = '4', long)]
    ipv4: bool,
    /// Show IPv6 sockets
    #[arg(short = '6', long)]
    ipv6: bool,
    /// Show only ports opened by processes of the current user
    #[arg(short, long)]
    mine: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let filter = Filter {
        tcp: args.tcp,
        udp: args.udp,
        ipv4: args.ipv4,
        ipv6: args.ipv6,
    };

    let sockets = ports::listening_sockets(filter)?;
    let processes = Processes::load(sockets.iter().flat_map(|socket| socket.pids.clone()))?;

    let mut entries: Vec<Entry> = sockets
        .iter()
        .flat_map(|socket| entries_for(socket, &processes))
        .filter(|entry| !args.mine || entry.is_mine)
        .collect();
    output::normalize(&mut entries);

    if entries.is_empty() {
        eprintln!("No open ports found.");
        return Ok(());
    }

    if let Err(err) = writeln!(io::stdout().lock(), "{}", output::render(&entries)) {
        // Output piped into e.g. `head` that exited early.
        if err.kind() != io::ErrorKind::BrokenPipe {
            return Err(err.into());
        }
    }

    let unknown = entries.iter().filter(|entry| entry.pid.is_none()).count();
    if unknown > 0 && !processes.is_root() {
        eprintln!(
            "note: {unknown} socket(s) belong to processes that are not visible to you. \
             Run as root (e.g. with sudo) to see them."
        );
    }

    Ok(())
}

/// One entry per process holding the socket, or a single entry without process details.
fn entries_for(socket: &Socket, processes: &Processes) -> Vec<Entry> {
    let socket_uid = socket.uid.and_then(uid_from_raw);
    let entry = |pid: Option<u32>| {
        let info = pid.and_then(|pid| processes.get(pid));
        let owner_uid = info
            .as_ref()
            .and_then(|info| info.uid.clone())
            .or_else(|| socket_uid.clone());
        let is_mine = owner_uid.is_some() && owner_uid.as_ref() == processes.current_uid();
        Entry {
            port: socket.port,
            protocol: socket.protocol,
            address: socket.address,
            pid,
            owner: owner_uid.as_ref().map(|uid| processes.user_name(uid)),
            process: info.as_ref().map(|info| info.name.clone()),
            command: info.map(|info| info.command),
            is_mine,
        }
    };

    if socket.pids.is_empty() {
        vec![entry(None)]
    } else {
        socket.pids.iter().map(|&pid| entry(Some(pid))).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_is_valid() {
        Args::command().debug_assert();
    }
}
