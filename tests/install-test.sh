#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/bin" "$tmp/home" "$tmp/target/release"
export XDG_CONFIG_HOME="$tmp/home/.config"
printf '#!/bin/sh\nexit 0\n' > "$tmp/target/release/glorious-service"
for command in npm cargo sudo systemctl; do
  printf '#!/bin/sh\nexit 0\n' > "$tmp/bin/$command"
  chmod +x "$tmp/bin/$command"
done
printf '#!/bin/sh\nprintf "%%s\\n" "$*" >> "$HOME/service-calls"\n' > "$tmp/bin/systemctl"
chmod +x "$tmp/target/release/glorious-service"
HOME="$tmp/home" XDG_DATA_HOME="$tmp/home/data with spaces" CARGO_TARGET_DIR="$tmp/target" PATH="$tmp/bin:$PATH" bash "$root/install.sh"
test -x "$tmp/home/data with spaces/glorious-launcher/glorious-service"
cmp "$root/data/glorious_devices.json" "$tmp/home/data with spaces/glorious-launcher/data/glorious_devices.json"
grep -Fq 'Exec=xdg-open http://127.0.0.1:25820/' "$tmp/home/data with spaces/applications/glorious-launcher.desktop"
grep -Fq -- '--no-browser' "$tmp/home/.config/systemd/user/glorious-launcher.service"
grep -Fxq -- '--user enable glorious-launcher.service' "$tmp/home/service-calls"
grep -Fxq -- '--user restart glorious-launcher.service' "$tmp/home/service-calls"
HOME="$tmp/home" XDG_DATA_HOME="$tmp/home/data with spaces" CARGO_TARGET_DIR="$tmp/target" PATH="$tmp/bin:$PATH" bash "$root/install.sh"
echo "Installer test passed"
