//! Cross-extension-safe interoperability with the installed `siderust` package.
//!
//! PyO3 classes are local to the extension module that registers them. A
//! downstream extension must therefore not compile its own copy of
//! `PyObserver`, `PyDirection`, or `PyHorizontalDirection` and expect Python
//! type identity to match. This module imports the installed canonical
//! extension and asks it to extract or construct its own classes. Only
//! primitive scalar values (`f64` payload fields and the `u32` protocol
//! version) cross that boundary. The installed extension must expose the same
//! [`BRIDGE_PROTOCOL_VERSION`].

use pyo3::exceptions::PyImportError;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use siderust::coordinates::centers::Geodetic;
use siderust::coordinates::frames::ECEF;
use siderust::coordinates::spherical::direction;
use siderust::qtty::{Degrees, Meters};

const EXTENSION_MODULE: &str = "siderust._siderust";
const BRIDGE_PROTOCOL_ATTRIBUTE: &str = "_bridge_protocol_version";
const OBSERVER_FROM_PARTS: &str = "_bridge_observer_from_parts";
const OBSERVER_TO_PARTS: &str = "_bridge_observer_to_parts";
const DIRECTION_FROM_PARTS: &str = "_bridge_direction_from_parts";
const DIRECTION_TO_PARTS: &str = "_bridge_direction_to_parts";
const HORIZONTAL_DIRECTION_FROM_PARTS: &str = "_bridge_horizontal_direction_from_parts";
const HORIZONTAL_DIRECTION_TO_PARTS: &str = "_bridge_horizontal_direction_to_parts";

/// Protocol implemented by the canonical Python extension bridge hooks.
///
/// Downstream extensions and the installed `siderust` Python package must use
/// compatible siderust-py releases which expose this same protocol version.
///
/// Protocol 2 adds horizontal-direction bridge hooks while keeping the
/// protocol-1 Observer and ICRS Direction APIs unchanged.
pub const BRIDGE_PROTOCOL_VERSION: u32 = 2;

fn bridge_module<'py>(py: Python<'py>) -> PyResult<Bound<'py, PyModule>> {
    let module = PyModule::import(py, EXTENSION_MODULE)?;
    let actual = module
        .getattr(BRIDGE_PROTOCOL_ATTRIBUTE)
        .and_then(|value| value.extract::<u32>())
        .map_err(|_| {
            PyImportError::new_err(format!(
                "installed {EXTENSION_MODULE} does not expose a valid \
                 {BRIDGE_PROTOCOL_ATTRIBUTE}; expected bridge protocol \
                 {BRIDGE_PROTOCOL_VERSION}. Install a compatible siderust Python package"
            ))
        })?;

    if actual != BRIDGE_PROTOCOL_VERSION {
        return Err(PyImportError::new_err(format!(
            "incompatible siderust bridge protocol: expected \
             {BRIDGE_PROTOCOL_VERSION}, found {actual}. Install matching \
             siderust-py Rust and Python package versions"
        )));
    }

    Ok(module)
}

/// Verify that the installed canonical extension supports this bridge API.
pub fn ensure_bridge_protocol(py: Python<'_>) -> PyResult<()> {
    bridge_module(py).map(|_| ())
}

/// Stable primitive representation of a WGS84 `Geodetic<ECEF>` observer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObserverParts {
    /// East-positive geodetic longitude in degrees.
    pub longitude_degrees: f64,
    /// North-positive geodetic latitude in degrees.
    pub latitude_degrees: f64,
    /// Ellipsoidal height above WGS84 in metres.
    pub height_metres: f64,
}

impl ObserverParts {
    /// Convert these WGS84 parts into Siderust's current observer type.
    pub fn into_observer(self) -> Geodetic<ECEF> {
        Geodetic::<ECEF>::new(
            Degrees::new(self.longitude_degrees),
            Degrees::new(self.latitude_degrees),
            Meters::new(self.height_metres),
        )
    }
}

