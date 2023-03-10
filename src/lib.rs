use std::hash::{Hash, Hasher};
use std::vec::IntoIter;

use im_rc::{HashMap, HashSet, Vector};
use pyo3::exceptions::PyIndexError;
use pyo3::pyclass::CompareOp;
use pyo3::types::{PyDict, PyTuple, PyType};
use pyo3::{exceptions::PyKeyError, types::PyMapping};
use pyo3::{prelude::*, AsPyPointer};

#[derive(Clone, Debug)]
struct Key {
    hash: isize,
    inner: PyObject,
}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_isize(self.hash);
    }
}

impl Eq for Key {}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        Python::with_gil(|py| {
            self.inner
                .call_method1(py, "__eq__", (&other.inner,))
                .and_then(|value| value.extract(py))
                .expect("__eq__ failed!")
        })
    }
}

impl IntoPy<PyObject> for Key {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.inner.into_py(py)
    }
}

impl AsPyPointer for Key {
    fn as_ptr(&self) -> *mut pyo3::ffi::PyObject {
        self.inner.as_ptr()
    }
}

impl<'source> FromPyObject<'source> for Key {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        Ok(Key {
            hash: ob.hash()?,
            inner: ob.into(),
        })
    }
}

#[repr(transparent)]
#[pyclass(name = "HashMap", module = "imrc", frozen, mapping, unsendable)]
struct HashMapPy {
    inner: HashMap<Key, PyObject>,
}

impl From<HashMap<Key, PyObject>> for HashMapPy {
    fn from(map: HashMap<Key, PyObject>) -> Self {
        HashMapPy { inner: map }
    }
}

impl<'source> FromPyObject<'source> for HashMapPy {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let mut ret = HashMap::new();
        if let Ok(mapping) = ob.downcast::<PyMapping>() {
            for each in mapping.items()?.iter()? {
                let (k, v): (Key, PyObject) = each?.extract()?;
                ret.insert(k, v);
            }
        } else {
            for each in ob.iter()? {
                let (k, v): (Key, PyObject) = each?.extract()?;
                ret.insert(k, v);
            }
        }
        Ok(HashMapPy { inner: ret })
    }
}

#[pymethods]
impl HashMapPy {
    #[new]
    #[pyo3(signature = (value=None, **kwds))]
    fn init(value: Option<HashMapPy>, kwds: Option<&PyDict>) -> PyResult<Self> {
        let mut map: HashMapPy;
        if let Some(value) = value {
            map = value;
        } else {
            map = HashMapPy {
                inner: HashMap::new(),
            };
        }
        if let Some(kwds) = kwds {
            for (k, v) in kwds {
                map.inner.insert(Key::extract(k)?, v.into());
            }
        }
        Ok(map)
    }

