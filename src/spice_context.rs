//! Immutable SPICE kernel context bindings.
//!
//! Wraps Rust [`siderust::spice::SpiceContext`] behind an [`Arc`] so Python
//! observers can share one owned configuration without dangling borrows or
//! global mutable SPICE state. Kernels are never downloaded automatically.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use pyo3::exceptions::{PyFileNotFoundError, PyOSError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyList;
use siderust::spice::{SpiceContext, SpiceContextError};

/// Owned, immutable SPICE kernel configuration shared by lunar observers.
///
/// Construct with [`Self::from_kernels`] or [`Self::from_de440`]. The context
/// does not mutate after construction; observers hold cloned [`Arc`] handles
/// and remain valid if the original Python variable is deleted.
#[pyclass(name = "SpiceContext", module = "siderust", frozen, from_py_object)]
#[derive(Clone)]
pub struct PySpiceContext {
    pub(crate) inner: Arc<SpiceContext>,
    /// User-facing load aliases in KernelSet provenance order.
    provenance: Vec<String>,
}

impl PySpiceContext {
    pub(crate) fn shared(&self) -> Arc<SpiceContext> {
        Arc::clone(&self.inner)
    }

    pub(crate) fn provenance_aliases(&self) -> &[String] {
        &self.provenance
    }

    fn wrap(context: SpiceContext) -> Self {
        let provenance: Vec<String> = context
            .kernel_set()
            .provenance()
            .into_iter()
            .map(str::to_owned)
            .collect();
        Self {
            inner: Arc::new(context),
            provenance,
        }
    }
}

/// Accept `str` or `pathlib.Path` (and other path-likes exposing `__fspath__`).
fn extract_path(obj: &Bound<'_, PyAny>) -> PyResult<PathBuf> {
    if let Ok(s) = obj.extract::<String>() {
        return Ok(PathBuf::from(s));
    }
    if let Ok(path) = obj.call_method0("__fspath__") {
        if let Ok(s) = path.extract::<String>() {
            return Ok(PathBuf::from(s));
        }
        if let Ok(s) = path.extract::<std::ffi::OsString>() {
            return Ok(PathBuf::from(s));
        }
    }
    Err(PyTypeError::new_err(
        "kernel path must be str or pathlib.Path",
    ))
}

fn extract_optional_path(obj: Option<&Bound<'_, PyAny>>) -> PyResult<Option<PathBuf>> {
    match obj {
        None => Ok(None),
        Some(v) if v.is_none() => Ok(None),
        Some(v) => Ok(Some(extract_path(v)?)),
    }
}

fn map_spice_err(err: SpiceContextError) -> PyErr {
    match err {
        SpiceContextError::Io(io) => {
            if io.kind() == std::io::ErrorKind::NotFound {
                PyFileNotFoundError::new_err(io.to_string())
            } else {
                PyOSError::new_err(format!("SPICE kernel I/O error: {io}"))
            }
        }
        other => PyValueError::new_err(other.to_string()),
    }
}

fn load_required(ctx: &mut SpiceContext, path: &Path, role: &str) -> PyResult<()> {
    if !path.exists() {
        return Err(PyFileNotFoundError::new_err(format!(
            "missing {role} kernel: {}",
            path.display()
        )));
    }
    ctx.load(path).map_err(map_spice_err)
}

fn load_optional(ctx: &mut SpiceContext, path: Option<&Path>, role: &str) -> PyResult<()> {
    if let Some(path) = path {
        load_required(ctx, path, role)?;
    }
    Ok(())
}

/// Standard DE440-compatible filenames used by [`PySpiceContext::from_de440`].
const DE440_SPK: &str = "de440s.bsp";
const DE440_BPC: &str = "moon_pa_de440_200625.bpc";
const DE440_FK: &str = "moon_de440_220930.tf";
const DE440_PCK: &str = "pck00011.tpc";
const DE440_LSK: &str = "naif0012.tls";

