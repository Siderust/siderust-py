# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0]

### Added

- Public `siderust.time` facade exposing the original `tempoch` Python time
  types, conversions, exceptions and interval helpers without duplicate classes.

### Changed

- Adopt published crates.io Siderust **0.18.0** with the `spice` feature in
  both the bindings and independent interoperability consumer. PyO3 0.30.1
  and bridge protocol 2 are unchanged.
- Inherit canonical IAU 2006 J2000 frame normalization for VSOP87/ELP2000
  outputs and the lunar TT-to-TDB correction from the core. Frame consistency
  does not imply greater intrinsic ephemeris-model accuracy.
- Install compatible `tempoch` automatically with `pip install siderust`;
  preserve its wheel/sdist validation and the time-period example.
- Synchronize Rust, Python and citation versions at 0.9.0.

### Fixed

- Return actionable Python `ValueError` for unavailable runtime ephemeris states
  and center-conversion failures instead of propagating a Rust panic.
- Reject the incorrect analytical `MoonPrincipalAxes` alias consistently in
  coordinate constructors, transforms, dictionaries and pickle reconstruction.
  Explicit kernel-backed SPICE `MOON_PA` / `MOON_ME` semantics are unchanged.

### Breaking changes

- Analytical lunar coordinates must name `MoonIauMeanEarth`. Legacy serialized
  `MoonPrincipalAxes` inputs now raise `ValueError`: explicitly rename them only
  if their original meaning was analytical `IAU_MOON`. Genuine principal-axes
  coordinates require their matching binary PCK/frame kernels; never relabel them.
- Corrected VSOP87/ELP2000 frame conventions and lunar time arguments change
  analytical numerical results. Review downstream snapshots using their physical
  frame, center, epoch and model conventions; no Python-side rotation is needed.
- Core SPK loaders strictly reject unsupported frames/types and preserve selected
  segment precedence. Kernel errors surface as Python exceptions, without
  silently substituting analytical data.

## [0.8.0] - 2026-10-29

### Added

