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

### Linux

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
session D-Bus, and `notify-send` are needed for desktop integration.

### Windows

#### For End Users (Recommended)

Windows users do **not** need Rust, Node.js, or any terminal tools installed:
1. Download **`GloriousWebLauncher-Setup.exe`** from the [Releases](https://gitlab.com/m24ih/glorious-web-launcher/releases) page.
2. Run the wizard setup ("Next ➔ Next ➔ Install").
3. Glorious Web Launcher will automatically appear in your System Tray and Start Menu. You can uninstall it anytime from Windows Settings (Installed Apps).

#### Building from Source / Developer Setup

Ensure Rust, Node.js and Visual Studio C++ Build Tools (MSVC) are installed.

To build the standalone GUI installer package:
```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\build-installer.ps1
```

Or run the quick local installation script:
```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

The installer builds the frontend and release binary, installs them under
`%LOCALAPPDATA%\glorious-launcher`, adds a Start Menu shortcut (`Glorious Web Launcher`),
and configures a background autostart shortcut in your Startup folder without any console window.
Profiles are saved under `%APPDATA%\glorious-launcher\profiles.json`.

To uninstall:
```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1 -Uninstall
```

For development:

```sh
cargo build
cargo run -p glorious-service -- --probe   # firmware/battery sanity check
cargo run -p glorious-service              # serve HTTP + WS
```

```sh
curl http://127.0.0.1:25820/api/devices
```
