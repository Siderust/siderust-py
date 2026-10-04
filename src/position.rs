//! Cartesian and spherical position types for Python.
//!
//! Provides `Position` (cartesian) and `SphericalPosition` with frame/center
//! metadata and coordinate transforms, mirroring the siderust type-safe API
//! through runtime dispatch.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use siderust::coordinates::cartesian::Position as CPosition;
use siderust::coordinates::centers::{Barycentric, Geocentric, Heliocentric};
use siderust::coordinates::frames::{
    self, EclipticMeanJ2000, EquatorialMeanJ2000, EquatorialMeanOfDate, EquatorialTrueOfDate,
};
use siderust::coordinates::transform::ext::PositionAstroExt;
use siderust::coordinates::transform::TransformCenter;
use siderust::qtty::*;
use siderust::time::JulianDate;

// =============================================================================
// Frame / Center string constants
// =============================================================================

pub const FRAME_ECL: &str = "EclipticMeanJ2000";
pub const FRAME_EQ: &str = "EquatorialMeanJ2000";
pub const FRAME_ICRS: &str = "ICRS";
pub const FRAME_ICRF: &str = "ICRF";
pub const FRAME_EMOD: &str = "EquatorialMeanOfDate";
pub const FRAME_ETOD: &str = "EquatorialTrueOfDate";

pub const CENTER_BARY: &str = "Barycentric";
pub const CENTER_HELIO: &str = "Heliocentric";
pub const CENTER_GEO: &str = "Geocentric";

pub const UNIT_AU: &str = "au";
pub const UNIT_KM: &str = "km";

const AU_KM: f64 = 149_597_870.700;

type PositionReduceArgs = (f64, f64, f64, String, String, String);

fn validate_frame(f: &str) -> PyResult<()> {
    match f {
        FRAME_ECL | FRAME_EQ | FRAME_ICRS | FRAME_ICRF | FRAME_EMOD | FRAME_ETOD => Ok(()),
        _ => Err(PyValueError::new_err(format!(
            "Unknown frame '{}'. Valid: EclipticMeanJ2000, EquatorialMeanJ2000, ICRS, ICRF, \
             EquatorialMeanOfDate, EquatorialTrueOfDate",
            f
        ))),
    }
}

fn validate_center(c: &str) -> PyResult<()> {
    match c {
        CENTER_BARY | CENTER_HELIO | CENTER_GEO => Ok(()),
        _ => Err(PyValueError::new_err(format!(
            "Unknown center '{}'. Valid: Barycentric, Heliocentric, Geocentric",
            c
        ))),
    }
}

fn validate_unit(u: &str) -> PyResult<()> {
    match u {
        UNIT_AU | UNIT_KM => Ok(()),
        _ => Err(PyValueError::new_err(format!(
            "Unknown unit '{}'. Valid: au, km",
            u
        ))),
    }
}

// =============================================================================
// Frame transform dispatch — rotation is center-independent
// =============================================================================

macro_rules! frame_conv {
    ($x:expr, $y:expr, $z:expr, $jd:expr, $F1:ty => $F2:ty) => {{
        let p = CPosition::<Barycentric, $F1, AstronomicalUnit>::new($x, $y, $z);
        let r: CPosition<Barycentric, $F2, AstronomicalUnit> = p.to_frame(&$jd);
        Ok::<(f64, f64, f64), PyErr>((r.x().value(), r.y().value(), r.z().value()))
    }};
}

macro_rules! to_frame_inner {
    ($x:expr, $y:expr, $z:expr, $jd:expr, $to:expr, $F:ty) => {
        match $to {
            FRAME_ECL => frame_conv!($x, $y, $z, $jd, $F => EclipticMeanJ2000),
            FRAME_EQ  => frame_conv!($x, $y, $z, $jd, $F => EquatorialMeanJ2000),
            FRAME_ICRS => frame_conv!($x, $y, $z, $jd, $F => frames::ICRS),
            FRAME_ICRF => frame_conv!($x, $y, $z, $jd, $F => frames::ICRF),
            FRAME_EMOD => frame_conv!($x, $y, $z, $jd, $F => EquatorialMeanOfDate),
            FRAME_ETOD => frame_conv!($x, $y, $z, $jd, $F => EquatorialTrueOfDate),
            _ => unreachable!(),
        }
    };
}

