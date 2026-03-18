#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<'EOF'
Usage: package-dist.sh --target <target-triple> --version <version> [--out-dir <dir>]

Packages release binaries into:
  myx-<version>-<target-triple>.tar.gz
  myx-<version>-<target-triple>.tar.gz.sha256

Required:
  --target   Rust target triple (e.g. aarch64-apple-darwin)
  --version  Release version without v prefix (e.g. 0.1.0)

Optional:
  --out-dir  Output directory (default: <repo>/dist)
EOF
}

target=""
version=""
out_dir="${ROOT_DIR}/dist"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      target="${2:-}"
      shift 2
      ;;
    --version)
      version="${2:-}"
      shift 2
      ;;
    --out-dir)
      out_dir="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [[ -z "${target}" || -z "${version}" ]]; then
  echo "error: --target and --version are required" >&2
  usage
  exit 2
fi

version="${version#v}"

bin_dir="${ROOT_DIR}/target/${target}/release"
for bin in myx myx-mcp-runner; do
  if [[ ! -x "${bin_dir}/${bin}" ]]; then
    echo "missing expected binary: ${bin_dir}/${bin}" >&2
    exit 1
  fi
done

mkdir -p "${out_dir}"

archive_base="myx-${version}-${target}"
archive_path="${out_dir}/${archive_base}.tar.gz"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

staging_dir="${tmp_dir}/${archive_base}"
mkdir -p "${staging_dir}/bin"
cp "${bin_dir}/myx" "${staging_dir}/bin/myx"
cp "${bin_dir}/myx-mcp-runner" "${staging_dir}/bin/myx-mcp-runner"
cp "${ROOT_DIR}/README.md" "${staging_dir}/README.md"
cp "${ROOT_DIR}/LICENSE" "${staging_dir}/LICENSE"

tar -C "${tmp_dir}" -czf "${archive_path}" "${archive_base}"
sha256="$(shasum -a 256 "${archive_path}" | awk '{print $1}')"
printf '%s  %s\n' "${sha256}" "$(basename "${archive_path}")" > "${archive_path}.sha256"

echo "wrote ${archive_path}"
echo "wrote ${archive_path}.sha256"
