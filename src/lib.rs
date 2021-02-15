use std::sync::{Arc, Mutex};
use std::vec::IntoIter;
use prc::*;
use prc::hash40::*;
use pyo3::prelude::*;
use pyo3::conversion::ToPyObject;
use pyo3::class::{basic::CompareOp, PyIterProtocol, PyObjectProtocol, PyMappingProtocol};
use pyo3::exceptions::{PyIndexError, PyTypeError};
use pyo3::types::{PyList, PyTuple};

#[pyclass(name = "param")]
#[derive(Debug)]
struct Param {
    inner: Arc<Mutex<ParamType>>,
}

#[derive(Debug, Clone, PartialEq)]
/// an enum designed to store sub-params as reference counted structs to allow python interaction
enum ParamType {
    Bool(bool),
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    Float(f32),
    Hash(Hash),
    Str(String),
    List(ParamList2),
    Struct(ParamStruct2)
}

#[derive(Debug, Clone, PartialEq)]
struct ParamList2(Vec<Param>);

#[derive(Debug, Clone, PartialEq)]
struct ParamStruct2(Vec<(Hash, Param)>);

#[pyclass(name = "hash")]
#[derive(Debug, Copy, Clone, Hash, PartialEq)]
struct Hash {
    inner: Hash40,
}

impl PartialEq for Param {
    fn eq(&self, other: &Self) -> bool {
        *self.inner.lock().unwrap() == *other.inner.lock().unwrap()
    }
}

macro_rules! conversions {
    ($($id:ident),*) => {
        impl From<ParamKind> for ParamType {
            fn from(f: ParamKind) -> Self {
                match f {
                    $(ParamKind::$id(v) => ParamType::$id(v.into())),*
                }
            }
        }
    }
}

conversions!(Bool, I8, U8, I16, U16, I32, U32, Float, Hash, Str, List, Struct);

impl From<ParamKind> for Param {
    fn from(f: ParamKind) -> Self {
        Param { inner:  Arc::new(Mutex::new(f.into())) }
    }
}

impl From<Hash40> for Hash {
    fn from(f: Hash40) -> Self {
        Hash { inner: f }
    }
}

impl From<ParamList> for ParamList2 {
    fn from(mut f: ParamList) -> Self {
        ParamList2(f.0.drain(..).map(From::from).collect())
    }
}

impl From<ParamStruct> for ParamStruct2 {
    fn from(mut f: ParamStruct) -> Self {
        ParamStruct2(f.0.drain(..).map(|(h, p)| (h.into(), p.into())).collect())
    }
}

impl From<&Param> for ParamKind {
    fn from(f: &Param) -> Self {
        let mutex = &*f.inner;
        let param_type = &*mutex.lock().unwrap();
        ParamKind::from(&param_type.clone())
    }
}

impl From<&ParamType> for ParamKind {
    fn from(f: &ParamType) -> Self {
        match f {
            ParamType::Bool(v) => ParamKind::Bool(*v),
            ParamType::I8(v) => ParamKind::I8(*v),
            ParamType::U8(v) => ParamKind::U8(*v),
            ParamType::I16(v) => ParamKind::I16(*v),
            ParamType::U16(v) => ParamKind::U16(*v),
            ParamType::I32(v) => ParamKind::I32(*v),
            ParamType::U32(v) => ParamKind::U32(*v),
            ParamType::Float(v) => ParamKind::Float(*v),
            ParamType::Hash(v) => ParamKind::Hash(v.into()),
            ParamType::Str(v) => ParamKind::Str(v.into()),
            ParamType::List(v) => ParamKind::List(v.into()),
            ParamType::Struct(v) => ParamKind::Struct(v.into()),
        }
    }
}

impl From<&Hash> for Hash40 {
    fn from(f: &Hash) -> Self {
        f.inner
    }
}

impl From<&ParamList2> for ParamList {
    fn from(f: &ParamList2) -> Self {
        ParamList(f.0.iter().map(From::from).collect())
    }
}