/// Subset of target frames for ECL source (no direct ECL→EMOD/ETOD provider).
macro_rules! to_frame_from_ecl {
    ($x:expr, $y:expr, $z:expr, $jd:expr, $to:expr) => {
        match $to {
            FRAME_ECL => Ok(($x, $y, $z)),
            FRAME_EQ  => frame_conv!($x, $y, $z, $jd, EclipticMeanJ2000 => EquatorialMeanJ2000),
            FRAME_ICRS => frame_conv!($x, $y, $z, $jd, EclipticMeanJ2000 => frames::ICRS),
            FRAME_ICRF => frame_conv!($x, $y, $z, $jd, EclipticMeanJ2000 => frames::ICRF),
            FRAME_EMOD | FRAME_ETOD => {
                // Chain: ECL → EQ → target
                let (ix, iy, iz) = frame_conv!($x, $y, $z, $jd,
                    EclipticMeanJ2000 => EquatorialMeanJ2000)?;
                convert_frame(ix, iy, iz, FRAME_EQ, $to, $jd)
            },
            _ => unreachable!(),
        }
    };
}

/// Subset of target frames for EMOD/ETOD source (no direct →ECL provider).
macro_rules! to_frame_from_time_dep {
    ($x:expr, $y:expr, $z:expr, $jd:expr, $to:expr, $F:ty) => {
        match $to {
            FRAME_EQ  => frame_conv!($x, $y, $z, $jd, $F => EquatorialMeanJ2000),
            FRAME_ICRS => frame_conv!($x, $y, $z, $jd, $F => frames::ICRS),
            FRAME_ICRF => frame_conv!($x, $y, $z, $jd, $F => frames::ICRF),
            FRAME_EMOD => frame_conv!($x, $y, $z, $jd, $F => EquatorialMeanOfDate),
            FRAME_ETOD => frame_conv!($x, $y, $z, $jd, $F => EquatorialTrueOfDate),
            FRAME_ECL => {
                // Chain: source → EQ → ECL
                let (ix, iy, iz) = frame_conv!($x, $y, $z, $jd,
                    $F => EquatorialMeanJ2000)?;
                convert_frame(ix, iy, iz, FRAME_EQ, FRAME_ECL, $jd)
            },
            _ => unreachable!(),
        }
    };
}

fn convert_frame(
    x: f64,
    y: f64,
    z: f64,
    from: &str,
    to: &str,
    jd: JulianDate,
) -> PyResult<(f64, f64, f64)> {
    if from == to {
        return Ok((x, y, z));
    }
    match from {
        FRAME_ECL => to_frame_from_ecl!(x, y, z, jd, to),
        FRAME_EQ => to_frame_inner!(x, y, z, jd, to, EquatorialMeanJ2000),
        FRAME_ICRS => to_frame_inner!(x, y, z, jd, to, frames::ICRS),
        FRAME_ICRF => to_frame_inner!(x, y, z, jd, to, frames::ICRF),
        FRAME_EMOD => to_frame_from_time_dep!(x, y, z, jd, to, EquatorialMeanOfDate),
        FRAME_ETOD => to_frame_from_time_dep!(x, y, z, jd, to, EquatorialTrueOfDate),
        _ => Err(PyValueError::new_err(format!(
            "Unknown source frame '{}'",
            from
        ))),
    }
}

// =============================================================================
// Center transform dispatch — uses EclipticMeanJ2000 internally, chains
// frame conversion if the position is in another frame.
// =============================================================================

