use prc::hash40::Hash40;
use prc::*;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

#[pyclass(name = "param")]
#[derive(Debug, Clone, PartialEq)]
struct Param {
    inner: ParamKind,
}

#[pyclass(name = "hash")]
#[derive(Debug, Copy, Clone, PartialEq)]
struct Hash {
    inner: Hash40,
}

#[pymodule]
fn pyprc(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Param>()?;
    m.add_class::<Hash>()?;
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

    #[staticmethod]
    fn list(mut value: Vec<Param>) -> Self {
        Param {
            inner: ParamKind::from(ParamList(value
                .drain(..)
                .map(|p| p.inner)
                .collect::<Vec<_>>()
            ))
        }
    }

    #[staticmethod]
    fn r#struct(mut value: Vec<(Hash, Param)>) -> Self {
        Param {
            inner: ParamKind::from(ParamStruct(value
                .drain(..)
                .map(|(h, p)| (h.inner, p.inner))
                .collect::<Vec<_>>()
            ))
        }
    }

    fn save(&self, filename: &str) -> PyResult<()> {
        let ps = self
            .inner
            .try_into_ref()
            .map_err(|_| PyTypeError::new_err("Only struct-type Params can be saved to a file"))?;
        save(filename, ps)?;
        Ok(())
    }

    fn clone(&self) -> Self {
        Param { inner: self.inner.clone() }
    }

    #[getter]
    fn get_value(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ob = match &self.inner {
            ParamKind::Bool(v) => v.to_object(py),
            ParamKind::I8(v) => v.to_object(py),
            ParamKind::U8(v) => v.to_object(py),
            ParamKind::I16(v) => v.to_object(py),
            ParamKind::U16(v) => v.to_object(py),
            ParamKind::I32(v) => v.to_object(py),
            ParamKind::U32(v) => v.to_object(py),
            ParamKind::Float(v) => v.to_object(py),
            ParamKind::Hash(v) => PyCell::new(py, Hash { inner: *v }).unwrap().into(),
            ParamKind::Str(v) => v.to_object(py),
            ParamKind::List(_) => return Err(PyTypeError::new_err("Cannot access value on a list-type param")),
            ParamKind::Struct(_) => return Err(PyTypeError::new_err("Cannot access value on a list-type param")),
        };
        Ok(ob)
    }

    #[setter]
    fn set_value(&mut self, value: PyObject) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        match &mut self.inner {
            ParamKind::Bool(v) => *v = value.extract(py)?,
            ParamKind::I8(v) => *v = value.extract(py)?,
            ParamKind::U8(v) => *v = value.extract(py)?,
            ParamKind::I16(v) => *v = value.extract(py)?,
            ParamKind::U16(v) => *v = value.extract(py)?,
            ParamKind::I32(v) => *v = value.extract(py)?,
            ParamKind::U32(v) => *v = value.extract(py)?,
            ParamKind::Float(v) => *v = value.extract(py)?,
            ParamKind::Hash(v) => *v = value.extract::<Hash>(py)?.inner,
            ParamKind::Str(v) => *v = value.extract(py)?,
            ParamKind::List(_) => return Err(PyTypeError::new_err("Cannot assign value on a list-type param")),
            ParamKind::Struct(_) => return Err(PyTypeError::new_err("Cannot assign value on a list-type param")),
        }
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
    (str, String),
    (hash, Hash)
);

impl From<Hash> for ParamKind {
    fn from(f: Hash) -> Self {
        ParamKind::from(f.inner)
    }
}
