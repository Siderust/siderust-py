//! Error handling utilities for siderust Python bindings.

use pyo3::exceptions::*;
use pyo3::prelude::*;

/// Creates an error for invalid body names.
#[allow(dead_code)]
pub fn invalid_body_error(name: &str) -> PyErr {
    PyValueError::new_err(format!(
        "Unknown body '{}'. Valid bodies: Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune",
        name
    ))
}

/// Creates an error for invalid star names.
pub fn unknown_star_error(name: &str) -> PyErr {
    PyValueError::new_err(format!("Unknown star '{}'", name))
}

/// Creates an error for invalid time windows.
pub fn invalid_period_error() -> PyErr {
    PyValueError::new_err("Invalid time window: start must be before end")
}

/// Extract an MJD value from either a raw float or an object with `.value` attribute.
///
/// Accepts:
/// - Plain f64 value
/// - Any object with a `.value` attribute (like tempoch.ModifiedJulianDate)
pub fn extract_mjd(time: &Bound<'_, PyAny>) -> PyResult<f64> {
    // Try raw float first (most common case)
    if let Ok(v) = time.extract::<f64>() {
        return Ok(v);
    }
    // Try object with `.value` attribute (tempoch.ModifiedJulianDate, JulianDate, etc.)
    if let Ok(value_attr) = time.getattr("value") {
        if let Ok(v) = value_attr.extract::<f64>() {
            return Ok(v);
        }
    }
    Err(PyTypeError::new_err(
        "time must be a float or an object with a .value attribute (e.g., tempoch.ModifiedJulianDate)",
    ))
}
