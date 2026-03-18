#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MYX_BIN="${MYX_BIN:-$ROOT_DIR/target/debug/myx}"
PACKAGE_SPEC="${PACKAGE_SPEC:-$ROOT_DIR/examples/github-capability}"
OUT_PATH="${OUT_PATH:-$ROOT_DIR/warm-cache-add.json}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bin)
      MYX_BIN="$2"
      shift 2
      ;;
    --package)
      PACKAGE_SPEC="$2"
      shift 2
      ;;
    --out)
      OUT_PATH="$2"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

timestamp_ms() {
  local candidate
  candidate="$(date +%s%3N 2>/dev/null || true)"
  if [[ "$candidate" =~ ^[0-9]+$ ]]; then
    echo "$candidate"
    return
  fi
  python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
}

measure_add_ms() {
  local workspace="$1"
  local start end
  start="$(timestamp_ms)"
  (
    cd "$workspace"
    MYX_POLICY_MODE=permissive MYX_NON_INTERACTIVE=1 "$MYX_BIN" add "$PACKAGE_SPEC" --json >/dev/null
  )
  end="$(timestamp_ms)"
  echo $((end - start))
}

if [[ ! -x "$MYX_BIN" ]]; then
  (cd "$ROOT_DIR" && cargo build -p myx-cli >/dev/null)
fi

workspace="$(mktemp -d)"
cleanup() {
  rm -rf "$workspace"
}
trap cleanup EXIT

cold_ms="$(measure_add_ms "$workspace")"
warm_ms="$(measure_add_ms "$workspace")"

mkdir -p "$(dirname "$OUT_PATH")"
cat >"$OUT_PATH" <<EOF
{
  "benchmark": "warm-cache-myx-add",
  "package": "$(printf "%s" "$PACKAGE_SPEC")",
  "workspace": "$(printf "%s" "$workspace")",
  "cold_ms": $cold_ms,
  "warm_ms": $warm_ms,
  "timestamp_utc": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

echo "warm-cache benchmark report: $OUT_PATH"
cat "$OUT_PATH"
