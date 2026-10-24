# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.1] - 2026-10-24

### Fixed

- Eliminate unsound Python-object-to-Rust-payload casts in query dispatch by
  retaining PyO3 borrow guards throughout Star and Direction operations, without
  cloning targets or changing public Python or Rust signatures.
- Cover Body, catalog and custom Star, and Direction dispatch with numerical
  altitude/azimuth method parity, threshold period partitions, crossing roots
  and directions, culmination extrema, determinism, and invalid-input regressions
  for [#13](https://github.com/Siderust/siderust-py/issues/13).
- Reject non-finite search window endpoints, as well as empty or reversed windows,
  before passing them to threshold and event searches.

## [0.5.0] - 2026-10-23

### Changed

- **Breaking:** upgrade the Rust dependency to `siderust` 0.15.0 with public Affn 0.12.1 and Tempoch 0.11.1 type identity.
- **Breaking:** align PyO3 with 0.30.1; drop CPython 3.8 (PyO3 0.30 floor is 3.9+).
- Synchronize the Rust crate and Python distribution versions at 0.5.0.
- Exercise Siderust's typed UTC times and periods through the published
  `tempoch-py` 0.5 bridge in the downstream consumer regression suite.
- Validate isolated sdist rebuilds and installed package metadata before release;
  publish to PyPI only after artifact validation and crates.io completion.
- Commit the release Cargo.lock and use locked Rust/wheel validation builds.

### Fixed

- Derive the packaged consumer's PyO3 requirement from Cargo metadata instead of
  retaining an incompatible 0.29 requirement after the 0.30 upgrade.
- Run the complete Python 3.9–3.14 test matrix on Ubuntu 24.04, which provides
  setup-python binaries for Python 3.9.

## [0.4.0] - 2026-10-12

### Changed

- Temporarily pin the Siderust and Principia release preparation commits until their crates.io releases.

- **Breaking:** upgrade the Rust dependency to `siderust` 0.14.0, using `tempoch` 0.10 time primitives.
- Synchronize the Rust crate and Python distribution versions at 0.4.0.

## [0.3.0] - 2026-10-11

### Changed

- Updated the Rust dependency to `siderust` 0.13.0, including its `tempoch` 0.9.x time primitives.
- Aligned the direct `affn` dependency with Siderust at 0.11.x.
- Synchronized the Rust crate and Python distribution versions at 0.3.0.

## [0.2.2] - 2026-10-04

### Fixed

- Reject non-finite longitude, latitude, height, right ascension, and declination
  values when constructing canonical Python `Observer` and `Direction` objects.
- Apply the same finite-value invariant to cross-extension bridge construction so
  downstream PyO3 consumers cannot recreate invalid canonical Python objects.

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
