"""Validate the reference environment against the published DE440 provenance."""

import hashlib
import json


def provenance(fixtures):
    return json.loads((fixtures / "provenance.json").read_text(encoding="ascii"))


def check_kernel(path, name, expected_hashes):
    expected = expected_hashes[name]
    if not path.is_file():
        raise ValueError(f"missing kernel: {path}")
    with path.open("rb") as kernel:
        actual = hashlib.file_digest(kernel, "sha256").hexdigest()
    if actual != expected:
        raise ValueError(f"SHA-256 mismatch for {path}: expected {expected}, got {actual}")
    return actual


def check_kernels(directory, expected_hashes):
    return {name: check_kernel(directory / name, name, expected_hashes) for name in expected_hashes}


def check_toolkit(spice, expected):
    generator = f"SpiceyPy {spice.__version__}"
    toolkit = spice.tkvrsn("TOOLKIT")
    if (generator, toolkit) != (expected["generator"], expected["toolkit"]):
        raise ValueError(
            f"expected {expected['generator']} / {expected['toolkit']}, got {generator} / {toolkit}"
        )
