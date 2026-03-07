//! Keplerian orbit and comet types for Python.
//!
//! Wraps `siderust::astro::orbit::Orbit` and `siderust::bodies::comet::Comet`
//! for orbital element representation and Kepler-based position propagation.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use qtty::*;
use siderust::astro::orbit::Orbit;
use siderust::bodies::comet;
use tempoch::JulianDate;

use crate::position::{PyPosition, CENTER_HELIO, FRAME_ECL, UNIT_AU};

// =============================================================================
// PyOrbit
// =============================================================================

/// Keplerian orbital elements.
///
/// Describes an elliptical (or hyperbolic) orbit around the Sun using the
/// six classical orbital elements plus the epoch.
///
/// Example:
///     >>> orbit = Orbit(
///     ...     semi_major_axis_au=1.0, eccentricity=0.0167,
///     ...     inclination_deg=0.00005, lon_ascending_node_deg=-11.26,
///     ...     arg_perihelion_deg=102.95, mean_anomaly_deg=100.46,
///     ...     epoch_jd=2451545.0)
///     >>> pos = orbit.kepler_position(2451545.0)
#[pyclass(name = "Orbit", module = "siderust", skip_from_py_object)]
#[derive(Clone)]
pub struct PyOrbit {
    pub(crate) inner: Orbit,
}

