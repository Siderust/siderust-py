//! Lunar surface observer and geometric Alt/Az bindings.
//!
//! Wraps `SurfaceSite<Moon>` and the body-surface horizontal pipeline from
//! Siderust 0.16. Results are geometric only (no refraction, aberration,
//! light-time, or terrain). Orientation is NAIF `IAU_MOON` analytical
//! Mean-Earth (`MoonIauMeanEarth`); the legacy `MoonPrincipalAxes` name is
//! the same marker, **not** JPL DE `MOON_PA`.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use siderust::bodies::solar_system::Moon;
use siderust::coordinates::cartesian::{Direction, Position as CPosition};
use siderust::coordinates::centers::Selenocentric;
use siderust::coordinates::frames::{
    EclipticMeanJ2000, EquatorialMeanJ2000, EquatorialMeanOfDate, EquatorialTrueOfDate,
    MoonIauMeanEarth, ICRF, ICRS,
};
use siderust::coordinates::spherical::direction as sph_dir;
use siderust::coordinates::surface::{
    BodySurfaceModel, SurfaceError, SurfaceSite, MOON_IAU_SPHERE,
};
use siderust::coordinates::transform::context::{DefaultEop, DefaultEphemeris};
use siderust::coordinates::transform::{AstroContext, SurfaceHorizontalExt, TransformCenter};
use siderust::ephemeris::{Ephemeris, Vsop87Ephemeris};
use siderust::qtty::*;
use siderust::time::JulianDate;

type DefaultCtx = AstroContext<DefaultEphemeris, DefaultEop>;

fn default_ctx() -> DefaultCtx {
    AstroContext::default()
}

use crate::coordinates::{PyHorizontal, PyIcrs};
use crate::position::{
    canonicalize_frame, convert_center, PyPosition, CENTER_BARY, CENTER_GEO, CENTER_HELIO,
    CENTER_SELENO, FRAME_ECL, FRAME_EMOD, FRAME_EQ, FRAME_ETOD, FRAME_ICRF, FRAME_ICRS,
    FRAME_MOON_IAU, UNIT_AU, UNIT_KM,
};

const AU_KM: f64 = 149_597_870.700;

fn surface_err(err: SurfaceError) -> PyErr {
    PyValueError::new_err(err.to_string())
}

fn require_finite_jd(jd_tt: f64) -> PyResult<JulianDate> {
    if !jd_tt.is_finite() {
        return Err(PyValueError::new_err("jd_tt must be finite"));
    }
    Ok(JulianDate::new(jd_tt))
}

fn normalize_icrs_direction(x: f64, y: f64, z: f64) -> PyResult<Direction<ICRS>> {
    for (name, value) in [("x", x), ("y", y), ("z", z)] {
        if !value.is_finite() {
            return Err(PyValueError::new_err(format!("{name} must be finite")));
        }
    }
    let norm = (x * x + y * y + z * z).sqrt();
    if !norm.is_finite() || norm == 0.0 {
        return Err(PyValueError::new_err(
            "ICRS direction must be a non-zero finite vector",
        ));
    }
    Ok(Direction::<ICRS>::new_unchecked(
        x / norm,
        y / norm,
        z / norm,
    ))
}

fn to_horizontal(alt: Degrees, az: Degrees) -> PyHorizontal {
    PyHorizontal::from_inner(sph_dir::Horizontal::new(alt, az))
}

fn au_to_km(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    (x * AU_KM, y * AU_KM, z * AU_KM)
}

/// Geometric lunar-surface observation (altitude, azimuth, optional range).
///
/// Direction-only observations omit range. Finite-distance observations
/// report slant range in the same unit as the input position (`au` or `km`).
#[pyclass(name = "LunarObservation", module = "siderust", frozen, from_py_object)]
#[derive(Clone)]
pub struct PyLunarObservation {
    alt_deg: f64,
    az_deg: f64,
    range: Option<f64>,
    range_unit: Option<String>,
}

