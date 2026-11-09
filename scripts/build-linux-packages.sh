#!/usr/bin/env bash
set -euo pipefail

readonly VERSION="0.1.6"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
root_dir="$(cd -- "${script_dir}/.." && pwd)"
package_dir="${root_dir}/target/packages"

skip_frontend=false
skip_cargo=false
for argument in "$@"; do
    case "${argument}" in
        --skip-frontend) skip_frontend=true ;;
        --skip-cargo) skip_cargo=true ;;
        *) printf 'Usage: %s [--skip-frontend] [--skip-cargo]\n' "$0" >&2; exit 2 ;;
    esac
done

die() {
    printf 'Error: %s\n' "$*" >&2
    exit 1
}

command -v tar >/dev/null 2>&1 || die "tar is required."
command -v gzip >/dev/null 2>&1 || die "gzip is required."
mkdir -p "${package_dir}"

if ! "${skip_frontend}"; then
    printf '[1/4] Building frontend...\n'
    (cd "${root_dir}/frontend" && npm ci && npm run build)
else
    printf '[1/4] Skipping frontend build.\n'
fi

if ! "${skip_cargo}"; then
    printf '[2/4] Building Rust release binary...\n'
    (cd "${root_dir}" && cargo build --release --locked -p glorious-service)
else
    printf '[2/4] Skipping Cargo build.\n'
fi

binary="${root_dir}/target/release/glorious-service"
[[ -f "${binary}" ]] || die "Release binary not found: ${binary}"

bundle_root="$(mktemp -d)"
trap 'rm -rf -- "${bundle_root}"' EXIT
bundle_name="glorious-launcher-v${VERSION}-linux-x86_64"
bundle_dir="${bundle_root}/${bundle_name}"
mkdir -p "${bundle_dir}"
install -Dm755 "${binary}" "${bundle_dir}/glorious-service"
install -Dm644 "${root_dir}/packaging/99-glorious.rules" "${bundle_dir}/99-glorious.rules"
install -Dm644 "${root_dir}/packaging/linux/glorious-launcher.desktop" \
    "${bundle_dir}/glorious-launcher.desktop"
install -Dm644 "${root_dir}/packaging/linux/glorious-launcher.svg" \
    "${bundle_dir}/glorious-launcher.svg"
install -Dm755 "${script_dir}/quick-install.sh" "${bundle_dir}/quick-install.sh"

tar -C "${bundle_root}" -czf "${package_dir}/${bundle_name}.tar.gz" "${bundle_name}"
printf '[3/4] Created %s\n' "${package_dir}/${bundle_name}.tar.gz"

printf '[4/4] Building Debian package...\n'
"${script_dir}/build-deb.sh"

if command -v makepkg >/dev/null 2>&1; then
    printf 'Building Arch package...\n'
    if "${script_dir}/build-arch-package.sh"; then
        :
    else
        printf 'Warning: Arch package build failed; continuing without the Arch package.\n' >&2
    fi
else
    printf 'Warning: makepkg is not installed; skipping Arch package.\n' >&2
fi
