#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: render-homebrew-formula.sh \
  --version <version> \
  --darwin-arm64-sha <sha256> \
  --darwin-amd64-sha <sha256> \
  [--repo <owner/repo>]

Renders a Formula/myx.rb file to stdout.
EOF
}

version=""
repo="myx-hq/myx"
darwin_arm64_sha=""
darwin_amd64_sha=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      version="${2:-}"
      shift 2
      ;;
    --repo)
      repo="${2:-}"
      shift 2
      ;;
    --darwin-arm64-sha)
      darwin_arm64_sha="${2:-}"
      shift 2
      ;;
    --darwin-amd64-sha)
      darwin_amd64_sha="${2:-}"
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

if [[ -z "${version}" || -z "${darwin_arm64_sha}" || -z "${darwin_amd64_sha}" ]]; then
  echo "error: --version, --darwin-arm64-sha, and --darwin-amd64-sha are required" >&2
  usage
  exit 2
fi

version="${version#v}"
tag="v${version}"

cat <<EOF
class Myx < Formula
  desc "Package manager and compatibility layer for agent capabilities"
  homepage "https://github.com/${repo}"
  version "${version}"
  license "Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/${repo}/releases/download/${tag}/myx-${version}-aarch64-apple-darwin.tar.gz"
      sha256 "${darwin_arm64_sha}"
    else
      url "https://github.com/${repo}/releases/download/${tag}/myx-${version}-x86_64-apple-darwin.tar.gz"
      sha256 "${darwin_amd64_sha}"
    end
  end

  def install
    bin.install "bin/myx"
    bin.install "bin/myx-mcp-runner"
  end

  test do
    assert_match "myx", shell_output("#{bin}/myx --help")
  end
end
EOF
