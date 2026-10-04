//! Solar system body enum for Python.
//!
//! Maps to the concrete zero-sized body types in `siderust::bodies::solar_system`.
//! Each variant can be used directly for altitude/azimuth queries.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use siderust::bodies::solar_system;
use siderust::coordinates::centers::Geodetic;
use siderust::coordinates::frames::ECEF;
use siderust::qtty::*;
use siderust::time::{JulianDate, ModifiedJulianDate};
use siderust::AltitudeProvider;
use siderust::AzimuthProvider;

use crate::observer::PyObserver;
use crate::position::{
    PyPosition, CENTER_BARY, CENTER_GEO, CENTER_HELIO, FRAME_ECL, UNIT_AU, UNIT_KM,
};

/// A solar system body for observation queries.
///
/// Access bodies as enum variants:
/// >>> Body.Sun
/// >>> Body.Moon
/// >>> Body.Mars
#[pyclass(
    name = "Body",
    module = "siderust",
    eq,
    eq_int,
    hash,
    frozen,
    from_py_object
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PyBody {
    /// The Sun.
    Sun = 0,
    /// Earth's Moon.
    Moon = 1,
    /// Mercury.
    Mercury = 2,
    /// Venus.
    Venus = 3,
    /// Earth (ephemeris only — cannot be observed from itself).
    Earth = 9,
    /// Mars.
    Mars = 4,
    /// Jupiter.
    Jupiter = 5,
    /// Saturn.
    Saturn = 6,
    /// Uranus.
    Uranus = 7,
    /// Neptune.
    Neptune = 8,
}

impl PyBody {
    /// Compute altitude for this body at a given observer and time.
    pub(crate) fn altitude_at_inner(
        &self,
        observer: &Geodetic<ECEF>,
        mjd: ModifiedJulianDate,
    ) -> f64 {
        match self {
            PyBody::Sun => solar_system::Sun
                .altitude_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Moon => solar_system::Moon
                .altitude_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Mercury => solar_system::Mercury
                .altitude_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Venus => solar_system::Venus
                .altitude_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Earth => 0.0, // Cannot observe Earth from Earth
            PyBody::Mars => solar_system::Mars
                .altitude_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Jupiter => solar_system::Jupiter
                .altitude_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Saturn => solar_system::Saturn
                .altitude_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Uranus => solar_system::Uranus
                .altitude_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Neptune => solar_system::Neptune
                .altitude_at(observer, mjd)
                .to::<Degree>()
                .value(),
        }
    }

    /// Compute azimuth for this body at a given observer and time.
    pub(crate) fn azimuth_at_inner(
        &self,
        observer: &Geodetic<ECEF>,
        mjd: ModifiedJulianDate,
    ) -> f64 {
        match self {
            PyBody::Sun => solar_system::Sun
                .azimuth_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Moon => solar_system::Moon
                .azimuth_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Mercury => solar_system::Mercury
                .azimuth_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Venus => solar_system::Venus
                .azimuth_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Earth => 0.0, // Cannot observe Earth from Earth
            PyBody::Mars => solar_system::Mars
                .azimuth_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Jupiter => solar_system::Jupiter
                .azimuth_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Saturn => solar_system::Saturn
                .azimuth_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Uranus => solar_system::Uranus
                .azimuth_at(observer, mjd)
                .to::<Degree>()
                .value(),
            PyBody::Neptune => solar_system::Neptune
                .azimuth_at(observer, mjd)
                .to::<Degree>()
                .value(),
        }
    }
}

/// Macro to dispatch body operations to the concrete Rust type.
/// Returns a Vec or scalar from the callback.
macro_rules! dispatch_body {
    ($body:expr, |$provider:ident| $action:expr) => {
        match $body {
            PyBody::Sun => {
                let $provider = siderust::bodies::solar_system::Sun;
                $action
            }
            PyBody::Moon => {
                let $provider = siderust::bodies::solar_system::Moon;
                $action
            }
            PyBody::Mercury => {
                let $provider = siderust::bodies::solar_system::Mercury;
                $action
            }
            PyBody::Venus => {
                let $provider = siderust::bodies::solar_system::Venus;
                $action
            }
            PyBody::Earth => {
                // Earth has no altitude/azimuth provider; dispatch as Sun
                // (callers that dispatch for observation should guard against Earth)
                let $provider = siderust::bodies::solar_system::Sun;
                $action
            }
            PyBody::Mars => {
                let $provider = siderust::bodies::solar_system::Mars;
                $action
            }
            PyBody::Jupiter => {
                let $provider = siderust::bodies::solar_system::Jupiter;
                $action
            }
            PyBody::Saturn => {
                let $provider = siderust::bodies::solar_system::Saturn;
                $action
            }
            PyBody::Uranus => {
                let $provider = siderust::bodies::solar_system::Uranus;
                $action
            }
            PyBody::Neptune => {
                let $provider = siderust::bodies::solar_system::Neptune;
                $action
            }
        }
    };
}

pub(crate) use dispatch_body;

#[pymethods]
impl PyBody {
    /// Altitude of this body in degrees at the given observer and MJD.
    ///
    /// Args:
    ///     observer: Observer location.
    ///     mjd: Modified Julian Date (float).
    ///
    /// Returns:
    ///     Altitude in degrees.
    fn altitude_at(&self, observer: &PyObserver, mjd: f64) -> f64 {
        self.altitude_at_inner(&observer.inner, ModifiedJulianDate::new(mjd))
    }

