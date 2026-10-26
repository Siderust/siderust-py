# Cross-extension interoperability

The public `siderust_py::interop` Rust module lets an independently compiled
PyO3 extension accept and return the canonical `siderust.Observer`,
`siderust.direction.ICRS`, and `siderust.direction.Horizontal` classes from the
installed Python package.

## Supported baseline

- Siderust: `0.15.x`
- qtty: `0.8.x`
- tempoch: `0.11.x`
- affn: `0.12.x`
- PyO3: `0.30.x`
- tempoch-py: `0.5.x` when exchanging `Time<UTC>` or `Period<UTC>`
- siderust-py library outputs: `cdylib` and `rlib`

Use maturin 1.9.4 or newer. PyO3's `extension-module` feature is deliberately
not enabled in Cargo manifests: maturin sets `PYO3_BUILD_EXTENSION_MODULE` for
extension builds, while ordinary Rust tests and consumers remain linkable.

## Why the boundary uses primitives

A PyO3 `#[pyclass]` belongs to the extension module that registered it. If a
downstream shared library compiles another copy of siderust-py's private
wrapper, that copy is not the same Python type as the installed package's
class.

The interop API instead imports `siderust._siderust` and invokes narrow bridge
functions there. The canonical extension performs its own type checks and
constructs its own objects. Only primitive scalar values (`f64` payload fields
and the `u32` protocol version) cross the shared-library boundary.

- `ObserverParts` carries east-positive WGS84 geodetic longitude in degrees,
  north-positive latitude in degrees, and ellipsoidal height in metres. It
  reconstructs exactly `Geodetic<ECEF>`.
- `IcrsParts` carries right ascension and declination in degrees in the
  Siderust `direction::ICRS` frame.
- `HorizontalParts` carries altitude above the geometric horizon and
  North-clockwise azimuth (through East) in degrees for
  `direction::Horizontal`. No atmospheric refraction is applied.

## Protocol compatibility

The canonical extension exposes a private integer `_bridge_protocol_version`.
The Rust interop API checks it before calling any bridge hook. Protocol 1
defined Observer and implicit ICRS direction primitives. Protocol 2 uses explicit
`_bridge_icrs_{from,to}_parts` and `_bridge_horizontal_{from,to}_parts` hooks
for the classes in `siderust.direction`; Observer hooks are unchanged. The
provisional `_bridge_direction_*` and `_bridge_horizontal_direction_*` hooks
are removed. Rebuild consumers against 0.6; no protocol 1 fallback exists.
A missing, malformed, or different protocol produces `ImportError` with the expected and installed
versions instead of failing later with an obscure missing-function error.

The Rust `siderust-py` dependency linked into a downstream extension and the
installed Python `siderust` package must support the same bridge protocol.
This protocol version, rather than the package semantic version, is the
compatibility contract.

## Cargo setup

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
pyo3 = "0.30"
siderust-py = "0.6"
tempoch-py = "0.5"
```

For unreleased siderust-py changes, pin a Git dependency to an immutable tag or
full commit hash rather than tracking a moving branch:

```toml
siderust-py = { git = "https://github.com/Siderust/siderust-py.git", rev = "<40-character commit SHA>" }
```

For adjacent checkouts during workspace development, use
`siderust-py = { path = "../siderust.py" }`.

## Observer example

```rust
use pyo3::prelude::*;
use siderust_py::interop::{observer_from_python, observer_to_python, ObserverParts};

#[pyfunction]
fn inspect_observer(value: &Bound<'_, PyAny>) -> PyResult<(f64, f64, f64)> {
    let observer = observer_from_python(value)?;
    let parts = ObserverParts::from(&observer);
    Ok((parts.longitude_degrees, parts.latitude_degrees, parts.height_metres))
}

#[pyfunction]
fn copy_observer(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    observer_to_python(py, &observer_from_python(value)?)
}
```

`copy_observer()` returns the installed package's actual class, so
`type(result) is siderust.Observer` is true.

## ICRS example

```rust
use pyo3::prelude::*;
use siderust_py::interop::{icrs_from_python, icrs_to_python, IcrsParts};

#[pyfunction]
fn inspect_direction(value: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    let direction = icrs_from_python(value)?;
    let parts = IcrsParts::from(&direction);
    Ok((parts.right_ascension_degrees, parts.declination_degrees))
}

#[pyfunction]
fn copy_direction(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    icrs_to_python(py, &icrs_from_python(value)?)
}
```

Objects of the wrong canonical type raise Python `TypeError`. Downstream code
should use these functions rather than depending on siderust-py's private
`#[pyclass]` implementation types.

## Horizontal example

```rust
use pyo3::prelude::*;
use siderust_py::interop::{
    horizontal_from_python, horizontal_to_python,
    HorizontalParts,
};

#[pyfunction]
fn inspect_horizontal(value: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    let direction = horizontal_from_python(value)?;
    let parts = HorizontalParts::from(&direction);
    Ok((parts.altitude_degrees, parts.azimuth_degrees))
}

#[pyfunction]
fn copy_horizontal(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    horizontal_to_python(py, &horizontal_from_python(value)?)
}
```

## Coordinated release and downstream validation

Siderust 0.15.0, Tempoch 0.11.1, Affn 0.12.1 and PyO3 0.30.1 are published
upstream dependencies. Both bridges use the same provider identities; do not
combine the 0.4 Rust bridges with the 0.5 graph. PyO3 0.30 requires Python 3.9+
and Rust 1.85+. This project retains its existing Python 3.9–3.14 matrix.

Tempoch's 0.11 upgrade changes its Affn identity; Siderust 0.15 changes its
public Affn identity as well. The canonical observer, direction and typed time
APIs remain usable without casts or conversion wrappers. PyO3 0.30 removes
the deprecated `skip_from_py_object` attribute, so downstream extensions must
remove that attribute during migration.

The downstream fixture links both bridges and explicitly assigns Tempoch
conversion results to `siderust::time::Time<UTC>` and `Period<UTC>`. Its Python
tests exercise UTC offset normalization, subsecond precision and rejection of
naive datetimes alongside the canonical Observer/ICRS contract. The
bridge-contract job runs on Linux, macOS and Windows.

For this release, both the Rust crate and Python package remain at 0.6.0.
Consumers of 0.5 must replace `DirectionParts` with `IcrsParts`,
`direction_{from,to}_python` with `icrs_{from,to}_python`, and
`direction_parts_from_python` with `icrs_parts_from_python`. Horizontal uses
`HorizontalParts`, `horizontal_{from,to}_python` and
`horizontal_parts_from_python`. Never register copies of the private wrappers.
The Python migration is documented in [the direction API](../direction.md).

Both release pipelines validate rebuilt source distributions outside the
checkout, then validate wheel/sdist metadata before crates.io publication.
PyPI publication depends on successful crates.io publication or verified
existence of that version. OIDC Trusted Publishing remains enabled; no release
is initiated by these validation steps.
