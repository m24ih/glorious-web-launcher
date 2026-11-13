#!/usr/bin/env bash
set -euo pipefail

readonly VERSION="0.1.9"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root_dir="$(cd -- "${script_dir}/.." && pwd)"
package_dir="${root_dir}/target/packages"
output="${package_dir}/glorious-launcher-${VERSION}-1.x86_64.rpm"

mkdir -p "${package_dir}"
rm -f -- "${output}"

if command -v rpmbuild >/dev/null 2>&1; then
    work_dir="$(mktemp -d)"
    trap 'rm -rf -- "${work_dir}"' EXIT
    for directory in BUILD BUILDROOT RPMS SOURCES SPECS; do
        mkdir -p "${work_dir}/${directory}"
    done

    binary="${GLORIOUS_BINARY:-${root_dir}/target/release/glorious-service}"
    [[ -f "${binary}" ]] || {
        printf 'Error: Release binary not found: %s\n' "${binary}" >&2
        exit 1
    }

    install -Dm755 "${binary}" "${work_dir}/SOURCES/glorious-service"
    install -Dm644 "${root_dir}/data/glorious_devices.json" \
        "${work_dir}/SOURCES/glorious_devices.json"
    install -Dm644 "${root_dir}/packaging/linux/glorious-launcher.desktop" \
        "${work_dir}/SOURCES/glorious-launcher.desktop"
    install -Dm644 "${root_dir}/packaging/linux/glorious-launcher.svg" \
        "${work_dir}/SOURCES/glorious-launcher.svg"
    install -Dm644 "${root_dir}/packaging/linux/glorious-launcher.service" \
        "${work_dir}/SOURCES/glorious-launcher.service"
    install -Dm644 "${root_dir}/packaging/99-glorious.rules" \
        "${work_dir}/SOURCES/99-glorious.rules"
    install -Dm644 "${root_dir}/LICENSE" "${work_dir}/SOURCES/LICENSE"
    install -Dm644 "${root_dir}/packaging/linux/rpm/glorious-launcher.spec" \
        "${work_dir}/SPECS/glorious-launcher.spec"

    rpmbuild -bb \
        --define "_topdir ${work_dir}" \
        "${work_dir}/SPECS/glorious-launcher.spec"
    rpm_path="${work_dir}/RPMS/x86_64/glorious-launcher-${VERSION}-1.x86_64.rpm"
    [[ -f "${rpm_path}" ]] || {
        printf 'Error: rpmbuild did not create %s\n' "${rpm_path}" >&2
        exit 1
    }
    mv -f -- "${rpm_path}" "${output}"
    printf 'Created %s\n' "${output}"
elif command -v cargo-generate-rpm >/dev/null 2>&1; then
    (cd "${root_dir}" && cargo generate-rpm)
    generated_rpm="$(find "${root_dir}/target" -type f \
        -name "glorious-launcher-${VERSION}-1.x86_64.rpm" -print -quit)"
    [[ -n "${generated_rpm}" ]] || {
        printf 'Error: cargo-generate-rpm did not create %s\n' "${output}" >&2
        exit 1
    }
    [[ "${generated_rpm}" == "${output}" ]] || mv -f -- "${generated_rpm}" "${output}"
    printf 'Created %s\n' "${output}"
else
    printf 'Warning: neither rpmbuild nor cargo-generate-rpm is installed; skipping RPM package.\n' >&2
fi
