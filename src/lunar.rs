//! Lunar surface observer and geometric Alt/Az bindings.
//!
//! Two backends are supported:
//!
//! - **Analytical** (`LunarObserver(lon, lat, height)`): VSOP/ELP ephemerides
//!   with NAIF `IAU_MOON` Mean-Earth orientation (`MoonIauMeanEarth`). The
//!   legacy `MoonPrincipalAxes` name remains the same analytical marker, **not**
//!   JPL DE `MOON_PA`.
//! - **SPICE** (`LunarObserver.from_spice(...)`): DE-specific frames from an
//!   immutable [`crate::spice_context::PySpiceContext`] (SPK + binary PCK + FK).
//!   No analytical fallback.
//!
//! Results are geometric only (`abcorr=NONE`): no refraction, aberration,
//! light-time, or terrain.

use std::sync::Arc;

use pyo3::exceptions::{PyTypeError, PyValueError};
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
    BodySurfaceModel, LatitudeConvention, ReferenceSurface, SurfaceError, SurfaceSite,
};
use siderust::coordinates::transform::context::{DefaultEop, DefaultEphemeris};
use siderust::coordinates::transform::{AstroContext, SurfaceHorizontalExt, TransformCenter};
use siderust::ephemeris::{Ephemeris, Vsop87Ephemeris};
use siderust::qtty::*;
use siderust::spice::{
    LunarObserver as SpiceLunarObserver, LunarTarget as SpiceLunarTarget, SpiceContext,
};
use siderust::tempoch::{Time, TDB};
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
use crate::spice_context::{spice_err, PySpiceContext};

const AU_KM: f64 = 149_597_870.700;
const BACKEND_ANALYTICAL: &str = "analytical";
const BACKEND_SPICE: &str = "spice";
const SITE_HEIGHT: &str = "height";
const SITE_RADIAL: &str = "radial";

fn surface_err(err: SurfaceError) -> PyErr {
    PyValueError::new_err(err.to_string())
}

fn require_finite_jd(jd_tt: f64) -> PyResult<JulianDate> {
    if !jd_tt.is_finite() {
        return Err(PyValueError::new_err("jd_tt must be finite"));
    }
    Ok(JulianDate::new(jd_tt))
}

fn require_finite_et(et_tdb_seconds: f64) -> PyResult<Time<TDB>> {
    if !et_tdb_seconds.is_finite() {
        return Err(PyValueError::new_err(
            "et_tdb_seconds must be finite TDB seconds past J2000",
        ));
    }
    Ok(Time::<TDB>::new(et_tdb_seconds))
}

/// Convert a TT Julian Date into the typed TDB epoch used by Rust SPICE.
fn jd_tt_to_tdb(jd_tt: f64) -> PyResult<Time<TDB>> {
    let jd = require_finite_jd(jd_tt)?;
    Ok(jd.to::<TDB>().to_j2000s())
}

/// Normalize a finite nonzero ICRS vector without overflow/underflow.
///
/// Scales by the maximum absolute component before forming the Euclidean
/// norm so extreme but finite magnitudes (e.g. `1e±200`) remain usable.
/// Affn/`Direction::try_new` use an unscaled magnitude and would reject or
/// overflow these cases, so this binding keeps a local scale-invariant path.
fn normalize_icrs_direction(x: f64, y: f64, z: f64) -> PyResult<Direction<ICRS>> {
    for (name, value) in [("x", x), ("y", y), ("z", z)] {
        if !value.is_finite() {
            return Err(PyValueError::new_err(format!("{name} must be finite")));
        }
    }
    let scale = x.abs().max(y.abs()).max(z.abs());
    if scale == 0.0 {
        return Err(PyValueError::new_err(
            "ICRS direction must be a non-zero finite vector",
        ));
    }
    let (sx, sy, sz) = (x / scale, y / scale, z / scale);
    let norm = (sx * sx + sy * sy + sz * sz).sqrt();
    if !norm.is_finite() || norm == 0.0 {
        return Err(PyValueError::new_err(
            "ICRS direction must be a non-zero finite vector",
        ));
    }
    Ok(Direction::<ICRS>::new_unchecked(
        sx / norm,
        sy / norm,
        sz / norm,
    ))
}

