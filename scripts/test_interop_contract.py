"""Build both PyO3 extensions and run their interoperability contract tests."""

from __future__ import annotations

import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONSUMER = ROOT / "tests" / "interop_consumer"
TEST = ROOT / "tests" / "test_interop.py"


def run(*args: str, cwd: Path = ROOT) -> None:
    subprocess.run(args, cwd=cwd, check=True)


def main() -> None:
    python = sys.executable

    run(python, "-m", "pip", "install", "--upgrade", "pip")
    run(python, "-m", "pip", "install", "pytest")
    run(python, "-m", "pip", "install", "--force-reinstall", str(ROOT))
    run(python, "-m", "pip", "install", "--force-reinstall", str(CONSUMER))

    with tempfile.TemporaryDirectory() as temp_dir:
        run(
            python,
            "-m",
            "pytest",
            "--rootdir",
            str(ROOT),
            str(TEST),
            cwd=Path(temp_dir),
        )


if __name__ == "__main__":
    main()
