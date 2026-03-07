//! Observer / observatory type for Python.
//!
//! Wraps `Geodetic<ECEF>` as a Python class with named constructors for
//! major observatory sites.

use pyo3::prelude::*;
use qtty::{Degrees, Meters};
use siderust::coordinates::centers::Geodetic;
use siderust::coordinates::frames::ECEF;

/// An observer location on the Earth's surface (WGS84 geodetic).
///
/// Create from longitude/latitude/height or use a predefined observatory.
///
/// Example:
/// >>> obs = Observer(-17.8925, 28.7543, 2396.0)       # custom
/// >>> obs = Observer.roque_de_los_muchachos()           # catalog
#[pyclass(name = "Observer", module = "siderust", from_py_object)]
#[derive(Clone)]
pub struct PyObserver {
    pub(crate) inner: Geodetic<ECEF>,
}

impl PyObserver {
    pub fn from_inner(inner: Geodetic<ECEF>) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl PyObserver {
    /// Create a custom observer at a geodetic position.
    ///
    /// Args:
    ///     lon_deg: Longitude in degrees (east positive).
    ///     lat_deg: Latitude in degrees (north positive).
    ///     height_m: Height above ellipsoid in metres (default 0).
    #[new]
    #[pyo3(signature = (lon_deg, lat_deg, height_m = 0.0))]
    fn new(lon_deg: f64, lat_deg: f64, height_m: f64) -> Self {
        Self {
            inner: Geodetic::<ECEF>::new(
                Degrees::new(lon_deg),
                Degrees::new(lat_deg),
                Meters::new(height_m),
            ),
        }
    }

    // ── Predefined observatories ──────────────────────────────────────

    /// Roque de los Muchachos Observatory (La Palma, Spain).
    #[staticmethod]
    fn roque_de_los_muchachos() -> Self {
        Self {
            inner: siderust::observatories::ROQUE_DE_LOS_MUCHACHOS,
        }
    }

    /// El Paranal Observatory (Chile).
    #[staticmethod]
    fn el_paranal() -> Self {
        Self {
            inner: siderust::observatories::EL_PARANAL,
        }
    }

    /// Mauna Kea Observatory (Hawaiʻi, USA).
    #[staticmethod]
    fn mauna_kea() -> Self {
        Self {
            inner: siderust::observatories::MAUNA_KEA,
        }
    }

    /// La Silla Observatory (Chile).
    #[staticmethod]
    fn la_silla() -> Self {
        Self {
            inner: siderust::observatories::LA_SILLA_OBSERVATORY,
        }
    }

    // ── Properties ────────────────────────────────────────────────────

    /// Longitude in degrees.
    #[getter]
    fn lon_deg(&self) -> f64 {
        self.inner.lon.value()
    }

    /// Latitude in degrees.
    #[getter]
    fn lat_deg(&self) -> f64 {
        self.inner.lat.value()
    }

    /// Height above ellipsoid in metres.
    #[getter]
    fn height_m(&self) -> f64 {
        self.inner.height.value()
    }

    fn __repr__(&self) -> String {
        format!(
            "Observer(lon={:.4}°, lat={:.4}°, h={:.1}m)",
            self.inner.lon.value(),
            self.inner.lat.value(),
            self.inner.height.value()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn __eq__(&self, other: &PyObserver) -> bool {
        (self.inner.lon.value() - other.inner.lon.value()).abs() < 1e-10
            && (self.inner.lat.value() - other.inner.lat.value()).abs() < 1e-10
            && (self.inner.height.value() - other.inner.height.value()).abs() < 1e-6
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.inner.lon.value().to_bits().hash(&mut hasher);
        self.inner.lat.value().to_bits().hash(&mut hasher);
        self.inner.height.value().to_bits().hash(&mut hasher);
        hasher.finish()
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, (f64, f64, f64))> {
        let cls = py.get_type::<Self>().into_any().unbind();
        Ok((
            cls,
            (
                self.inner.lon.value(),
                self.inner.lat.value(),
                self.inner.height.value(),
            ),
        ))
    }
}