macro_rules! center_conv {
    ($x:expr, $y:expr, $z:expr, $jd:expr, $C1:ty => $C2:ty) => {{
        let p = CPosition::<$C1, EclipticMeanJ2000, AstronomicalUnit>::new($x, $y, $z);
        let r: CPosition<$C2, EclipticMeanJ2000, AstronomicalUnit> = p.to_center($jd);
        Ok((r.x().value(), r.y().value(), r.z().value()))
    }};
}

fn convert_center_ecl(
    x: f64,
    y: f64,
    z: f64,
    from_center: &str,
    to_center: &str,
    jd: JulianDate,
) -> PyResult<(f64, f64, f64)> {
    if from_center == to_center {
        return Ok((x, y, z));
    }
    match (from_center, to_center) {
        (CENTER_BARY, CENTER_HELIO) => center_conv!(x, y, z, jd, Barycentric => Heliocentric),
        (CENTER_BARY, CENTER_GEO) => center_conv!(x, y, z, jd, Barycentric => Geocentric),
        (CENTER_HELIO, CENTER_BARY) => center_conv!(x, y, z, jd, Heliocentric => Barycentric),
        (CENTER_HELIO, CENTER_GEO) => center_conv!(x, y, z, jd, Heliocentric => Geocentric),
        (CENTER_GEO, CENTER_BARY) => center_conv!(x, y, z, jd, Geocentric => Barycentric),
        (CENTER_GEO, CENTER_HELIO) => center_conv!(x, y, z, jd, Geocentric => Heliocentric),
        _ => Err(PyValueError::new_err(format!(
            "Unsupported center conversion: {} -> {}",
            from_center, to_center
        ))),
    }
}

/// Center transform for any frame: converts to ECL, shifts center, converts back.
fn convert_center(
    x: f64,
    y: f64,
    z: f64,
    frame: &str,
    from_center: &str,
    to_center: &str,
    jd: JulianDate,
) -> PyResult<(f64, f64, f64)> {
    if from_center == to_center {
        return Ok((x, y, z));
    }
    // Convert to ECL frame if needed
    let (ecl_x, ecl_y, ecl_z) = convert_frame(x, y, z, frame, FRAME_ECL, jd)?;
    // Apply center shift in ECL
    let (out_x, out_y, out_z) =
        convert_center_ecl(ecl_x, ecl_y, ecl_z, from_center, to_center, jd)?;
    // Convert back to original frame
    convert_frame(out_x, out_y, out_z, FRAME_ECL, frame, jd)
}

// =============================================================================
// PyPosition — Cartesian position with frame/center/unit
// =============================================================================

/// A 3D cartesian position with frame, center, and unit metadata.
///
/// Mirrors Rust's `Position<Center, Frame, Unit>` with runtime dispatch.
///
/// Example:
///     >>> pos = Position(1.0, 0.0, 0.0, frame="EclipticMeanJ2000",
///     ...                center="Heliocentric", unit="au")
///     >>> pos.distance()
///     1.0
#[pyclass(name = "Position", module = "siderust", skip_from_py_object)]
#[derive(Clone)]
pub struct PyPosition {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
    pub(crate) frame: String,
    pub(crate) center: String,
    pub(crate) unit: String,
}

impl PyPosition {
    pub fn new_internal(x: f64, y: f64, z: f64, frame: &str, center: &str, unit: &str) -> Self {
        Self {
            x,
            y,
            z,
            frame: frame.to_string(),
            center: center.to_string(),
            unit: unit.to_string(),
        }
    }

    fn to_au(&self) -> (f64, f64, f64) {
        if self.unit == UNIT_KM {
            (self.x / AU_KM, self.y / AU_KM, self.z / AU_KM)
        } else {
            (self.x, self.y, self.z)
        }
    }

    fn coordinates_from_au(&self, x: f64, y: f64, z: f64, unit: &str) -> (f64, f64, f64) {
        if unit == UNIT_KM {
            (x * AU_KM, y * AU_KM, z * AU_KM)
        } else {
            (x, y, z)
        }
    }
}