impl From<&ParamStruct2> for ParamStruct {
    fn from(f: &ParamStruct2) -> Self {
        ParamStruct(f.0.iter().map(|(h, p)| (h.into(), p.into())).collect())
    }
}

// odd one out here
impl From<Hash> for ParamKind {
    fn from(f: Hash) -> Self {
        ParamKind::from(f.inner)
    }
}

#[pymodule]
fn pyprc(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Param>()?;
    m.add_class::<Hash>()?;
    
    m.add("PARAM_TYPE_BOOL", 1)?;
    m.add("PARAM_TYPE_I8", 2)?;
    m.add("PARAM_TYPE_U8", 3)?;
    m.add("PARAM_TYPE_I16", 4)?;
    m.add("PARAM_TYPE_U16", 5)?;
    m.add("PARAM_TYPE_I32", 6)?;
    m.add("PARAM_TYPE_U32", 7)?;
    m.add("PARAM_TYPE_FLOAT", 8)?;
    m.add("PARAM_TYPE_HASH", 9)?;
    m.add("PARAM_TYPE_STR", 10)?;
    m.add("PARAM_TYPE_LIST", 11)?;
    m.add("PARAM_TYPE_STRUCT", 12)?;
    Ok(())
}

impl Param {
    fn clone_ref(&self) -> Self {
        Param { inner: self.inner.clone() }
    }
}

impl Clone for Param {
    fn clone(&self) -> Self {
        Param { inner: Arc::new(Mutex::new(self.inner.lock().unwrap().clone())) }
    }
}

