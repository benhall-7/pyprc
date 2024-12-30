use prc::hash40::*;
use prc::*;
use pyo3::class::basic::CompareOp;
use pyo3::conversion::IntoPy;
use pyo3::exceptions::{PyIndexError, PyLookupError, PyTypeError};
use pyo3::prelude::*;
use std::sync::{Arc, Mutex};
use std::vec::IntoIter;

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
    Struct(ParamStruct2),
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
        // if the two objects being compared are the same object, locking both will deadlock
        Arc::ptr_eq(&self.inner, &other.inner)
            || *self.inner.lock().unwrap() == *other.inner.lock().unwrap()
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
        Param {
            inner: Arc::new(Mutex::new(f.into())),
        }
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
fn pyprc(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
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
        Param {
            inner: self.inner.clone(),
        }
    }
}

impl Clone for Param {
    fn clone(&self) -> Self {
        Param {
            inner: Arc::new(Mutex::new(self.inner.lock().unwrap().clone())),
        }
    }
}

macro_rules! make_impl {
    ($(($name:ident, $set_name:ident, $t:ty)),*) => {

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

        fn $set_name(&mut self, value: $t) {
            *self.inner.lock().unwrap() = ParamKind::from(value).into()
        }
    )*

    #[staticmethod]
    fn list(mut value: Vec<PyRef<Self>>) -> Self {
        Param {
            inner: Arc::new(Mutex::new(ParamType::List(ParamList2(value
                .drain(..)
                .map(|p| p.clone_ref())
                .collect()
            ))))
        }
    }

    fn set_list(&mut self, mut value: Vec<PyRef<Self>>) {
        *self.inner.lock().unwrap() = ParamType::List(ParamList2(value
            .drain(..)
            .map(|p| p.clone_ref())
            .collect()
        ))
    }

    #[staticmethod]
    fn r#struct(mut value: Vec<(Hash, PyRef<Self>)>) -> Self {
        Param {
            inner: Arc::new(Mutex::new(ParamType::Struct(ParamStruct2(value
                .drain(..)
                .map(|(h, p)| (h, p.clone_ref()))
                .collect()
            ))))
        }
    }

    fn set_struct(&mut self, mut value: Vec<(Hash, PyRef<Self>)>) {
        *self.inner.lock().unwrap() = ParamType::Struct(ParamStruct2(value
            .drain(..)
            .map(|(h, p)| (h, p.clone_ref()))
            .collect()
        ))
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
        Clone::clone(self)
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
    fn get_value(&self, py: Python) -> PyResult<PyObject> {
        let ob = match &*self.inner.lock().unwrap() {
            ParamType::Bool(v) => v.into_py(py),
            ParamType::I8(v) => v.into_py(py),
            ParamType::U8(v) => v.into_py(py),
            ParamType::I16(v) => v.into_py(py),
            ParamType::U16(v) => v.into_py(py),
            ParamType::I32(v) => v.into_py(py),
            ParamType::U32(v) => v.into_py(py),
            ParamType::Float(v) => v.into_py(py),
            ParamType::Hash(v) => v.into_py(py),
            ParamType::Str(v) => v.into_py(py),
            ParamType::List(_) => return Err(PyTypeError::new_err("Cannot access value on a list-type param")),
            ParamType::Struct(_) => return Err(PyTypeError::new_err("Cannot access value on a list-type param")),
        };
        Ok(ob)
    }

    #[setter]
    fn set_value(&mut self, py: Python, value: PyObject) -> PyResult<()> {
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
    (bool, set_bool, bool),
    (i8, set_i8, i8),
    (u8, set_u8, u8),
    (i16, set_i16, i16),
    (u16, set_u16, u16),
    (i32, set_i32, i32),
    (u32, set_u32, u32),
    (float, set_float, f32),
    (str, set_str, String),
    (hash, set_hash, Hash)
);

#[pymethods]
impl Param {
    fn __len__(&self) -> PyResult<usize> {
        match &*self.inner.lock().unwrap() {
            ParamType::List(v) => Ok(v.0.len()),
            ParamType::Struct(v) => Ok(v.0.len()),
            _ => Err(PyTypeError::new_err(
                "Cannot get length for params other than list or struct-type params",
            )),
        }
    }

    fn __getitem__(&self, py: Python, key: PyObject) -> PyResult<PyObject> {
        match &*self.inner.lock().unwrap() {
            ParamType::List(v) => {
                let index: usize = key.extract(py)?;
                if index >= v.0.len() {
                    Err(PyIndexError::new_err("Index out of bounds"))
                } else {
                    Ok(v.0[index].clone_ref().into_py(py))
                }
            }
            ParamType::Struct(v) => {
                let index: Hash = key.extract(py)?;
                let mut col: Vec<Param> =
                    v.0.iter()
                        .filter(|(hash, _)| *hash == index)
                        .map(|(_, p)| p.clone_ref())
                        .collect();
                if col.is_empty() {
                    Err(PyIndexError::new_err("Hash not found in child params"))
                } else if col.len() == 1 {
                    Ok(col.remove(0).into_py(py))
                } else {
                    Ok(col.into_py(py))
                }
            }
            _ => Err(PyTypeError::new_err(
                "Cannot index params other than list or struct-type params",
            )),
        }
    }

    fn __setitem__(&mut self, py: Python, key: PyObject, value: PyObject) -> PyResult<()> {
        match &mut *self.inner.lock().unwrap() {
            ParamType::List(v) => {
                let index: usize = key.extract(py)?;
                if index >= v.0.len() {
                    Err(PyIndexError::new_err("Index out of bounds"))
                } else {
                    let set: Param = value.extract(py)?;
                    v.0[index] = set;
                    Ok(())
                }
            }
            ParamType::Struct(v) => {
                let index: Hash = key.extract(py)?;
                let mut col: Vec<&mut Param> =
                    v.0.iter_mut()
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
                    Err(PyTypeError::new_err(
                        "Cannot assign param to this hash; more than one match was found",
                    ))
                }
            }
            _ => Err(PyTypeError::new_err(
                "Cannot index params other than list or struct-type params",
            )),
        }
    }

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
            _ => Err(PyTypeError::new_err(
                "Only == or != comparisons valid for param",
            )),
        }
    }
}