#[pymethods]
impl PyPosition {
    /// Create a cartesian position.
    ///
    /// Args:
    ///     x: X coordinate.
    ///     y: Y coordinate.
    ///     z: Z coordinate.
    ///     frame: Reference frame (default: "EclipticMeanJ2000").
    ///     center: Reference center (default: "Heliocentric").
    ///     unit: Length unit, "au" or "km" (default: "au").
    #[new]
    #[pyo3(signature = (x, y, z, frame = "EclipticMeanJ2000", center = "Heliocentric", unit = "au"))]
    fn new(x: f64, y: f64, z: f64, frame: &str, center: &str, unit: &str) -> PyResult<Self> {
        validate_frame(frame)?;
        validate_center(center)?;
        validate_unit(unit)?;
        Ok(Self::new_internal(x, y, z, frame, center, unit))
    }

    // ── Component access ────────────────────────────────────────────

    /// X coordinate.
    #[getter]
    fn x(&self) -> f64 {
        self.x
    }

    /// Y coordinate.
    #[getter]
    fn y(&self) -> f64 {
        self.y
    }

    /// Z coordinate.
    #[getter]
    fn z(&self) -> f64 {
        self.z
    }

    /// Reference frame name.
    #[getter]
    fn frame(&self) -> &str {
        &self.frame
    }

    /// Reference center name.
    #[getter]
    fn center(&self) -> &str {
        &self.center
    }

    /// Length unit ("au" or "km").
    #[getter]
    fn unit(&self) -> &str {
        &self.unit
    }

    // ── Derived quantities ──────────────────────────────────────────

    /// Distance from center (magnitude of position vector).
    fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Euclidean distance to another position (must be same frame/center/unit).
    fn distance_to(&self, other: &PyPosition) -> PyResult<f64> {
        if self.frame != other.frame || self.center != other.center || self.unit != other.unit {
            return Err(PyValueError::new_err(
                "distance_to requires same frame, center, and unit",
            ));
        }
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        Ok((dx * dx + dy * dy + dz * dz).sqrt())
    }

    // ── Transforms ──────────────────────────────────────────────────

    /// Transform to a different reference frame.
    ///
    /// Args:
    ///     target_frame: Target frame name.
    ///     jd: Julian Date for time-dependent transforms.
    ///
    /// Returns:
    ///     New Position in the target frame (same center and unit).
    fn to_frame(&self, target_frame: &str, jd: f64) -> PyResult<PyPosition> {
        validate_frame(target_frame)?;
        let jd = JulianDate::new(jd);
        let (ax, ay, az) = self.to_au();
        let (rx, ry, rz) = convert_frame(ax, ay, az, &self.frame, target_frame, jd)?;
        let (ox, oy, oz) = self.coordinates_from_au(rx, ry, rz, &self.unit);
        Ok(PyPosition::new_internal(
            ox,
            oy,
            oz,
            target_frame,
            &self.center,
            &self.unit,
        ))
    }

    /// Transform to a different reference center.
    ///
    /// Args:
    ///     target_center: Target center name.
    ///     jd: Julian Date for ephemeris lookup.
    ///
    /// Returns:
    ///     New Position with the target center (same frame and unit).
    fn to_center(&self, target_center: &str, jd: f64) -> PyResult<PyPosition> {
        validate_center(target_center)?;
        let jd = JulianDate::new(jd);
        let (ax, ay, az) = self.to_au();
        let (rx, ry, rz) =
            convert_center(ax, ay, az, &self.frame, &self.center, target_center, jd)?;
        let (ox, oy, oz) = self.coordinates_from_au(rx, ry, rz, &self.unit);
        Ok(PyPosition::new_internal(
            ox,
            oy,
            oz,
            &self.frame,
            target_center,
            &self.unit,
        ))
    }

