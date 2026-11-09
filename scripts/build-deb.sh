#!/usr/bin/env bash
set -euo pipefail

readonly VERSION="0.1.5"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root_dir="$(cd -- "${script_dir}/.." && pwd)"
package_dir="${root_dir}/target/packages"
work_dir="$(mktemp -d)"
trap 'rm -rf -- "${work_dir}"' EXIT

die() {
    printf 'Error: %s\n' "$*" >&2
    exit 1
}

for command in ar tar gzip md5sum install; do
    command -v "${command}" >/dev/null 2>&1 || die "Missing required command: ${command}"
done

binary="${GLORIOUS_BINARY:-${root_dir}/target/release/glorious-service}"
[[ -f "${binary}" ]] || die "Release binary not found: ${binary}"

control_dir="${work_dir}/control"
data_dir="${work_dir}/data"
mkdir -p "${control_dir}" "${data_dir}" "${package_dir}"

install -Dm644 "${root_dir}/packaging/linux/deb/control" "${control_dir}/control"
install -Dm755 "${root_dir}/packaging/linux/deb/postinst" "${control_dir}/postinst"
install -Dm755 "${root_dir}/packaging/linux/deb/postrm" "${control_dir}/postrm"

install -Dm755 "${binary}" "${data_dir}/usr/bin/glorious-service"
install -Dm644 "${root_dir}/data/glorious_devices.json" \
    "${data_dir}/usr/share/glorious-launcher/data/glorious_devices.json"
install -Dm644 "${root_dir}/packaging/linux/glorious-launcher.desktop" \
    "${data_dir}/usr/share/applications/glorious-launcher.desktop"
install -Dm644 "${root_dir}/packaging/linux/glorious-launcher.svg" \
    "${data_dir}/usr/share/icons/hicolor/scalable/apps/glorious-launcher.svg"
install -Dm644 "${root_dir}/packaging/linux/glorious-launcher.service" \
    "${data_dir}/usr/lib/systemd/user/glorious-launcher.service"
install -Dm644 "${root_dir}/packaging/99-glorious.rules" \
    "${data_dir}/usr/lib/udev/rules.d/99-glorious.rules"
install -Dm644 "${root_dir}/LICENSE" \
    "${data_dir}/usr/share/doc/glorious-launcher/copyright"

(cd "${data_dir}" && find usr -type f -print0 | sort -z | xargs -0 md5sum > "${control_dir}/md5sums")

printf '2.0\n' > "${work_dir}/debian-binary"
tar -C "${control_dir}" -czf "${work_dir}/control.tar.gz" .
tar -C "${data_dir}" --sort=name --owner=0 --group=0 --numeric-owner -czf "${work_dir}/data.tar.gz" .

output="${package_dir}/glorious-launcher_${VERSION}_amd64.deb"
rm -f -- "${output}"
(cd "${work_dir}" && ar crD "${output}" debian-binary control.tar.gz data.tar.gz)
printf 'Created %s\n' "${output}"
