#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$tmp/bin" "$tmp/home" "$tmp/target/release"
printf '#!/bin/sh\nexit 0\n' > "$tmp/target/release/glorious-service"
for command in npm cargo sudo; do
  printf '#!/bin/sh\nexit 0\n' > "$tmp/bin/$command"
  chmod +x "$tmp/bin/$command"
done
chmod +x "$tmp/target/release/glorious-service"
HOME="$tmp/home" XDG_DATA_HOME="$tmp/home/data with spaces" CARGO_TARGET_DIR="$tmp/target" PATH="$tmp/bin:$PATH" bash "$root/install.sh"
test -x "$tmp/home/data with spaces/glorious-launcher/glorious-service"
cmp "$root/data/glorious_devices.json" "$tmp/home/data with spaces/glorious-launcher/data/glorious_devices.json"
grep -Fq 'Exec="' "$tmp/home/data with spaces/applications/glorious-launcher.desktop"
HOME="$tmp/home" XDG_DATA_HOME="$tmp/home/data with spaces" CARGO_TARGET_DIR="$tmp/target" PATH="$tmp/bin:$PATH" bash "$root/install.sh"
echo "Installer test passed"