impl PyLunarObservation {
    fn with_range(alt: Degrees, az: Degrees, range: f64, unit: &str) -> Self {
        Self {
            alt_deg: alt.value(),
            az_deg: az.value(),
            range: Some(range),
            range_unit: Some(unit.to_string()),
        }
    }
}

#[pymethods]
impl PyLunarObservation {
    /// Geometric altitude above the local horizon in degrees.
    #[getter]
    fn alt_deg(&self) -> f64 {
        self.alt_deg
    }

    /// Azimuth in degrees (north = 0°, east = 90°, north through east).
    #[getter]
    fn az_deg(&self) -> f64 {
        self.az_deg
    }

    /// Slant range when the target had a known distance; otherwise `None`.
    #[getter]
    fn range(&self) -> Option<f64> {
        self.range
    }

    /// Unit of [`Self::range`] (`"au"` or `"km"`), or `None` for direction-only.
    #[getter]
    fn range_unit(&self) -> Option<&str> {
        self.range_unit.as_deref()
    }

    /// Local horizontal sky direction (`direction.Horizontal`).
    fn direction(&self) -> PyHorizontal {
        to_horizontal(Degrees::new(self.alt_deg), Degrees::new(self.az_deg))
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let d = pyo3::types::PyDict::new(py);
        d.set_item("alt_deg", self.alt_deg)?;
        d.set_item("az_deg", self.az_deg)?;
        d.set_item("range", self.range)?;
        d.set_item("range_unit", self.range_unit.as_deref())?;
        Ok(d)
    }

    fn __repr__(&self) -> String {
        match (&self.range, &self.range_unit) {
            (Some(r), Some(u)) => format!(
                "LunarObservation(alt_deg={:.6}, az_deg={:.6}, range={:.6} {})",
                self.alt_deg, self.az_deg, r, u
            ),
            _ => format!(
                "LunarObservation(alt_deg={:.6}, az_deg={:.6})",
                self.alt_deg, self.az_deg
            ),
        }
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Observer site on the Moon's IAU mean sphere.
///
/// Longitude is east-positive; latitude is planetocentric; height is metres
/// above the IAU sphere of radius 1737.4 km. Distinct from the Earth
/// [`super::observer::PyObserver`] (WGS84 / ECEF).
///
/// Epochs for Alt/Az are **Julian Date TT**. Lunar orientation converts TT→TDB
/// through tempoch. Results are geometric only.
///
/// Example:
/// >>> site = LunarObserver(20.0, -3.5)
/// >>> from siderust.direction import ICRS
/// >>> hz = site.observe_direction(ICRS(0.0, 90.0), 2451545.0)
#[pyclass(name = "LunarObserver", module = "siderust", from_py_object)]
#[derive(Clone)]
pub struct PyLunarObserver {
    pub(crate) site: SurfaceSite<Moon>,
}

#[pymethods]
impl PyLunarObserver {
    /// Create a lunar surface site.
    ///
    /// Args:
    ///     lon_deg: Longitude in degrees, east-positive.
    ///     lat_deg: Planetocentric latitude in degrees, north-positive.
    ///     height_m: Height above the IAU mean sphere in metres (default 0).
    #[new]
    #[pyo3(signature = (lon_deg, lat_deg, height_m = 0.0))]
    fn new(lon_deg: f64, lat_deg: f64, height_m: f64) -> PyResult<Self> {
        for (name, value) in [
            ("lon_deg", lon_deg),
            ("lat_deg", lat_deg),
            ("height_m", height_m),
        ] {
            if !value.is_finite() {
                return Err(PyValueError::new_err(format!("{name} must be finite")));
            }
        }
        let site = SurfaceSite::<Moon>::new(
            Degrees::new(lon_deg),
            Degrees::new(lat_deg),
            Meters::new(height_m),
        )
        .map_err(surface_err)?;
        Ok(Self { site })
    }

    /// Longitude in degrees (east-positive, normalized to (−180°, 180]).
    #[getter]
    fn lon_deg(&self) -> f64 {
        self.site.lon().value()
    }

    /// Planetocentric latitude in degrees.
    #[getter]
    fn lat_deg(&self) -> f64 {
        self.site.lat().value()
    }

    /// Height above the IAU mean sphere in metres.
    #[getter]
    fn height_m(&self) -> f64 {
        self.site.height().value()
    }

    /// IAU mean-sphere equatorial radius in metres (1_737_400).
    #[getter]
    fn equatorial_radius_m(&self) -> f64 {
        MOON_IAU_SPHERE.equatorial_radius().value()
    }

    /// Latitude convention for this site (`"planetocentric"`).
    #[getter]
    fn latitude_convention(&self) -> &'static str {
        self.site.latitude_convention().as_str()
    }

    /// Body name (`"Moon"`).
    #[getter]
    fn body_name(&self) -> &'static str {
        Moon::body_name()
    }

