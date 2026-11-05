# Changelog

## Unreleased

### Fixed

- Tests failed when run from an interactive terminal, because table rendering
  detected the terminal of the test process.

## 0.3.0

### Fixed

- Segmentation fault on current Rust versions, caused by unsound code in
  `prettytable-rs` 0.8 (RUSTSEC-2022-0091).
- Build failure of `netstat2` 0.9 with current `libc` versions.
- Panics when a process exits while being inspected, when a user id has no
  name (e.g. container users), and when truncating non-ASCII commands.
- Panic when piping output into a command that exits early (e.g. `ports | head`).
- Socket errors are reported instead of silently showing an empty list.

### Changed

- **Breaking:** all TCP and UDP sockets on IPv4 and IPv6 are shown by default.
  `--tcp`/`-t`, `--udp`/`-u`, `--ipv4`/`-4` and `--ipv6`/`-6` now narrow the
  output instead of extending it.
- One row per socket and process, sorted by port, protocol and address.
- The full command line is shown and truncated to the terminal width.
- On Linux, the socket owner is shown even if the process is not visible.
- A hint is printed when processes are hidden due to missing permissions.
- Plain, borderless output when stdout is not a terminal; `NO_COLOR` is respected.
- Much faster: process information is loaded once instead of once per socket.
- Updated to Rust edition 2024 (minimum Rust version 1.95) and current
  dependencies: clap 4, sysinfo 0.40, netstat2 0.11, comfy-table 8.
- Removed the unmaintained `users` crate and the `log`, `env_logger` and
  `exitcode` dependencies.

## 0.2.2

- Initial release series.
