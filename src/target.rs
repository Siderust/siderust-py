//! Target and ProperMotion types for Python.
//!
//! Provides:
//! - `Target`: timestamped coordinate snapshot wrapping a Position or Direction
//! - `ProperMotion`: stellar proper motion in RA/Dec
//! - `apply_proper_motion()`: free function to propagate a direction

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use qtty::*;
use siderust::astro::proper_motion;
use siderust::bodies::solar_system;
use siderust::coordinates::spherical::direction;
use siderust::targets::Trackable;
use tempoch::JulianDate;

use crate::bodies::PyBody;
use crate::coordinates::PyDirection;
use crate::position::{PyPosition, CENTER_BARY, CENTER_GEO, FRAME_ECL, UNIT_AU, UNIT_KM};
use crate::star::PyStar;

// =============================================================================
// PyProperMotion
// =============================================================================

/// Proper motion of a star in RA and Dec.
///
/// Proper motion is the apparent angular displacement of a star across
/// the sky due to its real motion through space.
///
/// Example:
///     >>> pm = ProperMotion(pm_ra_mas_yr=27.54, pm_dec_mas_yr=10.86)
#[pyclass(name = "ProperMotion", module = "siderust", from_py_object)]
#[derive(Clone)]
pub struct PyProperMotion {
    pub(crate) inner: proper_motion::ProperMotion,
    pub(crate) ra_mas: f64,
    pub(crate) dec_mas: f64,
    pub(crate) mu_alpha_star: bool,
}

#[pymethods]
impl PyProperMotion {
    /// Create a proper motion from RA and Dec rates.
    ///
    /// Args:
    ///     pm_ra_mas_yr: RA proper motion in milliarcseconds/year.
    ///     pm_dec_mas_yr: Dec proper motion in milliarcseconds/year.
    ///     mu_alpha_star: If True (default), pm_ra is µα⋆ = µα·cos(δ)
    ///                    (the convention used by Gaia/Hipparcos catalogs).
    ///                    If False, pm_ra is the true RA angular rate µα.
    #[new]
    #[pyo3(signature = (pm_ra_mas_yr, pm_dec_mas_yr, mu_alpha_star = true))]
    fn new(pm_ra_mas_yr: f64, pm_dec_mas_yr: f64, mu_alpha_star: bool) -> Self {
        type MasPerYear = Per<MilliArcsecond, Year>;
        type MasPerYearQ = Quantity<MasPerYear>;

        let inner = if mu_alpha_star {
            proper_motion::ProperMotion::from_mu_alpha_star::<MasPerYear>(
                MasPerYearQ::new(pm_ra_mas_yr),
                MasPerYearQ::new(pm_dec_mas_yr),
            )
        } else {
            proper_motion::ProperMotion::from_mu_alpha::<MasPerYear>(
                MasPerYearQ::new(pm_ra_mas_yr),
                MasPerYearQ::new(pm_dec_mas_yr),
            )
        };

        Self {
            inner,
            ra_mas: pm_ra_mas_yr,
            dec_mas: pm_dec_mas_yr,
            mu_alpha_star,
        }
    }

    /// RA proper motion in mas/yr.
    #[getter]
    fn pm_ra_mas_yr(&self) -> f64 {
        self.ra_mas
    }

    /// Dec proper motion in mas/yr.
    #[getter]
    fn pm_dec_mas_yr(&self) -> f64 {
        self.dec_mas
    }

    /// Whether RA rate uses µα⋆ convention.
    #[getter]
    fn mu_alpha_star(&self) -> bool {
        self.mu_alpha_star
    }

    fn __repr__(&self) -> String {
        format!(
            "ProperMotion(pm_ra={:.2} mas/yr, pm_dec={:.2} mas/yr, µα⋆={})",
            self.ra_mas, self.dec_mas, self.mu_alpha_star,
        )
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, (f64, f64, bool))> {
        let cls = py.get_type::<Self>().into_any().unbind();
        Ok((cls, (self.ra_mas, self.dec_mas, self.mu_alpha_star)))
    }
}

// =============================================================================
// PyTarget
// =============================================================================

/// Inner representation for Target position.
#[derive(Clone)]
enum TargetInner {
    Position(PyPosition),
    Direction(PyDirection),
}

