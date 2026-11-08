#!/usr/bin/env bash
set -euo pipefail

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
data_home=${XDG_DATA_HOME:-"$HOME/.local/share"}
app_dir="$data_home/glorious-launcher"
target_dir=${CARGO_TARGET_DIR:-"$root/target"}
[[ $target_dir = /* ]] || target_dir="$root/$target_dir"

if [[ $(uname -s) != Linux || $data_home != /* ]]; then
  echo 'Linux and an absolute XDG_DATA_HOME are required.' >&2
  exit 1
fi
for command in cargo npm sudo systemctl; do
  if ! command -v "$command" >/dev/null; then
    echo "Missing required command: $command" >&2
    exit 1
  fi
done

(cd "$root/frontend" && npm ci && npm run build)
(cd "$root" && cargo build --release --locked -p glorious-service)

install -Dm755 "$target_dir/release/glorious-service" "$app_dir/glorious-service"
install -Dm644 "$root/data/glorious_devices.json" "$app_dir/data/glorious_devices.json"
install -Dm644 "$root/frontend/public/favicon.svg" "$app_dir/icon.svg"
mkdir -p "$data_home/applications"
cat > "$data_home/applications/glorious-launcher.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Glorious Web Launcher
Comment=Configure Glorious mice
Exec=xdg-open http://127.0.0.1:25820/
Icon=$app_dir/icon.svg
Terminal=false
Categories=Settings;HardwareSettings;
EOF

sudo install -Dm644 "$root/packaging/99-glorious.rules" /etc/udev/rules.d/99-glorious.rules
sudo udevadm control --reload-rules
sudo udevadm trigger
unit_dir="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
mkdir -p "$unit_dir"
unit_path=$(printf '%s' "$app_dir" | sed 's/%/%%/g; s/\$/$$/g; s/[\\"]/\\&/g')
cat > "$unit_dir/glorious-launcher.service" <<EOF
[Unit]
Description=Glorious Web Launcher
After=graphical-session.target

[Service]
ExecStart="$unit_path/glorious-service" --devices-json "$unit_path/data/glorious_devices.json" --no-browser
Restart=on-failure

[Install]
WantedBy=default.target
EOF
systemctl --user daemon-reload
systemctl --user enable glorious-launcher.service
systemctl --user restart glorious-launcher.service
echo "Installed. Reconnect your mouse or receiver, then open Glorious Web Launcher from your applications menu."
