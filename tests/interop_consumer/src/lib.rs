use pyo3::prelude::*;
use siderust_py::interop::{
    direction_from_python, direction_to_python, ensure_bridge_protocol, observer_from_python,
    observer_to_python, DirectionParts, ObserverParts, BRIDGE_PROTOCOL_VERSION,
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
fn direction_parts(value: &Bound<'_, PyAny>) -> PyResult<(f64, f64)> {
    let direction = direction_from_python(value)?;
    let parts = DirectionParts::from(&direction);
    Ok((parts.right_ascension_degrees, parts.declination_degrees))
}

#[pyfunction]
fn direction_round_trip(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    direction_to_python(py, &direction_from_python(value)?)
}

#[pymodule]
fn _siderust_interop_consumer(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(bridge_protocol_version, m)?)?;
    m.add_function(wrap_pyfunction!(observer_parts, m)?)?;
    m.add_function(wrap_pyfunction!(observer_round_trip, m)?)?;
    m.add_function(wrap_pyfunction!(direction_parts, m)?)?;
    m.add_function(wrap_pyfunction!(direction_round_trip, m)?)?;
    Ok(())
}
