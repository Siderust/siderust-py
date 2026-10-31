#!/usr/bin/env bash

# Requires Python 3.11+ and Rust. Build solely from the archive with PEP 517,
# then run the existing Python suite against a fresh installation.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ "$#" -ne 1 || ! -f "$1" ]]; then
  echo "Usage: $0 path/to/siderust-version.tar.gz (exactly one sdist)" >&2
  exit 2
fi

workdir="$(mktemp -d /tmp/siderust-sdist.XXXXXXXX)"
trap 'rm -rf "$workdir"' EXIT
cp "$1" "$workdir/source.tar.gz"
cp "$ROOT/pyproject.toml" "$workdir/pyproject.toml"
mkdir "$workdir/tests"
cp "$ROOT/tests/test_api.py" "$ROOT/tests/test_direction.py" "$ROOT/tests/test_time.py" "$workdir/tests/"

python3 -m venv "$workdir/build-env"
python3 -m venv "$workdir/test-env"
cd "$workdir"

# A fresh Cargo home prevents local registry patches/config from hiding missing
# published dependencies. pip installs the archive's declared build backend.
CARGO_HOME="$workdir/cargo-home" CARGO_TARGET_DIR="$workdir/target" \
  "$workdir/build-env/bin/python" -I -m pip wheel \
    --use-pep517 --no-deps --no-cache-dir --wheel-dir "$workdir/wheels" \
    "$workdir/source.tar.gz"

"$workdir/test-env/bin/python" -I -m pip install pytest
"$workdir/test-env/bin/python" -I -m pip install "$workdir"/wheels/*.whl

"$workdir/test-env/bin/python" -I - <<'PY'
from importlib.metadata import distribution
from pathlib import Path
import sys
import tomllib

import siderust
import siderust._siderust as extension
import tempoch
from siderust import time

project = tomllib.loads(Path("pyproject.toml").read_text())["project"]
dist = distribution(project["name"])
assert dist.metadata["Name"] == project["name"], dist.metadata["Name"]
assert dist.version == project["version"], dist.version
assert siderust.__version__ == project["version"], siderust.__version__
assert any(req.lower().startswith("tempoch") for req in (dist.requires or []))
assert time.JulianDate is tempoch.JulianDate
prefix = Path(sys.prefix).resolve()
for location in (
    siderust.__file__, extension.__file__, time.__file__, tempoch.__file__,
    dist.locate_file(""), distribution("tempoch").locate_file(""),
):
    path = Path(location).resolve()
    assert path.is_relative_to(prefix), f"Import/metadata outside clean venv: {path}"
    print(f"Validated installed distribution: {path}")
print(f"Validated {dist.metadata['Name']} {dist.version}")
PY

"$workdir/test-env/bin/python" -I -m pytest \
  --import-mode=importlib --rootdir "$workdir" "$workdir/tests"
