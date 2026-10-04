# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-10-04

### Added

- Added automated PyPI publishing through GitHub Actions and PyPI Trusted Publishing/OIDC.
- Added cross-platform release wheels for Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x86_64, plus a source distribution.
- Added CI validation of built Python distribution artifacts before release tags are created.
- Added declared and tested CPython 3.13 and 3.14 support.

### Changed

- Consolidated tag-driven PyPI and crates.io publication into a single release workflow.
- Kept crates.io publication idempotent: an already-published crate version is detected and skipped instead of failing the release.

## [0.2.0] - 2026-10-04

### Added

- Added `rlib` output alongside the Python `cdylib` so downstream Rust/PyO3
  crates can depend on `siderust-py`.
- Added the public `siderust_py::interop` API for converting Observer and
  Direction values across independently compiled Python extensions.
- Added a versioned, primitive-only bridge protocol that preserves canonical
  Python type identity across extension boundaries.
- Added independent downstream-consumer and bridge-contract tests for the
  Observer/Direction interoperability surface.

### Changed

- Updated the bindings to Siderust 0.12.x and affn 0.10.x.
- Aligned the extension and downstream consumer with PyO3 0.29.x.
- Kept Rust and Python package version metadata synchronized at 0.2.0.

### Removed

- Removed the legacy Siderust submodule and obsolete dependency overrides.

## [0.1.0] - 2026-03-07

### Added

- Initial `siderust` Python bindings backed by the Siderust Rust library.
