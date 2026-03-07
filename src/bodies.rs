//! Solar system body enum for Python.
//!
//! Maps to the concrete zero-sized body types in `siderust::bodies::solar_system`.
//! Each variant can be used directly for altitude/azimuth queries.

use pyo3::prelude::*;
use qtty::*;
use siderust::bodies::solar_system;
use siderust::coordinates::centers::Geodetic;
use siderust::coordinates::frames::ECEF;
use siderust::AltitudePeriodsProvider;
use siderust::AzimuthProvider;
use tempoch::ModifiedJulianDate;

use crate::observer::PyObserver;

/// A solar system body for observation queries.
///
/// Access bodies as enum variants:
/// >>> Body.Sun
/// >>> Body.Moon
/// >>> Body.Mars
#[pyclass(name = "Body", module = "siderust", eq, eq_int, hash, frozen, from_py_object)]
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
            PyBody::Sun => solar_system::Sun.altitude_at(observer, mjd).to::<Degree>().value(),
            PyBody::Moon => solar_system::Moon.altitude_at(observer, mjd).to::<Degree>().value(),
            PyBody::Mercury => {
                solar_system::Mercury
                    .altitude_at(observer, mjd)
                    .to::<Degree>()
                    .value()
            }
            PyBody::Venus => {
                solar_system::Venus
                    .altitude_at(observer, mjd)
                    .to::<Degree>()
                    .value()
            }
            PyBody::Mars => solar_system::Mars.altitude_at(observer, mjd).to::<Degree>().value(),
            PyBody::Jupiter => {
                solar_system::Jupiter
                    .altitude_at(observer, mjd)
                    .to::<Degree>()
                    .value()
            }
            PyBody::Saturn => {
                solar_system::Saturn
                    .altitude_at(observer, mjd)
                    .to::<Degree>()
                    .value()
            }
            PyBody::Uranus => {
                solar_system::Uranus
                    .altitude_at(observer, mjd)
                    .to::<Degree>()
                    .value()
            }
            PyBody::Neptune => {
                solar_system::Neptune
                    .altitude_at(observer, mjd)
                    .to::<Degree>()
                    .value()
            }
        }
    }

    /// Compute azimuth for this body at a given observer and time.
    pub(crate) fn azimuth_at_inner(
        &self,
        observer: &Geodetic<ECEF>,
        mjd: ModifiedJulianDate,
    ) -> f64 {
        match self {
            PyBody::Sun => solar_system::Sun.azimuth_at(observer, mjd).to::<Degree>().value(),
            PyBody::Moon => solar_system::Moon.azimuth_at(observer, mjd).to::<Degree>().value(),
            PyBody::Mercury => {
                solar_system::Mercury
                    .azimuth_at(observer, mjd)
                    .to::<Degree>()
                    .value()
            }
            PyBody::Venus => {
                solar_system::Venus
                    .azimuth_at(observer, mjd)
                    .to::<Degree>()
                    .value()
            }
            PyBody::Mars => solar_system::Mars.azimuth_at(observer, mjd).to::<Degree>().value(),
            PyBody::Jupiter => {
                solar_system::Jupiter
                    .azimuth_at(observer, mjd)
                    .to::<Degree>()
                    .value()
            }
            PyBody::Saturn => {
                solar_system::Saturn
                    .azimuth_at(observer, mjd)
                    .to::<Degree>()
                    .value()
            }
            PyBody::Uranus => {
                solar_system::Uranus
                    .azimuth_at(observer, mjd)
                    .to::<Degree>()
                    .value()
            }
            PyBody::Neptune => {
                solar_system::Neptune
                    .azimuth_at(observer, mjd)
                    .to::<Degree>()
                    .value()
            }
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

    fn __repr__(&self) -> String {
        format!("Body.{:?}", self)
    }

    fn __str__(&self) -> &'static str {
        match self {
            Self::Sun => "Sun",
            Self::Moon => "Moon",
            Self::Mercury => "Mercury",
            Self::Venus => "Venus",
            Self::Mars => "Mars",
            Self::Jupiter => "Jupiter",
            Self::Saturn => "Saturn",
            Self::Uranus => "Uranus",
            Self::Neptune => "Neptune",
        }
    }
}