impl From<&Geodetic<ECEF>> for ObserverParts {
    fn from(observer: &Geodetic<ECEF>) -> Self {
        Self {
            longitude_degrees: observer.lon.value(),
            latitude_degrees: observer.lat.value(),
            height_metres: observer.height.value(),
        }
    }
}

/// Stable primitive representation of an ICRS spherical direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionParts {
    /// Right ascension in degrees in the ICRS frame.
    pub right_ascension_degrees: f64,
    /// Declination in degrees in the ICRS frame.
    pub declination_degrees: f64,
}

impl DirectionParts {
    /// Convert these ICRS parts into Siderust's current direction type.
    pub fn into_direction(self) -> direction::ICRS {
        direction::ICRS::new(
            Degrees::new(self.right_ascension_degrees),
            Degrees::new(self.declination_degrees),
        )
    }
}

impl From<&direction::ICRS> for DirectionParts {
    fn from(value: &direction::ICRS) -> Self {
        Self {
            right_ascension_degrees: value.azimuth.value(),
            declination_degrees: value.polar.value(),
        }
    }
}

/// Extract a canonical Python `siderust.Observer` into primitive parts.
pub fn observer_parts_from_python(value: &Bound<'_, PyAny>) -> PyResult<ObserverParts> {
    let (longitude_degrees, latitude_degrees, height_metres): (f64, f64, f64) =
        bridge_module(value.py())?
            .getattr(OBSERVER_TO_PARTS)?
            .call1((value,))?
            .extract()?;
    Ok(ObserverParts {
        longitude_degrees,
        latitude_degrees,
        height_metres,
    })
}

/// Extract a canonical Python `siderust.Observer` as `Geodetic<ECEF>`.
pub fn observer_from_python(value: &Bound<'_, PyAny>) -> PyResult<Geodetic<ECEF>> {
    Ok(observer_parts_from_python(value)?.into_observer())
}

/// Construct the actual `siderust.Observer` class owned by the installed package.
pub fn observer_to_python(py: Python<'_>, observer: &Geodetic<ECEF>) -> PyResult<Py<PyAny>> {
    let parts = ObserverParts::from(observer);
    bridge_module(py)?
        .getattr(OBSERVER_FROM_PARTS)?
        .call1((
            parts.longitude_degrees,
            parts.latitude_degrees,
            parts.height_metres,
        ))
        .map(Bound::unbind)
}

/// Extract a canonical Python `siderust.Direction` into primitive parts.
pub fn direction_parts_from_python(value: &Bound<'_, PyAny>) -> PyResult<DirectionParts> {
    let (right_ascension_degrees, declination_degrees): (f64, f64) = bridge_module(value.py())?
        .getattr(DIRECTION_TO_PARTS)?
        .call1((value,))?
        .extract()?;
    Ok(DirectionParts {
        right_ascension_degrees,
        declination_degrees,
    })
}

/// Extract a canonical Python `siderust.Direction` as the current ICRS type.
pub fn direction_from_python(value: &Bound<'_, PyAny>) -> PyResult<direction::ICRS> {
    Ok(direction_parts_from_python(value)?.into_direction())
}

/// Construct the actual `siderust.Direction` class owned by the installed package.
pub fn direction_to_python(py: Python<'_>, value: &direction::ICRS) -> PyResult<Py<PyAny>> {
    let parts = DirectionParts::from(value);
    bridge_module(py)?
        .getattr(DIRECTION_FROM_PARTS)?
        .call1((parts.right_ascension_degrees, parts.declination_degrees))
        .map(Bound::unbind)
}

/// Stable primitive representation of a horizontal spherical direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorizontalDirectionParts {
    /// Altitude above the geometric horizon in degrees.
    pub altitude_degrees: f64,
    /// Azimuth in degrees, North-clockwise through East.
    pub azimuth_degrees: f64,
}

