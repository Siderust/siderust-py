# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