#[pymethods]
impl PySpiceContext {
    /// Load an explicit SPK / binary-PCK / FK (+ optional text PCK / LSK) set.
    ///
    /// Args:
    ///     spk: Planetary/satellite SPK (``.bsp``), e.g. ``de440s.bsp``.
    ///     bpc: Lunar binary PCK Type 2 (``.bpc``).
    ///     fk: Lunar frame kernel (``.tf``) defining ``MOON_PA`` / ``MOON_ME``.
    ///     pck: Optional text PCK (physical constants / analytical metadata).
    ///     lsk: Optional leap-second kernel (not required for geometric ET
    ///         observations; useful for SPICE-consumer time metadata).
    ///
    /// Paths may be ``str`` or ``pathlib.Path``. Files are not downloaded.
    /// Filenames containing ``DE440`` do **not** prove mutual compatibility;
    /// the caller must supply a documented matching set.
    ///
    /// Loading order (and therefore NAIF last-file priority) is:
    /// ``spk``, ``bpc``, ``fk``, optional ``pck``, optional ``lsk``.
    #[staticmethod]
    #[pyo3(signature = (*, spk, bpc, fk, pck=None, lsk=None))]
    fn from_kernels(
        spk: &Bound<'_, PyAny>,
        bpc: &Bound<'_, PyAny>,
        fk: &Bound<'_, PyAny>,
        pck: Option<&Bound<'_, PyAny>>,
        lsk: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let spk = extract_path(spk)?;
        let bpc = extract_path(bpc)?;
        let fk = extract_path(fk)?;
        let pck = extract_optional_path(pck)?;
        let lsk = extract_optional_path(lsk)?;

        let mut ctx = SpiceContext::new();
        load_required(&mut ctx, &spk, "SPK")?;
        load_required(&mut ctx, &bpc, "binary PCK")?;
        load_required(&mut ctx, &fk, "FK")?;
        load_optional(&mut ctx, pck.as_deref(), "text PCK")?;
        load_optional(&mut ctx, lsk.as_deref(), "LSK")?;
        Ok(Self::wrap(ctx))
    }

    /// Load kernels from an ordered sequence of filesystem paths.
    ///
    /// Later files have NAIF last-loaded priority within each kernel type.
    /// Accepts ``str`` and ``pathlib.Path``. At least one path is required.
    #[staticmethod]
    #[pyo3(signature = (paths))]
    fn from_paths(paths: &Bound<'_, PyAny>) -> PyResult<Self> {
        let iter = paths.try_iter().map_err(|_| {
            PyTypeError::new_err("paths must be an iterable of str or pathlib.Path")
        })?;
        let mut loaded = 0usize;
        let mut ctx = SpiceContext::new();
        for item in iter {
            let path = extract_path(&item?)?;
            load_required(&mut ctx, &path, "kernel")?;
            loaded += 1;
        }
        if loaded == 0 {
            return Err(PyValueError::new_err(
                "from_paths requires at least one kernel path",
            ));
        }
        Ok(Self::wrap(ctx))
    }

    /// Convenience loader for the documented DE440 kernel directory layout.
    ///
    /// Expects these filenames in ``kernel_dir`` (same set as Siderust's
    /// ``doc/spice_lunar.md`` and Rust example ``24_spice_lunar``):
    ///
    /// - ``de440s.bsp``
    /// - ``moon_pa_de440_200625.bpc``
    /// - ``moon_de440_220930.tf``
    /// - ``pck00011.tpc``
    /// - ``naif0012.tls``
    ///
    /// This selects a *documented* compatible set by filename convention for
    /// that directory; it does not cryptographically authenticate DE releases.
    #[staticmethod]
    #[pyo3(signature = (kernel_dir))]
    fn from_de440(kernel_dir: &Bound<'_, PyAny>) -> PyResult<Self> {
        let dir = extract_path(kernel_dir)?;
        let mut ctx = SpiceContext::new();
        for (role, name) in [
            ("SPK", DE440_SPK),
            ("binary PCK", DE440_BPC),
            ("FK", DE440_FK),
            ("text PCK", DE440_PCK),
            ("LSK", DE440_LSK),
        ] {
            load_required(&mut ctx, &dir.join(name), role)?;
        }
        Ok(Self::wrap(ctx))
    }

    /// Loaded kernel aliases in KernelSet provenance order.
    ///
    /// This lists candidate aliases (with load order within each type), not a
    /// claim that every file contributed to a particular observation segment.
    #[getter]
    fn provenance(&self, py: Python<'_>) -> PyResult<Py<PyList>> {
        let list = PyList::empty(py);
        for alias in &self.provenance {
            list.append(alias)?;
        }
        Ok(list.unbind())
    }

    /// Number of loaded SPK kernels.
    #[getter]
    fn spk_count(&self) -> usize {
        self.inner.kernel_set().spk_count()
    }

    /// Whether a leap-second kernel is loaded.
    #[getter]
    fn has_lsk(&self) -> bool {
        self.inner.kernel_set().has_lsk()
    }

    fn __repr__(&self) -> String {
        format!(
            "SpiceContext(kernels={}, spk_count={}, has_lsk={})",
            self.provenance.len(),
            self.spk_count(),
            self.has_lsk()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    /// Identity equality: same shared Rust context allocation.
    fn __eq__(&self, other: &PySpiceContext) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        Arc::as_ptr(&self.inner).hash(&mut hasher);
        hasher.finish()
    }
}

/// Map Rust SPICE / surface errors into predictable Python exceptions.
pub(crate) fn spice_err(err: SpiceContextError) -> PyErr {
    map_spice_err(err)
}