/// Hash bits that agree with Python/`f64` equality for signed zeros.
fn hashable_float(value: f64) -> u64 {
    if value == 0.0 {
        0
    } else {
        value.to_bits()
    }
}

fn to_horizontal(alt: Degrees, az: Degrees) -> PyHorizontal {
    PyHorizontal::from_inner(sph_dir::Horizontal::new(alt, az))
}

fn au_to_km(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    (x * AU_KM, y * AU_KM, z * AU_KM)
}

fn require_finite_site(lon_deg: f64, lat_deg: f64, value: f64, value_name: &str) -> PyResult<()> {
    for (name, v) in [
        ("lon_deg", lon_deg),
        ("lat_deg", lat_deg),
        (value_name, value),
    ] {
        if !v.is_finite() {
            return Err(PyValueError::new_err(format!("{name} must be finite")));
        }
    }
    Ok(())
}

/// How a SPICE site radius/height was defined.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpiceSiteKind {
    /// Height above the Moon IAU mean sphere (1737.4 km).
    Height,
    /// Actual Moon-centre radial distance (radial local up).
    Radial,
}

/// Owned SPICE backend state. The Rust observer borrows `SpiceContext`; we keep
/// an [`Arc`] and rebuild the borrowing observer transiently per observation.
#[derive(Clone)]
struct SpiceBackend {
    context: Arc<SpiceContext>,
    /// Canonical uppercase frame name from the FK registry.
    frame: String,
    frame_id: i32,
    site_kind: SpiceSiteKind,
    /// Provenance aliases captured at observer construction (stable for eq/hash).
    provenance: Vec<String>,
}

/// Geometric lunar-surface observation (altitude, azimuth, optional range).
///
/// Direction-only observations omit range. Finite-distance observations
/// report slant range in the same unit as the input position (`au` or `km`).
/// SPICE-backed Sun/Earth observations always report range in kilometres.
#[pyclass(name = "LunarObservation", module = "siderust", frozen, from_py_object)]
#[derive(Clone)]
pub struct PyLunarObservation {
    alt_deg: f64,
    az_deg: f64,
    range: Option<f64>,
    range_unit: Option<String>,
    above_horizon: bool,
    backend: String,
    frame: Option<String>,
    frame_id: Option<i32>,
    target: Option<String>,
    epoch_et_tdb_seconds: Option<f64>,
    kernels: Vec<String>,
}

impl PyLunarObservation {
    fn analytical_with_range(alt: Degrees, az: Degrees, range: f64, unit: &str) -> Self {
        Self {
            alt_deg: alt.value(),
            az_deg: az.value(),
            range: Some(range),
            range_unit: Some(unit.to_string()),
            above_horizon: alt.value() > 0.0,
            backend: BACKEND_ANALYTICAL.to_string(),
            frame: Some("MoonIauMeanEarth".to_string()),
            frame_id: None,
            target: None,
            epoch_et_tdb_seconds: None,
            kernels: Vec::new(),
        }
    }