#[pymethods]
impl PyOrbit {
    /// Create a new set of Keplerian orbital elements.
    ///
    /// Args:
    ///     semi_major_axis_au: Semi-major axis in AU.
    ///     eccentricity: Orbital eccentricity (0 = circular, <1 = elliptical).
    ///     inclination_deg: Inclination in degrees.
    ///     lon_ascending_node_deg: Longitude of ascending node (Ω) in degrees.
    ///     arg_perihelion_deg: Argument of perihelion (ω) in degrees.
    ///     mean_anomaly_deg: Mean anomaly at epoch (M₀) in degrees.
    ///     epoch_jd: Epoch as Julian Date.
    #[new]
    #[pyo3(signature = (semi_major_axis_au, eccentricity, inclination_deg,
                        lon_ascending_node_deg, arg_perihelion_deg,
                        mean_anomaly_deg, epoch_jd))]
    fn new(
        semi_major_axis_au: f64,
        eccentricity: f64,
        inclination_deg: f64,
        lon_ascending_node_deg: f64,
        arg_perihelion_deg: f64,
        mean_anomaly_deg: f64,
        epoch_jd: f64,
    ) -> Self {
        Self {
            inner: Orbit::new(
                AstronomicalUnits::new(semi_major_axis_au),
                eccentricity,
                Degrees::new(inclination_deg),
                Degrees::new(lon_ascending_node_deg),
                Degrees::new(arg_perihelion_deg),
                Degrees::new(mean_anomaly_deg),
                JulianDate::new(epoch_jd),
            ),
        }
    }

    // ── Element access ──────────────────────────────────────────────

    /// Semi-major axis in AU.
    #[getter]
    fn semi_major_axis_au(&self) -> f64 {
        self.inner.semi_major_axis.value()
    }

    /// Orbital eccentricity.
    #[getter]
    fn eccentricity(&self) -> f64 {
        self.inner.eccentricity
    }

    /// Inclination in degrees.
    #[getter]
    fn inclination_deg(&self) -> f64 {
        self.inner.inclination.value()
    }

    /// Longitude of ascending node (Ω) in degrees.
    #[getter]
    fn lon_ascending_node_deg(&self) -> f64 {
        self.inner.longitude_of_ascending_node.value()
    }

    /// Argument of perihelion (ω) in degrees.
    #[getter]
    fn arg_perihelion_deg(&self) -> f64 {
        self.inner.argument_of_perihelion.value()
    }

    /// Mean anomaly at epoch (M₀) in degrees.
    #[getter]
    fn mean_anomaly_deg(&self) -> f64 {
        self.inner.mean_anomaly_at_epoch.value()
    }

    /// Epoch as Julian Date.
    #[getter]
    fn epoch_jd(&self) -> f64 {
        self.inner.epoch.value()
    }

    // ── Propagation ─────────────────────────────────────────────────

    /// Compute heliocentric ecliptic position at a given Julian Date.
    ///
    /// Uses Kepler's equation to propagate the orbit to the requested epoch.
    ///
    /// Args:
    ///     jd: Julian Date for evaluation.
    ///
    /// Returns:
    ///     Position in EclipticMeanJ2000 / Heliocentric / AU.
    fn kepler_position(&self, jd: f64) -> PyPosition {
        let pos = self.inner.kepler_position(JulianDate::new(jd));
        PyPosition::new_internal(
            pos.x().value(),
            pos.y().value(),
            pos.z().value(),
            FRAME_ECL,
            CENTER_HELIO,
            UNIT_AU,
        )
    }

    /// Approximate orbital period in Julian years (Kepler's third law).
    fn period_years(&self) -> f64 {
        self.inner.semi_major_axis.value().powf(1.5)
    }

    // ── Serialization ───────────────────────────────────────────────

    /// Convert to a dictionary.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        d.set_item("semi_major_axis_au", self.semi_major_axis_au())?;
        d.set_item("eccentricity", self.eccentricity())?;
        d.set_item("inclination_deg", self.inclination_deg())?;
        d.set_item("lon_ascending_node_deg", self.lon_ascending_node_deg())?;
        d.set_item("arg_perihelion_deg", self.arg_perihelion_deg())?;
        d.set_item("mean_anomaly_deg", self.mean_anomaly_deg())?;
        d.set_item("epoch_jd", self.epoch_jd())?;
        Ok(d)
    }

    /// Create from a dictionary.
    #[staticmethod]
    fn from_dict(d: &Bound<'_, PyDict>) -> PyResult<Self> {
        let get = |k: &str| -> PyResult<f64> {
            d.get_item(k)?
                .ok_or_else(|| PyValueError::new_err(format!("missing key '{}'", k)))?
                .extract()
        };
        Ok(Self::new(
            get("semi_major_axis_au")?,
            get("eccentricity")?,
            get("inclination_deg")?,
            get("lon_ascending_node_deg")?,
            get("arg_perihelion_deg")?,
            get("mean_anomaly_deg")?,
            get("epoch_jd")?,
        ))
    }

    fn __repr__(&self) -> String {
        format!(
            "Orbit(a={:.6} AU, e={:.6}, i={:.4}°, Ω={:.4}°, ω={:.4}°, M₀={:.4}°, epoch=JD {:.1})",
            self.inner.semi_major_axis.value(),
            self.inner.eccentricity,
            self.inner.inclination.value(),
            self.inner.longitude_of_ascending_node.value(),
            self.inner.argument_of_perihelion.value(),
            self.inner.mean_anomaly_at_epoch.value(),
            self.inner.epoch.value(),
        )
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, (f64, f64, f64, f64, f64, f64, f64))> {
        let cls = py.get_type::<Self>().into_any().unbind();
        Ok((
            cls,
            (
                self.semi_major_axis_au(),
                self.eccentricity(),
                self.inclination_deg(),
                self.lon_ascending_node_deg(),
                self.arg_perihelion_deg(),
                self.mean_anomaly_deg(),
                self.epoch_jd(),
            ),
        ))
    }
}

// =============================================================================
// PyComet
// =============================================================================

/// A comet with orbital elements and metadata.
///
/// Access preset comets with class methods or create custom ones.
///
/// Example:
///     >>> halley = Comet.halley()
///     >>> pos = halley.orbit.kepler_position(2451545.0)
#[pyclass(name = "Comet", module = "siderust", skip_from_py_object)]
#[derive(Clone)]
pub struct PyComet {
    pub(crate) name: String,
    pub(crate) tail_length_km: f64,
    pub(crate) orbit: Orbit,
    pub(crate) reference: String,
}

