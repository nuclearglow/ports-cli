use std::io::{self, IsTerminal};
use std::net::IpAddr;

use comfy_table::presets::{NOTHING, UTF8_FULL_CONDENSED};
use comfy_table::{
    Attribute, Cell, CellAlignment, Color, ColumnConstraint, ContentArrangement, Row, Table,
};

use crate::ports::Protocol;

/// One output row: a socket together with one of the processes holding it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    pub port: u16,
    pub protocol: Protocol,
    pub address: IpAddr,
    pub pid: Option<u32>,
    pub process: Option<String>,
    pub command: Option<String>,
    pub owner: Option<String>,
    pub is_mine: bool,
}

const HEADERS: [&str; 7] = [
    "Address", "Proto", "Port", "Owner", "Process", "PID", "Command",
];
const COMMAND_COLUMN: usize = HEADERS.len() - 1;
const MISSING: &str = "-";

/// Sort by port, protocol, address and pid, and drop duplicates
/// (e.g. several `SO_REUSEPORT` sockets held by the same process).
pub fn normalize(entries: &mut Vec<Entry>) {
    entries.sort();
    entries.dedup();
}

/// How to render the table. Decided once up front, so rendering never
/// depends on the environment it happens to run in (e.g. tests in a terminal).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// Draw borders and fit the table to the terminal width.
    pub terminal: bool,
    pub color: bool,
    /// Table width; `None` means the width of the terminal.
    pub width: Option<u16>,
}

impl Options {
    pub fn detect() -> Self {
        let terminal = io::stdout().is_terminal();
        let no_color = std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty());
        Self {
            terminal,
            color: terminal && !no_color,
            width: None,
        }
    }
}

pub fn render(entries: &[Entry], options: Options) -> String {
    let mut table = Table::new();
    let color = options.color;
    if color {
        table.enforce_styling();
    }

    if options.terminal {
        table
            .load_style(UTF8_FULL_CONDENSED.with_rounded_corners())
            .set_content_arrangement(ContentArrangement::Dynamic);
        if let Some(width) = options.width {
            table.set_width(width);
        }
    } else {
        table.force_no_tty();
        // Plain, borderless columns are easier to process with grep, awk & co.
        table.load_style(NOTHING);
    }

    table.set_header(HEADERS.iter().map(|header| {
        let cell = Cell::new(header);
        if color {
            cell.add_attribute(Attribute::Bold)
        } else {
            cell
        }
    }));

    for entry in entries {
        let fg = color.then(|| port_color(entry));
        let cells = [
            entry.address.to_string(),
            entry.protocol.to_string(),
            entry.port.to_string(),
            display(&entry.owner),
            display(&entry.process),
            entry
                .pid
                .map_or_else(|| MISSING.to_owned(), |pid| pid.to_string()),
            display(&entry.command),
        ]
        .into_iter()
        .map(|text| match fg {
            Some(fg) => Cell::new(text).fg(fg),
            None => Cell::new(text),
        });

        let mut row = Row::from(cells);
        // Long commands are truncated to the terminal width instead of wrapping.
        row.max_height(1);
        table.add_row(row);
    }

    for (index, column) in table.column_iter_mut().enumerate() {
        if index != COMMAND_COLUMN {
            column.set_constraint(ColumnConstraint::ContentWidth);
        }
        if index == 1 {
            column.set_cell_alignment(CellAlignment::Center);
        }
    }

    if options.terminal {
        table.to_string()
    } else {
        table.trim_fmt()
    }
}

fn display(value: &Option<String>) -> String {
    match value.as_deref() {
        Some(value) if !value.is_empty() => value.to_owned(),
        _ => MISSING.to_owned(),
    }
}

