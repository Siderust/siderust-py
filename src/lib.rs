//! Python bindings for siderust astrometry and astrodynamics.
//!
//! This crate provides Python bindings for the siderust Rust library, enabling
//! observation planning, coordinate transforms, altitude/azimuth queries, and
//! ephemeris access from Python — all backed by Rust for performance.

use pyo3::prelude::*;

mod bodies;
mod coordinates;
mod ephemeris;
mod errors;
mod events;
mod observer;
mod orbit;
mod phase;
mod position;
mod queries;
mod star;
mod target;

use bodies::PyBody;
use coordinates::PyDirection;
use ephemeris::PyRuntimeEphemeris;
use events::{PyCrossingDirection, PyCrossingEvent, PyCulminationEvent, PyCulminationKind};
use observer::PyObserver;
use orbit::{PyComet, PyOrbit};
use phase::{PyMoonPhaseGeometry, PyMoonPhaseLabel, PyPhaseEvent, PyPhaseKind};
use position::{PyDisplacement, PyPosition, PySphericalPosition};
use star::PyStar;
use target::{PyProperMotion, PyTarget};

/// siderust: Astrometry & Astrodynamics for Python
///
/// This module provides observation planning, coordinate transforms,
/// altitude/azimuth queries, and ephemeris access, powered by Rust.
///
/// Example:
/// >>> from siderust import Observer, Body
/// >>> obs = Observer.roque_de_los_muchachos()
/// >>> from tempoch import ModifiedJulianDate
/// >>> mjd = ModifiedJulianDate(60000.0)
/// >>> alt = Body.Sun.altitude_at(obs, mjd.value)
/// >>> print(f"Sun altitude: {alt:.4f} deg")
#[pymodule]
fn _siderust(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core types
    m.add_class::<PyObserver>()?;
    m.add_class::<PyBody>()?;
    m.add_class::<PyStar>()?;
    m.add_class::<PyDirection>()?;
    m.add_class::<PyPosition>()?;
    m.add_class::<PyDisplacement>()?;
    m.add_class::<PySphericalPosition>()?;

    // Event types
    m.add_class::<PyCrossingEvent>()?;
    m.add_class::<PyCulminationEvent>()?;
    m.add_class::<PyCrossingDirection>()?;
    m.add_class::<PyCulminationKind>()?;

    // Moon phase types
    m.add_class::<PyMoonPhaseGeometry>()?;
    m.add_class::<PyMoonPhaseLabel>()?;
    m.add_class::<PyPhaseEvent>()?;
    m.add_class::<PyPhaseKind>()?;

    // Orbit / target / ephemeris types
    m.add_class::<PyOrbit>()?;
    m.add_class::<PyComet>()?;
    m.add_class::<PyProperMotion>()?;
    m.add_class::<PyTarget>()?;
    m.add_class::<PyRuntimeEphemeris>()?;

    // Free functions
    m.add_function(wrap_pyfunction!(queries::altitude_at, m)?)?;
    m.add_function(wrap_pyfunction!(queries::above_threshold, m)?)?;
    m.add_function(wrap_pyfunction!(queries::below_threshold, m)?)?;
    m.add_function(wrap_pyfunction!(queries::crossings, m)?)?;
    m.add_function(wrap_pyfunction!(queries::culminations, m)?)?;
    m.add_function(wrap_pyfunction!(queries::azimuth_at, m)?)?;
    m.add_function(wrap_pyfunction!(queries::intersect_periods, m)?)?;
    m.add_function(wrap_pyfunction!(phase::moon_phase, m)?)?;
    m.add_function(wrap_pyfunction!(phase::find_moon_phases, m)?)?;
    m.add_function(wrap_pyfunction!(target::apply_proper_motion, m)?)?;

    // Twilight constants
    m.add("TWILIGHT_HORIZON", queries::TWILIGHT_HORIZON)?;
    m.add("TWILIGHT_CIVIL", queries::TWILIGHT_CIVIL)?;
    m.add("TWILIGHT_NAUTICAL", queries::TWILIGHT_NAUTICAL)?;
    m.add("TWILIGHT_ASTRONOMICAL", queries::TWILIGHT_ASTRONOMICAL)?;

    // Version
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
