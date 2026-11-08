# Glorious Web Launcher

Linux tooling and data extracted from Glorious CORE, intended to power a local
web launcher for Glorious mice without the Windows/Electron dependency.

## Contents

- `data/glorious_devices.json` — 16 Glorious mouse profiles (26 raw records) with
  VID/PID pairs for wired, 2.4 GHz dongle and Bluetooth connections.
- `assets/images/` — product images used by the launcher UI.
- `crates/glorious-service/` — Rust HID service with an HTTP + WebSocket API on
  `127.0.0.1:25820`. See the [service README](crates/glorious-service/README.md).
- `packaging/99-glorious.rules` — udev rule for `/dev/hidraw*` access.

## Quick start

For a Linux desktop installation, install Rust/Cargo, Node.js and npm, a C
compiler, `pkg-config`, and the libudev development files first (for example,
`base-devel` and `systemd` on Arch, or `build-essential`, `pkg-config` and
`libudev-dev` on Debian/Ubuntu). Then run from the repository:

```sh
./install.sh
```

The installer builds the frontend and release binary, installs them under
`${XDG_DATA_HOME:-~/.local/share}/glorious-launcher`, and adds an applications
menu entry. It asks for sudo to install and reload the HID udev rule. Reconnect
the mouse or receiver afterwards. Running it again updates the installation;
existing profiles and tray preferences are preserved. The installer enables
and starts a systemd user service, which starts automatically at login. The
application menu entry opens the running service in your browser. A tray host,
session D-Bus, and `notify-send` are
needed for desktop integration.

For development:

```sh
cargo build
cargo run -p glorious-service -- --probe   # firmware/battery sanity check
cargo run -p glorious-service              # serve HTTP + WS
```

```sh
curl http://127.0.0.1:25820/api/devices
```
