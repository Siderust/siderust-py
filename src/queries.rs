//! Altitude and azimuth query functions for Python.
//!
//! These are free functions that accept a body (Body enum), star, or
//! ICRS direction and delegate to the appropriate Rust implementation.

use pyo3::prelude::*;
use qtty::*;
use siderust::{AltitudePeriodsProvider, SearchOpts};
use tempoch::{Interval, ModifiedJulianDate, MJD};

use crate::bodies::{dispatch_body, PyBody};
use crate::coordinates::PyDirection;
use crate::errors::invalid_period_error;
use crate::events::{PyCrossingEvent, PyCulminationEvent};
use crate::observer::PyObserver;
use crate::star::PyStar;

// ═══════════════════════════════════════════════════════════════════════════
// Helper: dispatch to the right provider based on what the user passed
// ═══════════════════════════════════════════════════════════════════════════

/// Enum to unify the different subject types for dispatch.
enum Subject<'a> {
    Body(PyBody),
    Star(&'a PyStar),
    Dir(&'a PyDirection),
}

/// A Python-friendly "subject" that can be a Body, Star, or Direction. 
/// We accept `&Bound<'_, PyAny>` and try to extract each type.
fn extract_subject<'a>(
    target: &'a Bound<'_, PyAny>,
) -> PyResult<Subject<'a>> {
    // Try Body enum first
    if let Ok(body) = target.extract::<PyBody>() {
        return Ok(Subject::Body(body));
    }
    // Try Star
    if let Ok(star) = target.extract::<PyRef<'_, PyStar>>() {
        // We need to clone to avoid lifetime issues
        return Ok(Subject::Star(unsafe { &*(star.as_ptr() as *const PyStar) }));
    }
    // Try Direction
    if let Ok(dir) = target.extract::<PyRef<'_, PyDirection>>() {
        return Ok(Subject::Dir(unsafe { &*(dir.as_ptr() as *const PyDirection) }));
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "target must be a Body, Star, or Direction",
    ))
}

fn make_window(start_mjd: f64, end_mjd: f64) -> PyResult<tempoch::Period<MJD>> {
    if start_mjd >= end_mjd {
        return Err(invalid_period_error());
    }
    Ok(Interval::new(
        ModifiedJulianDate::new(start_mjd),
        ModifiedJulianDate::new(end_mjd),
    ))
}

// ═══════════════════════════════════════════════════════════════════════════
// Public Python functions
// ═══════════════════════════════════════════════════════════════════════════

/// Compute the altitude of a target in degrees.
///
/// Args:
///     target: A Body, Star, or Direction.
///     observer: Observer location.
///     mjd: Modified Julian Date (float).
///
/// Returns:
///     Altitude in degrees.
///
/// Example:
///     >>> altitude_at(Body.Sun, Observer.roque_de_los_muchachos(), 60000.0)
#[pyfunction]
pub fn altitude_at(target: &Bound<'_, PyAny>, observer: &PyObserver, mjd: f64) -> PyResult<f64> {
    let t = ModifiedJulianDate::new(mjd);
    match extract_subject(target)? {
        Subject::Body(body) => Ok(body.altitude_at_inner(&observer.inner, t)),
        Subject::Star(star) => Ok(star
            .inner
            .altitude_at(&observer.inner, t)
            .to::<Degree>()
            .value()),
        Subject::Dir(dir) => Ok(dir
            .inner
            .altitude_at(&observer.inner, t)
            .to::<Degree>()
            .value()),
    }
}

/// Compute the azimuth of a target in degrees.
///
/// Args:
///     target: A Body, Star, or Direction.
///     observer: Observer location.
///     mjd: Modified Julian Date (float).
///
/// Returns:
///     Azimuth in degrees (North = 0, East = 90).
#[pyfunction]
pub fn azimuth_at(target: &Bound<'_, PyAny>, observer: &PyObserver, mjd: f64) -> PyResult<f64> {
    use siderust::AzimuthProvider;
    let t = ModifiedJulianDate::new(mjd);
    match extract_subject(target)? {
        Subject::Body(body) => Ok(body.azimuth_at_inner(&observer.inner, t)),
        Subject::Star(star) => Ok(star
            .inner
            .azimuth_at(&observer.inner, t)
            .to::<Degree>()
            .value()),
        Subject::Dir(dir) => Ok(dir
            .inner
            .azimuth_at(&observer.inner, t)
            .to::<Degree>()
            .value()),
    }
}