macro_rules! make_impl {
    ($(($name:ident, $t:ty)),*) => {

// intentionally unindented

#[pymethods]
impl Param {
    #[new]
    fn new(filename: &str) -> PyResult<Self> {
        let p = open(filename).map(ParamKind::from)?;
        Ok(p.into())
    }

    $(
        #[staticmethod]
        fn $name(value: $t) -> Self {
            Param::from(ParamKind::from(value))
        }
    )*

    #[staticmethod]
    fn list(value: Vec<Param>) -> Self {
        Param { inner: Arc::new(Mutex::new(ParamType::List(ParamList2(value)))) }
    }

    #[staticmethod]
    fn r#struct(value: Vec<(Hash, Param)>) -> Self {
        Param { inner: Arc::new(Mutex::new(ParamType::Struct(ParamStruct2(value)))) }
    }

    fn save(&self, filename: &str) -> PyResult<()> {
        if let ParamType::Struct(ps2) = &*self.inner.lock().unwrap() {
            save(filename, &ps2.into())?;
            Ok(())
        } else {
            Err(PyTypeError::new_err("Only struct-type Params can be saved to a file"))
        }
    }

    fn clone(&self) -> Self {
        Param { inner: self.inner.clone() }
    }

    #[getter]
    fn get_type(&self) -> u8 {
        match &*self.inner.lock().unwrap() {
            ParamType::Bool(_) => 1,
            ParamType::I8(_) => 2,
            ParamType::U8(_) => 3,
            ParamType::I16(_) => 4,
            ParamType::U16(_) => 5,
            ParamType::I32(_) => 6,
            ParamType::U32(_) => 7,
            ParamType::Float(_) => 8,
            ParamType::Hash(_) => 9,
            ParamType::Str(_) => 10,
            ParamType::List(_) => 11,
            ParamType::Struct(_) => 12,
        }
    }

    #[getter]
    fn get_value(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let ob = match &*self.inner.lock().unwrap() {
            ParamType::Bool(v) => v.to_object(py),
            ParamType::I8(v) => v.to_object(py),
            ParamType::U8(v) => v.to_object(py),
            ParamType::I16(v) => v.to_object(py),
            ParamType::U16(v) => v.to_object(py),
            ParamType::I32(v) => v.to_object(py),
            ParamType::U32(v) => v.to_object(py),
            ParamType::Float(v) => v.to_object(py),
            ParamType::Hash(v) => PyCell::new(py, *v).unwrap().into(),
            ParamType::Str(v) => v.to_object(py),
            ParamType::List(_) => return Err(PyTypeError::new_err("Cannot access value on a list-type param")),
            ParamType::Struct(_) => return Err(PyTypeError::new_err("Cannot access value on a list-type param")),
        };
        Ok(ob)
    }

    #[setter]
    fn set_value(&mut self, value: PyObject) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        match &mut *self.inner.lock().unwrap() {
            ParamType::Bool(v) => *v = value.extract(py)?,
            ParamType::I8(v) => *v = value.extract(py)?,
            ParamType::U8(v) => *v = value.extract(py)?,
            ParamType::I16(v) => *v = value.extract(py)?,
            ParamType::U16(v) => *v = value.extract(py)?,
            ParamType::I32(v) => *v = value.extract(py)?,
            ParamType::U32(v) => *v = value.extract(py)?,
            ParamType::Float(v) => *v = value.extract(py)?,
            ParamType::Hash(v) => *v = value.extract(py)?,
            ParamType::Str(v) => *v = value.extract(py)?,
            ParamType::List(_) => return Err(PyTypeError::new_err("Cannot assign value on a list-type param")),
            ParamType::Struct(_) => return Err(PyTypeError::new_err("Cannot assign value on a list-type param")),
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

#[pyproto]
impl<'a> PyMappingProtocol<'a> for Param {
    fn __getitem__(&self, key: PyObject) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        match &*self.inner.lock().unwrap() {
            ParamType::List(v) => {
                let index: usize = key.extract(py)?;
                if index >= v.0.len() {
                    Err(PyIndexError::new_err("Index out of bounds").into())
                } else {
                    Ok(PyCell::new(py, Param { inner: v.0[index].inner.clone() })?.into())
                }
            }
            ParamType::Struct(v) => {
                let index: Hash = key.extract(py)?;
                let mut col: Vec<PyObject> = v.0.iter()
                    .filter(|(hash, _)| *hash == index)
                    .map(|(_, p)| {
                        PyCell::new(py, Param { inner: p.inner.clone() })
                    })
                    .collect::<Result<Vec<_>, _>>()?
                    .drain(..)
                    .map(From::from)
                    .collect();
                if col.is_empty() {
                    Err(PyIndexError::new_err("Hash not found in child params"))
                } else if col.len() == 1 {
                    Ok(col.remove(0))
                } else {
                    Ok(PyList::new(py, col).into())
                }
            }
            _ => Err(PyTypeError::new_err("Cannot index params other than list or struct-type params")),
        }
    }

    fn __setitem__(&mut self, key: PyObject, value: PyObject) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        match &mut *self.inner.lock().unwrap() {
            ParamType::List(v) => {
                let index: usize = key.extract(py)?;
                if index >= v.0.len() { 
                    Err(PyIndexError::new_err("Index out of bounds").into())
                } else {
                    let set: Param = value.extract(py)?;
                    v.0[index] = set;
                    Ok(())
                }
            }
            ParamType::Struct(v) => {
                let index: Hash = key.extract(py)?;
                let mut col: Vec<&mut Param> = v.0.iter_mut()
                    .filter(|(hash, _)| *hash == index)
                    .map(|(_, p)| p)
                    .collect();
                if col.is_empty() {
                    Err(PyIndexError::new_err("Hash not found in child params"))
                } else if col.len() == 1 {
                    let set: Param = value.extract(py)?;
                    *col[0] = set;
                    Ok(())
                } else {
                    Err(PyTypeError::new_err("Cannot assign param to this hash; more than one match was found"))
                }
            }
            _ => Err(PyTypeError::new_err("Cannot index params other than list or struct-type params")),
        }
    }
}

#[pymethods]
impl Hash {
    #[new]    
    fn new(value: PyObject) -> PyResult<Hash> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        if let Ok(v) = value.extract::<&str>(py) {
            Ok(Hash { inner: to_hash40(v) })
        } else if let Ok(v) = value.extract::<u64>(py) {
            Ok(Hash { inner: Hash40(v) })
        } else {
            Err(PyTypeError::new_err("Hash constructor accepts only string or unsigned int64"))
        }
    }

    #[staticmethod]
    fn load_labels(filepath: &str) -> PyResult<()> {
        let labels = read_custom_labels(filepath)?;
        set_custom_labels(labels.into_iter());
        Ok(())
    }
}

