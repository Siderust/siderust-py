//! Moon phase types and functions for Python.
//!
//! Wraps `siderust::calculus::lunar::phase` for geocentric/topocentric
//! moon phase queries, phase events, and illumination thresholds.

use pyo3::prelude::*;
use siderust::ephemeris::Vsop87Ephemeris;
use siderust::event::lunar::phase as lunar;
use siderust::qtty::*;
use siderust::time::{Interval, JulianDate, ModifiedJulianDate};

use crate::errors::invalid_period_error;
use crate::observer::PyObserver;

/// Moon phase label (8 principal phases).
#[pyclass(
    name = "MoonPhaseLabel",
    module = "siderust",
    eq,
    eq_int,
    hash,
    frozen,
    from_py_object
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PyMoonPhaseLabel {
    NewMoon = 0,
    WaxingCrescent = 1,
    FirstQuarter = 2,
    WaxingGibbous = 3,
    FullMoon = 4,
    WaningGibbous = 5,
    LastQuarter = 6,
    WaningCrescent = 7,
}

impl From<lunar::MoonPhaseLabel> for PyMoonPhaseLabel {
    fn from(l: lunar::MoonPhaseLabel) -> Self {
        match l {
            lunar::MoonPhaseLabel::NewMoon => Self::NewMoon,
            lunar::MoonPhaseLabel::WaxingCrescent => Self::WaxingCrescent,
            lunar::MoonPhaseLabel::FirstQuarter => Self::FirstQuarter,
            lunar::MoonPhaseLabel::WaxingGibbous => Self::WaxingGibbous,
            lunar::MoonPhaseLabel::FullMoon => Self::FullMoon,
            lunar::MoonPhaseLabel::WaningGibbous => Self::WaningGibbous,
            lunar::MoonPhaseLabel::LastQuarter => Self::LastQuarter,
            lunar::MoonPhaseLabel::WaningCrescent => Self::WaningCrescent,
        }
    }
}

#[pymethods]
impl PyMoonPhaseLabel {
    fn __repr__(&self) -> String {
        format!("MoonPhaseLabel.{:?}", self)
    }

    fn __str__(&self) -> &'static str {
        match self {
            Self::NewMoon => "New Moon",
            Self::WaxingCrescent => "Waxing Crescent",
            Self::FirstQuarter => "First Quarter",
            Self::WaxingGibbous => "Waxing Gibbous",
            Self::FullMoon => "Full Moon",
            Self::WaningGibbous => "Waning Gibbous",
            Self::LastQuarter => "Last Quarter",
            Self::WaningCrescent => "Waning Crescent",
        }
    }
}

/// Moon phase geometry at a given instant.
#[pyclass(name = "MoonPhaseGeometry", module = "siderust", from_py_object)]
#[derive(Clone)]
pub struct PyMoonPhaseGeometry {
    /// Phase angle in degrees.
    pub phase_angle_deg: f64,
    /// Illuminated fraction (0.0 – 1.0).
    pub illuminated_fraction: f64,
    /// Elongation in degrees.
    pub elongation_deg: f64,
    /// Whether the Moon is waxing.
    pub waxing: bool,
    /// Human-readable phase label.
    pub label: PyMoonPhaseLabel,
}

impl From<lunar::MoonPhaseGeometry> for PyMoonPhaseGeometry {
    fn from(g: lunar::MoonPhaseGeometry) -> Self {
        Self {
            phase_angle_deg: g.phase_angle.to::<Degree>().value(),
            illuminated_fraction: g.illuminated_fraction.value(),
            elongation_deg: g.elongation.to::<Degree>().value(),
            waxing: g.waxing,
            label: g.label().into(),
        }
    }
}

#[pymethods]
impl PyMoonPhaseGeometry {
    /// Phase angle in degrees.
    #[getter]
    fn phase_angle_deg(&self) -> f64 {
        self.phase_angle_deg
    }

    /// Illuminated fraction (0.0 – 1.0).
    #[getter]
    fn illuminated_fraction(&self) -> f64 {
        self.illuminated_fraction
    }

    /// Elongation in degrees.
    #[getter]
    fn elongation_deg(&self) -> f64 {
        self.elongation_deg
    }

    /// Whether the Moon is waxing.
    #[getter]
    fn waxing(&self) -> bool {
        self.waxing
    }

    /// Phase label.
    #[getter]
    fn label(&self) -> PyMoonPhaseLabel {
        self.label
    }

