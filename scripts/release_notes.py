#!/usr/bin/env python3
"""Validate CITATION.cff and extract exact-tag GitHub Release changelog notes."""

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TAG = re.compile(r"v(?P<version>\d+\.\d+\.\d+)\Z")
HEADING = re.compile(r"^## \[(?P<version>[^\]]+)\](?: - \d{4}-\d{2}-\d{2})?[ \t]*$", re.MULTILINE)
CFF_VERSION = re.compile(r"""^version:\s*["']?(?P<version>\d+\.\d+\.\d+)["']?[ \t]*$""", re.MULTILINE)


def extract_notes(changelog: str, version: str) -> str:
    headings = list(HEADING.finditer(changelog))
    indices = [i for i, heading in enumerate(headings) if heading.group("version") == version]
    if len(indices) != 1:
        raise ValueError(f"CHANGELOG.md must contain exactly one [{version}] section (found {len(indices)})")
    index = indices[0]
    start = headings[index].end()
    end = headings[index + 1].start() if index + 1 < len(headings) else len(changelog)
    notes = changelog[start:end].strip()
    if not any(line.strip() and not line.lstrip().startswith("#") for line in notes.splitlines()):
        raise ValueError(f"CHANGELOG.md section [{version}] has no release notes")
    return notes + "\n"


def citation_version(citation: str) -> str:
    versions = CFF_VERSION.findall(citation)
    if len(versions) != 1:
        raise ValueError("CITATION.cff must contain exactly one numeric version field")
    return versions[0]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--changelog", type=Path, default=ROOT / "CHANGELOG.md")
    parser.add_argument("--citation", type=Path, default=ROOT / "CITATION.cff")
    args = parser.parse_args()
    match = TAG.fullmatch(args.tag)
    if not match:
        parser.error(f"invalid release tag {args.tag!r}; expected vMAJOR.MINOR.PATCH")
    version = match.group("version")
    try:
        citation = args.citation.read_text(encoding="utf-8")
        changelog = args.changelog.read_text(encoding="utf-8")
        cff_version = citation_version(citation)
        if cff_version != version:
            raise ValueError(f"CITATION.cff version {cff_version} does not match release tag {args.tag}")
        notes = extract_notes(changelog, version)
    except (OSError, ValueError) as error:
        parser.error(str(error))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(notes, encoding="utf-8")


if __name__ == "__main__":
    main()