    /// Geometric Alt/Az of a fixed ICRS sky direction at Julian Date TT.
    ///
    /// Direction-only: no distance is invented, so there is no parallax.
    ///
    /// Args:
    ///     direction: `siderust.direction.ICRS` right ascension / declination.
    ///     jd_tt: Observation epoch as Julian Date on the TT time scale.
    ///
    /// Returns:
    ///     `siderust.direction.Horizontal` (geometric altitude and azimuth).
    fn observe_direction(&self, direction: &PyIcrs, jd_tt: f64) -> PyResult<PyHorizontal> {
        let cart = direction.inner.to_cartesian();
        self.observe_icrs_vector(cart.x(), cart.y(), cart.z(), jd_tt)
    }

    /// Geometric Alt/Az of an ICRS Cartesian direction at Julian Date TT.
    ///
    /// The vector is normalized; a zero-length vector raises `ValueError`.
    /// No parallax is applied.
    #[pyo3(signature = (x, y, z, jd_tt))]
    fn observe_icrs_vector(&self, x: f64, y: f64, z: f64, jd_tt: f64) -> PyResult<PyHorizontal> {
        let jd = require_finite_jd(jd_tt)?;
        let dir = normalize_icrs_direction(x, y, z)?;
        let ctx = default_ctx();
        let obs = self
            .site
            .observe_direction(&dir, &jd, &ctx)
            .map_err(surface_err)?;
        let sph = obs.to_spherical();
        Ok(to_horizontal(sph.alt(), sph.az()))
    }

    /// Finite-distance geometric Alt/Az of a cartesian `Position`.
    ///
    /// Supports centers `Selenocentric`, `Geocentric`, `Barycentric`, and
    /// `Heliocentric`, and frames including `ICRS`, `EclipticMeanJ2000`,
    /// `MoonIauMeanEarth` (alias `MoonPrincipalAxes`), and other inertial
    /// frames already exposed by `Position`. Range is preserved in the
    /// position's unit (`au` or `km`).
    ///
    /// Args:
    ///     position: Target position with known range.
    ///     jd_tt: Observation epoch as Julian Date TT.
    fn observe_position(&self, position: &PyPosition, jd_tt: f64) -> PyResult<PyLunarObservation> {
        let jd = require_finite_jd(jd_tt)?;
        let frame = canonicalize_frame(&position.frame)?;
        let center = position.center.as_str();
        match center {
            CENTER_SELENO | CENTER_GEO | CENTER_BARY | CENTER_HELIO => {}
            other => {
                return Err(PyValueError::new_err(format!(
                    "Unsupported center '{}' for lunar observation. Valid: \
                     Selenocentric, Geocentric, Barycentric, Heliocentric",
                    other
                )));
            }
        }
        let unit = position.unit.as_str();
        if unit != UNIT_AU && unit != UNIT_KM {
            return Err(PyValueError::new_err(format!(
                "Unknown unit '{}'. Valid: au, km",
                unit
            )));
        }

        let (ax, ay, az) = if unit == UNIT_KM {
            (position.x / AU_KM, position.y / AU_KM, position.z / AU_KM)
        } else {
            (position.x, position.y, position.z)
        };

        let (cx, cy, cz) = convert_center(ax, ay, az, &frame, center, CENTER_SELENO, jd)?;
        observe_selenocentric(&self.site, cx, cy, cz, &frame, unit, &jd)
    }