    /// Transform to a different frame and center simultaneously.
    ///
    /// Args:
    ///     target_frame: Target frame name.
    ///     target_center: Target center name.
    ///     jd: Julian Date.
    ///
    /// Returns:
    ///     New Position in the target frame and center.
    fn transform(&self, target_frame: &str, target_center: &str, jd: f64) -> PyResult<PyPosition> {
        validate_frame(target_frame)?;
        validate_center(target_center)?;
        let jd_val = JulianDate::new(jd);
        let (ax, ay, az) = self.to_au();
        // Center shift first (in source frame), then frame rotation
        let (cx, cy, cz) =
            convert_center(ax, ay, az, &self.frame, &self.center, target_center, jd_val)?;
        let (rx, ry, rz) = convert_frame(cx, cy, cz, &self.frame, target_frame, jd_val)?;
        let (ox, oy, oz) = self.coordinates_from_au(rx, ry, rz, &self.unit);
        Ok(PyPosition::new_internal(
            ox,
            oy,
            oz,
            target_frame,
            target_center,
            &self.unit,
        ))
    }

    /// Convert to a different length unit.
    ///
    /// Args:
    ///     unit: "au" or "km".
    fn to_unit(&self, unit: &str) -> PyResult<PyPosition> {
        validate_unit(unit)?;
        if unit == self.unit {
            return Ok(self.clone());
        }
        let (x, y, z) = if self.unit == UNIT_AU && unit == UNIT_KM {
            (self.x * AU_KM, self.y * AU_KM, self.z * AU_KM)
        } else {
            (self.x / AU_KM, self.y / AU_KM, self.z / AU_KM)
        };
        Ok(PyPosition::new_internal(
            x,
            y,
            z,
            &self.frame,
            &self.center,
            unit,
        ))
    }

    /// Convert to spherical coordinates.
    ///
    /// Returns:
    ///     SphericalPosition with same frame, center, and unit.
    fn to_spherical(&self) -> PySphericalPosition {
        let r = self.distance();
        let lat = if r == 0.0 {
            0.0
        } else {
            (self.z / r).asin().to_degrees()
        };
        let lon = self.y.atan2(self.x).to_degrees();
        PySphericalPosition {
            lon_deg: lon,
            lat_deg: lat,
            distance: r,
            frame: self.frame.clone(),
            center: self.center.clone(),
            unit: self.unit.clone(),
        }
    }

    // ── Arithmetic ──────────────────────────────────────────────────

    /// Subtract another position (same frame/center/unit), returning (dx, dy, dz).
    fn __sub__(&self, other: &PyPosition) -> PyResult<(f64, f64, f64)> {
        if self.frame != other.frame || self.center != other.center || self.unit != other.unit {
            return Err(PyValueError::new_err(
                "subtraction requires same frame, center, and unit",
            ));
        }
        Ok((self.x - other.x, self.y - other.y, self.z - other.z))
    }

    // ── Display ─────────────────────────────────────────────────────

    fn __repr__(&self) -> String {
        format!(
            "Position(x={:.6}, y={:.6}, z={:.6}, frame='{}', center='{}', unit='{}')",
            self.x, self.y, self.z, self.frame, self.center, self.unit
        )
    }

    fn __str__(&self) -> String {
        format!(
            "({:+.6}, {:+.6}, {:+.6}) {} {} [{}]",
            self.x, self.y, self.z, self.frame, self.center, self.unit
        )
    }

    fn __eq__(&self, other: &PyPosition) -> bool {
        (self.x - other.x).abs() < 1e-12
            && (self.y - other.y).abs() < 1e-12
            && (self.z - other.z).abs() < 1e-12
            && self.frame == other.frame
            && self.center == other.center
            && self.unit == other.unit
    }

