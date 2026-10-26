use pyo3::prelude::*;
use siderust::time::{Period, Time, UTC};
use siderust_py::interop::{
    ensure_bridge_protocol, horizontal_from_python, horizontal_to_python, icrs_from_python,
    icrs_to_python, observer_from_python, observer_to_python, HorizontalParts, IcrsParts,
    ObserverParts, BRIDGE_PROTOCOL_VERSION,
};

#[pyfunction]
fn bridge_protocol_version(py: Python<'_>) -> PyResult<u32> {
    ensure_bridge_protocol(py)?;
    Ok(BRIDGE_PROTOCOL_VERSION)
}

#[pyfunction]
fn observer_parts(value: &Bound<'_, PyAny>) -> PyResult<(f64, f64, f64)> {
    let observer = observer_from_python(value)?;
    let parts = ObserverParts::from(&observer);
    Ok((
        parts.longitude_degrees,
        parts.latitude_degrees,
        parts.height_metres,
    ))
}

#[pyfunction]
fn observer_round_trip(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    observer_to_python(py, &observer_from_python(value)?)
}

#[pyfunction]
fn observer_from_parts(
    py: Python<'_>,
    longitude_degrees: f64,
    latitude_degrees: f64,
    height_metres: f64,
) -> PyResult<Py<PyAny>> {
    let observer = ObserverParts {
        longitude_degrees,
        latitude_degrees,
        height_metres,
    }
    .into_observer();
    observer_to_python(py, &observer)
}

#[pyfunction]
fn icrs_parts(value: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    let direction = icrs_from_python(value)?;
    let parts = IcrsParts::from(&direction);
    Ok((parts.right_ascension_degrees, parts.declination_degrees))
}

#[pyfunction]
fn icrs_round_trip(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    icrs_to_python(py, &icrs_from_python(value)?)
}

#[pyfunction]
fn icrs_from_parts(
    py: Python<'_>,
    right_ascension_degrees: f64,
    declination_degrees: f64,
) -> PyResult<Py<PyAny>> {
    let direction = IcrsParts {
        right_ascension_degrees,
        declination_degrees,
    }
    .into_direction();
    icrs_to_python(py, &direction)
}

#[pyfunction]
fn horizontal_parts(value: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    let direction = horizontal_from_python(value)?;
    let parts = HorizontalParts::from(&direction);
    Ok((parts.altitude_degrees, parts.azimuth_degrees))
}

#[pyfunction]
fn horizontal_round_trip(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    horizontal_to_python(py, &horizontal_from_python(value)?)
}

#[pyfunction]
fn horizontal_from_parts(
    py: Python<'_>,
    altitude_degrees: f64,
    azimuth_degrees: f64,
) -> PyResult<Py<PyAny>> {
    let direction = HorizontalParts {
        altitude_degrees,
        azimuth_degrees,
    }
    .into_direction();
    horizontal_to_python(py, &direction)
}

/// These explicit annotations prove that both bridges use Siderust's actual
/// time types, rather than compiling separate Tempoch/Affn identities.
#[pyfunction]
fn period_round_trip<'py>(
    start: &Bound<'py, PyAny>,
    end: &Bound<'py, PyAny>,
) -> PyResult<(Bound<'py, PyAny>, Bound<'py, PyAny>)> {
    let start_time: Time<UTC> = tempoch_py::interop::datetime_to_time(start)?;
    let end_time: Time<UTC> = tempoch_py::interop::datetime_to_time(end)?;
    let period: Period<UTC> = Period::try_new(start_time, end_time)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))?;
    tempoch_py::interop::period_to_datetimes(start.py(), period)
}

#[pymodule]
fn _siderust_interop_consumer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(bridge_protocol_version, m)?)?;
    m.add_function(wrap_pyfunction!(observer_parts, m)?)?;
    m.add_function(wrap_pyfunction!(observer_round_trip, m)?)?;
    m.add_function(wrap_pyfunction!(observer_from_parts, m)?)?;
    m.add_function(wrap_pyfunction!(icrs_parts, m)?)?;
    m.add_function(wrap_pyfunction!(icrs_round_trip, m)?)?;
    m.add_function(wrap_pyfunction!(icrs_from_parts, m)?)?;
    m.add_function(wrap_pyfunction!(horizontal_parts, m)?)?;
    m.add_function(wrap_pyfunction!(horizontal_round_trip, m)?)?;
    m.add_function(wrap_pyfunction!(horizontal_from_parts, m)?)?;
    m.add_function(wrap_pyfunction!(period_round_trip, m)?)?;
    Ok(())
}