    /// Geometric Alt/Az of the Earth as seen from this lunar site (TT JD).
    ///
    /// Uses the VSOP/ELP lunar ephemeris: Earth (selenocentric) = −Moon(geocentric).
    /// Finite-distance parallax is included. Range is in kilometres.
    fn observe_earth(&self, jd_tt: f64) -> PyResult<PyLunarObservation> {
        let jd = require_finite_jd(jd_tt)?;
        let ctx = default_ctx();
        let moon_geo = Vsop87Ephemeris::moon_geocentric(jd);
        let earth_sel = CPosition::<Selenocentric, EclipticMeanJ2000, Kilometer>::new(
            -moon_geo.x(),
            -moon_geo.y(),
            -moon_geo.z(),
        );
        let obs = self
            .site
            .observe_position(&earth_sel, &jd, &ctx)
            .map_err(surface_err)?;
        Ok(PyLunarObservation::with_range(
            obs.alt(),
            obs.az(),
            obs.range().to::<Kilometer>().value(),
            UNIT_KM,
        ))
    }

    /// Geometric Alt/Az of the Sun as seen from this lunar site (TT JD).
    ///
    /// Converts the Sun's barycentric position to selenocentric before
    /// observing so the line of sight is Moon→Sun (finite range in au).
    fn observe_sun(&self, jd_tt: f64) -> PyResult<PyLunarObservation> {
        let jd = require_finite_jd(jd_tt)?;
        let ctx = default_ctx();
        let sun_bary = Vsop87Ephemeris::sun_barycentric(jd);
        let sun_sel: CPosition<Selenocentric, EclipticMeanJ2000, AstronomicalUnit> =
            TransformCenter::to_center_with(&sun_bary, (), jd, &ctx);
        let obs = self
            .site
            .observe_position(&sun_sel, &jd, &ctx)
            .map_err(surface_err)?;
        Ok(PyLunarObservation::with_range(
            obs.alt(),
            obs.az(),
            obs.range().to::<AstronomicalUnit>().value(),
            UNIT_AU,
        ))
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let d = pyo3::types::PyDict::new(py);
        d.set_item("lon_deg", self.lon_deg())?;
        d.set_item("lat_deg", self.lat_deg())?;
        d.set_item("height_m", self.height_m())?;
        Ok(d)
    }

    #[staticmethod]
    fn from_dict(d: &Bound<'_, pyo3::types::PyDict>) -> PyResult<Self> {
        let get = |k: &str| -> PyResult<f64> {
            d.get_item(k)?
                .ok_or_else(|| PyValueError::new_err(format!("missing key '{}'", k)))?
                .extract()
        };
        Self::new(get("lon_deg")?, get("lat_deg")?, get("height_m")?)
    }

