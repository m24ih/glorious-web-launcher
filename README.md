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

```sh
cargo build
cargo run -p glorious-service -- --probe   # firmware/battery sanity check
cargo run -p glorious-service              # serve HTTP + WS
```

```sh
curl http://127.0.0.1:25820/api/devices
```