    fn from_spice(obs: siderust::spice::LunarObservation<'_>, frame: &str, target: &str) -> Self {
        let alt = obs.horizontal.alt();
        let az = obs.horizontal.az();
        let range = obs.horizontal.range().to::<Kilometer>().value();
        let epoch = obs.epoch.to_j2000s().raw().value();
        let kernels: Vec<String> = obs.kernels.iter().map(|s| (*s).to_string()).collect();
        Self {
            alt_deg: alt.value(),
            az_deg: az.value(),
            range: Some(range),
            range_unit: Some(UNIT_KM.to_string()),
            above_horizon: obs.above_horizon(),
            backend: BACKEND_SPICE.to_string(),
            frame: Some(frame.to_string()),
            frame_id: Some(obs.frame_id),
            target: Some(target.to_string()),
            epoch_et_tdb_seconds: Some(epoch),
            kernels,
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

    /// Whether the target centre is strictly above the geometric horizon.
    #[getter]
    fn above_horizon(&self) -> bool {
        self.above_horizon
    }

    /// Observation backend: `"analytical"` or `"spice"`.
    #[getter]
    fn backend(&self) -> &str {
        &self.backend
    }

    /// Lunar frame name when known (`MoonIauMeanEarth`, `MOON_ME`, …).
    #[getter]
    fn frame(&self) -> Option<&str> {
        self.frame.as_deref()
    }

    /// NAIF frame ID for SPICE-backed results; `None` for analytical.
    #[getter]
    fn frame_id(&self) -> Option<i32> {
        self.frame_id
    }

    /// Target name for SPICE Sun/Earth observations (`"Sun"` / `"Earth"`).
    #[getter]
    fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    /// Evaluation epoch as TDB seconds past J2000 (SPICE ET), when recorded.
    #[getter]
    fn epoch_et_tdb_seconds(&self) -> Option<f64> {
        self.epoch_et_tdb_seconds
    }

    /// Loaded kernel aliases from the observation context (SPICE only).
    ///
    /// Lists candidate aliases with load order, not a claim that every file
    /// contributed the exact selected SPK/BPC segment.
    #[getter]
    fn kernels(&self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        let list = pyo3::types::PyList::empty(py);
        for alias in &self.kernels {
            list.append(alias)?;
        }
        Ok(list.unbind())
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
        d.set_item("above_horizon", self.above_horizon)?;
        d.set_item("backend", self.backend.as_str())?;
        d.set_item("frame", self.frame.as_deref())?;
        d.set_item("frame_id", self.frame_id)?;
        d.set_item("target", self.target.as_deref())?;
        d.set_item("epoch_et_tdb_seconds", self.epoch_et_tdb_seconds)?;
        d.set_item("kernels", self.kernels.clone())?;
        Ok(d)
    }

    fn __repr__(&self) -> String {
        match (&self.range, &self.range_unit) {
            (Some(r), Some(u)) => format!(
                "LunarObservation(alt_deg={:.6}, az_deg={:.6}, range={:.6} {}, backend={})",
                self.alt_deg, self.az_deg, r, u, self.backend
            ),
            _ => format!(
                "LunarObservation(alt_deg={:.6}, az_deg={:.6}, backend={})",
                self.alt_deg, self.az_deg, self.backend
            ),
        }
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Observer site on the Moon.
///
/// Construct analytically with [`Self::new`], or bind a DE-specific SPICE
/// frame with [`Self::from_spice`]. Analytical and SPICE observers at the same
/// longitude/latitude are **not** equivalent scientific objects.
///
/// Longitude is east-positive; latitude is planetocentric. Epochs for the
/// ordinary methods are **Julian Date TT**. SPICE paths also accept exact ET
/// via `observe_*_et`. Results are geometric only.
#[pyclass(name = "LunarObserver", module = "siderust", from_py_object)]
#[derive(Clone)]
pub struct PyLunarObserver {
    pub(crate) site: SurfaceSite<Moon>,
    spice: Option<SpiceBackend>,
}

impl PyLunarObserver {
    fn analytical(site: SurfaceSite<Moon>) -> Self {
        Self { site, spice: None }
    }

    fn spice_observer(&self) -> PyResult<(&SpiceBackend, SpiceLunarObserver<'_>)> {
        let backend = self.spice.as_ref().ok_or_else(|| {
            PyValueError::new_err("internal error: spice backend required for this path")
        })?;
        let observer = SpiceLunarObserver::new(backend.context.as_ref(), &backend.frame, self.site)
            .map_err(spice_err)?;
        Ok((backend, observer))
    }

    fn observe_spice_target(
        &self,
        target: SpiceLunarTarget,
        epoch: Time<TDB>,
    ) -> PyResult<PyLunarObservation> {
        let (backend, observer) = self.spice_observer()?;
        let obs = observer.observe(target, epoch).map_err(spice_err)?;
        let target_name = match target {
            SpiceLunarTarget::Sun => "Sun",
            SpiceLunarTarget::Earth => "Earth",
        };
        Ok(PyLunarObservation::from_spice(
            obs,
            &backend.frame,
            target_name,
        ))
    }

    fn reject_spice_for(&self, method: &str) -> PyResult<()> {
        if self.spice.is_some() {
            return Err(PyValueError::new_err(format!(
                "{method} is not supported for SPICE-backed LunarObserver; \
                 use observe_sun / observe_earth (or the *_et variants). \
                 Direction and arbitrary Position observation remain analytical-only \
                 because a coherent SPICE sky-direction semantics is not exposed here."
            )));
        }
        Ok(())
    }
}

#[pymethods]
impl PyLunarObserver {
    /// Create an analytical lunar surface site (IAU mean sphere / IAU_MOON).
    ///
    /// Args:
    ///     lon_deg: Longitude in degrees, east-positive.
    ///     lat_deg: Planetocentric latitude in degrees, north-positive.
    ///     height_m: Height above the IAU mean sphere in metres (default 0).
    #[new]
    #[pyo3(signature = (lon_deg, lat_deg, height_m = 0.0))]
    fn new(lon_deg: f64, lat_deg: f64, height_m: f64) -> PyResult<Self> {
        require_finite_site(lon_deg, lat_deg, height_m, "height_m")?;
        let site = SurfaceSite::<Moon>::new(
            Degrees::new(lon_deg),
            Degrees::new(lat_deg),
            Meters::new(height_m),
        )
        .map_err(surface_err)?;
        Ok(Self::analytical(site))
    }

    /// Bind a site to an immutable SPICE kernel context and lunar FK frame.
    ///
    /// Args:
    ///     context: [`SpiceContext`](PySpiceContext) with compatible SPK/BPC/FK.
    ///     frame: Runtime frame name, e.g. ``\"MOON_ME\"`` or ``\"MOON_PA\"``.
    ///     lon_deg: East-positive longitude in degrees (interpreted in `frame`).
    ///     lat_deg: Planetocentric latitude in degrees, north-positive.
    ///     height_m: Height above the IAU mean sphere (radius 1737.4 km).
    ///         Mutually exclusive with ``radius_m``. Defaults to 0 when neither
    ///         height nor radius is given.
    ///     radius_m: Actual Moon-centre radial distance in metres (radial up).
    ///         Mutually exclusive with ``height_m``.
    ///
    /// The observer retains a cloned ``Arc`` of the context and remains valid
    /// after the caller's `context` variable is deleted. There is no analytical
    /// fallback: missing coverage or incompatible frames raise ``ValueError``.
    #[classmethod]
    #[pyo3(signature = (context, frame, lon_deg, lat_deg, *, height_m=None, radius_m=None))]
    fn from_spice(
        _cls: &Bound<'_, pyo3::types::PyType>,
        context: &PySpiceContext,
        frame: &str,
        lon_deg: f64,
        lat_deg: f64,
        height_m: Option<f64>,
        radius_m: Option<f64>,
    ) -> PyResult<Self> {
        if height_m.is_some() && radius_m.is_some() {
            return Err(PyValueError::new_err(
                "specify height_m or radius_m, not both",
            ));
        }
        let frame = frame.trim();
        if frame.is_empty() {
            return Err(PyValueError::new_err("frame must be a non-empty string"));
        }

        let (site, site_kind) = if let Some(radius) = radius_m {
            require_finite_site(lon_deg, lat_deg, radius, "radius_m")?;
            let site = SurfaceSite::with_surface(
                Degrees::new(lon_deg),
                Degrees::new(lat_deg),
                Meters::new(0.0),
                ReferenceSurface::sphere(Meters::new(radius)).map_err(surface_err)?,
                LatitudeConvention::Planetocentric,
            )
            .map_err(surface_err)?;
            // Validate the frame chain via the Rust constructor.
            let _ = SpiceLunarObserver::radial(
                context.inner.as_ref(),
                frame,
                Degrees::new(lon_deg),
                Degrees::new(lat_deg),
                Meters::new(radius),
            )
            .map_err(spice_err)?;
            (site, SpiceSiteKind::Radial)
        } else {
            let height = height_m.unwrap_or(0.0);
            require_finite_site(lon_deg, lat_deg, height, "height_m")?;
            let site = SurfaceSite::<Moon>::new(
                Degrees::new(lon_deg),
                Degrees::new(lat_deg),
                Meters::new(height),
            )
            .map_err(surface_err)?;
            let _ =
                SpiceLunarObserver::new(context.inner.as_ref(), frame, site).map_err(spice_err)?;
            (site, SpiceSiteKind::Height)
        };

        let spec = context
            .inner
            .frame_registry()
            .frame_by_name(frame)
            .map_err(|e| spice_err(e.into()))?;
        let backend = SpiceBackend {
            context: context.shared(),
            frame: spec.name.clone(),
            frame_id: spec.id,
            site_kind,
            provenance: context.provenance_aliases().to_vec(),
        };
        Ok(Self {
            site,
            spice: Some(backend),
        })
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

    /// Height above the reference surface in metres.
    ///
    /// For radial SPICE sites this is ``0``; use [`Self::radius_m`] for the
    /// Moon-centre distance.
    #[getter]
    fn height_m(&self) -> f64 {
        self.site.height().value()
    }

    /// Reference equatorial / sphere radius in metres.
    #[getter]
    fn equatorial_radius_m(&self) -> f64 {
        self.site.surface().equatorial_radius().value()
    }

    /// Moon-centre radial distance in metres for radial SPICE sites; else `None`.
    #[getter]
    fn radius_m(&self) -> Option<f64> {
        match self.spice.as_ref().map(|b| b.site_kind) {
            Some(SpiceSiteKind::Radial) => Some(self.equatorial_radius_m()),
            _ => None,
        }
    }

    /// Site definition kind: `"height"`, `"radial"`, or analytical height.
    #[getter]
    fn site_kind(&self) -> &'static str {
        match self.spice.as_ref().map(|b| b.site_kind) {
            Some(SpiceSiteKind::Radial) => SITE_RADIAL,
            Some(SpiceSiteKind::Height) | None => SITE_HEIGHT,
        }
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

    /// Backend: `"analytical"` or `"spice"`.
    #[getter]
    fn backend(&self) -> &'static str {
        if self.spice.is_some() {
            BACKEND_SPICE
        } else {
            BACKEND_ANALYTICAL
        }
    }

    /// Lunar frame name (`MoonIauMeanEarth` or a SPICE FK name such as `MOON_ME`).
    #[getter]
    fn frame(&self) -> String {
        match &self.spice {
            Some(b) => b.frame.clone(),
            None => "MoonIauMeanEarth".to_string(),
        }
    }

    /// NAIF frame ID for SPICE observers; `None` for analytical.
    #[getter]
    fn frame_id(&self) -> Option<i32> {
        self.spice.as_ref().map(|b| b.frame_id)
    }

    /// Loaded kernel aliases for SPICE observers (empty for analytical).
    #[getter]
    fn kernels(&self, py: Python<'_>) -> PyResult<Py<pyo3::types::PyList>> {
        let list = pyo3::types::PyList::empty(py);
        if let Some(backend) = &self.spice {
            for alias in &backend.provenance {
                list.append(alias)?;
            }
        }
        Ok(list.unbind())
    }

    /// Geometric Alt/Az of a fixed ICRS sky direction at Julian Date TT.
    ///
    /// Direction-only: no distance is invented, so there is no parallax.
    /// Not available for SPICE-backed observers.
    fn observe_direction(&self, direction: &PyIcrs, jd_tt: f64) -> PyResult<PyHorizontal> {
        self.reject_spice_for("observe_direction")?;
        let cart = direction.inner.to_cartesian();
        self.observe_icrs_vector(cart.x(), cart.y(), cart.z(), jd_tt)
    }

    /// Geometric Alt/Az of an ICRS Cartesian direction at Julian Date TT.
    ///
    /// The vector is normalized; a zero-length vector raises `ValueError`.
    /// No parallax is applied. Not available for SPICE-backed observers.
    #[pyo3(signature = (x, y, z, jd_tt))]
    fn observe_icrs_vector(&self, x: f64, y: f64, z: f64, jd_tt: f64) -> PyResult<PyHorizontal> {
        self.reject_spice_for("observe_icrs_vector")?;
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
    /// Analytical backend only. SPICE-backed observers must use
    /// `observe_sun` / `observe_earth`.
    fn observe_position(&self, position: &PyPosition, jd_tt: f64) -> PyResult<PyLunarObservation> {
        self.reject_spice_for("observe_position")?;
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
    /// Analytical: VSOP/ELP lunar ephemeris. SPICE: SPK/PCK/FK pipeline with
    /// range in kilometres. Finite-distance parallax is included.
    fn observe_earth(&self, jd_tt: f64) -> PyResult<PyLunarObservation> {
        if self.spice.is_some() {
            return self.observe_spice_target(SpiceLunarTarget::Earth, jd_tt_to_tdb(jd_tt)?);
        }
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
        Ok(PyLunarObservation::analytical_with_range(
            obs.alt(),
            obs.az(),
            obs.range().to::<Kilometer>().value(),
            UNIT_KM,
        ))
    }

    /// Geometric Alt/Az of the Sun as seen from this lunar site (TT JD).
    ///
    /// Analytical: VSOP Sun with range in au. SPICE: SPK/PCK/FK with range in
    /// kilometres. Finite-distance parallax is included.
    fn observe_sun(&self, jd_tt: f64) -> PyResult<PyLunarObservation> {
        if self.spice.is_some() {
            return self.observe_spice_target(SpiceLunarTarget::Sun, jd_tt_to_tdb(jd_tt)?);
        }
        let jd = require_finite_jd(jd_tt)?;
        let ctx = default_ctx();
        let sun_bary = Vsop87Ephemeris::sun_barycentric(jd);
        let sun_sel: CPosition<Selenocentric, EclipticMeanJ2000, AstronomicalUnit> =
            TransformCenter::to_center_with(&sun_bary, (), jd, &ctx);
        let obs = self
            .site
            .observe_position(&sun_sel, &jd, &ctx)
            .map_err(surface_err)?;
        Ok(PyLunarObservation::analytical_with_range(
            obs.alt(),
            obs.az(),
            obs.range().to::<AstronomicalUnit>().value(),
            UNIT_AU,
        ))
    }

    /// Geometric Sun observation at an exact SPICE ET (TDB seconds past J2000).
    ///
    /// Prefer this for same-epoch CSPICE parity tests. Analytical observers
    /// raise ``ValueError``.
    fn observe_sun_et(&self, et_tdb_seconds: f64) -> PyResult<PyLunarObservation> {
        if self.spice.is_none() {
            return Err(PyValueError::new_err(
                "observe_sun_et requires a SPICE-backed LunarObserver",
            ));
        }
        self.observe_spice_target(SpiceLunarTarget::Sun, require_finite_et(et_tdb_seconds)?)
    }

    /// Geometric Earth observation at an exact SPICE ET (TDB seconds past J2000).
    fn observe_earth_et(&self, et_tdb_seconds: f64) -> PyResult<PyLunarObservation> {
        if self.spice.is_none() {
            return Err(PyValueError::new_err(
                "observe_earth_et requires a SPICE-backed LunarObserver",
            ));
        }
        self.observe_spice_target(SpiceLunarTarget::Earth, require_finite_et(et_tdb_seconds)?)
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let d = pyo3::types::PyDict::new(py);
        d.set_item("lon_deg", self.lon_deg())?;
        d.set_item("lat_deg", self.lat_deg())?;
        d.set_item("height_m", self.height_m())?;
        d.set_item("backend", self.backend())?;
        d.set_item("frame", self.frame())?;
        d.set_item("site_kind", self.site_kind())?;
        if let Some(backend) = &self.spice {
            d.set_item("frame_id", backend.frame_id)?;
            d.set_item("radius_m", self.radius_m())?;
            d.set_item("kernels", backend.provenance.clone())?;
            d.set_item(
                "note",
                "SPICE LunarObserver cannot be reconstructed from dict without the original kernels",
            )?;
        }
        Ok(d)
    }

    /// Reconstruct an **analytical** observer from a dict.
    ///
    /// SPICE-backed dicts (``backend == "spice"``) are rejected so a SPICE
    /// observer is never silently rebuilt as analytical.
    #[staticmethod]
    fn from_dict(d: &Bound<'_, pyo3::types::PyDict>) -> PyResult<Self> {
        if let Some(backend) = d.get_item("backend")? {
            let backend: String = backend.extract()?;
            if backend == BACKEND_SPICE {
                return Err(PyTypeError::new_err(
                    "cannot reconstruct a SPICE LunarObserver from dict; \
                     rebuild with LunarObserver.from_spice(context, ...)",
                ));
            }
        }
        let get = |k: &str| -> PyResult<f64> {
            d.get_item(k)?
                .ok_or_else(|| PyValueError::new_err(format!("missing key '{}'", k)))?
                .extract()
        };
        Self::new(get("lon_deg")?, get("lat_deg")?, get("height_m")?)
    }

    fn __repr__(&self) -> String {
        match &self.spice {
            Some(b) => match b.site_kind {
                SpiceSiteKind::Radial => format!(
                    "LunarObserver.from_spice(frame={:?}, lon={:.4}°, lat={:.4}°, radius_m={:.1})",
                    b.frame,
                    self.lon_deg(),
                    self.lat_deg(),
                    self.equatorial_radius_m()
                ),
                SpiceSiteKind::Height => format!(
                    "LunarObserver.from_spice(frame={:?}, lon={:.4}°, lat={:.4}°, height_m={:.1})",
                    b.frame,
                    self.lon_deg(),
                    self.lat_deg(),
                    self.height_m()
                ),
            },
            None => format!(
                "LunarObserver(lon={:.4}°, lat={:.4}°, h={:.1}m)",
                self.lon_deg(),
                self.lat_deg(),
                self.height_m()
            ),
        }
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    /// Exact equality of validated site coordinates and backend identity.
    ///
    /// Analytical observers compare longitude/latitude/height. SPICE observers
    /// also require the same frame, site kind, and kernel provenance aliases.
    /// Analytical and SPICE observers never compare equal.
    fn __eq__(&self, other: &PyLunarObserver) -> bool {
        if self.spice.is_some() != other.spice.is_some() {
            return false;
        }
        if self.lon_deg() != other.lon_deg()
            || self.lat_deg() != other.lat_deg()
            || self.height_m() != other.height_m()
            || self.equatorial_radius_m() != other.equatorial_radius_m()
        {
            return false;
        }
        match (&self.spice, &other.spice) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                a.frame == b.frame
                    && a.frame_id == b.frame_id
                    && a.site_kind == b.site_kind
                    && a.provenance == b.provenance
            }
            _ => false,
        }
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.backend().hash(&mut hasher);
        hashable_float(self.lon_deg()).hash(&mut hasher);
        hashable_float(self.lat_deg()).hash(&mut hasher);
        hashable_float(self.height_m()).hash(&mut hasher);
        hashable_float(self.equatorial_radius_m()).hash(&mut hasher);
        if let Some(backend) = &self.spice {
            backend.frame.hash(&mut hasher);
            backend.frame_id.hash(&mut hasher);
            (backend.site_kind as u8).hash(&mut hasher);
            for alias in &backend.provenance {
                alias.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Pickle support for analytical observers only.
    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, (f64, f64, f64))> {
        if self.spice.is_some() {
            return Err(PyTypeError::new_err(
                "SPICE LunarObserver cannot be pickled; rebuild with from_spice",
            ));
        }
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
                Ok(PyLunarObservation::analytical_with_range(
                    obs.alt(),
                    obs.az(),
                    obs.range().value(),
                    UNIT_KM,
                ))
            } else {
                let p = CPosition::<Selenocentric, $Frame, AstronomicalUnit>::new(x_au, y_au, z_au);
                let obs = site.observe_position(&p, jd, &ctx).map_err(surface_err)?;
                Ok(PyLunarObservation::analytical_with_range(
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
    use siderust::coordinates::surface::MOON_IAU_SPHERE;
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

    #[test]
    fn normalize_survives_extreme_finite_magnitudes() {
        let ref_dir = normalize_icrs_direction(1.0, 2.0, 3.0).unwrap();
        for scale in [1e-200_f64, 1e200_f64] {
            let dir = normalize_icrs_direction(1.0 * scale, 2.0 * scale, 3.0 * scale).unwrap();
            assert_abs_diff_eq!(dir.x(), ref_dir.x(), epsilon = 1e-12);
            assert_abs_diff_eq!(dir.y(), ref_dir.y(), epsilon = 1e-12);
            assert_abs_diff_eq!(dir.z(), ref_dir.z(), epsilon = 1e-12);
        }
        assert!(normalize_icrs_direction(0.0, 0.0, 0.0).is_err());
        assert!(normalize_icrs_direction(f64::NAN, 0.0, 0.0).is_err());
        assert!(normalize_icrs_direction(f64::INFINITY, 0.0, 0.0).is_err());
    }

    #[test]
    fn equality_matches_hash_for_exact_and_signed_zero() {
        fn hash_bits(obs: &PyLunarObserver) -> u64 {
            obs.__hash__()
        }

        let a = PyLunarObserver::new(20.0, -3.5, 0.0).unwrap();
        let b = PyLunarObserver::new(20.0, -3.5, 0.0).unwrap();
        assert!(a.__eq__(&b));
        assert_eq!(hash_bits(&a), hash_bits(&b));

        let near = PyLunarObserver::new(20.0, -3.5, 1e-7).unwrap();
        assert!(!a.__eq__(&near));

        let pos = PyLunarObserver::new(0.0, 0.0, 0.0).unwrap();
        let neg = PyLunarObserver::new(-0.0, -0.0, -0.0).unwrap();
        assert!(pos.__eq__(&neg));
        assert_eq!(hash_bits(&pos), hash_bits(&neg));

        let east = PyLunarObserver::new(190.0, 0.0, 0.0).unwrap();
        let west = PyLunarObserver::new(-170.0, 0.0, 0.0).unwrap();
        assert!(east.__eq__(&west));
        assert_eq!(hash_bits(&east), hash_bits(&west));
    }

    #[test]
    fn analytical_iau_sphere_radius_unchanged() {
        let site = PyLunarObserver::new(0.0, 0.0, 0.0).unwrap();
        assert_eq!(
            site.equatorial_radius_m(),
            MOON_IAU_SPHERE.equatorial_radius().value()
        );
        assert_eq!(site.backend(), BACKEND_ANALYTICAL);
        assert!(site.radius_m().is_none());
    }
}
