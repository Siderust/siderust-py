//! Altitude/azimuth event types for Python.

use pyo3::prelude::*;
use siderust::{CrossingDirection, CrossingEvent, CulminationEvent, CulminationKind};

/// Direction of a threshold crossing.
#[pyclass(
    name = "CrossingDirection",
    module = "siderust",
    eq,
    eq_int,
    hash,
    frozen,
    from_py_object
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PyCrossingDirection {
    /// Body is rising through the threshold.
    Rising = 0,
    /// Body is setting through the threshold.
    Setting = 1,
}

impl From<CrossingDirection> for PyCrossingDirection {
    fn from(d: CrossingDirection) -> Self {
        match d {
            CrossingDirection::Rising => Self::Rising,
            CrossingDirection::Setting => Self::Setting,
        }
    }
}

#[pymethods]
impl PyCrossingDirection {
    fn __repr__(&self) -> String {
        format!("CrossingDirection.{:?}", self)
    }

    fn __str__(&self) -> &'static str {
        match self {
            Self::Rising => "Rising",
            Self::Setting => "Setting",
        }
    }
}

/// A threshold-crossing event (rise or set).
#[pyclass(name = "CrossingEvent", module = "siderust", from_py_object)]
#[derive(Clone)]
pub struct PyCrossingEvent {
    /// MJD of the crossing.
    pub mjd: f64,
    /// Direction (rising or setting).
    pub direction: PyCrossingDirection,
}

impl From<CrossingEvent> for PyCrossingEvent {
    fn from(e: CrossingEvent) -> Self {
        Self {
            mjd: e.mjd.value(),
            direction: e.direction.into(),
        }
    }
}

#[pymethods]
impl PyCrossingEvent {
    /// MJD of the crossing.
    #[getter]
    fn mjd(&self) -> f64 {
        self.mjd
    }

    /// Direction of the crossing (Rising or Setting).
    #[getter]
    fn direction(&self) -> PyCrossingDirection {
        self.direction
    }

    fn __repr__(&self) -> String {
        format!(
            "CrossingEvent(mjd={:.6}, direction={:?})",
            self.mjd, self.direction
        )
    }

    fn __str__(&self) -> String {
        format!("MJD {:.6} {:?}", self.mjd, self.direction)
    }
}

/// Kind of culmination event.
#[pyclass(
    name = "CulminationKind",
    module = "siderust",
    eq,
    eq_int,
    hash,
    frozen,
    from_py_object
)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PyCulminationKind {
    /// Upper culmination (maximum altitude).
    Max = 0,
    /// Lower culmination (minimum altitude).
    Min = 1,
}

impl From<CulminationKind> for PyCulminationKind {
    fn from(k: CulminationKind) -> Self {
        match k {
            CulminationKind::Max => Self::Max,
            CulminationKind::Min => Self::Min,
        }
    }
}

#[pymethods]
impl PyCulminationKind {
    fn __repr__(&self) -> String {
        format!("CulminationKind.{:?}", self)
    }

    fn __str__(&self) -> &'static str {
        match self {
            Self::Max => "Max",
            Self::Min => "Min",
        }
    }
}

/// A culmination event (local altitude extremum).
#[pyclass(name = "CulminationEvent", module = "siderust", from_py_object)]
#[derive(Clone)]
pub struct PyCulminationEvent {
    /// MJD of the culmination.
    pub mjd: f64,
    /// Altitude at culmination in degrees.
    pub altitude_deg: f64,
    /// Kind (Max or Min).
    pub kind: PyCulminationKind,
}

impl From<CulminationEvent> for PyCulminationEvent {
    fn from(e: CulminationEvent) -> Self {
        Self {
            mjd: e.mjd.value(),
            altitude_deg: e.altitude.value(),
            kind: e.kind.into(),
        }
    }
}

#[pymethods]
impl PyCulminationEvent {
    /// MJD of the culmination.
    #[getter]
    fn mjd(&self) -> f64 {
        self.mjd
    }

    /// Altitude at culmination in degrees.
    #[getter]
    fn altitude_deg(&self) -> f64 {
        self.altitude_deg
    }

    /// Kind (Max or Min).
    #[getter]
    fn kind(&self) -> PyCulminationKind {
        self.kind
    }

    fn __repr__(&self) -> String {
        format!(
            "CulminationEvent(mjd={:.6}, alt={:.4}°, kind={:?})",
            self.mjd, self.altitude_deg, self.kind
        )
    }

    fn __str__(&self) -> String {
        format!(
            "MJD {:.6}: {:.4}° ({:?})",
            self.mjd, self.altitude_deg, self.kind
        )
    }
}
