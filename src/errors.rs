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