- SPICE kernel context bindings (`SpiceContext.from_kernels`, `from_paths`,
  `from_de440`) wrapping an immutable Rust `SpiceContext` behind `Arc`
  ([#19](https://github.com/Siderust/siderust-py/issues/19)).
- `LunarObserver.from_spice(...)` for DE-specific `MOON_ME` / `MOON_PA` (and
  related FK) geometric Sun/Earth observations with finite-distance parallax,
  slant range in kilometres, and `above_horizon` semantics.
- Height-above-IAU-sphere and actual radial-distance site constructors
  (`height_m` / `radius_m`, mutually exclusive).
- Exact-ET observation methods `observe_sun_et` / `observe_earth_et` for
  same-epoch CSPICE parity; ordinary `observe_sun` / `observe_earth` keep the
  Julian Date TT convention with explicit TT→TDB conversion.
- Independent CSPICE DE440 fixture suite under `tests/fixtures/lunar_de440`
  (from Siderust PR #125) and beginner example
  `examples/15_spice_lunar_de440.py`.

### Changed

- Depend on published crates.io `siderust` **0.17.0** with the `spice` feature
  (DE440 SPICE lunar surface geometry from
  [Siderust/siderust#125](https://github.com/Siderust/siderust/pull/125)).
- Synchronize crate / PyPI / `__version__` metadata at 0.8.0.
- Document the Siderust compatibility baseline as `0.17.x` for interop consumers.
- Extend `LunarObservation` with optional provenance fields (`backend`,
  `frame`, `frame_id`, `above_horizon`, `epoch_et_tdb_seconds`, `kernels`)
  without breaking existing `alt_deg` / `az_deg` / `range` / `range_unit` /
  `direction()` usage.

### Fixed

- Keep analytical and SPICE lunar observers scientifically distinct for
  equality, hashing, `to_dict` / `from_dict`, and pickle (SPICE observers are
  never silently reconstructed as analytical).
- SPICE observer equality/hashing keys on the shared `Arc<SpiceContext>`
  allocation so independently loaded kernel configurations remain distinct
  even when path aliases match.
- Reject unknown or malformed `LunarObserver.from_dict` backends instead of
  falling through to the analytical constructor.

## [0.7.0] - 2026-10-28

### Added

- Lunar-surface observer bindings (`LunarObserver`, `LunarObservation`) wrapping
  Rust `SurfaceSite<Moon>` geometric Alt/Az for ICRS directions, finite-distance
  positions, and Earth/Sun from a lunar site ([#17](https://github.com/Siderust/siderust-py/issues/17)).
- Selenocentric centre and `MoonIauMeanEarth` body-fixed frame support on
  `Position` / `SphericalPosition`, with `MoonPrincipalAxes` accepted as the
  documented legacy alias for NAIF `IAU_MOON` Mean-Earth (not JPL DE `MOON_PA`).
- Scientific validation tests mirroring Siderust v0.16.0 body-surface references
  (including offline CSPICE composite Alt/Az) and example
  `examples/14_lunar_surface_altaz.py`.
- Documentation for lunar-surface conventions in `doc/lunar_surface.md`.
- Citation metadata for the Python bindings and guidance to cite the core
  Siderust library and underlying scientific models and datasets.
- Publish a GitHub Release for each successful version-tagged PyPI/crates.io
  release, with version-matched CHANGELOG notes and safe reruns.

### Changed

- Upgrade the Rust dependency to published `siderust` 0.16.0 (lunar surface
  geometry, selenocentric centres, and IAU Moon orientation fixes).
- Synchronize the Rust crate and Python distribution versions at 0.7.0.
- Document the Siderust compatibility baseline as `0.16.x` for interop consumers.
- Show the Python distribution's PyPI version in the README badge, while
  keeping docs.rs for the Rust interoperability crate.

### Fixed

- Make `LunarObserver` equality exact and hash-consistent (including signed zeros
  and longitude-normalized equivalents), restoring the Python `a == b ⇒ hash(a)
  == hash(b)` invariant.
- Normalize ICRS observation vectors with a max-abs scale so extreme but finite
  magnitudes (e.g. `1e±200`) do not overflow or underflow.
- Route `Position.from_dict` / `SphericalPosition.from_dict` through the public
  constructors so non-finite coordinates cannot bypass validation.

## [0.6.0] - 2026-10-26

### Added

- Real importable `siderust.direction` module with canonical `ICRS` and
  `Horizontal` classes backed by Rust spherical directions. Horizontal contains
  geometric altitude and North-clockwise azimuth through East, without observer,
  epoch, refraction or object identity.

### Changed

- **Breaking:** `Direction` → `direction.ICRS` and the provisional
  `HorizontalDirection` → `direction.Horizontal`. Import via
  `from siderust import direction` or `from siderust.direction import ICRS, Horizontal`.
  No legacy aliases remain. Target, Star tracking, observing queries and proper
  motion use ICRS explicitly; horizontal directions are rejected by these APIs.
- **Breaking:** bridge protocol 2 uses `IcrsParts`, `icrs_{from,to}_python`,
  `icrs_parts_from_python`, `HorizontalParts`, `horizontal_{from,to}_python`
  and `horizontal_parts_from_python`. Private hooks use `_bridge_icrs_*` and
  `_bridge_horizontal_*`. Observer hooks are unchanged. Consumers must rebuild
  against siderust-py 0.6 and install a matching protocol-2 Python package.
- Directions are immutable values with exact canonical-coordinate equality.
  Hashing agrees with equality, including signed zero; use angular separation
  and an explicit tolerance for approximate comparisons.
- ICRS vector conversion, dot product and separation delegate to the Rust core.
- Reject non-finite ProperMotion rates and propagation dates; validate propagated
  directions before exposing them. Reject non-finite custom Star coordinates
  so tracking cannot produce invalid ICRS values. Document the core's J2000 catalog
  approximation for proper-motion propagation.

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
