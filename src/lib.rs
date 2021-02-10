use prc::{open, save, ParamKind};
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
struct Param {
    inner: ParamKind
}

#[pymodule]
fn pyprc(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Param>()?;
    Ok(())
}

#[pymethods]
impl Param {
    #[staticmethod]
    fn from_file(filename: &str) -> PyResult<Self> {
        let p = open(filename).map(ParamKind::from)?;
        Ok(Param { inner: p })
    }

    fn to_file(&self, filename: &str) -> PyResult<()> {
        let ps = self.inner
            .try_into_ref()
            .map_err(|_| PyValueError::new_err("Expected Struct-type Param"))?;
        save(filename, ps)?;
        Ok(())
    }
}