    fn __contains__(&self, key: Key) -> bool {
        self.inner.contains_key(&key)
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<KeyIterator>> {
        Py::new(
            slf.py(),
            KeyIterator {
                inner: slf.keys().into_iter(),
            },
        )
    }

    fn __getitem__(&self, key: Key) -> PyResult<PyObject> {
        match self.inner.get(&key) {
            Some(value) => Ok(value.to_owned()),
            None => Err(PyKeyError::new_err(key)),
        }
    }

    fn __len__(&self) -> usize {
        self.inner.len().into()
    }

    fn __repr__(&self, py: Python) -> String {
        let contents = self.inner.iter().map(|(k, v)| {
            format!(
                "{}: {}",
                k.into_py(py),
                v.call_method0(py, "__repr__")
                    .and_then(|r| r.extract(py))
                    .unwrap_or("<repr error>".to_owned())
            )
        });
        format!("HashMap({{{}}})", contents.collect::<Vec<_>>().join(", "))
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> PyResult<PyObject> {
        match op {
            CompareOp::Eq => Ok((self.inner.len() == other.inner.len()
                && self
                    .inner
                    .iter()
                    .map(|(k1, v1)| (v1, other.inner.get(&k1)))
                    .map(|(v1, v2)| PyAny::eq(v1.extract(py)?, v2))
                    .all(|r| r.unwrap_or(false)))
            .into_py(py)),
            CompareOp::Ne => Ok((self.inner.len() != other.inner.len()
                || self
                    .inner
                    .iter()
                    .map(|(k1, v1)| (v1, other.inner.get(&k1)))
                    .map(|(v1, v2)| PyAny::ne(v1.extract(py)?, v2))
                    .all(|r| r.unwrap_or(true)))
            .into_py(py)),
            _ => Ok(py.NotImplemented()),
        }
    }

    #[classmethod]
    fn convert(_cls: &PyType, value: &PyAny, py: Python) -> PyResult<PyObject> {
        if value.is_instance_of::<HashMapPy>()? {
            Ok(value.into())
        } else {
            Ok(HashMapPy::extract(value)?.into_py(py))
        }
    }

    fn get(&self, key: Key) -> Option<&PyObject> {
        self.inner.get(&key)
    }

    fn keys(&self) -> Vec<Key> {
        self.inner.keys().map(|key| key.clone()).collect()
    }

    fn values(&self) -> Vec<&PyObject> {
        self.inner.values().collect::<Vec<&PyObject>>().to_owned()
    }

    fn items(&self) -> Vec<(&Key, &PyObject)> {
        self.inner
            .iter()
            .collect::<Vec<(&Key, &PyObject)>>()
            .to_owned()
    }

    fn discard(&self, key: Key) -> PyResult<HashMapPy> {
        match self.inner.contains_key(&key) {
            true => Ok(HashMapPy {
                inner: self.inner.without(&key),
            }),
            false => Ok(HashMapPy {
                inner: self.inner.clone(),
            }),
        }
    }

    fn insert(&self, key: Key, value: &PyAny) -> HashMapPy {
        HashMapPy {
            inner: self.inner.update(Key::from(key), value.into()),
        }
    }

    fn remove(&self, key: Key) -> PyResult<HashMapPy> {
        match self.inner.contains_key(&key) {
            true => Ok(HashMapPy {
                inner: self.inner.without(&key),
            }),
            false => Err(PyKeyError::new_err(key)),
        }
    }

    #[pyo3(signature = (*maps, **kwds))]
    fn update(&self, maps: &PyTuple, kwds: Option<&PyDict>) -> PyResult<HashMapPy> {
        let mut inner = self.inner.clone();
        for value in maps {
            let map = HashMapPy::extract(value)?;
            for (k, v) in &map.inner {
                inner.insert(k.to_owned(), v.to_owned());
            }
        }
        if let Some(kwds) = kwds {
            for (k, v) in kwds {
                inner.insert(Key::extract(k)?, v.extract()?);
            }
        }
        Ok(HashMapPy { inner })
    }
}

#[pyclass(module = "imrc", unsendable)]
struct KeyIterator {
    inner: IntoIter<Key>,
}

#[pymethods]
impl KeyIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Key> {
        slf.inner.next()
    }
}

#[repr(transparent)]
#[pyclass(name = "HashSet", module = "imrc", frozen, unsendable)]
struct HashSetPy {
    inner: HashSet<Key>,
}

impl<'source> FromPyObject<'source> for HashSetPy {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let mut ret = HashSet::new();
        for each in ob.iter()? {
            let k: Key = each?.extract()?;
            ret.insert(k);
        }
        Ok(HashSetPy { inner: ret })
    }
}

fn is_subset(one: &HashSet<Key>, two: &HashSet<Key>) -> bool {
    one.iter().all(|v| two.contains(v))
}

#[pymethods]
impl HashSetPy {
    #[new]
    fn init(value: Option<HashSetPy>) -> Self {
        if let Some(value) = value {
            value
        } else {
            HashSetPy {
                inner: HashSet::new(),
            }
        }
    }

    fn __and__(&self, other: &Self) -> Self {
        self.intersection(&other)
    }

    fn __or__(&self, other: &Self) -> Self {
        self.union(&other)
    }

    fn __sub__(&self, other: &Self) -> Self {
        self.difference(&other)
    }

