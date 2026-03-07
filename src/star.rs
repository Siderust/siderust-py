//! Star type for Python.
//!
//! Wraps `siderust::bodies::Star<'static>` with catalog lookup and
//! custom construction.

use pyo3::prelude::*;
use qtty::*;
use siderust::bodies::catalog;
use siderust::bodies::Star;
use siderust::coordinates::spherical::direction;
use siderust::AltitudePeriodsProvider;
use siderust::AzimuthProvider;
use tempoch::ModifiedJulianDate;

use crate::errors::unknown_star_error;
use crate::observer::PyObserver;

/// A star with physical properties and sky coordinates.
///
/// Create from the built-in catalog or with custom RA/Dec:
///
/// >>> vega = Star.catalog("Vega")
/// >>> custom = Star.from_ra_dec("HD 12345", 123.456, 45.678)
#[pyclass(name = "Star", module = "siderust", from_py_object)]
#[derive(Clone)]
pub struct PyStar {
    pub(crate) inner: Star<'static>,
}

impl PyStar {
    pub fn from_inner(inner: Star<'static>) -> Self {
        Self { inner }
    }
}

/// Look up a star by name from the built-in catalog.
fn lookup_star(name: &str) -> Option<Star<'static>> {
    let lower = name.to_lowercase();
    match lower.as_str() {
        "sirius" => Some(catalog::SIRIUS.clone()),
        "vega" => Some(catalog::VEGA.clone()),
        "polaris" => Some(catalog::POLARIS.clone()),
        "canopus" => Some(catalog::CANOPUS.clone()),
        "arcturus" => Some(catalog::ARCTURUS.clone()),
        "rigel" => Some(catalog::RIGEL.clone()),
        "betelgeuse" => Some(catalog::BETELGEUSE.clone()),
        "procyon" => Some(catalog::PROCYON.clone()),
        "aldebaran" => Some(catalog::ALDEBARAN.clone()),
        "altair" => Some(catalog::ALTAIR.clone()),
        _ => None,
    }
}

#[pymethods]
impl PyStar {
    /// Look up a star from the built-in catalog.
    ///
    /// Available stars: Sirius, Vega, Polaris, Canopus, Arcturus,
    /// Rigel, Betelgeuse, Procyon, Aldebaran, Altair.
    ///
    /// Args:
    ///     name: Star name (case-insensitive).
    ///
    /// Raises:
    ///     ValueError: If the star name is not in the catalog.
    #[staticmethod]
    fn catalog(name: &str) -> PyResult<Self> {
        lookup_star(name)
            .map(Self::from_inner)
            .ok_or_else(|| unknown_star_error(name))
    }

    /// Create a star with full physical parameters.
    ///
    /// Use this method when you have complete stellar metadata. For coordinate-only
    /// sky positions (e.g., for altitude/azimuth queries), prefer `Direction` instead.
    ///
    /// Args:
    ///     name: Display name for the star.
    ///     ra_deg: Right ascension in degrees.
    ///     dec_deg: Declination in degrees.
    ///     distance_ly: Distance in light-years.
    ///     mass_solar: Mass in solar masses.
    ///     radius_solar: Radius in solar radii.
    ///     luminosity_solar: Luminosity in solar luminosities.
    ///
    /// Example:
    ///     >>> star = Star.custom(
    ///     ...     name="Proxima Centauri",
    ///     ...     ra_deg=217.429,
    ///     ...     dec_deg=-62.679,
    ///     ...     distance_ly=4.24,
    ///     ...     mass_solar=0.12,
    ///     ...     radius_solar=0.15,
    ///     ...     luminosity_solar=0.0017
    ///     ... )
    #[staticmethod]
    #[pyo3(signature = (name, ra_deg, dec_deg, distance_ly, mass_solar, radius_solar, luminosity_solar))]
    fn custom(
        name: &str,
        ra_deg: f64,
        dec_deg: f64,
        distance_ly: f64,
        mass_solar: f64,
        radius_solar: f64,
        luminosity_solar: f64,
    ) -> Self {
        use siderust::coordinates::centers::Geocentric;
        use siderust::coordinates::frames::EquatorialMeanJ2000;
        use siderust::targets::CoordinateWithPM;
        use tempoch::JulianDate;

        let pos = affn::spherical::Position::<Geocentric, EquatorialMeanJ2000, LightYear>::new(
            Degrees::new(ra_deg),
            Degrees::new(dec_deg),
            LightYears::new(distance_ly),
        );

        let coord = CoordinateWithPM::new_static(pos, JulianDate::J2000);

        Self {
            inner: Star::new(
                name.to_string(),
                LightYears::new(distance_ly),
                SolarMasses::new(mass_solar),
                qtty::length::nominal::SolarRadiuses::new(radius_solar),
                SolarLuminosities::new(luminosity_solar),
                coord,
            ),
        }
    }