    /// Convert to a dictionary.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let d = pyo3::types::PyDict::new(py);
        d.set_item("x", self.x)?;
        d.set_item("y", self.y)?;
        d.set_item("z", self.z)?;
        d.set_item("frame", &self.frame)?;
        d.set_item("center", &self.center)?;
        d.set_item("unit", &self.unit)?;
        Ok(d)
    }

    /// Create from a dictionary.
    #[staticmethod]
    fn from_dict(d: &Bound<'_, pyo3::types::PyDict>) -> PyResult<Self> {
        let get_f64 = |k: &str| -> PyResult<f64> {
            d.get_item(k)?
                .ok_or_else(|| PyValueError::new_err(format!("missing key '{}'", k)))?
                .extract()
        };
        let get_str = |k: &str| -> PyResult<String> {
            d.get_item(k)?
                .ok_or_else(|| PyValueError::new_err(format!("missing key '{}'", k)))?
                .extract()
        };
        let frame = get_str("frame")?;
        let center = get_str("center")?;
        let unit = get_str("unit")?;
        validate_frame(&frame)?;
        validate_center(&center)?;
        validate_unit(&unit)?;
        Ok(Self::new_internal(
            get_f64("x")?,
            get_f64("y")?,
            get_f64("z")?,
            &frame,
            &center,
            &unit,
        ))
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, PositionReduceArgs)> {
        let cls = py.get_type::<Self>().into_any().unbind();
        Ok((
            cls,
            (
                self.x,
                self.y,
                self.z,
                self.frame.clone(),
                self.center.clone(),
                self.unit.clone(),
            ),
        ))
    }
}

// =============================================================================
// PySphericalPosition
// =============================================================================

/// A spherical position (longitude, latitude, distance) with frame/center/unit.
///
/// Convention: longitude = azimuthal angle, latitude = polar angle from equator.
///
/// Example:
///     >>> sph = SphericalPosition(lon_deg=100.0, lat_deg=0.0, distance=1.0,
///     ...                          frame="EclipticMeanJ2000", center="Heliocentric")
#[pyclass(name = "SphericalPosition", module = "siderust", skip_from_py_object)]
#[derive(Clone)]
pub struct PySphericalPosition {
    pub lon_deg: f64,
    pub lat_deg: f64,
    pub distance: f64,
    pub frame: String,
    pub center: String,
    pub unit: String,
}

#[pymethods]
impl PySphericalPosition {
    /// Create a spherical position.
    ///
    /// Args:
    ///     lon_deg: Longitude (or RA) in degrees.
    ///     lat_deg: Latitude (or Dec) in degrees.
    ///     distance: Radial distance in the specified unit.
    ///     frame: Reference frame (default: "EclipticMeanJ2000").
    ///     center: Reference center (default: "Heliocentric").
    ///     unit: Length unit, "au" or "km" (default: "au").
    #[new]
    #[pyo3(signature = (lon_deg, lat_deg, distance, frame = "EclipticMeanJ2000", center = "Heliocentric", unit = "au"))]
    fn new(
        lon_deg: f64,
        lat_deg: f64,
        distance: f64,
        frame: &str,
        center: &str,
        unit: &str,
    ) -> PyResult<Self> {
        validate_frame(frame)?;
        validate_center(center)?;
        validate_unit(unit)?;
        Ok(Self {
            lon_deg,
            lat_deg,
            distance,
            frame: frame.to_string(),
            center: center.to_string(),
            unit: unit.to_string(),
        })
    }

    /// Longitude (or RA) in degrees.
    #[getter]
    fn lon_deg(&self) -> f64 {
        self.lon_deg
    }

    /// Latitude (or Dec) in degrees.
    #[getter]
    fn lat_deg(&self) -> f64 {
        self.lat_deg
    }

    /// Radial distance.
    #[getter]
    fn distance(&self) -> f64 {
        self.distance
    }

    /// Reference frame name.
    #[getter]
    fn frame(&self) -> &str {
        &self.frame
    }

    /// Reference center name.
    #[getter]
    fn center(&self) -> &str {
        &self.center
    }

    /// Length unit.
    #[getter]
    fn unit(&self) -> &str {
        &self.unit
    }

    /// Angular separation to another spherical position in degrees (Vincenty formula).
    ///
    /// Both positions must be in the same frame.
    fn angular_separation(&self, other: &PySphericalPosition) -> PyResult<f64> {
        if self.frame != other.frame {
            return Err(PyValueError::new_err(
                "angular_separation requires same frame",
            ));
        }
        let lon1 = self.lon_deg.to_radians();
        let lat1 = self.lat_deg.to_radians();
        let lon2 = other.lon_deg.to_radians();
        let lat2 = other.lat_deg.to_radians();
        Ok(vincenty_separation(lon1, lat1, lon2, lat2).to_degrees())
    }

