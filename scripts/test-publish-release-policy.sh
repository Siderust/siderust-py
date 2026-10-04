#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

expect_failure() {
  local name="$1"
  local expected="$2"
  shift 2

  local log="$tmp_dir/${name}.log"
  if "$@" >"$log" 2>&1; then
    echo "error: expected '$name' to fail" >&2
    cat "$log" >&2
    exit 1
  fi

  if ! grep -F "$expected" "$log" >/dev/null; then
    echo "error: '$name' failed without expected message: $expected" >&2
    cat "$log" >&2
    exit 1
  fi
}

cp pyproject.toml "$tmp_dir/pyproject-mismatch.toml"
python3 - "$tmp_dir/pyproject-mismatch.toml" <<'PY'
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
updated, count = re.subn(
    r'(?ms)^(\[project\]\s.*?^version\s*=\s*)"[^"]+"',
    r'\1"9.9.9"',
    text,
    count=1,
)
if count != 1:
    raise SystemExit("could not rewrite [project].version for mismatch test")
path.write_text(updated, encoding="utf-8")
PY

expect_failure   version-mismatch   "does not match pyproject.toml"   env SIDERUST_PY_PYPROJECT_FILE="$tmp_dir/pyproject-mismatch.toml"   bash scripts/publish-release.sh --dry-run

expect_failure   wrong-tag   "does not match manifest version"   bash scripts/publish-release.sh --dry-run --tag v0.2.2

expect_failure   publish-without-tag   "publishing requires a version tag"   env -u GITHUB_REF_TYPE -u GITHUB_REF_NAME   bash scripts/publish-release.sh

echo "Release policy guards passed."
