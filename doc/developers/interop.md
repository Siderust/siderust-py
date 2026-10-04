# Cross-extension interoperability

The public `siderust_py::interop` Rust module lets an independently compiled
PyO3 extension accept and return the canonical `siderust.Observer` and
`siderust.Direction` classes from the installed Python package.

## Supported baseline

- Siderust: `0.12.x`
- qtty: `0.8.x`
- tempoch: `0.7.x`
- affn: `0.10.x`
- PyO3: `0.29.x`
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
constructs its own objects. Only `f64` values cross the shared-library
boundary.

- `ObserverParts` carries east-positive WGS84 geodetic longitude in degrees,
  north-positive latitude in degrees, and ellipsoidal height in metres. It
  reconstructs exactly `Geodetic<ECEF>`.
- `DirectionParts` carries right ascension and declination in degrees in the
  current Siderust `direction::ICRS` frame (ICRS axes aligned to J2000).

## Cargo setup

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
pyo3 = "0.29"
siderust-py = { git = "https://github.com/Siderust/siderust-py.git" }
```

For adjacent checkouts, use
`siderust-py = { path = "../siderust.py" }` instead.

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

## Direction example

```rust
use pyo3::prelude::*;
use siderust_py::interop::{direction_from_python, direction_to_python, DirectionParts};

#[pyfunction]
fn inspect_direction(value: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    let direction = direction_from_python(value)?;
    let parts = DirectionParts::from(&direction);
    Ok((parts.right_ascension_degrees, parts.declination_degrees))
}

#[pyfunction]
fn copy_direction(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    direction_to_python(py, &direction_from_python(value)?)
}
```

Objects of the wrong canonical type raise Python `TypeError`. Downstream code
should use these functions rather than depending on siderust-py's private
`#[pyclass]` implementation types.
