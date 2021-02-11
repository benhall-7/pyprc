use prc::{open, save, ParamKind};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, Clone, PartialEq)]
struct Param {
    inner: ParamKind,
}

#[pymodule]
fn pyprc(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Param>()?;
    Ok(())
}

macro_rules! make_impl {
    ($(($name:ident, $t:ty)),*) => {

// intentionally unindented

#[pymethods]
impl Param {
    #[new]
    fn new(filename: &str) -> PyResult<Self> {
        let p = open(filename).map(ParamKind::from)?;
        Ok(Param { inner: p })
    }

    $(
        #[staticmethod]
        fn $name(value: $t) -> Self {
            Param { inner: ParamKind::from(value) }
        }
    )*

    fn save(&self, filename: &str) -> PyResult<()> {
        let ps = self
            .inner
            .try_into_ref()
            .map_err(|_| PyValueError::new_err("Expected Struct-type Param"))?;
        save(filename, ps)?;
        Ok(())
    }
}

    };
}

make_impl!(
    (bool, bool),
    (i8, i8),
    (u8, u8),
    (i16, i16),
    (u16, u16),
    (i32, i32),
    (u32, u32),
    (float, f32),
    (str, String)
);
