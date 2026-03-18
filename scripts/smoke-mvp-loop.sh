#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MYX_BIN="${MYX_BIN:-$ROOT_DIR/target/debug/myx}"
PACKAGE_DIR="${PACKAGE_DIR:-$ROOT_DIR/examples/smoke-capability}"

if [[ ! -x "$MYX_BIN" ]]; then
  (cd "$ROOT_DIR" && cargo build -p myx-cli >/dev/null)
fi

workspace="$(mktemp -d)"
cleanup() {
  rm -rf "$workspace"
}
trap cleanup EXIT

scaffold_dir="$workspace/scaffold"
consumer_dir="$workspace/consumer"
mkdir -p "$consumer_dir"

"$MYX_BIN" init "$scaffold_dir" --json >/dev/null

digest="$(
  cd "$consumer_dir"
  MYX_POLICY_MODE=permissive MYX_NON_INTERACTIVE=1 "$MYX_BIN" add "$PACKAGE_DIR" --json >/dev/null
  python3 - <<'PY'
import json
from pathlib import Path
lock = json.loads(Path("myx.lock").read_text())
print(lock["packages"][0]["digest"])
PY
)"
rm -f "$consumer_dir/myx.lock"

index_path="$consumer_dir/index.json"
cat >"$index_path" <<EOF
{
  "schema_version": 1,
  "packages": [
    {
      "name": "smoke",
      "version": "0.1.0",
      "source": "$PACKAGE_DIR",
      "digest": "$digest"
    }
  ]
}
EOF

cat >"$consumer_dir/myx.config.toml" <<EOF
[index]
sources = ["$index_path"]

[policy]
mode = "permissive"
EOF

(
  cd "$consumer_dir"
  MYX_POLICY_MODE=permissive MYX_NON_INTERACTIVE=1 "$MYX_BIN" add smoke --json >/dev/null
  MYX_POLICY_MODE=permissive MYX_NON_INTERACTIVE=1 "$MYX_BIN" inspect smoke --json >inspect.json
  MYX_POLICY_MODE=permissive MYX_NON_INTERACTIVE=1 "$MYX_BIN" run smoke.echo_text --input '{"text":"hello"}' --json >run.json
  MYX_POLICY_MODE=permissive MYX_NON_INTERACTIVE=1 "$MYX_BIN" build --target mcp --package smoke --json >/dev/null

  python3 - <<'PY'
import json
from pathlib import Path

inspect_payload = json.loads(Path("inspect.json").read_text())
assert inspect_payload["ok"] is True
assert inspect_payload["identity"]["name"] == "smoke"

run_payload = json.loads(Path("run.json").read_text())
assert run_payload["ok"] is True
assert run_payload["result"]["kind"] == "subprocess"
assert "hello" in (run_payload["result"]["stdout"] or "")

assert Path(".myx/mcp/server.json").exists()
assert Path(".myx/mcp/runtime-config.json").exists()
assert Path(".myx/mcp/launch.json").exists()
assert Path(".myx/mcp/run.sh").exists()
PY
)

echo "MVP smoke loop passed (init -> add -> inspect -> run -> build)."
