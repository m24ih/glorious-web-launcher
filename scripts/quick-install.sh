#!/usr/bin/env bash
set -euo pipefail

readonly VERSION="0.1.6"
readonly PROJECT_URL="https://gitlab.com/m24ih/glorious-web-launcher"
readonly BIN_DIR="${HOME}/.local/bin"
readonly DATA_DIR="${HOME}/.local/share"
readonly CONFIG_DIR="${HOME}/.config/systemd/user"
readonly SERVICE_NAME="glorious-launcher.service"
readonly SERVICE_PATH="${CONFIG_DIR}/${SERVICE_NAME}"
readonly BINARY_PATH="${BIN_DIR}/glorious-service"
readonly DESKTOP_PATH="${DATA_DIR}/applications/glorious-launcher.desktop"
readonly ICON_PATH="${DATA_DIR}/icons/hicolor/scalable/apps/glorious-launcher.svg"
readonly UDEV_PATH="/etc/udev/rules.d/99-glorious.rules"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
script_root="$(cd -- "${script_dir}/.." && pwd)"
download_dir="$(mktemp -d)"
trap 'rm -rf -- "${download_dir}"' EXIT

die() {
    printf 'Error: %s\n' "$*" >&2
    exit 1
}

have() {
    command -v "$1" >/dev/null 2>&1
}

run_sudo() {
    if have sudo; then
        sudo "$@"
    else
        die "sudo is required for udev rule installation."
    fi
}

download() {
    local url="$1"
    local destination="$2"
    if have curl; then
        curl --fail --location --silent --show-error "$url" --output "$destination"
    elif have wget; then
        wget --quiet --output-document="$destination" "$url"
    else
        die "curl or wget is required to download the release binary."
    fi
}

detect_arch() {
    case "$(uname -m)" in
        x86_64) printf 'x86_64\n' ;;
        aarch64) printf 'aarch64\n' ;;
        *) die "Unsupported architecture: $(uname -m). Supported architectures are x86_64 and aarch64." ;;
    esac
}

find_local_file() {
    local relative
    local base
    for base in "$PWD" "$script_root" "$script_dir" "${PWD}/bundle" "${script_root}/bundle"; do
        for relative in "$@"; do
            if [[ -f "${base}/${relative}" ]]; then
                printf '%s\n' "${base}/${relative}"
                return 0
            fi
        done
    done
    return 1
}

install_assets_from_release() {
    local raw_base="${GLORIOUS_RAW_BASE_URL:-${PROJECT_URL}/-/raw/v${VERSION}}"
    local asset_name="$1"
    local destination="$2"
    download "${raw_base}/${asset_name}" "${destination}"
}

uninstall() {
    if have systemctl; then
        systemctl --user disable --now "${SERVICE_NAME}" >/dev/null 2>&1 || true
        systemctl --user daemon-reload >/dev/null 2>&1 || true
    fi

    rm -f -- "${BINARY_PATH}" "${DESKTOP_PATH}" "${ICON_PATH}" "${SERVICE_PATH}"
    printf 'Removed Glorious Web Launcher files from your user account.\n'

    local answer="n"
    if [[ -t 0 ]]; then
        printf 'Remove the system udev rule (%s)? [y/N] ' "${UDEV_PATH}"
        read -r answer || answer="n"
    elif [[ -t 1 ]]; then
        printf 'Remove the system udev rule (%s)? [y/N] ' "${UDEV_PATH}"
        read -r answer </dev/tty 2>/dev/null || answer="n"
    fi
    if [[ "${answer,,}" == "y" || "${answer,,}" == "yes" ]]; then
        run_sudo rm -f -- "${UDEV_PATH}"
        run_sudo udevadm control --reload-rules
        run_sudo udevadm trigger
        printf 'Removed the udev rule.\n'
    fi
}

if [[ "${1:-}" == "--uninstall" ]]; then
    [[ $# -eq 1 ]] || die "Usage: $0 [--uninstall]"
    uninstall
    exit 0
fi
[[ $# -eq 0 ]] || die "Usage: $0 [--uninstall]"
[[ "$(uname -s)" == "Linux" ]] || die "This installer supports Linux only."

arch="$(detect_arch)"
if ! have systemctl; then
    die "systemctl is required."
fi

binary_source="$(find_local_file target/release/glorious-service glorious-service bundle/glorious-service 2>/dev/null || true)"
if [[ -z "${binary_source}" ]]; then
    download_url="${GLORIOUS_DOWNLOAD_URL:-${PROJECT_URL}/-/releases/v${VERSION}/downloads/glorious-service-linux-${arch}}"
    binary_source="${download_dir}/glorious-service"
    printf 'Downloading Glorious Web Launcher for %s...\n' "${arch}"
    download "${download_url}" "${binary_source}"
fi

desktop_source="$(find_local_file packaging/linux/glorious-launcher.desktop packaging/glorious-launcher.desktop glorious-launcher.desktop 2>/dev/null || true)"
icon_source="$(find_local_file packaging/linux/glorious-launcher.svg packaging/glorious-launcher.svg glorious-launcher.svg 2>/dev/null || true)"
udev_source="$(find_local_file packaging/99-glorious.rules packaging/linux/99-glorious.rules 99-glorious.rules 2>/dev/null || true)"

mkdir -p -- "${BIN_DIR}" "$(dirname -- "${DESKTOP_PATH}")" "$(dirname -- "${ICON_PATH}")" "${CONFIG_DIR}"
install -Dm755 "${binary_source}" "${BINARY_PATH}"

if [[ -n "${desktop_source}" ]]; then
    install -Dm644 "${desktop_source}" "${DESKTOP_PATH}"
else
    install_assets_from_release packaging/linux/glorious-launcher.desktop "${DESKTOP_PATH}"
fi
if [[ -n "${icon_source}" ]]; then
    install -Dm644 "${icon_source}" "${ICON_PATH}"
else
    install_assets_from_release packaging/linux/glorious-launcher.svg "${ICON_PATH}"
fi

if [[ -z "${udev_source}" ]]; then
    udev_source="${download_dir}/99-glorious.rules"
    install_assets_from_release packaging/99-glorious.rules "${udev_source}"
fi
printf 'Installing device permissions (sudo may prompt for your password)...\n'
run_sudo install -Dm644 "${udev_source}" "${UDEV_PATH}"
run_sudo udevadm control --reload-rules
run_sudo udevadm trigger

cat > "${SERVICE_PATH}" <<EOF
[Unit]
Description=Glorious Web Launcher
After=graphical-session.target

[Service]
ExecStart=${BINARY_PATH} --no-browser
Restart=on-failure

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable --now "${SERVICE_NAME}"

cat <<EOF

Glorious Web Launcher installed successfully.
Open http://127.0.0.1:25820/ in your browser.
EOF
