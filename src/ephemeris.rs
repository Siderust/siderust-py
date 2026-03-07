//! Runtime-loaded JPL DE4xx ephemeris backend for Python.
//!
//! Wraps `siderust::calculus::ephemeris::RuntimeEphemeris` to load BSP files
//! at runtime and query Sun, Earth, and Moon positions.

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use siderust::calculus::ephemeris::{DynEphemeris, RuntimeEphemeris};
use tempoch::JulianDate;

use crate::position::{
    PyPosition, CENTER_BARY, CENTER_GEO, CENTER_HELIO, FRAME_ECL, UNIT_AU, UNIT_KM,
};

/// A runtime-loaded JPL DE4xx ephemeris backend.
///
/// Load a BSP file from disk or from bytes, then query Sun, Earth, and Moon
/// positions without compile-time feature flags.
///
/// Example:
///     >>> eph = RuntimeEphemeris.from_bsp("de440.bsp")
///     >>> sun = eph.sun_barycentric(2451545.0)
///     >>> print(sun.distance())
#[pyclass(name = "RuntimeEphemeris", module = "siderust", skip_from_py_object)]
#[derive(Clone)]
pub struct PyRuntimeEphemeris {
    inner: RuntimeEphemeris,
}

#[pymethods]
impl PyRuntimeEphemeris {
    /// Load a runtime ephemeris from a BSP file on disk.
    ///
    /// Args:
    ///     path: Path to a JPL DE4xx BSP file (e.g. "de440.bsp").
    ///
    /// Raises:
    ///     IOError: If the file cannot be read or parsed.
    #[staticmethod]
    fn from_bsp(path: &str) -> PyResult<Self> {
        RuntimeEphemeris::from_bsp(path)
            .map(|inner| Self { inner })
            .map_err(|e| PyIOError::new_err(format!("Failed to load BSP: {}", e)))
    }

    /// Load a runtime ephemeris from raw BSP bytes in memory.
    ///
    /// Args:
    ///     data: BSP file contents as bytes.
    ///
    /// Raises:
    ///     ValueError: If the data cannot be parsed as a valid BSP file.
    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        RuntimeEphemeris::from_bytes(data)
            .map(|inner| Self { inner })
            .map_err(|e| PyValueError::new_err(format!("Failed to parse BSP: {}", e)))
    }

    /// Sun barycentric position at a given Julian Date.
    ///
    /// Returns Position in EclipticMeanJ2000 / Barycentric / AU.
    fn sun_barycentric(&self, jd: f64) -> PyPosition {
        let p = self.inner.sun_barycentric(JulianDate::new(jd));
        PyPosition::new_internal(
            p.x().value(),
            p.y().value(),
            p.z().value(),
            FRAME_ECL,
            CENTER_BARY,
            UNIT_AU,
        )
    }

    /// Earth barycentric position at a given Julian Date.
    ///
    /// Returns Position in EclipticMeanJ2000 / Barycentric / AU.
    fn earth_barycentric(&self, jd: f64) -> PyPosition {
        let p = self.inner.earth_barycentric(JulianDate::new(jd));
        PyPosition::new_internal(
            p.x().value(),
            p.y().value(),
            p.z().value(),
            FRAME_ECL,
            CENTER_BARY,
            UNIT_AU,
        )
    }

    /// Earth heliocentric position at a given Julian Date.
    ///
    /// Returns Position in EclipticMeanJ2000 / Heliocentric / AU.
    fn earth_heliocentric(&self, jd: f64) -> PyPosition {
        let p = self.inner.earth_heliocentric(JulianDate::new(jd));
        PyPosition::new_internal(
            p.x().value(),
            p.y().value(),
            p.z().value(),
            FRAME_ECL,
            CENTER_HELIO,
            UNIT_AU,
        )
    }

    /// Earth barycentric velocity at a given Julian Date.
    ///
    /// Returns (vx, vy, vz) in AU/day in EclipticMeanJ2000.
    fn earth_barycentric_velocity(&self, jd: f64) -> (f64, f64, f64) {
        let v = self.inner.earth_barycentric_velocity(JulianDate::new(jd));
        (v.x().value(), v.y().value(), v.z().value())
    }

    /// Moon geocentric position at a given Julian Date.
    ///
    /// Returns Position in EclipticMeanJ2000 / Geocentric / km.
    fn moon_geocentric(&self, jd: f64) -> PyPosition {
        let p = self.inner.moon_geocentric(JulianDate::new(jd));
        PyPosition::new_internal(
            p.x().value(),
            p.y().value(),
            p.z().value(),
            FRAME_ECL,
            CENTER_GEO,
            UNIT_KM,
        )
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.inner)
    }
}