    // ── Properties ────────────────────────────────────────────────────

    /// Star name.
    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    /// Distance in light-years.
    #[getter]
    fn distance_ly(&self) -> f64 {
        self.inner.distance.value()
    }

    /// Mass in solar masses.
    #[getter]
    fn mass_solar(&self) -> f64 {
        self.inner.mass.value()
    }

    /// Radius in solar radii.
    #[getter]
    fn radius_solar(&self) -> f64 {
        self.inner.radius.value()
    }

    /// Luminosity in solar luminosities.
    #[getter]
    fn luminosity_solar(&self) -> f64 {
        self.inner.luminosity.value()
    }

    /// Right ascension in degrees.
    #[getter]
    fn ra_deg(&self) -> f64 {
        let icrs: direction::ICRS = (&self.inner).into();
        icrs.azimuth.to::<Degree>().value()
    }

    /// Declination in degrees.
    #[getter]
    fn dec_deg(&self) -> f64 {
        let icrs: direction::ICRS = (&self.inner).into();
        icrs.polar.to::<Degree>().value()
    }

    // ── Observation methods ───────────────────────────────────────────

    /// Altitude of this star in degrees at the given observer and MJD.
    fn altitude_at(&self, observer: &PyObserver, mjd: f64) -> f64 {
        self.inner
            .altitude_at(&observer.inner, ModifiedJulianDate::new(mjd))
            .to::<Degree>()
            .value()
    }

    /// Azimuth of this star in degrees at the given observer and MJD.
    fn azimuth_at(&self, observer: &PyObserver, mjd: f64) -> f64 {
        self.inner
            .azimuth_at(&observer.inner, ModifiedJulianDate::new(mjd))
            .to::<Degree>()
            .value()
    }

    // ── Dunder methods ────────────────────────────────────────────────

    /// Track this star at a given Julian Date.
    ///
    /// Returns a Target wrapping the star's ICRS direction.
    ///
    /// Args:
    ///     jd: Julian Date.
    ///
    /// Returns:
    ///     Target with the star's Direction at the given epoch.
    fn track(&self, jd: f64) -> crate::target::PyTarget {
        crate::target::track_star(self, jd)
    }

    fn __repr__(&self) -> String {
        format!("Star('{}')", self.inner.name)
    }

    fn __str__(&self) -> String {
        format!(
            "{} (RA={:.4}°, Dec={:.4}°, d={:.2} ly)",
            self.inner.name,
            self.ra_deg(),
            self.dec_deg(),
            self.inner.distance.value()
        )
    }

    fn __eq__(&self, other: &PyStar) -> bool {
        self.inner.name == other.inner.name
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.inner.name.hash(&mut hasher);
        hasher.finish()
    }

    /// Convert to a dictionary.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let d = pyo3::types::PyDict::new(py);
        d.set_item("name", self.name())?;
        d.set_item("ra_deg", self.ra_deg())?;
        d.set_item("dec_deg", self.dec_deg())?;
        d.set_item("distance_ly", self.distance_ly())?;
        d.set_item("mass_solar", self.mass_solar())?;
        d.set_item("radius_solar", self.radius_solar())?;
        d.set_item("luminosity_solar", self.luminosity_solar())?;
        Ok(d)
    }

    /// Create from a dictionary containing all physical parameters.
    ///
    /// Required keys: name, ra_deg, dec_deg, distance_ly, mass_solar,
    /// radius_solar, luminosity_solar.
    #[staticmethod]
    fn from_dict(d: &Bound<'_, pyo3::types::PyDict>) -> PyResult<Self> {
        let name: String = d
            .get_item("name")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing 'name'"))?
            .extract()?;
        let ra: f64 = d
            .get_item("ra_deg")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing 'ra_deg'"))?
            .extract()?;
        let dec: f64 = d
            .get_item("dec_deg")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing 'dec_deg'"))?
            .extract()?;
        let distance_ly: f64 = d
            .get_item("distance_ly")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing 'distance_ly'"))?
            .extract()?;
        let mass_solar: f64 = d
            .get_item("mass_solar")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing 'mass_solar'"))?
            .extract()?;
        let radius_solar: f64 = d
            .get_item("radius_solar")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing 'radius_solar'"))?
            .extract()?;
        let luminosity_solar: f64 = d
            .get_item("luminosity_solar")?
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("missing 'luminosity_solar'"))?
            .extract()?;
        Ok(Self::custom(
            &name,
            ra,
            dec,
            distance_ly,
            mass_solar,
            radius_solar,
            luminosity_solar,
        ))
    }
}