/// A timestamped coordinate snapshot with optional proper motion.
///
/// Target couples a position (or direction) with an epoch and optional proper
/// motion. It is the Python equivalent of Rust's `CoordinateWithPM<T>`.
///
/// Example:
///     >>> target = Body.Mars.track(2451545.0)
///     >>> print(target.time, target.position.distance())
///     >>> # With proper motion:
///     >>> pm = ProperMotion(pm_ra_mas_yr=27.54, pm_dec_mas_yr=10.86)
///     >>> target = Target(direction, jd=2451545.0, proper_motion=pm)
#[pyclass(name = "Target", module = "siderust", skip_from_py_object)]
#[derive(Clone)]
pub struct PyTarget {
    inner: TargetInner,
    time: f64, // Julian Date
    proper_motion: Option<PyProperMotion>,
}

#[pymethods]
impl PyTarget {
    /// Create a Target from a Position and a Julian Date.
    ///
    /// Args:
    ///     position: A Position or Direction object.
    ///     jd: Julian Date of the snapshot.
    ///     proper_motion: Optional ProperMotion for stars/moving objects.
    #[new]
    #[pyo3(signature = (position, jd, proper_motion = None))]
    fn new(
        position: &Bound<'_, PyAny>,
        jd: f64,
        proper_motion: Option<PyProperMotion>,
    ) -> PyResult<Self> {
        if let Ok(pos) = position.cast::<PyPosition>() {
            Ok(Self {
                inner: TargetInner::Position(pos.borrow().clone()),
                time: jd,
                proper_motion,
            })
        } else if let Ok(dir) = position.extract::<PyDirection>() {
            Ok(Self {
                inner: TargetInner::Direction(dir),
                time: jd,
                proper_motion,
            })
        } else {
            Err(PyValueError::new_err(
                "Target expects a Position or Direction object",
            ))
        }
    }

    /// Julian Date of the snapshot.
    #[getter]
    fn time(&self) -> f64 {
        self.time
    }

    /// Proper motion of this target (None for static objects).
    #[getter]
    fn proper_motion(&self) -> Option<PyProperMotion> {
        self.proper_motion.clone()
    }

    /// The coordinate as a Position (raises ValueError if this is a Direction target).
    #[getter]
    fn position(&self) -> PyResult<PyPosition> {
        match &self.inner {
            TargetInner::Position(p) => Ok(p.clone()),
            TargetInner::Direction(_) => Err(PyValueError::new_err(
                "This target holds a Direction, not a Position. Use .direction instead.",
            )),
        }
    }

    /// The coordinate as a Direction (raises ValueError if this is a Position target).
    #[getter]
    fn direction(&self) -> PyResult<PyDirection> {
        match &self.inner {
            TargetInner::Direction(d) => Ok(*d),
            TargetInner::Position(_) => Err(PyValueError::new_err(
                "This target holds a Position, not a Direction. Use .position instead.",
            )),
        }
    }

    /// Whether this target holds a Position.
    #[getter]
    fn is_position(&self) -> bool {
        matches!(self.inner, TargetInner::Position(_))
    }

    /// Whether this target holds a Direction.
    #[getter]
    fn is_direction(&self) -> bool {
        matches!(self.inner, TargetInner::Direction(_))
    }

    /// Update the target with a new position/direction and time.
    #[pyo3(signature = (position, jd, proper_motion = None))]
    fn update(
        &mut self,
        position: &Bound<'_, PyAny>,
        jd: f64,
        proper_motion: Option<PyProperMotion>,
    ) -> PyResult<()> {
        if let Ok(pos) = position.cast::<PyPosition>() {
            self.inner = TargetInner::Position(pos.borrow().clone());
            self.time = jd;
            self.proper_motion = proper_motion;
            Ok(())
        } else if let Ok(dir) = position.extract::<PyDirection>() {
            self.inner = TargetInner::Direction(dir);
            self.time = jd;
            self.proper_motion = proper_motion;
            Ok(())
        } else {
            Err(PyValueError::new_err(
                "update() expects a Position or Direction object",
            ))
        }
    }

    fn __repr__(&self) -> String {
        let pm_str = match &self.proper_motion {
            Some(pm) => format!(", proper_motion={}", pm.__repr__()),
            None => String::new(),
        };
        match &self.inner {
            TargetInner::Position(p) => format!(
                "Target(position=Position({:.6}, {:.6}, {:.6}, {}, {}, {}), jd={:.1}{})",
                p.x, p.y, p.z, p.frame, p.center, p.unit, self.time, pm_str
            ),
            TargetInner::Direction(d) => format!(
                "Target(direction=Direction(ra={:.4}°, dec={:.4}°), jd={:.1}{})",
                d.inner.ra(),
                d.inner.dec(),
                self.time,
                pm_str
            ),
        }
    }
}

// =============================================================================
// track() methods — added to Body, Star, Direction
// =============================================================================