    fn __xor__(&self, other: &Self) -> Self {
        self.symmetric_difference(&other)
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<KeyIterator>> {
        let iter = slf
            .inner
            .iter()
            .map(|k| k.to_owned())
            .collect::<Vec<_>>()
            .into_iter();
        Py::new(slf.py(), KeyIterator { inner: iter })
    }

    fn __len__(&self) -> usize {
        self.inner.len().into()
    }

    fn __repr__(&self, py: Python) -> String {
        let contents = self.inner.iter().map(|k| {
            k.into_py(py)
                .call_method0(py, "__repr__")
                .and_then(|r| r.extract(py))
                .unwrap_or("<repr failed>".to_owned())
        });
        format!("HashSet({{{}}})", contents.collect::<Vec<_>>().join(", "))
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> PyResult<PyObject> {
        match op {
            CompareOp::Eq => Ok((self.inner.len() == other.inner.len()
                && is_subset(&self.inner, &other.inner))
            .into_py(py)),
            CompareOp::Ne => Ok((self.inner.len() != other.inner.len()
                || self.inner.iter().any(|k| !other.inner.contains(k)))
            .into_py(py)),
            CompareOp::Lt => Ok((self.inner.len() < other.inner.len()
                && is_subset(&self.inner, &other.inner))
            .into_py(py)),
            CompareOp::Le => Ok(is_subset(&self.inner, &other.inner).into_py(py)),
            _ => Ok(py.NotImplemented()),
        }
    }

    fn insert(&self, value: Key) -> HashSetPy {
        HashSetPy {
            inner: self.inner.update(Key::from(value)),
        }
    }

    fn discard(&self, value: Key) -> PyResult<HashSetPy> {
        match self.inner.contains(&value) {
            true => Ok(HashSetPy {
                inner: self.inner.without(&value),
            }),
            false => Ok(HashSetPy {
                inner: self.inner.clone(),
            }),
        }
    }

    fn remove(&self, value: Key) -> PyResult<HashSetPy> {
        match self.inner.contains(&value) {
            true => Ok(HashSetPy {
                inner: self.inner.without(&value),
            }),
            false => Err(PyKeyError::new_err(value)),
        }
    }

    fn difference(&self, other: &Self) -> Self {
        let mut inner = self.inner.clone();
        for value in other.inner.iter() {
            inner.remove(value);
        }
        HashSetPy { inner }
    }

    fn intersection(&self, other: &Self) -> Self {
        let mut inner: HashSet<Key> = HashSet::new();
        let larger: &HashSet<Key>;
        let iter;
        if self.inner.len() > other.inner.len() {
            larger = &self.inner;
            iter = other.inner.iter();
        } else {
            larger = &other.inner;
            iter = self.inner.iter();
        }
        for value in iter {
            if larger.contains(value) {
                inner.insert(value.to_owned());
            }
        }
        HashSetPy { inner }
    }

    fn symmetric_difference(&self, other: &Self) -> Self {
        let mut inner: HashSet<Key>;
        let iter;
        if self.inner.len() > other.inner.len() {
            inner = self.inner.clone();
            iter = other.inner.iter();
        } else {
            inner = other.inner.clone();
            iter = self.inner.iter();
        }
        for value in iter {
            if inner.contains(value) {
                inner.remove(value);
            } else {
                inner.insert(value.to_owned());
            }
        }
        HashSetPy { inner }
    }

    fn union(&self, other: &Self) -> Self {
        let mut inner: HashSet<Key>;
        let iter;
        if self.inner.len() > other.inner.len() {
            inner = self.inner.clone();
            iter = other.inner.iter();
        } else {
            inner = other.inner.clone();
            iter = self.inner.iter();
        }
        for value in iter {
            inner.insert(value.to_owned());
        }
        HashSetPy { inner }
    }

    #[pyo3(signature = (*iterables))]
    fn update(&self, iterables: &PyTuple) -> PyResult<HashSetPy> {
        let mut inner = self.inner.clone();
        for each in iterables {
            let iter = each.iter()?;
            for value in iter {
                inner.insert(Key::extract(value?)?.to_owned());
            }
        }
        Ok(HashSetPy { inner })
    }
}

#[repr(transparent)]
#[pyclass(name = "Vector", module = "imrc", frozen, sequence, unsendable)]
struct VectorPy {
    inner: Vector<PyObject>,
}

impl From<Vector<PyObject>> for VectorPy {
    fn from(elements: Vector<PyObject>) -> Self {
        VectorPy { inner: elements }
    }
}

impl<'source> FromPyObject<'source> for VectorPy {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let mut ret: Vector<PyObject> = Vector::new();
        for each in ob.iter()? {
            ret.push_back(each?.extract()?);
        }
        Ok(VectorPy { inner: ret })
    }
}

#[pymethods]
impl VectorPy {
    #[new]
    #[pyo3(signature = (*elements))]
    fn init(elements: &PyTuple) -> PyResult<Self> {
        let mut ret: VectorPy;
        if elements.len() == 1 {
            ret = elements.get_item(0)?.extract()?;
        } else {
            ret = VectorPy {
                inner: Vector::new(),
            };
            if elements.len() > 1 {
                for each in elements {
                    ret.inner.push_back(each.extract()?);
                }
            }
        }
        Ok(ret)
    }