    /// Azimuth of this body in degrees at the given observer and MJD.
    ///
    /// Args:
    ///     observer: Observer location.
    ///     mjd: Modified Julian Date (float).
    ///
    /// Returns:
    ///     Azimuth in degrees (North = 0, East = 90).
    fn azimuth_at(&self, observer: &PyObserver, mjd: f64) -> f64 {
        self.azimuth_at_inner(&observer.inner, ModifiedJulianDate::new(mjd))
    }

    /// Heliocentric ecliptic cartesian position at a given Julian Date (VSOP87a).
    ///
    /// Returns a Position in EclipticMeanJ2000 / Heliocentric / AU.
    /// Not available for Moon (use geocentric_position instead).
    fn heliocentric_position(&self, jd: f64) -> PyResult<PyPosition> {
        let jd = JulianDate::new(jd);
        macro_rules! vsop87a {
            ($body:ty) => {{
                let p = <$body>::vsop87a(jd);
                Ok(PyPosition::new_internal(
                    p.x().value(),
                    p.y().value(),
                    p.z().value(),
                    FRAME_ECL,
                    CENTER_HELIO,
                    UNIT_AU,
                ))
            }};
        }
        match self {
            PyBody::Sun => {
                // Sun at heliocentric origin
                Ok(PyPosition::new_internal(
                    0.0,
                    0.0,
                    0.0,
                    FRAME_ECL,
                    CENTER_HELIO,
                    UNIT_AU,
                ))
            }
            PyBody::Moon => Err(PyValueError::new_err(
                "Moon has no heliocentric VSOP87 model. Use Body.Moon.geocentric_position(jd).",
            )),
            PyBody::Mercury => vsop87a!(solar_system::Mercury),
            PyBody::Venus => vsop87a!(solar_system::Venus),
            PyBody::Earth => vsop87a!(solar_system::Earth),
            PyBody::Mars => vsop87a!(solar_system::Mars),
            PyBody::Jupiter => vsop87a!(solar_system::Jupiter),
            PyBody::Saturn => vsop87a!(solar_system::Saturn),
            PyBody::Uranus => vsop87a!(solar_system::Uranus),
            PyBody::Neptune => vsop87a!(solar_system::Neptune),
        }
    }

    /// Barycentric ecliptic cartesian position at a given Julian Date (VSOP87e).
    ///
    /// Returns a Position in EclipticMeanJ2000 / Barycentric / AU.
    fn barycentric_position(&self, jd: f64) -> PyResult<PyPosition> {
        let jd = JulianDate::new(jd);
        macro_rules! vsop87e {
            ($body:ty) => {{
                let p = <$body>::vsop87e(jd);
                Ok(PyPosition::new_internal(
                    p.x().value(),
                    p.y().value(),
                    p.z().value(),
                    FRAME_ECL,
                    CENTER_BARY,
                    UNIT_AU,
                ))
            }};
        }
        match self {
            PyBody::Sun => vsop87e!(solar_system::Sun),
            PyBody::Moon => Err(PyValueError::new_err(
                "Moon has no barycentric VSOP87 model. Use Body.Moon.geocentric_position(jd).",
            )),
            PyBody::Mercury => vsop87e!(solar_system::Mercury),
            PyBody::Venus => vsop87e!(solar_system::Venus),
            PyBody::Earth => vsop87e!(solar_system::Earth),
            PyBody::Mars => vsop87e!(solar_system::Mars),
            PyBody::Jupiter => vsop87e!(solar_system::Jupiter),
            PyBody::Saturn => vsop87e!(solar_system::Saturn),
            PyBody::Uranus => vsop87e!(solar_system::Uranus),
            PyBody::Neptune => vsop87e!(solar_system::Neptune),
        }
    }

    /// Geocentric ecliptic position of the Moon in km.
    ///
    /// Returns a Position in EclipticMeanJ2000 / Geocentric / km.
    /// Only available for the Moon.
    fn geocentric_position(&self, jd: f64) -> PyResult<PyPosition> {
        let jd = JulianDate::new(jd);
        match self {
            PyBody::Moon => {
                let p = solar_system::Moon::get_geo_position::<Kilometer>(jd);
                Ok(PyPosition::new_internal(
                    p.x().value(), p.y().value(), p.z().value(),
                    FRAME_ECL, CENTER_GEO, UNIT_KM,
                ))
            }
            _ => Err(PyValueError::new_err(
                "geocentric_position is only available for Moon. Use heliocentric_position or barycentric_position."
            )),
        }
    }

    /// Track this body at a given Julian Date.
    ///
    /// Returns a Target wrapping the body's natural position:
    /// - Planets/Sun: barycentric ecliptic position (AU)
    /// - Moon: geocentric ecliptic position (km)
    ///
    /// Args:
    ///     jd: Julian Date.
    ///
    /// Returns:
    ///     Target with the body's position at the given epoch.
    fn track(&self, jd: f64) -> crate::target::PyTarget {
        crate::target::track_body(self, jd)
    }

    fn __repr__(&self) -> String {
        format!("Body.{:?}", self)
    }

    fn __str__(&self) -> &'static str {
        match self {
            Self::Sun => "Sun",
            Self::Moon => "Moon",
            Self::Mercury => "Mercury",
            Self::Venus => "Venus",
            Self::Earth => "Earth",
            Self::Mars => "Mars",
            Self::Jupiter => "Jupiter",
            Self::Saturn => "Saturn",
            Self::Uranus => "Uranus",
            Self::Neptune => "Neptune",
        }
    }
}