    fn __repr__(&self) -> String {
        format!(
            "LunarObserver(lon={:.4}°, lat={:.4}°, h={:.1}m)",
            self.lon_deg(),
            self.lat_deg(),
            self.height_m()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    fn __eq__(&self, other: &PyLunarObserver) -> bool {
        (self.lon_deg() - other.lon_deg()).abs() < 1e-10
            && (self.lat_deg() - other.lat_deg()).abs() < 1e-10
            && (self.height_m() - other.height_m()).abs() < 1e-6
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.lon_deg().to_bits().hash(&mut hasher);
        self.lat_deg().to_bits().hash(&mut hasher);
        self.height_m().to_bits().hash(&mut hasher);
        hasher.finish()
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, (f64, f64, f64))> {
        let cls = py.get_type::<Self>().into_any().unbind();
        Ok((cls, (self.lon_deg(), self.lat_deg(), self.height_m())))
    }
}

fn observe_selenocentric(
    site: &SurfaceSite<Moon>,
    x_au: f64,
    y_au: f64,
    z_au: f64,
    frame: &str,
    unit: &str,
    jd: &JulianDate,
) -> PyResult<PyLunarObservation> {
    let ctx = default_ctx();
    macro_rules! observe {
        ($Frame:ty) => {{
            if unit == UNIT_KM {
                let (x, y, z) = au_to_km(x_au, y_au, z_au);
                let p = CPosition::<Selenocentric, $Frame, Kilometer>::new(x, y, z);
                let obs = site.observe_position(&p, jd, &ctx).map_err(surface_err)?;
                Ok(PyLunarObservation::with_range(
                    obs.alt(),
                    obs.az(),
                    obs.range().value(),
                    UNIT_KM,
                ))
            } else {
                let p = CPosition::<Selenocentric, $Frame, AstronomicalUnit>::new(x_au, y_au, z_au);
                let obs = site.observe_position(&p, jd, &ctx).map_err(surface_err)?;
                Ok(PyLunarObservation::with_range(
                    obs.alt(),
                    obs.az(),
                    obs.range().value(),
                    UNIT_AU,
                ))
            }
        }};
    }

    match frame {
        FRAME_ICRS => observe!(ICRS),
        FRAME_ICRF => observe!(ICRF),
        FRAME_ECL => observe!(EclipticMeanJ2000),
        FRAME_EQ => observe!(EquatorialMeanJ2000),
        FRAME_EMOD => observe!(EquatorialMeanOfDate),
        FRAME_ETOD => observe!(EquatorialTrueOfDate),
        FRAME_MOON_IAU => observe!(MoonIauMeanEarth),
        other => Err(PyValueError::new_err(format!(
            "Unsupported frame '{}' for lunar observation",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use siderust::J2000;

    #[test]
    fn quickstart_ncp_at_j2000() {
        let site = PyLunarObserver::new(20.0, -3.5, 0.0).unwrap();
        let hz = site
            .observe_icrs_vector(0.0, 0.0, 1.0, J2000.value())
            .unwrap();
        assert_abs_diff_eq!(hz.inner.alt().value(), 17.765928422856998, epsilon = 1e-9);
        assert_abs_diff_eq!(hz.inner.az().value(), 12.045094675187446, epsilon = 1e-9);
    }

    #[test]
    fn cspice_composite_body_fixed_altaz() {
        let site = PyLunarObserver::new(20.0, -3.5, 0.0).unwrap();
        let target = PyPosition::new_internal(
            1617.7391512763131,
            642.0177864954426,
            -96.38962730131595,
            FRAME_MOON_IAU,
            CENTER_SELENO,
            UNIT_KM,
        );
        let obs = site.observe_position(&target, 2_451_545.0).unwrap();
        assert_abs_diff_eq!(obs.alt_deg, 5.60040918481664, epsilon = 1e-9);
        assert_abs_diff_eq!(obs.az_deg, 78.69006752597979, epsilon = 1e-9);
        assert_abs_diff_eq!(obs.range.unwrap(), 51.23475382979799, epsilon = 1e-9);
    }

    #[test]
    fn rejects_invalid_latitude() {
        assert!(PyLunarObserver::new(0.0, 95.0, 0.0).is_err());
    }

    #[test]
    fn rejects_zero_direction() {
        let site = PyLunarObserver::new(0.0, 0.0, 0.0).unwrap();
        assert!(site
            .observe_icrs_vector(0.0, 0.0, 0.0, J2000.value())
            .is_err());
    }

    #[test]
    fn moon_principal_axes_alias_canonicalizes() {
        let frame = canonicalize_frame("MoonPrincipalAxes").unwrap();
        assert_eq!(frame, FRAME_MOON_IAU);
    }
}
