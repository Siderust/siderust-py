"""Offline checks for invalid generator configuration and kernel provenance."""

import hashlib
import json
import subprocess
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace

from provenance import check_kernel, check_toolkit, provenance


class ProvenanceTests(unittest.TestCase):
    def test_missing_and_corrupt_kernel(self):
        with TemporaryDirectory() as directory:
            path = Path(directory) / "de440s.bsp"
            hashes = {path.name: hashlib.sha256(b"expected").hexdigest()}
            with self.assertRaisesRegex(ValueError, "missing kernel"):
                check_kernel(path, path.name, hashes)
            path.write_bytes(b"invalid")
            with self.assertRaisesRegex(ValueError, "SHA-256 mismatch"):
                check_kernel(path, path.name, hashes)

    def test_matching_kernel(self):
        content = b"kernel contents"
        digest = hashlib.sha256(content).hexdigest()
        with TemporaryDirectory() as directory:
            path = Path(directory) / "example.bsp"
            path.write_bytes(content)
            (path.parent / "provenance.json").write_text(
                json.dumps({"source_sha256": {path.name: digest}}), encoding="ascii"
            )
            hashes = provenance(path.parent)["source_sha256"]
            self.assertEqual(check_kernel(path, path.name, hashes), digest)

    def test_toolkit_mismatch(self):
        expected = {"generator": "SpiceyPy 8.3.0", "toolkit": "CSPICE_N0067"}
        version = expected["generator"].removeprefix("SpiceyPy ")
        check_toolkit(
            SimpleNamespace(__version__=version, tkvrsn=lambda _: expected["toolkit"]), expected
        )
        with self.assertRaisesRegex(ValueError, "expected"):
            check_toolkit(SimpleNamespace(__version__=version, tkvrsn=lambda _: "other"), expected)

    def test_invalid_output_mode_exits_before_generation(self):
        script = Path(__file__).with_name("cspice_lunar_de440_references.py")
        with TemporaryDirectory() as directory:
            result = subprocess.run(
                [sys.executable, script, "--verify", "--output-dir", directory],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 2)
            self.assertIn("not allowed with argument", result.stderr)
