//! ICRS direction (RA/Dec) for Python.
//!
//! Wraps `spherical::direction::ICRS` as a lightweight Python class for
//! fixed-coordinate altitude/azimuth queries.

use pyo3::prelude::*;
use qtty::*;
use siderust::coordinates::spherical::direction;
use siderust::AltitudePeriodsProvider;
use siderust::AzimuthProvider;
use tempoch::ModifiedJulianDate;

use crate::observer::PyObserver;

/// A fixed ICRS sky direction (right ascension, declination).
///
/// Use for altitude/azimuth queries of fixed sky positions:
///
/// >>> d = Direction(ra_deg=83.633, dec_deg=22.014)  # Betelgeuse-ish
/// >>> d.altitude_at(observer, 60000.0)
#[pyclass(name = "Direction", module = "siderust", from_py_object)]
#[derive(Clone, Copy)]
pub struct PyDirection {
    pub(crate) inner: direction::ICRS,
}

impl PyDirection {
    pub fn from_inner(inner: direction::ICRS) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyDirection {
    /// Create an ICRS direction from right ascension and declination.
    ///
    /// Args:
    ///     ra_deg: Right ascension in degrees.
    ///     dec_deg: Declination in degrees.
    #[new]
    fn new(ra_deg: f64, dec_deg: f64) -> Self {
        Self {
            inner: direction::ICRS::new(Degrees::new(ra_deg), Degrees::new(dec_deg)),
        }
    }

    /// Right ascension in degrees.
    #[getter]
    fn ra_deg(&self) -> f64 {
        self.inner.azimuth.to::<Degree>().value()
    }

    /// Declination in degrees.
    #[getter]
    fn dec_deg(&self) -> f64 {
        self.inner.polar.to::<Degree>().value()
    }

    /// Altitude of this direction in degrees at the given observer and MJD.
    fn altitude_at(&self, observer: &PyObserver, mjd: f64) -> f64 {
        self.inner
            .altitude_at(&observer.inner, ModifiedJulianDate::new(mjd))
            .to::<Degree>()
            .value()
    }

    /// Azimuth of this direction in degrees at the given observer and MJD.
    fn azimuth_at(&self, observer: &PyObserver, mjd: f64) -> f64 {
        self.inner
            .azimuth_at(&observer.inner, ModifiedJulianDate::new(mjd))
            .to::<Degree>()
            .value()
    }

    fn __repr__(&self) -> String {
        format!(
            "Direction(ra={:.4}°, dec={:.4}°)",
            self.ra_deg(),
            self.dec_deg()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn __eq__(&self, other: &PyDirection) -> bool {
        (self.ra_deg() - other.ra_deg()).abs() < 1e-10
            && (self.dec_deg() - other.dec_deg()).abs() < 1e-10
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.inner.azimuth.value().to_bits().hash(&mut hasher);
        self.inner.polar.value().to_bits().hash(&mut hasher);
        hasher.finish()
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, (f64, f64))> {
        let cls = py.get_type::<Self>().into_any().unbind();
        Ok((cls, (self.ra_deg(), self.dec_deg())))
    }
}