impl PyComet {
    fn from_rust(c: &comet::Comet<'_>) -> Self {
        Self {
            name: c.name.to_string(),
            tail_length_km: c.tail_length.value(),
            orbit: c.orbit,
            reference: match c.reference {
                comet::OrbitFrame::Heliocentric => "Heliocentric".to_string(),
                comet::OrbitFrame::Barycentric => "Barycentric".to_string(),
            },
        }
    }
}

#[pymethods]
impl PyComet {
    /// Create a custom comet.
    ///
    /// Args:
    ///     name: Comet designation or name.
    ///     orbit: Keplerian orbital elements.
    ///     tail_length_km: Approximate tail length in km (default: 0).
    ///     reference: Orbit reference frame, "Heliocentric" or "Barycentric" (default: "Heliocentric").
    #[new]
    #[pyo3(signature = (name, orbit, tail_length_km = 0.0, reference = "Heliocentric"))]
    fn new(name: &str, orbit: &PyOrbit, tail_length_km: f64, reference: &str) -> PyResult<Self> {
        match reference {
            "Heliocentric" | "Barycentric" => {}
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Unknown reference '{}'. Valid: Heliocentric, Barycentric",
                    reference
                )));
            }
        }
        Ok(Self {
            name: name.to_string(),
            tail_length_km,
            orbit: orbit.inner,
            reference: reference.to_string(),
        })
    }

    // ── Preset comets ───────────────────────────────────────────────

    /// 1P/Halley — archetype periodic comet (heliocentric elements).
    #[staticmethod]
    fn halley() -> Self {
        Self::from_rust(&comet::HALLEY)
    }

    /// 2P/Encke — shortest-period named comet (heliocentric).
    #[staticmethod]
    fn encke() -> Self {
        Self::from_rust(&comet::ENCKE)
    }

    /// C/1995 O1 (Hale-Bopp) — great comet of 1997 (barycentric elements).
    #[staticmethod]
    fn hale_bopp() -> Self {
        Self::from_rust(&comet::HALE_BOPP)
    }

    // ── Properties ──────────────────────────────────────────────────

    /// Comet name/designation.
    #[getter]
    fn name(&self) -> &str {
        &self.name
    }

    /// Approximate tail length in km.
    #[getter]
    fn tail_length_km(&self) -> f64 {
        self.tail_length_km
    }

    /// Keplerian orbital elements.
    #[getter]
    fn orbit(&self) -> PyOrbit {
        PyOrbit {
            inner: self.orbit,
        }
    }

    /// Orbit reference frame ("Heliocentric" or "Barycentric").
    #[getter]
    fn reference(&self) -> &str {
        &self.reference
    }

    /// Approximate orbital period in Julian years.
    fn period_years(&self) -> f64 {
        self.orbit.semi_major_axis.value().powf(1.5)
    }

    /// Heliocentric ecliptic position at a given Julian Date.
    ///
    /// Uses Kepler's equation to propagate the comet's orbit.
    fn kepler_position(&self, jd: f64) -> PyPosition {
        let pos = self.orbit.kepler_position(JulianDate::new(jd));
        PyPosition::new_internal(
            pos.x().value(),
            pos.y().value(),
            pos.z().value(),
            FRAME_ECL,
            CENTER_HELIO,
            UNIT_AU,
        )
    }

    fn __repr__(&self) -> String {
        format!(
            "Comet('{}', period={:.1} yr, ref={})",
            self.name,
            self.period_years(),
            self.reference
        )
    }

    fn __str__(&self) -> String {
        format!(
            "{} (a={:.3} AU, e={:.6}, P={:.1} yr)",
            self.name,
            self.orbit.semi_major_axis.value(),
            self.orbit.eccentricity,
            self.period_years(),
        )
    }
}