impl HorizontalDirectionParts {
    /// Convert these horizontal parts into Siderust's current direction type.
    pub fn into_direction(self) -> direction::Horizontal {
        direction::Horizontal::new(
            Degrees::new(self.altitude_degrees),
            Degrees::new(self.azimuth_degrees),
        )
    }
}

impl From<&direction::Horizontal> for HorizontalDirectionParts {
    fn from(value: &direction::Horizontal) -> Self {
        Self {
            altitude_degrees: value.polar.value(),
            azimuth_degrees: value.azimuth.value(),
        }
    }
}

/// Extract a canonical Python `siderust.HorizontalDirection` into primitive parts.
pub fn horizontal_direction_parts_from_python(
    value: &Bound<'_, PyAny>,
) -> PyResult<HorizontalDirectionParts> {
    let (altitude_degrees, azimuth_degrees): (f64, f64) = bridge_module(value.py())?
        .getattr(HORIZONTAL_DIRECTION_TO_PARTS)?
        .call1((value,))?
        .extract()?;
    Ok(HorizontalDirectionParts {
        altitude_degrees,
        azimuth_degrees,
    })
}

/// Extract a canonical Python `siderust.HorizontalDirection` as the current type.
pub fn horizontal_direction_from_python(
    value: &Bound<'_, PyAny>,
) -> PyResult<direction::Horizontal> {
    Ok(horizontal_direction_parts_from_python(value)?.into_direction())
}

/// Construct the actual `siderust.HorizontalDirection` class owned by the package.
pub fn horizontal_direction_to_python(
    py: Python<'_>,
    value: &direction::Horizontal,
) -> PyResult<Py<PyAny>> {
    let parts = HorizontalDirectionParts::from(value);
    bridge_module(py)?
        .getattr(HORIZONTAL_DIRECTION_FROM_PARTS)?
        .call1((parts.altitude_degrees, parts.azimuth_degrees))
        .map(Bound::unbind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observer_parts_round_trip() {
        let parts = ObserverParts {
            longitude_degrees: -17.8925,
            latitude_degrees: 28.7543,
            height_metres: 2396.0,
        };
        let round_trip = ObserverParts::from(&parts.into_observer());
        assert!((round_trip.longitude_degrees - parts.longitude_degrees).abs() < 1e-12);
        assert!((round_trip.latitude_degrees - parts.latitude_degrees).abs() < 1e-12);
        assert_eq!(round_trip.height_metres, parts.height_metres);
    }

    #[test]
    fn direction_parts_round_trip() {
        let parts = DirectionParts {
            right_ascension_degrees: 83.633,
            declination_degrees: 22.014,
        };
        let round_trip = DirectionParts::from(&parts.into_direction());
        assert!((round_trip.right_ascension_degrees - parts.right_ascension_degrees).abs() < 1e-12);
        assert!((round_trip.declination_degrees - parts.declination_degrees).abs() < 1e-12);
    }

    #[test]
    fn horizontal_direction_parts_round_trip() {
        let parts = HorizontalDirectionParts {
            altitude_degrees: 45.0,
            azimuth_degrees: 180.0,
        };
        let round_trip = HorizontalDirectionParts::from(&parts.into_direction());
        assert!((round_trip.altitude_degrees - parts.altitude_degrees).abs() < 1e-12);
        assert!((round_trip.azimuth_degrees - parts.azimuth_degrees).abs() < 1e-12);
    }

    #[test]
    fn horizontal_direction_canonicalizes_altitude() {
        // Altitude 100° folds to 80° with azimuth shifted by 180°.
        let parts = HorizontalDirectionParts {
            altitude_degrees: 100.0,
            azimuth_degrees: 10.0,
        };
        let direction = parts.into_direction();
        assert!((direction.alt().value() - 80.0).abs() < 1e-12);
        assert!((direction.az().value() - 190.0).abs() < 1e-12);
    }
}