/// Find periods when a target is above a threshold altitude.
///
/// Args:
///     target: A Body, Star, or Direction.
///     observer: Observer location.
///     start_mjd: Start of the search window (MJD).
///     end_mjd: End of the search window (MJD).
///     threshold_deg: Altitude threshold in degrees.
///
/// Returns:
///     List of (start_mjd, end_mjd) tuples for above-threshold periods.
#[pyfunction]
pub fn above_threshold(
    target: &Bound<'_, PyAny>,
    observer: &PyObserver,
    start_mjd: f64,
    end_mjd: f64,
    threshold_deg: f64,
) -> PyResult<Vec<(f64, f64)>> {
    let window = make_window(start_mjd, end_mjd)?;
    let threshold = Degrees::new(threshold_deg);
    let opts = SearchOpts::default();

    let periods = match extract_subject(target)? {
        Subject::Body(body) => dispatch_body!(body, |p| {
            siderust::above_threshold(&p, &observer.inner, window, threshold, opts)
        }),
        Subject::Star(star) => {
            siderust::above_threshold(&star.inner, &observer.inner, window, threshold, opts)
        }
        Subject::Dir(dir) => {
            siderust::above_threshold(&dir.inner, &observer.inner, window, threshold, opts)
        }
    };

    Ok(periods
        .into_iter()
        .map(|p| (p.start.value(), p.end.value()))
        .collect())
}

/// Find periods when a target is below a threshold altitude.
///
/// Args:
///     target: A Body, Star, or Direction.
///     observer: Observer location.
///     start_mjd: Start of the search window (MJD).
///     end_mjd: End of the search window (MJD).
///     threshold_deg: Altitude threshold in degrees.
///
/// Returns:
///     List of (start_mjd, end_mjd) tuples for below-threshold periods.
#[pyfunction]
pub fn below_threshold(
    target: &Bound<'_, PyAny>,
    observer: &PyObserver,
    start_mjd: f64,
    end_mjd: f64,
    threshold_deg: f64,
) -> PyResult<Vec<(f64, f64)>> {
    let window = make_window(start_mjd, end_mjd)?;
    let threshold = Degrees::new(threshold_deg);
    let opts = SearchOpts::default();

    let periods = match extract_subject(target)? {
        Subject::Body(body) => dispatch_body!(body, |p| {
            siderust::below_threshold(&p, &observer.inner, window, threshold, opts)
        }),
        Subject::Star(star) => {
            siderust::below_threshold(&star.inner, &observer.inner, window, threshold, opts)
        }
        Subject::Dir(dir) => {
            siderust::below_threshold(&dir.inner, &observer.inner, window, threshold, opts)
        }
    };

    Ok(periods
        .into_iter()
        .map(|p| (p.start.value(), p.end.value()))
        .collect())
}

/// Find threshold-crossing events (rise/set) for a target.
///
/// Args:
///     target: A Body, Star, or Direction.
///     observer: Observer location.
///     start_mjd: Start of the search window (MJD).
///     end_mjd: End of the search window (MJD).
///     threshold_deg: Altitude threshold in degrees.
///
/// Returns:
///     List of CrossingEvent objects.
#[pyfunction]
pub fn crossings(
    target: &Bound<'_, PyAny>,
    observer: &PyObserver,
    start_mjd: f64,
    end_mjd: f64,
    threshold_deg: f64,
) -> PyResult<Vec<PyCrossingEvent>> {
    let window = make_window(start_mjd, end_mjd)?;
    let threshold = Degrees::new(threshold_deg);
    let opts = SearchOpts::default();

    let events = match extract_subject(target)? {
        Subject::Body(body) => dispatch_body!(body, |p| {
            siderust::crossings(&p, &observer.inner, window, threshold, opts)
        }),
        Subject::Star(star) => {
            siderust::crossings(&star.inner, &observer.inner, window, threshold, opts)
        }
        Subject::Dir(dir) => {
            siderust::crossings(&dir.inner, &observer.inner, window, threshold, opts)
        }
    };

    Ok(events.into_iter().map(PyCrossingEvent::from).collect())
}

/// Find culmination events (local altitude extrema) for a target.
///
/// Args:
///     target: A Body, Star, or Direction.
///     observer: Observer location.
///     start_mjd: Start of the search window (MJD).
///     end_mjd: End of the search window (MJD).
///
/// Returns:
///     List of CulminationEvent objects.
#[pyfunction]
pub fn culminations(
    target: &Bound<'_, PyAny>,
    observer: &PyObserver,
    start_mjd: f64,
    end_mjd: f64,
) -> PyResult<Vec<PyCulminationEvent>> {
    let window = make_window(start_mjd, end_mjd)?;
    let opts = SearchOpts::default();

    let events = match extract_subject(target)? {
        Subject::Body(body) => dispatch_body!(body, |p| {
            siderust::culminations(&p, &observer.inner, window, opts)
        }),
        Subject::Star(star) => siderust::culminations(&star.inner, &observer.inner, window, opts),
        Subject::Dir(dir) => siderust::culminations(&dir.inner, &observer.inner, window, opts),
    };

    Ok(events.into_iter().map(PyCulminationEvent::from).collect())
}