    fn __repr__(&self) -> String {
        format!(
            "MoonPhaseGeometry(phase={:.1}°, illum={:.1}%, label={:?})",
            self.phase_angle_deg,
            self.illuminated_fraction * 100.0,
            self.label
        )
    }

    fn __str__(&self) -> String {
        format!(
            "{} ({:.1}% illuminated)",
            self.label.__str__(),
            self.illuminated_fraction * 100.0
        )
    }
}

/// Kind of principal phase event.
#[pyclass(
    name = "PhaseKind",
    module = "siderust",
    eq,
    eq_int,
    hash,
    frozen,
    from_py_object
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PyPhaseKind {
    NewMoon = 0,
    FirstQuarter = 1,
    FullMoon = 2,
    LastQuarter = 3,
}

impl From<lunar::PhaseKind> for PyPhaseKind {
    fn from(k: lunar::PhaseKind) -> Self {
        match k {
            lunar::PhaseKind::NewMoon => Self::NewMoon,
            lunar::PhaseKind::FirstQuarter => Self::FirstQuarter,
            lunar::PhaseKind::FullMoon => Self::FullMoon,
            lunar::PhaseKind::LastQuarter => Self::LastQuarter,
        }
    }
}

#[pymethods]
impl PyPhaseKind {
    fn __repr__(&self) -> String {
        format!("PhaseKind.{:?}", self)
    }

    fn __str__(&self) -> &'static str {
        match self {
            Self::NewMoon => "New Moon",
            Self::FirstQuarter => "First Quarter",
            Self::FullMoon => "Full Moon",
            Self::LastQuarter => "Last Quarter",
        }
    }
}

/// A principal moon-phase event.
#[pyclass(name = "PhaseEvent", module = "siderust", from_py_object)]
#[derive(Clone)]
pub struct PyPhaseEvent {
    /// MJD of the event.
    pub mjd: f64,
    /// Kind of phase.
    pub kind: PyPhaseKind,
}

impl From<lunar::PhaseEvent> for PyPhaseEvent {
    fn from(e: lunar::PhaseEvent) -> Self {
        Self {
            mjd: e.mjd.value(),
            kind: e.kind.into(),
        }
    }
}

#[pymethods]
impl PyPhaseEvent {
    /// MJD of the event.
    #[getter]
    fn mjd(&self) -> f64 {
        self.mjd
    }

    /// Kind of phase.
    #[getter]
    fn kind(&self) -> PyPhaseKind {
        self.kind
    }

    fn __repr__(&self) -> String {
        format!("PhaseEvent(mjd={:.6}, kind={:?})", self.mjd, self.kind)
    }

    fn __str__(&self) -> String {
        format!("MJD {:.6}: {}", self.mjd, self.kind.__str__())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Free functions
// ═══════════════════════════════════════════════════════════════════════════

/// Compute Moon phase geometry at a given Julian Date.
///
/// If an observer is provided, computes topocentric phase; otherwise geocentric.
///
/// Args:
///     jd: Julian Date (float).
///     observer: Optional observer for topocentric computation.
///
/// Returns:
///     MoonPhaseGeometry with phase angle, illumination, etc.
#[pyfunction]
#[pyo3(signature = (jd, observer = None))]
pub fn moon_phase(jd: f64, observer: Option<&PyObserver>) -> PyMoonPhaseGeometry {
    let jd = JulianDate::new(jd);
    let geom = match observer {
        Some(obs) => lunar::moon_phase_topocentric::<Vsop87Ephemeris>(jd, obs.inner),
        None => lunar::moon_phase_geocentric::<Vsop87Ephemeris>(jd),
    };
    geom.into()
}

/// Find principal Moon phase events in a time window.
///
/// Args:
///     start_mjd: Start of the search window (MJD).
///     end_mjd: End of the search window (MJD).
///
/// Returns:
///     List of PhaseEvent objects.
#[pyfunction]
pub fn find_moon_phases(start_mjd: f64, end_mjd: f64) -> PyResult<Vec<PyPhaseEvent>> {
    if start_mjd >= end_mjd {
        return Err(invalid_period_error());
    }
    let window = Interval::new(
        ModifiedJulianDate::new(start_mjd),
        ModifiedJulianDate::new(end_mjd),
    );
    let events = lunar::find_phase_events::<Vsop87Ephemeris>(window, Default::default());
    Ok(events.into_iter().map(PyPhaseEvent::from).collect())
}
