"""Release metadata and exact-tag changelog extraction regression tests."""

import subprocess
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "release_notes.py"
CHANGELOG = """# Changelog

## [Unreleased]
- Planned.

## [1.2.3] - 2026-10-26

### Added
- New directions.

### Fixed
- Canonical imports.

## [1.2.2] - 2026-10-20
- Older fix.
"""


def invoke(tmp_path, tag="v1.2.3", log=CHANGELOG, citation='version: "1.2.3"\n'):
    changelog = tmp_path / "CHANGELOG.md"
    cff = tmp_path / "CITATION.cff"
    output = tmp_path / "notes.md"
    changelog.write_text(log, encoding="utf-8")
    cff.write_text(citation, encoding="utf-8")
    result = subprocess.run(
        [sys.executable, str(SCRIPT), "--tag", tag, "--changelog", str(changelog),
         "--citation", str(cff), "--output", str(output)],
        check=False, capture_output=True, text=True,
    )
    return result, output


def test_exact_section_only(tmp_path):
    result, output = invoke(tmp_path)
    assert result.returncode == 0, result.stderr
    notes = output.read_text(encoding="utf-8")
    assert "New directions." in notes and "Canonical imports." in notes
    assert "Planned." not in notes and "Older fix." not in notes
    assert not notes.startswith("## [1.2.3]")


def test_missing_section_fails(tmp_path):
    result, output = invoke(tmp_path, tag="v1.2.4", citation='version: "1.2.4"\n')
    assert result.returncode != 0 and "exactly one" in result.stderr
    assert not output.exists()


def test_duplicate_section_fails(tmp_path):
    result, _ = invoke(tmp_path, log=CHANGELOG + "\n## [1.2.3]\n- Duplicated.\n")
    assert result.returncode != 0 and "exactly one" in result.stderr


def test_empty_release_notes_fail(tmp_path):
    result, _ = invoke(tmp_path, log="## [1.2.3]\n### Added\n\n## [1.2.2]\n- Old.\n")
    assert result.returncode != 0 and "no release notes" in result.stderr


def test_citation_mismatch_fails(tmp_path):
    result, output = invoke(tmp_path, citation='version: "1.2.2"\n')
    assert result.returncode != 0 and "does not match" in result.stderr
    assert not output.exists()


def test_missing_citation_version_fails(tmp_path):
    result, _ = invoke(tmp_path, citation="cff-version: 1.2.0\n")
    assert result.returncode != 0 and "exactly one numeric version" in result.stderr


def test_invalid_tags_fail(tmp_path):
    for tag in ("1.2.3", "v1.2.3-rc1", "v1.2", "v1.2.3\nx"):
        result, output = invoke(tmp_path, tag=tag)
        assert result.returncode != 0 and "invalid release tag" in result.stderr
        assert not output.exists()