fn port_color(entry: &Entry) -> Color {
    if entry.is_mine {
        return Color::Yellow;
    }
    match entry.port {
        // Well-known ports
        0..=1023 => Color::DarkRed,
        // Registered ports
        1024..=49151 => Color::Green,
        // Dynamic / private ports
        _ => Color::DarkGreen,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    const PLAIN: Options = Options {
        terminal: false,
        color: false,
        width: None,
    };

    fn entry(port: u16, protocol: Protocol, address: IpAddr, pid: Option<u32>) -> Entry {
        Entry {
            port,
            protocol,
            address,
            pid,
            process: None,
            command: None,
            owner: None,
            is_mine: false,
        }
    }

    #[test]
    fn normalize_sorts_and_dedups() {
        let v4 = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let v6 = IpAddr::V6(Ipv6Addr::LOCALHOST);
        let mut entries = vec![
            entry(8080, Protocol::Tcp, v4, Some(2)),
            entry(22, Protocol::Udp, v4, None),
            entry(22, Protocol::Tcp, v6, None),
            entry(8080, Protocol::Tcp, v4, Some(1)),
            entry(22, Protocol::Tcp, v4, None),
            entry(8080, Protocol::Tcp, v4, Some(2)),
        ];
        normalize(&mut entries);
        assert_eq!(
            entries,
            vec![
                entry(22, Protocol::Tcp, v4, None),
                entry(22, Protocol::Tcp, v6, None),
                entry(22, Protocol::Udp, v4, None),
                entry(8080, Protocol::Tcp, v4, Some(1)),
                entry(8080, Protocol::Tcp, v4, Some(2)),
            ]
        );
    }

    #[test]
    fn colors_by_port_range_and_ownership() {
        let v4 = IpAddr::V4(Ipv4Addr::LOCALHOST);
        assert_eq!(
            port_color(&entry(22, Protocol::Tcp, v4, None)),
            Color::DarkRed
        );
        assert_eq!(
            port_color(&entry(1024, Protocol::Tcp, v4, None)),
            Color::Green
        );
        assert_eq!(
            port_color(&entry(49151, Protocol::Tcp, v4, None)),
            Color::Green
        );
        assert_eq!(
            port_color(&entry(49152, Protocol::Tcp, v4, None)),
            Color::DarkGreen
        );
        let mine = Entry {
            is_mine: true,
            ..entry(22, Protocol::Tcp, v4, None)
        };
        assert_eq!(port_color(&mine), Color::Yellow);
    }

    #[test]
    fn missing_values_render_as_dash() {
        let v4 = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
        let output = render(&[entry(22, Protocol::Tcp, v4, None)], PLAIN);
        let row = output.lines().nth(1).expect("data row");
        let columns: Vec<&str> = row.split_whitespace().collect();
        assert_eq!(columns, ["0.0.0.0", "TCP", "22", "-", "-", "-", "-"]);
    }

    #[test]
    fn plain_output_has_no_styling_or_padding() {
        let v4 = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let output = render(&[entry(22, Protocol::Tcp, v4, None)], PLAIN);
        assert!(!output.contains('\x1b'));
        assert!(!output.contains('│'));
        assert!(output.lines().all(|line| !line.ends_with(' ')));
    }

    #[test]
    fn terminal_output_fits_width_and_truncates_command() {
        let v4 = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let long = Entry {
            command: Some("x".repeat(200)),
            ..entry(8080, Protocol::Tcp, v4, Some(1))
        };
        let options = Options {
            terminal: true,
            color: false,
            width: Some(80),
        };
        let output = render(&[long], options);
        assert!(output.contains('│'));
        assert!(output.contains('…'));
        assert!(!output.contains('\x1b'));
        assert!(output.lines().all(|line| line.chars().count() <= 80));
    }

    #[test]
    fn color_option_styles_output() {
        let v4 = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let options = Options {
            terminal: true,
            color: true,
            width: Some(80),
        };
        let output = render(&[entry(22, Protocol::Tcp, v4, None)], options);
        assert!(output.contains('\x1b'));
    }
}