    fn __len__(&self) -> usize {
        self.inner.len()
    }

    fn __repr__(&self, py: Python) -> String {
        let contents = self.inner.iter().map(|k| {
            k.into_py(py)
                .call_method0(py, "__repr__")
                .and_then(|r| r.extract(py))
                .unwrap_or("<repr failed>".to_owned())
        });
        format!("Vector([{}])", contents.collect::<Vec<_>>().join(", "))
    }

    fn __reversed__(&self) -> Self {
        let mut inner = Vector::new();
        for each in self.inner.iter() {
            inner.push_front(each.to_owned())
        }
        VectorPy { inner }
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp, py: Python<'_>) -> PyResult<PyObject> {
        match op {
            CompareOp::Eq => Ok((self.inner.len() == other.inner.len()
                && self
                    .inner
                    .iter()
                    .zip(other.inner.iter())
                    .map(|(e1, e2)| PyAny::eq(e1.extract(py)?, e2))
                    .all(|r| r.unwrap_or(false)))
            .into_py(py)),
            CompareOp::Ne => Ok((self.inner.len() != other.inner.len()
                || self
                    .inner
                    .iter()
                    .zip(other.inner.iter())
                    .map(|(e1, e2)| PyAny::ne(e1.extract(py)?, e2))
                    .any(|r| r.unwrap_or(true)))
            .into_py(py)),
            _ => Ok(py.NotImplemented()),
        }
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<VectorIterator>> {
        let iter = slf
            .inner
            .iter()
            .map(|k| k.to_owned())
            .collect::<Vec<_>>()
            .into_iter();
        Py::new(slf.py(), VectorIterator { inner: iter })
    }

    #[getter]
    fn first(&self) -> PyResult<&PyObject> {
        self.inner
            .front()
            .ok_or_else(|| PyIndexError::new_err("empty list has no first element"))
    }

    fn push_front(&self, other: PyObject) -> VectorPy {
        let mut inner = self.inner.clone();
        inner.push_front(other);
        VectorPy { inner }
    }

    #[getter]
    fn rest(&self) -> VectorPy {
        let mut inner = self.inner.clone();
        inner.pop_front();
        VectorPy { inner }
    }
}

#[pyclass(module = "imrc", unsendable)]
struct VectorIterator {
    inner: IntoIter<PyObject>,
}

#[pymethods]
impl VectorIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyObject> {
        slf.inner.next()
    }
}

#[pymodule]
#[pyo3(name = "imrc")]
fn imrc(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<HashMapPy>()?;
    PyMapping::register::<HashMapPy>(py)?;
    m.add_class::<HashSetPy>()?;
    m.add_class::<VectorPy>()?;
    Ok(())
}