/// Track a solar system body at a given Julian Date.
///
/// Returns a Target wrapping the body's natural position:
/// - Planets/Sun: barycentric ecliptic position (AU)
/// - Moon: geocentric ecliptic position (km)
pub(crate) fn track_body(body: &PyBody, jd: f64) -> PyTarget {
    let jd_val = JulianDate::new(jd);
    match body {
        PyBody::Moon => {
            let pos = solar_system::Moon.track(jd_val);
            PyTarget {
                inner: TargetInner::Position(PyPosition::new_internal(
                    pos.x().value(),
                    pos.y().value(),
                    pos.z().value(),
                    FRAME_ECL,
                    CENTER_GEO,
                    UNIT_KM,
                )),
                time: jd,
                proper_motion: None, // Solar system bodies have no proper motion
            }
        }
        _ => {
            let (x, y, z) = track_body_vsop87(body, jd_val);
            PyTarget {
                inner: TargetInner::Position(PyPosition::new_internal(
                    x,
                    y,
                    z,
                    FRAME_ECL,
                    CENTER_BARY,
                    UNIT_AU,
                )),
                time: jd,
                proper_motion: None, // Solar system bodies have no proper motion
            }
        }
    }
}

fn track_body_vsop87(body: &PyBody, jd: JulianDate) -> (f64, f64, f64) {
    macro_rules! vsop {
        ($body:ident) => {{
            let t = solar_system::$body.track(jd);
            (
                t.position.x().value(),
                t.position.y().value(),
                t.position.z().value(),
            )
        }};
    }
    match body {
        PyBody::Sun => vsop!(Sun),
        PyBody::Mercury => vsop!(Mercury),
        PyBody::Venus => vsop!(Venus),
        PyBody::Earth => vsop!(Earth),
        PyBody::Mars => vsop!(Mars),
        PyBody::Jupiter => vsop!(Jupiter),
        PyBody::Saturn => vsop!(Saturn),
        PyBody::Uranus => vsop!(Uranus),
        PyBody::Neptune => vsop!(Neptune),
        PyBody::Moon => unreachable!(),
    }
}

/// Track a star at a given Julian Date — returns an ICRS direction with proper motion.
pub(crate) fn track_star(star: &PyStar, jd: f64) -> PyTarget {
    let dir = star.inner.track(JulianDate::new(jd));

    // Extract proper motion from the star's coordinate if available
    let pm = star
        .inner
        .coordinate
        .get_proper_motion()
        .map(|pm| {
            // Convert from Degrees/Year to MilliArcseconds/Year
            type MasPerYear = Per<MilliArcsecond, Year>;
            let ra_mas = pm.pm_ra.to::<MasPerYear>().value();
            let dec_mas = pm.pm_dec.to::<MasPerYear>().value();
            let mu_alpha_star = pm.ra_convention
                == proper_motion::RaProperMotionConvention::MuAlphaStar;
            PyProperMotion {
                inner: pm.clone(),
                ra_mas,
                dec_mas,
                mu_alpha_star,
            }
        });

    PyTarget {
        inner: TargetInner::Direction(PyDirection { inner: dir }),
        time: jd,
        proper_motion: pm,
    }
}

/// Track a direction at a given Julian Date — returns self (time-invariant).
pub(crate) fn track_direction(dir: &PyDirection, jd: f64) -> PyTarget {
    let tracked = dir.inner.track(JulianDate::new(jd));
    PyTarget {
        inner: TargetInner::Direction(PyDirection { inner: tracked }),
        time: jd,
        proper_motion: None, // Directions don't have proper motion
    }
}

// =============================================================================
// apply_proper_motion
// =============================================================================

/// Apply proper motion to a direction, propagating it from J2000 to a target epoch.
///
/// Args:
///     direction: ICRS direction at J2000.
///     proper_motion: ProperMotion object.
///     jd: Target Julian Date.
///
/// Returns:
///     New Direction with proper motion applied.
#[pyfunction]
pub fn apply_proper_motion(
    direction: &PyDirection,
    pm: &PyProperMotion,
    jd: f64,
) -> PyResult<PyDirection> {
    use siderust::coordinates::{
        centers::Geocentric, frames::EquatorialMeanJ2000, spherical::Position,
    };

    // Convert direction to a spherical position for proper motion application.
    let pos = Position::<Geocentric, EquatorialMeanJ2000, LightYear>::new(
        direction.inner.azimuth,
        direction.inner.polar,
        LightYears::new(1.0),
    );

    let moved =
        proper_motion::set_proper_motion_since_j2000(pos, pm.inner.clone(), JulianDate::new(jd))
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(PyDirection {
        inner: direction::ICRS::new(moved.ra(), moved.dec()),
    })
}
