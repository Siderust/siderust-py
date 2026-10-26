//! Explicit ICRS and Horizontal spherical directions for Python.
//!
//! Wraps `spherical::direction::ICRS` as a lightweight Python class for
//! fixed-coordinate altitude/azimuth queries.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use siderust::coordinates::spherical::direction;
use siderust::qtty::*;
use siderust::time::ModifiedJulianDate;
use siderust::AltitudeProvider;
use siderust::AzimuthProvider;

use crate::observer::PyObserver;

/// A fixed ICRS sky direction (right ascension, declination).
///
/// Use for altitude/azimuth queries of fixed sky positions:
///
/// >>> d = direction.ICRS(ra_deg=83.633, dec_deg=22.014)  # Betelgeuse-ish
/// >>> d.altitude_at(observer, 60000.0)
#[pyclass(name = "ICRS", module = "siderust.direction", frozen, from_py_object)]
#[derive(Clone, Copy)]
pub struct PyIcrs {
    pub(crate) inner: direction::ICRS,
}

impl PyIcrs {
    pub fn from_inner(inner: direction::ICRS) -> Self {
        Self { inner }
    }
}

#[pyfunction]
pub(crate) fn _bridge_icrs_to_parts(value: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    let direction = value.cast::<PyIcrs>()?.borrow();
    Ok((direction.ra_deg(), direction.dec_deg()))
}

#[pyfunction]
pub(crate) fn _bridge_icrs_from_parts(
    right_ascension_degrees: f64,
    declination_degrees: f64,
) -> PyResult<PyIcrs> {
    PyIcrs::new(right_ascension_degrees, declination_degrees)
}

#[allow(clippy::wrong_self_convention)]
#[pymethods]
impl PyIcrs {
    /// Create an ICRS direction from right ascension and declination.
    ///
    /// Args:
    ///     ra_deg: Right ascension in degrees.
    ///     dec_deg: Declination in degrees.
    #[new]
    pub(crate) fn new(ra_deg: f64, dec_deg: f64) -> PyResult<Self> {
        for (name, value) in [("ra_deg", ra_deg), ("dec_deg", dec_deg)] {
            if !value.is_finite() {
                return Err(PyValueError::new_err(format!("{name} must be finite")));
            }
        }

        Ok(Self {
            inner: direction::ICRS::new(Degrees::new(ra_deg), Degrees::new(dec_deg)),
        })
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

    /// Angular separation to another direction in degrees (Vincenty formula).
    fn angular_separation(&self, other: &PyIcrs) -> f64 {
        self.inner.angular_separation(&other.inner).value()
    }

    /// Convert to a unit Cartesian vector in ICRS.
    fn to_cartesian(&self) -> (f64, f64, f64) {
        let cart = self.inner.to_cartesian();
        (cart.x(), cart.y(), cart.z())
    }

    /// Dot product with another ICRS direction.
    fn dot(&self, other: &PyIcrs) -> f64 {
        self.inner.to_cartesian().dot(&other.inner.to_cartesian())
    }

    /// Track this direction at a given Julian Date.
    ///
    /// Fixed directions are time-invariant, so this simply wraps the
    /// direction in a Target with the given epoch.
    ///
    /// Args:
    ///     jd: Julian Date.
    ///
    /// Returns:
    ///     Target with this direction.ICRS at the given epoch.
    fn track(&self, jd: f64) -> crate::target::PyTarget {
        crate::target::track_direction(self, jd)
    }

    /// Convert to a dictionary.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let d = pyo3::types::PyDict::new(py);
        d.set_item("ra_deg", self.ra_deg())?;
        d.set_item("dec_deg", self.dec_deg())?;
        Ok(d)
    }

    /// Create from a dictionary.
    #[staticmethod]
    fn from_dict(d: &Bound<'_, pyo3::types::PyDict>) -> PyResult<Self> {
        let ra: f64 = d
            .get_item("ra_deg")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing 'ra_deg'"))?
            .extract()?;
        let dec: f64 = d
            .get_item("dec_deg")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing 'dec_deg'"))?
            .extract()?;
        Self::new(ra, dec)
    }