#[pyclass]
struct ParamIter {
    inner: IntoIter<PyObject>
}

#[pyproto]
impl<'a> PyIterProtocol<'a> for Param {
    fn __iter__(this: PyRef<Self>) -> PyResult<Py<ParamIter>> {
        let py = this.py();
        match &*this.inner.lock().unwrap() {
            ParamType::List(v) => {
                let refs: IntoIter<PyObject> = v.0
                    .iter()
                    .map(|p| PyCell::new(py, Param { inner: p.inner.clone() }).unwrap().into())
                    .collect::<Vec<_>>()
                    .into_iter();
                Py::new(py, ParamIter { inner: refs })
            }
            ParamType::Struct(v) => {
                let refs: IntoIter<PyObject> = v.0
                    .iter()
                    .map(|(h, p)| [PyCell::new(py, *h).unwrap().into(), PyCell::new(py, p.clone_ref()).unwrap().into()])
                    .map(|arr2: [PyObject; 2]| PyTuple::new(py, arr2.iter()).to_object(py))
                    .collect::<Vec<_>>()
                    .into_iter();
                Py::new(py, ParamIter { inner: refs })
            }
            _ => Err(PyTypeError::new_err("Cannot iterate params other than list or struct-type params"))
        }
    }
}

#[pyproto]
impl PyIterProtocol for ParamIter {
    fn __next__(mut this: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
        Ok(this.inner.next())
    }
}

#[pyproto]
impl<'a> PyObjectProtocol<'a> for Param {
    fn __str__(&self) -> String {
        match &*self.inner.lock().unwrap() {
            ParamType::Bool(v) => format!("param bool ({})", v),
            ParamType::I8(v) => format!("param i8 ({})", v),
            ParamType::U8(v) => format!("param u8 ({})", v),
            ParamType::I16(v) => format!("param i16 ({})", v),
            ParamType::U16(v) => format!("param u16 ({})", v),
            ParamType::I32(v) => format!("param i32 ({})", v),
            ParamType::U32(v) => format!("param u32 ({})", v),
            ParamType::Float(v) => format!("param float ({})", v),
            ParamType::Hash(v) => format!("param hash ({})", v.inner),
            ParamType::Str(v) => format!("param str ({})", v),
            ParamType::List(v) => format!("param list (len = {})", v.0.len()),
            ParamType::Struct(v) => format!("param struct (len = {})", v.0.len()),
        }
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    fn __richcmp__(&self, other: PyRef<Self>, co: CompareOp) -> PyResult<bool> {
        match co {
            CompareOp::Eq => Ok(self == &*other),
            CompareOp::Ne => Ok(self != &*other),
            _ => Err(PyTypeError::new_err("Only == or != comparisons valid for hash"))
        }
    }
}

#[pyproto]
impl<'a> PyObjectProtocol<'a> for Hash {
    fn __str__(&self) -> String {
        // utilizes the global static labels for Hash40s
        format!("hash ({})", self.inner)
    }

    fn __repr__(&self) -> String {
        self.__str__()
    }

    fn __hash__(&self) -> u64 {
        self.inner.0
    }

    fn __richcmp__(&self, other: PyRef<Self>, co: CompareOp) -> PyResult<bool> {
        match co {
            CompareOp::Eq => Ok(self == &*other),
            CompareOp::Ne => Ok(self != &*other),
            _ => Err(PyTypeError::new_err("Only == or != comparisons valid for hash"))
        }
    }
}

