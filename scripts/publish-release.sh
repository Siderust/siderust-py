#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRATE_NAME="siderust-py"
DRY_RUN=false
TAG=""

usage() {
  cat <<'EOF'
Usage: scripts/publish-release.sh [--dry-run] [--tag vX.Y.Z]

Validates release metadata and the packaged crate before publishing.
When --dry-run is supplied, all verification is performed without uploading.
EOF
}

while (($#)); do
  case "$1" in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --tag)
      if (($# < 2)); then
        echo "error: --tag requires a value" >&2
        exit 2
      fi
      TAG="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

cd "$ROOT_DIR"

cargo_version="$(
  cargo metadata --no-deps --format-version 1 |
    python3 -c 'import json, sys; data = json.load(sys.stdin); print(next(p["version"] for p in data["packages"] if p["name"] == "siderust-py"))'
)"

python_version="$(
  python3 - <<'PY'
from pathlib import Path
import re

text = Path("pyproject.toml").read_text(encoding="utf-8")
match = re.search(r'(?ms)^\[project\]\s.*?^version\s*=\s*"([^"]+)"', text)
if match is None:
    raise SystemExit("could not read [project].version from pyproject.toml")
print(match.group(1))
PY
)"

if [[ "$cargo_version" != "$python_version" ]]; then
  echo "error: Cargo.toml version ($cargo_version) does not match pyproject.toml ($python_version)" >&2
  exit 1
fi

if [[ -z "$TAG" && "${GITHUB_REF_TYPE:-}" == "tag" ]]; then
  TAG="${GITHUB_REF_NAME:-}"
fi

expected_tag="v$cargo_version"
if [[ -n "$TAG" && "$TAG" != "$expected_tag" ]]; then
  echo "error: release tag '$TAG' does not match manifest version '$expected_tag'" >&2
  exit 1
fi

crate_url="https://crates.io/api/v1/crates/$CRATE_NAME/$cargo_version"
http_status="$(
  curl --retry 3 --silent --show-error --output /dev/null --write-out '%{http_code}'     --user-agent "siderust-py-release-script" "$crate_url"
)"

already_published=false
case "$http_status" in
  200)
    already_published=true
    ;;
  404)
    ;;
  *)
    echo "error: crates.io version check failed with HTTP $http_status" >&2
    exit 1
    ;;
esac

if [[ "$already_published" == true && "$DRY_RUN" == false ]]; then
  echo "$CRATE_NAME $cargo_version is already published; nothing to do."
  exit 0
fi

echo "Validating $CRATE_NAME $cargo_version..."
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo test --doc
cargo package

package_archive="target/package/$CRATE_NAME-$cargo_version.crate"
if [[ ! -f "$package_archive" ]]; then
  echo "error: cargo package did not produce $package_archive" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

mkdir -p "$tmp_dir/package" "$tmp_dir/consumer/src"
tar -xzf "$package_archive" -C "$tmp_dir/package"

version_requirement="${cargo_version%.*}"
cat > "$tmp_dir/consumer/Cargo.toml" <<EOF
[package]
name = "siderust-py-packaged-consumer"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
siderust-py = { path = "../package/$CRATE_NAME-$cargo_version", version = "$version_requirement" }
EOF

cat > "$tmp_dir/consumer/src/lib.rs" <<'EOF'
use siderust_py::interop::{
    direction_from_python, direction_to_python, observer_from_python, observer_to_python,
};

pub fn assert_public_interop_api() {
    let _ = observer_from_python;
    let _ = observer_to_python;
    let _ = direction_from_python;
    let _ = direction_to_python;
}
EOF

cargo check --manifest-path "$tmp_dir/consumer/Cargo.toml"
cargo publish --dry-run

if [[ "$DRY_RUN" == true ]]; then
  echo "Dry-run validation succeeded for $CRATE_NAME $cargo_version."
  exit 0
fi

cargo publish