    fn __repr__(&self) -> String {
        format!(
            "siderust.direction.ICRS(ra_deg={:.4}, dec_deg={:.4})",
            self.ra_deg(),
            self.dec_deg()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn __eq__(&self, other: &PyIcrs) -> bool {
        self.inner.azimuth.value() == other.inner.azimuth.value()
            && self.inner.polar.value() == other.inner.polar.value()
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        hashable_float(self.inner.azimuth.value()).hash(&mut hasher);
        hashable_float(self.inner.polar.value()).hash(&mut hasher);
        hasher.finish()
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, (f64, f64))> {
        let cls = py.get_type::<Self>().into_any().unbind();
        Ok((cls, (self.ra_deg(), self.dec_deg())))
    }
}

/// A local horizontal sky direction (altitude, azimuth).
///
/// Altitude is measured above the geometric horizon; azimuth follows the
/// North-clockwise convention (0° = North, increasing through East). Values
/// are geometric (no atmospheric refraction).
///
/// >>> h = direction.Horizontal(alt_deg=45.0, az_deg=180.0)  # due South at 45°
#[pyclass(
    name = "Horizontal",
    module = "siderust.direction",
    frozen,
    from_py_object
)]
#[derive(Clone, Copy)]
pub struct PyHorizontal {
    pub(crate) inner: direction::Horizontal,
}

impl PyHorizontal {
    pub fn from_inner(inner: direction::Horizontal) -> Self {
        Self { inner }
    }
}

#[pyfunction]
pub(crate) fn _bridge_horizontal_to_parts(value: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    let direction = value.cast::<PyHorizontal>()?.borrow();
    Ok((direction.alt_deg(), direction.az_deg()))
}

#[pyfunction]
pub(crate) fn _bridge_horizontal_from_parts(alt_deg: f64, az_deg: f64) -> PyResult<PyHorizontal> {
    PyHorizontal::new(alt_deg, az_deg)
}

#[allow(clippy::wrong_self_convention)]
#[pymethods]
impl PyHorizontal {
    /// Create a horizontal direction from altitude and azimuth.
    ///
    /// Args:
    ///     alt_deg: Altitude above the geometric horizon in degrees
    ///         (preferably in ``[-90, 90]``; out-of-range values are
    ///         canonicalized by Siderust).
    ///     az_deg: Azimuth in degrees, North-clockwise through East.
    ///
    /// Both angles must be finite. No atmospheric refraction is applied.
    #[new]
    fn new(alt_deg: f64, az_deg: f64) -> PyResult<Self> {
        for (name, value) in [("alt_deg", alt_deg), ("az_deg", az_deg)] {
            if !value.is_finite() {
                return Err(PyValueError::new_err(format!("{name} must be finite")));
            }
        }

        Ok(Self {
            inner: direction::Horizontal::new(Degrees::new(alt_deg), Degrees::new(az_deg)),
        })
    }

    /// Altitude above the geometric horizon in degrees.
    #[getter]
    fn alt_deg(&self) -> f64 {
        self.inner.alt().to::<Degree>().value()
    }

    /// Azimuth in degrees (North-clockwise through East).
    #[getter]
    fn az_deg(&self) -> f64 {
        self.inner.az().to::<Degree>().value()
    }

    /// Convert to a dictionary.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let d = pyo3::types::PyDict::new(py);
        d.set_item("alt_deg", self.alt_deg())?;
        d.set_item("az_deg", self.az_deg())?;
        Ok(d)
    }

    /// Create from a dictionary.
    #[staticmethod]
    fn from_dict(d: &Bound<'_, pyo3::types::PyDict>) -> PyResult<Self> {
        let alt: f64 = d
            .get_item("alt_deg")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing 'alt_deg'"))?
            .extract()?;
        let az: f64 = d
            .get_item("az_deg")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing 'az_deg'"))?
            .extract()?;
        Self::new(alt, az)
    }

    fn __repr__(&self) -> String {
        format!(
            "siderust.direction.Horizontal(alt_deg={:.4}, az_deg={:.4})",
            self.alt_deg(),
            self.az_deg()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn __eq__(&self, other: &PyHorizontal) -> bool {
        self.inner.polar.value() == other.inner.polar.value()
            && self.inner.azimuth.value() == other.inner.azimuth.value()
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        hashable_float(self.inner.polar.value()).hash(&mut hasher);
        hashable_float(self.inner.azimuth.value()).hash(&mut hasher);
        hasher.finish()
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, (f64, f64))> {
        let cls = py.get_type::<Self>().into_any().unbind();
        Ok((cls, (self.alt_deg(), self.az_deg())))
    }
}

// Python considers signed zeros equal, so their hashes must also agree.
fn hashable_float(value: f64) -> u64 {
    if value == 0.0 {
        0
    } else {
        value.to_bits()
    }
}