    /// Euclidean 3D distance to another position.
    fn distance_to(&self, other: &PySphericalPosition) -> PyResult<f64> {
        let c1 = self.to_cartesian()?;
        let c2 = other.to_cartesian()?;
        c1.distance_to(&c2)
    }

    /// Convert to cartesian coordinates.
    fn to_cartesian(&self) -> PyResult<PyPosition> {
        let lon = self.lon_deg.to_radians();
        let lat = self.lat_deg.to_radians();
        let x = self.distance * lat.cos() * lon.cos();
        let y = self.distance * lat.cos() * lon.sin();
        let z = self.distance * lat.sin();
        Ok(PyPosition::new_internal(
            x,
            y,
            z,
            &self.frame,
            &self.center,
            &self.unit,
        ))
    }

    fn __repr__(&self) -> String {
        format!(
            "SphericalPosition(lon={:.4}°, lat={:.4}°, r={:.6} {}, frame='{}', center='{}')",
            self.lon_deg, self.lat_deg, self.distance, self.unit, self.frame, self.center
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    /// Convert to a dictionary.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        let d = pyo3::types::PyDict::new(py);
        d.set_item("lon_deg", self.lon_deg)?;
        d.set_item("lat_deg", self.lat_deg)?;
        d.set_item("distance", self.distance)?;
        d.set_item("frame", &self.frame)?;
        d.set_item("center", &self.center)?;
        d.set_item("unit", &self.unit)?;
        Ok(d)
    }

    /// Create from a dictionary.
    #[staticmethod]
    fn from_dict(d: &Bound<'_, pyo3::types::PyDict>) -> PyResult<Self> {
        let get_f64 = |k: &str| -> PyResult<f64> {
            d.get_item(k)?
                .ok_or_else(|| PyValueError::new_err(format!("missing key '{}'", k)))?
                .extract()
        };
        let get_str = |k: &str| -> PyResult<String> {
            d.get_item(k)?
                .ok_or_else(|| PyValueError::new_err(format!("missing key '{}'", k)))?
                .extract()
        };
        let frame = get_str("frame")?;
        let center = get_str("center")?;
        let unit = get_str("unit")?;
        validate_frame(&frame)?;
        validate_center(&center)?;
        validate_unit(&unit)?;
        Ok(Self {
            lon_deg: get_f64("lon_deg")?,
            lat_deg: get_f64("lat_deg")?,
            distance: get_f64("distance")?,
            frame,
            center,
            unit,
        })
    }

    fn __reduce__(&self, py: Python<'_>) -> PyResult<(Py<PyAny>, PositionReduceArgs)> {
        let cls = py.get_type::<Self>().into_any().unbind();
        Ok((
            cls,
            (
                self.lon_deg,
                self.lat_deg,
                self.distance,
                self.frame.clone(),
                self.center.clone(),
                self.unit.clone(),
            ),
        ))
    }
}

// =============================================================================
// Angular separation (Vincenty formula)
// =============================================================================

/// Vincenty great-circle distance formula — numerically stable for all separations.
/// Arguments and return value in radians.
pub(crate) fn vincenty_separation(lon1: f64, lat1: f64, lon2: f64, lat2: f64) -> f64 {
    let dlon = lon2 - lon1;
    let cos_lat1 = lat1.cos();
    let cos_lat2 = lat2.cos();
    let sin_lat1 = lat1.sin();
    let sin_lat2 = lat2.sin();

    let a = (cos_lat2 * dlon.sin()).powi(2)
        + (cos_lat1 * sin_lat2 - sin_lat1 * cos_lat2 * dlon.cos()).powi(2);
    let b = sin_lat1 * sin_lat2 + cos_lat1 * cos_lat2 * dlon.cos();

    a.sqrt().atan2(b)
}
