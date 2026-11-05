# ports

[![crates.io](https://img.shields.io/crates/v/ports-cli.svg)](https://crates.io/crates/ports-cli)
[![CI](https://github.com/nuclearglow/ports-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/nuclearglow/ports-cli/actions/workflows/ci.yml)

A minimal CLI tool to list open ports and associated processes.

Works on Linux, macOS and Windows.

## Installation

```shell
cargo install ports-cli
```

This installs the `ports` binary. Requires Rust 1.95 or newer.

## Usage

```text
Usage: ports [OPTIONS]

Options:
  -t, --tcp      Show TCP sockets
  -u, --udp      Show UDP sockets
  -4, --ipv4     Show IPv4 sockets
  -6, --ipv6     Show IPv6 sockets
  -m, --mine     Show only ports opened by processes of the current user
  -h, --help     Print help
  -V, --version  Print version
```

Without filters, all listening TCP and bound UDP sockets on IPv4 and IPv6 are
shown. Filters can be combined, e.g. `ports -t -4` shows IPv4 TCP listeners only.

```text
$ ports -t
╭───────────┬───────┬──────┬───────┬───────────┬────────┬───────────────────────────╮
│ Address   ┆ Proto ┆ Port ┆ Owner ┆ Process   ┆ PID    ┆ Command                   │
╞═══════════╪═══════╪══════╪═══════╪═══════════╪════════╪═══════════════════════════╡
│ 0.0.0.0   ┆  TCP  ┆ 22   ┆ root  ┆ -         ┆ -      ┆ -                         │
│ ::        ┆  TCP  ┆ 22   ┆ root  ┆ -         ┆ -      ┆ -                         │
│ 0.0.0.0   ┆  TCP  ┆ 3306 ┆ mysql ┆ -         ┆ -      ┆ -                         │
│ 127.0.0.1 ┆  TCP  ┆ 8080 ┆ nuky  ┆ python3   ┆ 122069 ┆ python3 -m http.server …  │
╰───────────┴───────┴──────┴───────┴───────────┴────────┴───────────────────────────╯
```

- **Colors:** yellow = your own processes, red = well-known ports (0-1023),
  green = registered ports (1024-49151), dark green = dynamic ports (49152-65535).
  Set `NO_COLOR` to disable colors.
- **Permissions:** processes of other users are only visible when running as root,
  e.g. `sudo ports`. On Linux the owner of a socket is shown regardless.
- **Scripting:** when the output is not a terminal, a plain table without borders
  and without truncation is printed, so it works well with `grep`, `awk` & co.

## Alternatives

Linux:

```shell
ss -tulpn
```

macOS:

```shell
lsof -iTCP -sTCP:LISTEN -n -P
```

## License

[MIT](LICENSE)
