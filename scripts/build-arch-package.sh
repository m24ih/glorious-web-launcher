#!/usr/bin/env bash
set -euo pipefail

readonly VERSION="0.1.6"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root_dir="$(cd -- "${script_dir}/.." && pwd)"
package_dir="${root_dir}/target/packages"
binary="${root_dir}/target/release/glorious-service"
work_dir="$(mktemp -d)"
trap 'rm -rf -- "${work_dir}"' EXIT

die() {
    printf 'Error: %s\n' "$*" >&2
    exit 1
}

command -v makepkg >/dev/null 2>&1 || die "makepkg is required."
[[ -f "${binary}" ]] || die "Release binary not found: ${binary}"

for file in \
    "${root_dir}/data/glorious_devices.json" \
    "${root_dir}/packaging/linux/glorious-launcher.desktop" \
    "${root_dir}/packaging/linux/glorious-launcher.svg" \
    "${root_dir}/packaging/linux/glorious-launcher.service" \
    "${root_dir}/packaging/99-glorious.rules"; do
    [[ -f "${file}" ]] || die "Packaging file not found: ${file}"
done

mkdir -p "${package_dir}"
cat > "${work_dir}/PKGBUILD" <<EOF
pkgname=glorious-launcher
pkgver=${VERSION}
pkgrel=1
pkgdesc='Web GUI and HID service for Glorious gaming mice'
arch=('x86_64')
license=('MIT')
source=()

package() {
    install -Dm755 '${binary}' "\$pkgdir/usr/bin/glorious-service"
    install -Dm644 '${root_dir}/data/glorious_devices.json' \
        "\$pkgdir/usr/share/glorious-launcher/data/glorious_devices.json"
    install -Dm644 '${root_dir}/packaging/linux/glorious-launcher.desktop' \
        "\$pkgdir/usr/share/applications/glorious-launcher.desktop"
    install -Dm644 '${root_dir}/packaging/linux/glorious-launcher.svg' \
        "\$pkgdir/usr/share/icons/hicolor/scalable/apps/glorious-launcher.svg"
    install -Dm644 '${root_dir}/packaging/linux/glorious-launcher.service' \
        "\$pkgdir/usr/lib/systemd/user/glorious-launcher.service"
    install -Dm644 '${root_dir}/packaging/99-glorious.rules' \
        "\$pkgdir/usr/lib/udev/rules.d/99-glorious.rules"
}
EOF

(cd "${work_dir}" && PKGDEST="${package_dir}" makepkg --noconfirm -f)
printf 'Created %s\n' "${package_dir}/glorious-launcher-${VERSION}-1-x86_64.pkg.tar.zst"