#[pymethods]
impl Hash {
    #[new]
    fn new(py: Python, value: PyObject) -> PyResult<Hash> {
        if let Ok(v) = value.extract::<&str>(py) {
            let labels = Hash40::label_map();
            let lock = labels.lock().unwrap();
            lock.hash_of(v).map(|hash| hash.into()).ok_or_else(|| {
                PyLookupError::new_err(
                    "Could not convert this string into a hash. The label map does not contain the string, and is using strict conversion"
                )
            })
        } else if let Ok(v) = value.extract::<u64>(py) {
            Ok(Hash { inner: Hash40(v) })
        } else {
            Err(PyTypeError::new_err(
                "Hash constructor accepts only string or unsigned int64",
            ))
        }
    }

    #[staticmethod]
    fn algo(string: &str) -> Hash {
        hash40(string).into()
    }

    #[staticmethod]
    fn load_labels(filepath: &str) -> PyResult<()> {
        let label_map = Hash40::label_map();
        let mut labels = label_map.lock().unwrap();
        labels.add_custom_labels_from_path(filepath).unwrap();
        Ok(())
    }

    #[staticmethod]
    fn set_strict(strict: bool) {
        let label_map = Hash40::label_map();
        let mut lock = label_map.lock().unwrap();
        lock.strict = strict;
    }

    #[getter]
    fn get_value(&self) -> u64 {
        self.inner.0
    }

    fn __str__(&self) -> String {
        // utilizes the global static labels for Hash40s
        format!("{}", self.inner)
    }

    fn __repr__(&self) -> String {
        format!("hash ({})", self.inner)
    }

    fn __hash__(&self) -> u64 {
        self.inner.0
    }

    fn __richcmp__(&self, other: PyRef<Self>, co: CompareOp) -> PyResult<bool> {
        match co {
            CompareOp::Eq => Ok(self == &*other),
            CompareOp::Ne => Ok(self != &*other),
            _ => Err(PyTypeError::new_err(
                "Only == or != comparisons valid for hash",
            )),
        }
    }
}

#[pyclass]
struct ParamIter {
    inner: IntoIter<PyObject>,
}

#[pymethods]
impl Param {
    fn __iter__(this: PyRef<Self>) -> PyResult<Py<ParamIter>> {
        let py = this.py();
        match &*this.inner.lock().unwrap() {
            ParamType::List(v) => {
                let refs: IntoIter<PyObject> =
                    v.0.iter()
                        .map(|p| p.clone_ref().into_py(py))
                        .collect::<Vec<_>>()
                        .into_iter();
                Py::new(py, ParamIter { inner: refs })
            }
            ParamType::Struct(v) => {
                let refs: IntoIter<PyObject> =
                    v.0.iter()
                        .map(|(h, p)| (*h, p.clone_ref()).into_py(py))
                        .collect::<Vec<_>>()
                        .into_iter();
                Py::new(py, ParamIter { inner: refs })
            }
            _ => Err(PyTypeError::new_err(
                "Cannot iterate params other than list or struct-type params",
            )),
        }
    }
}

#[pymethods]
impl ParamIter {
    fn __next__(mut this: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
        Ok(this.inner.next())
    }
}
