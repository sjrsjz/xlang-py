use arc_unsafe_refcell::ArcUnsafeRefCellWrapper;
use pyo3::types::{PyBytes, PyDict, PyFloat, PyInt, PyList, PyNone, PyString, PyTuple};
use pyo3::{create_exception, prelude::*};
use xlang::{Lambda, WrappedPyFunction};
use xlang_vm_core::executor::variable::{
    VMBytes as XlangVMBytes, VMFloat as XlangVMFloat, VMInt as XlangVMInt,
    VMKeyVal as XlangVMKeyVal, VMNamed as XlangVMNamed, VMNull as XlangVMNull,
    VMRange as XlangVMRange, VMString as XlangVMString, VMTuple as XlangVMTuple,
    VMWrapper as XlangVMWrapper,
};
use xlang_vm_core::gc::GCRef as XlangGCRef;
use xlang_vm_core::gc::GCSystem as XlangGCSystem;

mod arc_unsafe_refcell;
mod xlang;

// type ArcUnsafeGCWrapper = Arc<RefCell<UnsafeGCWrapper>>;

#[pyclass(unsendable)]
struct GCSystem {
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
}

#[allow(dead_code)]
trait GCRef {
    fn get_ref(&self) -> &XlangGCRef;
    fn get_mut_ref(&mut self) -> &mut XlangGCRef;
    fn clone_ref(&mut self) -> XlangGCRef;
    fn drop_ref(&mut self);
}

#[pyclass(unsendable)]
#[derive(Clone)]
struct VMInt {
    gc_ref: XlangGCRef,
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
}

impl VMInt {
    fn create(gc: &mut GCSystem, value: i64) -> Self {
        let gc_ref = match gc.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XlangVMInt::new(value)),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        VMInt {
            gc_ref,
            gc_system: gc.gc_system.clone(),
        }
    }
}

impl GCRef for VMInt {
    fn get_ref(&self) -> &XlangGCRef {
        &self.gc_ref
    }
    fn get_mut_ref(&mut self) -> &mut XlangGCRef {
        &mut self.gc_ref
    }
    fn clone_ref(&mut self) -> XlangGCRef {
        self.gc_ref.clone_ref()
    }
    fn drop_ref(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pymethods]
impl VMInt {
    #[new]
    #[pyo3(text_signature = "($cls, gc, value)")]
    fn new(gc: &mut GCSystem, value: i64) -> Self {
        VMInt::create(gc, value)
    }

    #[pyo3(text_signature = "($self)")]
    fn get_value(&self) -> i64 {
        self.gc_ref.as_const_type::<XlangVMInt>().value
    }

    #[pyo3(text_signature = "($self, value)")]
    fn set_value(&mut self, value: i64) {
        self.gc_ref.as_type::<XlangVMInt>().value = value;
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("VMInt({})", self.get_value()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.get_value()))
    }

    #[pyo3(text_signature = "($self)")]
    fn clone(&mut self) -> Self {
        let value = XlangVMInt::new(self.get_value());
        let gc_ref = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(value),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        return VMInt {
            gc_ref,
            gc_system: self.gc_system.clone(),
        };
    }

    #[pyo3(text_signature = "($self, py)")]
    fn to_py(&self, py: Python) -> PyResult<PyObject> {
        let value = self.get_value();
        let py_int = PyInt::new(py, value);
        Ok(py_int.into())
    }
}

impl Drop for VMInt {
    fn drop(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pyclass(unsendable)]
#[derive(Clone)]
struct VMFloat {
    gc_ref: XlangGCRef,
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
}

impl VMFloat {
    fn create(gc: &mut GCSystem, value: f64) -> Self {
        let gc_ref = match gc.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XlangVMFloat::new(value)),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        VMFloat {
            gc_ref,
            gc_system: gc.gc_system.clone(),
        }
    }
}

impl GCRef for VMFloat {
    fn get_ref(&self) -> &XlangGCRef {
        &self.gc_ref
    }
    fn get_mut_ref(&mut self) -> &mut XlangGCRef {
        &mut self.gc_ref
    }
    fn clone_ref(&mut self) -> XlangGCRef {
        self.gc_ref.clone_ref()
    }
    fn drop_ref(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pymethods]
impl VMFloat {
    #[new]
    #[pyo3(text_signature = "($cls, gc, value)")]
    fn new(gc: &mut GCSystem, value: f64) -> Self {
        VMFloat::create(gc, value)
    }

    #[pyo3(text_signature = "($self)")]
    fn get_value(&self) -> f64 {
        self.gc_ref.as_const_type::<XlangVMFloat>().value
    }

    #[pyo3(text_signature = "($self, value)")]
    fn set_value(&mut self, value: f64) {
        self.gc_ref.as_type::<XlangVMFloat>().value = value;
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("VMFloat({})", self.get_value()))
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.get_value()))
    }

    #[pyo3(text_signature = "($self)")]
    fn clone(&mut self) -> Self {
        let value = XlangVMFloat::new(self.get_value());
        let gc_ref = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(value),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        return VMFloat {
            gc_ref,
            gc_system: self.gc_system.clone(),
        };
    }

    #[pyo3(text_signature = "($self, py)")]
    fn to_py(&self, py: Python) -> PyResult<PyObject> {
        let value = self.get_value();
        let py_float = PyFloat::new(py, value);
        Ok(py_float.into())
    }
}

impl Drop for VMFloat {
    fn drop(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pyclass(unsendable)]
#[derive(Clone)]
struct VMString {
    gc_ref: XlangGCRef,
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
}

impl VMString {
    fn create(gc: &mut GCSystem, value: String) -> Self {
        let gc_ref = match gc.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XlangVMString::new(&value)),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        VMString {
            gc_ref,
            gc_system: gc.gc_system.clone(),
        }
    }
}

impl GCRef for VMString {
    fn get_ref(&self) -> &XlangGCRef {
        &self.gc_ref
    }
    fn get_mut_ref(&mut self) -> &mut XlangGCRef {
        &mut self.gc_ref
    }
    fn clone_ref(&mut self) -> XlangGCRef {
        self.gc_ref.clone_ref()
    }
    fn drop_ref(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pymethods]
impl VMString {
    #[new]
    #[pyo3(text_signature = "($cls, gc, value)")]
    fn new(gc: &mut GCSystem, value: String) -> Self {
        VMString::create(gc, value)
    }

    #[pyo3(text_signature = "($self)")]
    fn get_value(&self) -> String {
        self.gc_ref.as_const_type::<XlangVMString>().value.clone()
    }

    #[pyo3(text_signature = "($self, value)")]
    fn set_value(&mut self, value: String) {
        self.gc_ref.as_type::<XlangVMString>().value = value;
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("VMString(\"{}\")", self.get_value()))
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.get_value()))
    }

    fn __len__(&self) -> usize {
        self.gc_ref.as_const_type::<XlangVMString>().value.len()
    }

    fn clone(&mut self) -> Self {
        let value = XlangVMString::new(&self.get_value());
        let gc_ref = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(value),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        return VMString {
            gc_ref,
            gc_system: self.gc_system.clone(),
        };
    }

    #[pyo3(text_signature = "($self, py)")]
    fn to_py(&self, py: Python) -> PyResult<PyObject> {
        let value = self.get_value();
        let py_str = PyString::new(py, &value);
        Ok(py_str.into())
    }
}

impl Drop for VMString {
    fn drop(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pyclass(unsendable)]
#[derive(Clone)]
struct VMNull {
    gc_ref: XlangGCRef,
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
}

impl VMNull {
    fn create(gc: &mut GCSystem) -> Self {
        let gc_ref = match gc.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XlangVMNull::new()),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        VMNull {
            gc_ref,
            gc_system: gc.gc_system.clone(),
        }
    }
}

impl GCRef for VMNull {
    fn get_ref(&self) -> &XlangGCRef {
        &self.gc_ref
    }
    fn get_mut_ref(&mut self) -> &mut XlangGCRef {
        &mut self.gc_ref
    }
    fn clone_ref(&mut self) -> XlangGCRef {
        self.gc_ref.clone_ref()
    }
    fn drop_ref(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pymethods]
impl VMNull {
    #[new]
    #[pyo3(text_signature = "($cls, gc)")]
    fn new(gc: &mut GCSystem) -> Self {
        VMNull::create(gc)
    }

    #[pyo3(text_signature = "($self)")]
    fn get_value(&self) -> PyResult<PyObject> {
        let py_none = Python::with_gil(|py| py.None());
        Ok(py_none.into())
    }
    fn __repr__(&self) -> PyResult<String> {
        Ok("VMNull()".to_string())
    }
    fn __str__(&self) -> PyResult<String> {
        Ok("None".to_string())
    }

    #[pyo3(text_signature = "($self)")]
    fn clone(&mut self) -> Self {
        let gc_ref = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XlangVMNull::new()),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        return VMNull {
            gc_ref,
            gc_system: self.gc_system.clone(),
        };
    }

    #[pyo3(text_signature = "($self, py)")]
    fn to_py(&self, py: Python) -> PyResult<PyObject> {
        let py_none = py.None();
        Ok(py_none.into())
    }
}

impl Drop for VMNull {
    fn drop(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pyclass(unsendable)]
#[derive(Clone)]
struct VMBytes {
    gc_ref: XlangGCRef,
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
}

impl VMBytes {
    fn create(gc: &mut GCSystem, value: Vec<u8>) -> Self {
        let gc_ref = match gc.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XlangVMBytes::new(&value)),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        VMBytes {
            gc_ref,
            gc_system: gc.gc_system.clone(),
        }
    }
}

impl GCRef for VMBytes {
    fn get_ref(&self) -> &XlangGCRef {
        &self.gc_ref
    }
    fn get_mut_ref(&mut self) -> &mut XlangGCRef {
        &mut self.gc_ref
    }
    fn clone_ref(&mut self) -> XlangGCRef {
        self.gc_ref.clone_ref()
    }
    fn drop_ref(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pymethods]
impl VMBytes {
    #[new]
    #[pyo3(text_signature = "($cls, gc, value)")]
    fn new(gc: &mut GCSystem, value: Vec<u8>) -> Self {
        VMBytes::create(gc, value)
    }

    #[pyo3(text_signature = "($self)")]
    fn get_value(&self) -> Vec<u8> {
        self.gc_ref.as_const_type::<XlangVMBytes>().value.clone()
    }

    #[pyo3(text_signature = "($self, value)")]
    fn set_value(&mut self, value: Vec<u8>) {
        self.gc_ref.as_type::<XlangVMBytes>().value = value;
    }

    fn __repr__(&self) -> PyResult<String> {
        // Represent bytes as a string, similar to Python's b"..."
        // This might need a more robust way to escape non-printable characters
        let bytes_val = self.get_value();
        let repr_str = bytes_val
            .iter()
            .map(|b| {
                if *b >= 32 && *b <= 126 {
                    (*b as char).to_string()
                } else {
                    format!("\\x{:02x}", b)
                }
            })
            .collect::<String>();
        Ok(format!("VMBytes(b\"{}\")", repr_str))
    }

    fn __str__(&self) -> PyResult<String> {
        // Convert bytes to a string representation
        let bytes_val = self.get_value();
        let str_val = String::from_utf8_lossy(&bytes_val);
        Ok(str_val.to_string())
    }

    fn __len__(&self) -> usize {
        self.gc_ref.as_const_type::<XlangVMBytes>().value.len()
    }

    #[pyo3(text_signature = "($self)")]
    fn clone(&mut self) -> Self {
        let value = XlangVMBytes::new(&self.get_value());
        let gc_ref = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(value),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        return VMBytes {
            gc_ref,
            gc_system: self.gc_system.clone(),
        };
    }

    #[pyo3(text_signature = "($self, py)")]
    fn to_py(&self, py: Python) -> PyResult<PyObject> {
        let value = self.get_value();
        let py_bytes = PyBytes::new(py, &value);
        Ok(py_bytes.into())
    }
}

impl Drop for VMBytes {
    fn drop(&mut self) {
        self.gc_ref.drop_ref();
    }
}
// Helper function to handle Python basic types conversion using GC system
fn extract_xlang_gc_ref_with_gc_arc(
    obj: &Bound<'_, PyAny>,
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
) -> PyResult<XlangGCRef> {
    // First try to extract as a VM type
    if let Ok(gc_ref) = extract_xlang_gc_ref(obj) {
        return Ok(gc_ref);
    }
    // If not a VM type, handle basic Python types
    if let Ok(py_int) = obj.downcast::<PyInt>() {
        let value = py_int.extract::<i64>()?;
        let xlang_int = XlangVMInt::new(value);
        let new_gc_ref = match gc_system.borrow_mut() {
            Ok(mut mut_gc_system_guard) => mut_gc_system_guard.new_object(xlang_int),
            Err(_) => {
                panic!("Failed to borrow GC system for PyInt conversion");
            }
        };
        Ok(new_gc_ref)
    } else if let Ok(py_float) = obj.downcast::<PyFloat>() {
        let value = py_float.extract::<f64>()?;
        let xlang_float = XlangVMFloat::new(value);
        let new_gc_ref = match gc_system.borrow_mut() {
            Ok(mut mut_gc_system_guard) => mut_gc_system_guard.new_object(xlang_float),
            Err(_) => {
                panic!("Failed to borrow GC system for PyFloat conversion");
            }
        };
        Ok(new_gc_ref)
    } else if let Ok(py_str) = obj.downcast::<PyString>() {
        let value = py_str.to_string_lossy().to_string();
        let xlang_string = XlangVMString::new(&value);
        let new_gc_ref = match gc_system.borrow_mut() {
            Ok(mut mut_gc_system_guard) => mut_gc_system_guard.new_object(xlang_string),
            Err(_) => {
                panic!("Failed to borrow GC system for PyString conversion");
            }
        };
        Ok(new_gc_ref)
    } else if let Ok(py_bytes) = obj.downcast::<PyBytes>() {
        let value: Vec<u8> = py_bytes.extract()?;
        let xlang_bytes = XlangVMBytes::new(&value);
        let new_gc_ref = match gc_system.borrow_mut() {
            Ok(mut mut_gc_system_guard) => mut_gc_system_guard.new_object(xlang_bytes),
            Err(_) => {
                panic!("Failed to borrow GC system for PyBytes conversion");
            }
        };
        Ok(new_gc_ref)
    } else if let Ok(py_list) = obj.downcast::<PyList>() {
        let mut xlang_list: Vec<XlangGCRef> = Vec::new();
        for item in py_list.iter() {
            let item_ref = extract_xlang_gc_ref_with_gc_arc(&item, gc_system.clone())?;
            xlang_list.push(item_ref);
        }
        let new_gc_ref = match gc_system.borrow_mut() {
            Ok(mut mut_gc_system_guard) => mut_gc_system_guard
                .new_object(XlangVMTuple::new(&mut xlang_list.iter_mut().collect())),
            Err(_) => {
                panic!("Failed to borrow GC system for PyList conversion");
            }
        };
        for item in &mut xlang_list {
            item.drop_ref();
        }
        Ok(new_gc_ref)
    } else if let Ok(py_tuple) = obj.downcast::<PyTuple>() {
        // Changed py_list to py_tuple for clarity
        let mut xlang_list: Vec<XlangGCRef> = Vec::new();
        for item in py_tuple.iter() {
            // Changed py_list to py_tuple
            let item_ref = extract_xlang_gc_ref_with_gc_arc(&item, gc_system.clone())?; // Changed Arc::clone to gc_system.clone()
            xlang_list.push(item_ref);
        }
        let new_gc_ref = match gc_system.borrow_mut() {
            Ok(mut mut_gc_system_guard) => mut_gc_system_guard
                .new_object(XlangVMTuple::new(&mut xlang_list.iter_mut().collect())),
            Err(_) => {
                panic!("Failed to borrow GC system for PyTuple conversion");
            }
        };
        for item in &mut xlang_list {
            item.drop_ref();
        }
        Ok(new_gc_ref)
    } else if let Ok(_) = obj.downcast::<PyNone>() {
        let xlang_none = XlangVMNull::new();
        let new_gc_ref = match gc_system.borrow_mut() {
            Ok(mut mut_gc_system_guard) => mut_gc_system_guard.new_object(xlang_none),
            Err(_) => {
                panic!("Failed to borrow GC system for PyNone conversion");
            }
        };
        Ok(new_gc_ref)
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Expected a xlang VM type or basic Python type for extraction",
        ))
    }
}

// Helper function to handle Python basic types conversion using GC system
fn extract_xlang_gc_ref_with_gc(
    obj: &Bound<'_, PyAny>,
    gc_system: &mut XlangGCSystem,
) -> PyResult<XlangGCRef> {
    // First try to extract as a VM type
    if let Ok(gc_ref) = extract_xlang_gc_ref(obj) {
        return Ok(gc_ref);
    }
    // If not a VM type, handle basic Python types
    if let Ok(py_int) = obj.downcast::<PyInt>() {
        let value = py_int.extract::<i64>()?;
        let xlang_int = XlangVMInt::new(value);
        let new_gc_ref = gc_system.new_object(xlang_int);
        Ok(new_gc_ref)
    } else if let Ok(py_float) = obj.downcast::<PyFloat>() {
        let value = py_float.extract::<f64>()?;
        let xlang_float = XlangVMFloat::new(value);
        let new_gc_ref = gc_system.new_object(xlang_float);
        Ok(new_gc_ref)
    } else if let Ok(py_str) = obj.downcast::<PyString>() {
        let value = py_str.to_string_lossy().to_string();
        let xlang_string = XlangVMString::new(&value);
        let new_gc_ref = gc_system.new_object(xlang_string);
        Ok(new_gc_ref)
    } else if let Ok(py_bytes) = obj.downcast::<PyBytes>() {
        let value: Vec<u8> = py_bytes.extract()?;
        let xlang_bytes = XlangVMBytes::new(&value);
        let new_gc_ref = gc_system.new_object(xlang_bytes);
        Ok(new_gc_ref)
    } else if let Ok(py_list) = obj.downcast::<PyList>() {
        let mut xlang_list: Vec<XlangGCRef> = Vec::new();
        for item in py_list.iter() {
            let item_ref = extract_xlang_gc_ref_with_gc(&item, gc_system)?;
            xlang_list.push(item_ref);
        }
        let new_gc_ref =
            gc_system.new_object(XlangVMTuple::new(&mut xlang_list.iter_mut().collect()));
        for item in &mut xlang_list {
            item.drop_ref();
        }
        Ok(new_gc_ref)
    } else if let Ok(py_list) = obj.downcast::<PyTuple>() {
        let mut xlang_list: Vec<XlangGCRef> = Vec::new();
        for item in py_list.iter() {
            let item_ref = extract_xlang_gc_ref_with_gc(&item, gc_system)?;
            xlang_list.push(item_ref);
        }
        let new_gc_ref =
            gc_system.new_object(XlangVMTuple::new(&mut xlang_list.iter_mut().collect()));
        for item in &mut xlang_list {
            item.drop_ref();
        }
        Ok(new_gc_ref)
    } else if let Ok(_) = obj.downcast::<PyNone>() {
        let xlang_none = XlangVMNull::new();
        let new_gc_ref = gc_system.new_object(xlang_none);
        Ok(new_gc_ref)
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Expected a xlang VM type or basic Python type for extraction",
        ))
    }
}

// Helper function to extract XlangGCRef from a PyObject holding one of our VM types
// This function will need to be updated as more types are added or a more generic solution is found.
fn extract_xlang_gc_ref(obj: &Bound<'_, PyAny>) -> PyResult<XlangGCRef> {
    if let Ok(mut vm_int) = obj.extract::<PyRefMut<VMInt>>() {
        Ok(vm_int.gc_ref.clone_ref())
    } else if let Ok(mut vm_float) = obj.extract::<PyRefMut<VMFloat>>() {
        Ok(vm_float.gc_ref.clone_ref())
    } else if let Ok(mut vm_string) = obj.extract::<PyRefMut<VMString>>() {
        Ok(vm_string.gc_ref.clone_ref())
    } else if let Ok(mut vm_null) = obj.extract::<PyRefMut<VMNull>>() {
        Ok(vm_null.gc_ref.clone_ref())
    } else if let Ok(mut vm_bytes) = obj.extract::<PyRefMut<VMBytes>>() {
        Ok(vm_bytes.gc_ref.clone_ref())
    } else if let Ok(mut vm_key_val) = obj.extract::<PyRefMut<VMKeyVal>>() {
        Ok(vm_key_val.gc_ref.clone_ref())
    } else if let Ok(mut vm_named) = obj.extract::<PyRefMut<VMNamed>>() {
        Ok(vm_named.gc_ref.clone_ref())
    } else if let Ok(mut vm_tuple) = obj.extract::<PyRefMut<VMTuple>>() {
        Ok(vm_tuple.gc_ref.clone_ref())
    } else if let Ok(mut vm_wrapper) = obj.extract::<PyRefMut<VMWrapper>>() {
        Ok(vm_wrapper.gc_ref.clone_ref())
    } else if let Ok(mut vm_range) = obj.extract::<PyRefMut<VMRange>>() {
        Ok(vm_range.gc_ref.clone_ref())
    } else if let Ok(mut vm_wrapped_pyfunction) = obj.extract::<PyRefMut<WrappedPyFunction>>() {
        match &mut vm_wrapped_pyfunction.function_object {
            Some(wrapped) => {
                let xlang_ref = wrapped.clone_ref();
                Ok(xlang_ref)
            }
            None => Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "WrappedPyFunction is None",
            )),
        }
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Expected a xlang VM type for extraction",
        ))
    }
}

// Helper function to convert XlangGCRef to a PyObject wrapper
pub(crate) fn xlang_gc_ref_to_py_object(
    // Changed to pub(crate)
    gc_ref: &mut XlangGCRef, // Take ownership as we are creating a new Py wrapper
    gc_system_arc: ArcUnsafeRefCellWrapper<XlangGCSystem>,
    py: Python,
) -> PyResult<PyObject> {
    if gc_ref.isinstance::<XlangVMInt>() {
        let py_obj = VMInt {
            gc_ref: gc_ref.clone_ref(),
            gc_system: gc_system_arc,
        };
        Ok(Py::new(py, py_obj)?.into_pyobject(py)?.into())
    } else if gc_ref.isinstance::<XlangVMFloat>() {
        let py_obj = VMFloat {
            gc_ref: gc_ref.clone_ref(),
            gc_system: gc_system_arc,
        };
        Ok(Py::new(py, py_obj)?.into_pyobject(py)?.into())
    } else if gc_ref.isinstance::<XlangVMString>() {
        let py_obj = VMString {
            gc_ref: gc_ref.clone_ref(),
            gc_system: gc_system_arc,
        };
        Ok(Py::new(py, py_obj)?.into_pyobject(py)?.into())
    } else if gc_ref.isinstance::<XlangVMNull>() {
        let py_obj = VMNull {
            gc_ref: gc_ref.clone_ref(),
            gc_system: gc_system_arc,
        };
        Ok(Py::new(py, py_obj)?.into_pyobject(py)?.into())
    } else if gc_ref.isinstance::<XlangVMBytes>() {
        let py_obj = VMBytes {
            gc_ref: gc_ref.clone_ref(),
            gc_system: gc_system_arc,
        };
        Ok(Py::new(py, py_obj)?.into_pyobject(py)?.into())
    } else if gc_ref.isinstance::<XlangVMKeyVal>() {
        let py_obj = VMKeyVal {
            gc_ref: gc_ref.clone_ref(),
            gc_system: gc_system_arc,
        };
        Ok(Py::new(py, py_obj)?.into_pyobject(py)?.into())
    } else if gc_ref.isinstance::<XlangVMNamed>() {
        let py_obj = VMNamed {
            gc_ref: gc_ref.clone_ref(),
            gc_system: gc_system_arc,
        };
        Ok(Py::new(py, py_obj)?.into_pyobject(py)?.into())
    } else if gc_ref.isinstance::<XlangVMTuple>() {
        let py_obj = VMTuple {
            gc_ref: gc_ref.clone_ref(),
            gc_system: gc_system_arc,
        };
        Ok(Py::new(py, py_obj)?.into_pyobject(py)?.into())
    } else if gc_ref.isinstance::<XlangVMWrapper>() {
        // Handle VMWrapper case
        let py_obj = VMWrapper {
            gc_ref: gc_ref.clone_ref(),
            gc_system: gc_system_arc,
        };
        Ok(Py::new(py, py_obj)?.into_pyobject(py)?.into())
    } else if gc_ref.isinstance::<XlangVMRange>() {
        // Handle VMRange case
        let py_obj = VMRange {
            gc_ref: gc_ref.clone_ref(),
            gc_system: gc_system_arc,
        };
        Ok(Py::new(py, py_obj)?.into_pyobject(py)?.into())
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Unknown xlang VM type for PyObject conversion",
        ))
    }
}

#[pyclass(unsendable)]
#[derive(Clone)]
struct VMKeyVal {
    gc_ref: XlangGCRef,
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
}

impl VMKeyVal {
    fn create(
        gc: &mut GCSystem,
        py_key: PyObject,
        py_value: PyObject,
        py: Python,
    ) -> PyResult<Self> {
        let mut xlang_key_ref =
            extract_xlang_gc_ref_with_gc_arc(py_key.bind(py), gc.gc_system.clone())?;
        let mut xlang_value_ref =
            extract_xlang_gc_ref_with_gc_arc(py_value.bind(py), gc.gc_system.clone())?;

        let xlang_kv = XlangVMKeyVal::new(&mut xlang_key_ref, &mut xlang_value_ref);
        let new_gc_ref = match gc.gc_system.borrow_mut() {
            Ok(mut mut_gc_system_guard) => mut_gc_system_guard.new_object(xlang_kv),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };

        xlang_key_ref.drop_ref(); // Dropping the cloned refs from extract_xlang_gc_ref
        xlang_value_ref.drop_ref();

        Ok(VMKeyVal {
            gc_ref: new_gc_ref,
            gc_system: gc.gc_system.clone(),
        })
    }
}

impl GCRef for VMKeyVal {
    fn get_ref(&self) -> &XlangGCRef {
        &self.gc_ref
    }
    fn get_mut_ref(&mut self) -> &mut XlangGCRef {
        &mut self.gc_ref
    }
    fn clone_ref(&mut self) -> XlangGCRef {
        self.gc_ref.clone_ref()
    }
    fn drop_ref(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pymethods]
impl VMKeyVal {
    #[new]
    #[pyo3(text_signature = "($cls, gc, key, value, py)")]
    fn new(gc: &mut GCSystem, key: PyObject, value: PyObject, py: Python) -> PyResult<Self> {
        VMKeyVal::create(gc, key, value, py)
    }

    #[pyo3(text_signature = "($self, py)")]
    fn get_key(&mut self, py: Python) -> PyResult<PyObject> {
        let xlang_kv = self.gc_ref.as_type::<XlangVMKeyVal>();
        xlang_gc_ref_to_py_object(&mut xlang_kv.key, self.gc_system.clone(), py)
    }

    #[pyo3(text_signature = "($self, py_key, py)")]
    fn set_key(&mut self, py_key: PyObject, py: Python) -> PyResult<()> {
        let mut new_key_ref =
            extract_xlang_gc_ref_with_gc_arc(py_key.bind(py), self.gc_system.clone())?;
        let mut old_ref = self.gc_ref.as_type::<XlangVMKeyVal>().key.clone(); // Drop old key
        self.gc_ref.as_type::<XlangVMKeyVal>().key = new_key_ref.clone(); // Assign new key (takes ownership)
        self.gc_ref.get_traceable().add_reference(&mut new_key_ref);
        self.gc_ref.get_traceable().remove_reference(&mut old_ref);
        new_key_ref.drop_ref(); // Drop the new key reference
        Ok(())
    }

    #[pyo3(text_signature = "($self, py)")]
    fn get_value(&mut self, py: Python) -> PyResult<PyObject> {
        let xlang_kv = self.gc_ref.as_type::<XlangVMKeyVal>();
        xlang_gc_ref_to_py_object(&mut xlang_kv.value, self.gc_system.clone(), py)
    }

    #[pyo3(text_signature = "($self, py_value, py)")]
    fn set_value(&mut self, py_value: PyObject, py: Python) -> PyResult<()> {
        let mut new_value_ref =
            extract_xlang_gc_ref_with_gc_arc(py_value.bind(py), self.gc_system.clone())?;
        let mut old_ref = self.gc_ref.as_type::<XlangVMKeyVal>().value.clone(); // Drop old key
        self.gc_ref.as_type::<XlangVMKeyVal>().value = new_value_ref.clone(); // Assign new key (takes ownership)
        self.gc_ref
            .get_traceable()
            .add_reference(&mut new_value_ref);
        self.gc_ref.get_traceable().remove_reference(&mut old_ref);
        new_value_ref.drop_ref(); // Drop the new key reference
        Ok(())
    }

    fn __repr__(&mut self, py: Python) -> PyResult<String> {
        let xlang_kv = self.gc_ref.as_type::<XlangVMKeyVal>();

        let key_obj = xlang_gc_ref_to_py_object(&mut xlang_kv.key, self.gc_system.clone(), py)?;
        let value_obj = xlang_gc_ref_to_py_object(&mut xlang_kv.value, self.gc_system.clone(), py)?;

        let key_repr = key_obj.bind(py).repr()?.extract::<String>()?;
        let value_repr = value_obj.bind(py).repr()?.extract::<String>()?;
        Ok(format!("VMKeyVal({}, {})", key_repr, value_repr))
    }

    fn __str__(&mut self, py: Python) -> PyResult<String> {
        let xlang_kv = self.gc_ref.as_type::<XlangVMKeyVal>();

        let key_obj = xlang_gc_ref_to_py_object(&mut xlang_kv.key, self.gc_system.clone(), py)?;
        let value_obj = xlang_gc_ref_to_py_object(&mut xlang_kv.value, self.gc_system.clone(), py)?;

        let key_str = key_obj.bind(py).str()?.extract::<String>()?;
        let value_str = value_obj.bind(py).str()?.extract::<String>()?;
        Ok(format!("{} : {}", key_str, value_str))
    }

    #[pyo3(text_signature = "($self, py)")]
    fn clone(&mut self, _py: Python) -> PyResult<Self> {
        let xlang_kv_orig = self.gc_ref.as_type::<XlangVMKeyVal>();

        // We need to create new Xlang objects for the cloned key and value if they are to be distinct
        // For now, assuming clone means new VMKeyVal wrapper with new XlangVMKeyVal containing *cloned* Xlang objects
        // This requires deepcopy semantics from xlang_vm_core if we want true deep clone.
        // The current xlang_vm_core::VMObject::deepcopy is what we need.
        // For simplicity, this Python clone will create a new XlangVMKeyVal with *references* to the *same* key/value Xlang objects.
        // If a deep clone of underlying data is needed, that's a different operation.
        // The current `VMInt.clone()` creates a new XlangVMInt. We should follow that.

        // This is tricky: to truly clone, we'd need to deepcopy the underlying Xlang objects.
        // For now, let's make a new XlangVMKeyVal that refers to the *same* underlying key/value objects.
        // This is a shallow clone of the KeyVal structure, but shares the underlying K and V.
        // To match VMInt.clone(), we should create new Xlang objects for key and value.
        // This requires knowing their types and calling their respective Xlang::new methods.
        // This is complex. A simpler clone for now:
        let new_xlang_kv = XlangVMKeyVal::new(&mut xlang_kv_orig.key, &mut xlang_kv_orig.value);
        let new_gc_ref = match self.gc_system.borrow_mut() {
            Ok(mut mut_gc_system_guard) => mut_gc_system_guard.new_object(new_xlang_kv),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };

        Ok(VMKeyVal {
            gc_ref: new_gc_ref,
            gc_system: self.gc_system.clone(),
        })
    }

    #[pyo3(text_signature = "($self, py)")]
    fn to_py(&mut self, py: Python) -> PyResult<PyObject> {
        let xlang_kv = self.gc_ref.as_type::<XlangVMKeyVal>();
        let key_obj = xlang_gc_ref_to_py_object(&mut xlang_kv.key, self.gc_system.clone(), py)?;
        let value_obj = xlang_gc_ref_to_py_object(&mut xlang_kv.value, self.gc_system.clone(), py)?;
        let py_dict = PyDict::new(py);
        py_dict.set_item(key_obj, value_obj)?;
        Ok(py_dict.into())
    }
}

impl Drop for VMKeyVal {
    fn drop(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pyclass(unsendable)]
#[derive(Clone)]
struct VMNamed {
    gc_ref: XlangGCRef,
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
}

impl VMNamed {
    fn create(
        gc: &mut GCSystem,
        py_name: PyObject,
        py_value: PyObject,
        py: Python,
    ) -> PyResult<Self> {
        let mut xlang_name_ref =
            extract_xlang_gc_ref_with_gc_arc(py_name.bind(py), gc.gc_system.clone())?;
        let mut xlang_value_ref =
            extract_xlang_gc_ref_with_gc_arc(py_value.bind(py), gc.gc_system.clone())?;

        let xlang_named = XlangVMNamed::new(&mut xlang_name_ref, &mut xlang_value_ref);
        let new_gc_ref = match gc.gc_system.borrow_mut() {
            Ok(mut mut_gc_system_guard) => mut_gc_system_guard.new_object(xlang_named),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };

        xlang_name_ref.drop_ref();
        xlang_value_ref.drop_ref();

        Ok(VMNamed {
            gc_ref: new_gc_ref,
            gc_system: gc.gc_system.clone(),
        })
    }
}

impl GCRef for VMNamed {
    fn get_ref(&self) -> &XlangGCRef {
        &self.gc_ref
    }
    fn get_mut_ref(&mut self) -> &mut XlangGCRef {
        &mut self.gc_ref
    }
    fn clone_ref(&mut self) -> XlangGCRef {
        self.gc_ref.clone_ref()
    }
    fn drop_ref(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pymethods]
impl VMNamed {
    #[new]
    #[pyo3(text_signature = "($cls, gc, name, value, py)")]
    fn new(gc: &mut GCSystem, name: PyObject, value: PyObject, py: Python) -> PyResult<Self> {
        VMNamed::create(gc, name, value, py)
    }

    #[pyo3(text_signature = "($self, py)")]
    fn get_name(&mut self, py: Python) -> PyResult<PyObject> {
        let xlang_named = self.gc_ref.as_type::<XlangVMNamed>();
        xlang_gc_ref_to_py_object(&mut xlang_named.key, self.gc_system.clone(), py)
    }

    #[pyo3(text_signature = "($self, py_name, py)")]
    fn set_name(&mut self, py_name: PyObject, py: Python) -> PyResult<()> {
        let mut new_name_ref =
            extract_xlang_gc_ref_with_gc_arc(py_name.bind(py), self.gc_system.clone())?;
        let mut old_ref = self.gc_ref.as_type::<XlangVMNamed>().key.clone(); // Drop old key
        self.gc_ref.get_traceable().add_reference(&mut new_name_ref);
        self.gc_ref.get_traceable().remove_reference(&mut old_ref);
        new_name_ref.drop_ref(); // Drop the new key reference
        Ok(())
    }

    #[pyo3(text_signature = "($self, py)")]
    fn get_value(&mut self, py: Python) -> PyResult<PyObject> {
        let xlang_named = self.gc_ref.as_type::<XlangVMNamed>();
        xlang_gc_ref_to_py_object(&mut xlang_named.value, self.gc_system.clone(), py)
    }

    #[pyo3(text_signature = "($self, py_value, py)")]
    fn set_value(&mut self, py_value: PyObject, py: Python) -> PyResult<()> {
        let mut new_value_ref =
            extract_xlang_gc_ref_with_gc_arc(py_value.bind(py), self.gc_system.clone())?;
        let mut old_ref = self.gc_ref.as_type::<XlangVMNamed>().value.clone(); // Drop old key
        self.gc_ref.as_type::<XlangVMNamed>().value = new_value_ref.clone(); // Assign new key (takes ownership)
        self.gc_ref
            .get_traceable()
            .add_reference(&mut new_value_ref);
        self.gc_ref.get_traceable().remove_reference(&mut old_ref);
        new_value_ref.drop_ref(); // Drop the new key reference
        Ok(())
    }

    fn __repr__(&mut self, py: Python) -> PyResult<String> {
        let xlang_named = self.gc_ref.as_type::<XlangVMNamed>();

        let name_obj = xlang_gc_ref_to_py_object(&mut xlang_named.key, self.gc_system.clone(), py)?;
        let value_obj =
            xlang_gc_ref_to_py_object(&mut xlang_named.value, self.gc_system.clone(), py)?;

        let name_repr = name_obj.bind(py).repr()?.extract::<String>()?;
        let value_repr = value_obj.bind(py).repr()?.extract::<String>()?;
        Ok(format!("VMNamed({} => {})", name_repr, value_repr))
    }

    fn __str__(&mut self, py: Python) -> PyResult<String> {
        let xlang_named = self.gc_ref.as_type::<XlangVMNamed>();

        let name_obj = xlang_gc_ref_to_py_object(&mut xlang_named.key, self.gc_system.clone(), py)?;
        let value_obj =
            xlang_gc_ref_to_py_object(&mut xlang_named.value, self.gc_system.clone(), py)?;

        let name_str = name_obj.bind(py).str()?.extract::<String>()?;
        let value_str = value_obj.bind(py).str()?.extract::<String>()?;
        Ok(format!("{} => {}", name_str, value_str))
    }
    #[pyo3(text_signature = "($self, py)")]
    fn clone(&mut self, _py: Python) -> PyResult<Self> {
        // Similar to VMKeyVal, this is a shallow clone of the structure for now.
        let xlang_named_orig = self.gc_ref.as_type::<XlangVMNamed>();

        let new_xlang_named =
            XlangVMNamed::new(&mut xlang_named_orig.key, &mut xlang_named_orig.value);
        let new_gc_ref = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(new_xlang_named),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };

        Ok(VMNamed {
            gc_ref: new_gc_ref,
            gc_system: self.gc_system.clone(),
        })
    }

    #[pyo3(text_signature = "($self, py)")]
    fn to_py(&mut self, py: Python) -> PyResult<PyObject> {
        let xlang_named = self.gc_ref.as_type::<XlangVMNamed>();
        let name_obj = xlang_gc_ref_to_py_object(&mut xlang_named.key, self.gc_system.clone(), py)?;
        let value_obj =
            xlang_gc_ref_to_py_object(&mut xlang_named.value, self.gc_system.clone(), py)?;
        let py_dict = PyDict::new(py);
        py_dict.set_item(name_obj, value_obj)?;
        Ok(py_dict.into())
    }
}

impl Drop for VMNamed {
    fn drop(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pyclass(unsendable)]
#[derive(Clone)]
struct VMTuple {
    gc_ref: XlangGCRef,
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
}

impl VMTuple {
    fn create(gc: &mut GCSystem, py_values: Vec<PyObject>, py: Python) -> PyResult<Self> {
        let mut xlang_refs_vec: Vec<XlangGCRef> = Vec::with_capacity(py_values.len());
        for py_obj in py_values {
            xlang_refs_vec.push(extract_xlang_gc_ref_with_gc_arc(
                py_obj.bind(py),
                gc.gc_system.clone(),
            )?);
        }

        // XlangVMTuple::new expects &mut Vec<&mut GCRef>
        // We have Vec<GCRef>. Need to convert.
        let mut refs_for_xlang_constructor: Vec<&mut XlangGCRef> =
            xlang_refs_vec.iter_mut().collect();

        let xlang_tuple = XlangVMTuple::new(&mut refs_for_xlang_constructor);
        let new_gc_ref = match gc.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(xlang_tuple),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };

        // Drop the GCRefs in xlang_refs_vec as XlangVMTuple::new clones them.
        for mut r in xlang_refs_vec {
            r.drop_ref();
        }

        Ok(VMTuple {
            gc_ref: new_gc_ref,
            gc_system: gc.gc_system.clone(),
        })
    }
}

impl GCRef for VMTuple {
    fn get_ref(&self) -> &XlangGCRef {
        &self.gc_ref
    }
    fn get_mut_ref(&mut self) -> &mut XlangGCRef {
        &mut self.gc_ref
    }
    fn clone_ref(&mut self) -> XlangGCRef {
        self.gc_ref.clone_ref()
    }
    fn drop_ref(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pymethods]
impl VMTuple {
    #[new]
    #[pyo3(text_signature = "($cls, gc, values, py)")]
    fn new(gc: &mut GCSystem, values: Vec<PyObject>, py: Python) -> PyResult<Self> {
        VMTuple::create(gc, values, py)
    }

    fn __len__(&self) -> usize {
        self.gc_ref.as_const_type::<XlangVMTuple>().values.len()
    }

    // get_item is complex due to Python's rich indexing. For now, simple usize index.
    fn __getitem__(&mut self, idx: usize, py: Python) -> PyResult<PyObject> {
        let xlang_tuple = self.gc_ref.as_type::<XlangVMTuple>();
        if idx < xlang_tuple.values.len() {
            xlang_gc_ref_to_py_object(&mut xlang_tuple.values[idx], self.gc_system.clone(), py)
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "Tuple index out of range",
            ))
        }
    }

    fn __getattr__(&mut self, attr: &str, py: Python) -> PyResult<PyObject> {
        let xlang_tuple = self.gc_ref.as_type::<XlangVMTuple>();
        for item_ref in &mut xlang_tuple.values {
            if item_ref.isinstance::<XlangVMNamed>() {
                let xlang_named = item_ref.as_type::<XlangVMNamed>();
                if !xlang_named.key.isinstance::<XlangVMString>() {
                    continue;
                }
                let xlang_key = xlang_named.key.as_const_type::<XlangVMString>();
                let key_str = xlang_key.value.as_str();
                if key_str == attr {
                    return xlang_gc_ref_to_py_object(
                        &mut xlang_named.value,
                        self.gc_system.clone(),
                        py,
                    );
                }
            } else if item_ref.isinstance::<XlangVMKeyVal>() {
                let xlang_kv = item_ref.as_type::<XlangVMKeyVal>();
                if !xlang_kv.key.isinstance::<XlangVMString>() {
                    continue;
                }
                let xlang_key = xlang_kv.key.as_const_type::<XlangVMString>();
                let key_str = xlang_key.value.as_str();
                if key_str == attr {
                    return xlang_gc_ref_to_py_object(
                        &mut xlang_kv.value,
                        self.gc_system.clone(),
                        py,
                    );
                }
            }
        }
        Err(PyErr::new::<pyo3::exceptions::PyAttributeError, _>(
            format!("Attribute {} not found in tuple", attr),
        ))
    }

    #[pyo3(text_signature = "($self, py)")]
    fn to_list(&mut self, py: Python) -> PyResult<Vec<PyObject>> {
        let xlang_tuple = self.gc_ref.as_type::<XlangVMTuple>();
        let mut py_list = Vec::with_capacity(xlang_tuple.values.len());
        for item_ref in &mut xlang_tuple.values {
            py_list.push(xlang_gc_ref_to_py_object(
                item_ref,
                self.gc_system.clone(),
                py,
            )?);
        }
        Ok(py_list)
    }

    fn __repr__(&mut self, py: Python) -> PyResult<String> {
        let xlang_tuple = self.gc_ref.as_type::<XlangVMTuple>();
        let mut reprs = Vec::new();
        for item_ref in &mut xlang_tuple.values {
            let item_obj = xlang_gc_ref_to_py_object(item_ref, self.gc_system.clone(), py)?;
            reprs.push(item_obj.bind(py).repr()?.extract::<String>()?);
        }
        if reprs.len() == 1 {
            Ok(format!("VMTuple(({},))", reprs.join(", ")))
        } else {
            Ok(format!("VMTuple(({}))", reprs.join(", ")))
        }
    }

    fn __str__(&mut self, py: Python) -> PyResult<String> {
        let xlang_tuple = self.gc_ref.as_type::<XlangVMTuple>();
        let mut str_items = Vec::new();
        for item_ref in &mut xlang_tuple.values {
            let item_obj = xlang_gc_ref_to_py_object(item_ref, self.gc_system.clone(), py)?;
            str_items.push(item_obj.bind(py).str()?.extract::<String>()?);
        }

        if str_items.len() == 1 {
            Ok(format!("({},)", str_items.join(", ")))
        } else {
            Ok(format!("({})", str_items.join(", ")))
        }
    }

    #[pyo3(text_signature = "($self, py)")]
    fn clone(&mut self, _py: Python) -> PyResult<Self> {
        // This will be a shallow clone of the tuple structure, elements are shared.
        // For a deep clone, each element would need to be cloned.
        let xlang_tuple_orig = self.gc_ref.as_type::<XlangVMTuple>();
        let new_tuple = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XlangVMTuple::new(
                &mut xlang_tuple_orig.values.iter_mut().collect(),
            )),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };

        Ok(VMTuple {
            gc_ref: new_tuple,
            gc_system: self.gc_system.clone(),
        })
    }

    #[pyo3(text_signature = "($self, py)")]
    fn to_py(&mut self, py: Python) -> PyResult<PyObject> {
        let xlang_tuple = self.gc_ref.as_type::<XlangVMTuple>();
        let py_tuple = PyList::empty(py);
        for item_ref in &mut xlang_tuple.values {
            let item_obj = xlang_gc_ref_to_py_object(item_ref, self.gc_system.clone(), py)?;
            py_tuple.append(item_obj)?;
        }
        Ok(py_tuple.into())
    }
}

impl Drop for VMTuple {
    fn drop(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pyclass(unsendable)]
#[derive(Clone)]
struct VMWrapper {
    gc_ref: XlangGCRef,
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
}
impl VMWrapper {
    fn create(gc: &mut GCSystem, value: &mut XlangGCRef) -> Self {
        let gc_ref = match gc.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XlangVMWrapper::new(value)),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        VMWrapper {
            gc_ref,
            gc_system: gc.gc_system.clone(),
        }
    }
}
impl GCRef for VMWrapper {
    fn get_ref(&self) -> &XlangGCRef {
        &self.gc_ref
    }
    fn get_mut_ref(&mut self) -> &mut XlangGCRef {
        &mut self.gc_ref
    }
    fn clone_ref(&mut self) -> XlangGCRef {
        self.gc_ref.clone_ref()
    }
    fn drop_ref(&mut self) {
        self.gc_ref.drop_ref();
    }
}
impl Drop for VMWrapper {
    fn drop(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pymethods]
impl VMWrapper {
    #[new]
    #[pyo3(text_signature = "($cls, gc, value, py)")]
    fn new(gc: &mut GCSystem, value: PyObject, py: Python) -> PyResult<Self> {
        let mut xlang_ref = extract_xlang_gc_ref_with_gc_arc(value.bind(py), gc.gc_system.clone())?;
        let wrapped = VMWrapper::create(gc, &mut xlang_ref);
        xlang_ref.drop_ref(); // Drop the cloned ref
        Ok(wrapped)
    }

    #[pyo3(text_signature = "($self, py)")]
    fn get_value(&mut self, py: Python) -> PyResult<PyObject> {
        let xlang_wrapper = self.gc_ref.as_type::<XlangVMWrapper>();
        xlang_gc_ref_to_py_object(&mut xlang_wrapper.value_ref, self.gc_system.clone(), py)
    }

    #[pyo3(text_signature = "($self, value, py)")]
    fn set_value(&mut self, value: PyObject, py: Python) -> PyResult<()> {
        let mut new_value_ref =
            extract_xlang_gc_ref_with_gc_arc(value.bind(py), self.gc_system.clone())?;
        let mut old_ref = self.gc_ref.as_type::<XlangVMWrapper>().value_ref.clone(); // Drop old key
        self.gc_ref.as_type::<XlangVMWrapper>().value_ref = new_value_ref.clone(); // Assign new key (takes ownership)
        self.gc_ref
            .get_traceable()
            .add_reference(&mut new_value_ref);
        self.gc_ref.get_traceable().remove_reference(&mut old_ref);
        new_value_ref.drop_ref(); // Drop the new key reference
        Ok(())
    }
    fn __repr__(&mut self, py: Python) -> PyResult<String> {
        let xlang_wrapper = self.gc_ref.as_type::<XlangVMWrapper>();
        let value_obj =
            xlang_gc_ref_to_py_object(&mut xlang_wrapper.value_ref, self.gc_system.clone(), py)?;
        let value_repr = value_obj.bind(py).repr()?.extract::<String>()?;
        Ok(format!("VMWrapper({})", value_repr))
    }
    fn __str__(&mut self, py: Python) -> PyResult<String> {
        let xlang_wrapper = self.gc_ref.as_type::<XlangVMWrapper>();
        let value_obj =
            xlang_gc_ref_to_py_object(&mut xlang_wrapper.value_ref, self.gc_system.clone(), py)?;
        let value_str = value_obj.bind(py).str()?.extract::<String>()?;
        Ok(format!("wrap({})", value_str))
    }

    #[pyo3(text_signature = "($self, py)")]
    fn clone(&mut self, _py: Python) -> PyResult<Self> {
        // This will be a shallow clone of the wrapper structure, elements are shared.
        // For a deep clone, each element would need to be cloned.
        let xlang_wrapper_origin = self.gc_ref.as_type::<XlangVMWrapper>();
        let new_wrapper = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => {
                gc_system.new_object(XlangVMWrapper::new(&mut xlang_wrapper_origin.value_ref))
            }
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };

        Ok(VMWrapper {
            gc_ref: new_wrapper,
            gc_system: self.gc_system.clone(),
        })
    }
}

#[pyclass(unsendable)]
#[derive(Clone)]
struct VMRange {
    gc_ref: XlangGCRef,
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
}
impl VMRange {
    fn create(gc: &mut GCSystem, start: i64, end: i64) -> Self {
        let gc_ref = match gc.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XlangVMRange::new(start, end)),
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        VMRange {
            gc_ref,
            gc_system: gc.gc_system.clone(),
        }
    }
}

impl GCRef for VMRange {
    fn get_ref(&self) -> &XlangGCRef {
        &self.gc_ref
    }
    fn get_mut_ref(&mut self) -> &mut XlangGCRef {
        &mut self.gc_ref
    }
    fn clone_ref(&mut self) -> XlangGCRef {
        self.gc_ref.clone_ref()
    }
    fn drop_ref(&mut self) {
        self.gc_ref.drop_ref();
    }
}
impl Drop for VMRange {
    fn drop(&mut self) {
        self.gc_ref.drop_ref();
    }
}

#[pymethods]
impl VMRange {
    #[new]
    #[pyo3(text_signature = "($cls, gc, start, end)")]
    fn new(gc: &mut GCSystem, start: i64, end: i64) -> Self {
        VMRange::create(gc, start, end)
    }

    #[pyo3(text_signature = "($self)")]
    fn get_start(&self) -> i64 {
        self.gc_ref.as_const_type::<XlangVMRange>().start
    }

    #[pyo3(text_signature = "($self)")]
    fn get_end(&self) -> i64 {
        self.gc_ref.as_const_type::<XlangVMRange>().end
    }

    #[pyo3(text_signature = "($self)")]
    fn get_key(&self) -> i64 {
        self.get_start()
    }

    #[pyo3(text_signature = "($self)")]
    fn get_value(&self) -> i64 {
        self.get_end()
    }

    fn __repr__(&self) -> PyResult<String> {
        let start = self.get_start();
        let end = self.get_end();
        Ok(format!("VMRange({}, {})", start, end))
    }

    fn __str__(&self) -> PyResult<String> {
        let start = self.get_start();
        let end = self.get_end();
        Ok(format!("VMRange({}, {})", start, end))
    }

    fn __len__(&self) -> usize {
        (self.get_end() - self.get_start()) as usize
    }

    #[pyo3(text_signature = "($self)")]
    fn clone(&self) -> Self {
        let new_gc_ref = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => {
                gc_system.new_object(XlangVMRange::new(self.get_start(), self.get_end()))
            }
            Err(_) => {
                panic!("Failed to borrow GC system");
            }
        };
        VMRange {
            gc_ref: new_gc_ref,
            gc_system: self.gc_system.clone(),
        }
    }

    #[pyo3(text_signature = "($self, py)")]
    fn to_py(&self, py: Python) -> PyResult<PyObject> {
        let start = self.get_start();
        let end = self.get_end();

        // 创建一个 Python range 对象
        let range_module = py.import("builtins")?;
        let range_fn = range_module.getattr("range")?;
        let py_range = range_fn.call1((start, end))?;

        Ok(py_range.into())
    }
}

#[pymethods]
impl GCSystem {
    #[new]
    #[pyo3(text_signature = "($cls)")]
    fn new() -> Self {
        GCSystem {
            gc_system: ArcUnsafeRefCellWrapper::new(XlangGCSystem::new(None)),
        }
    }

    #[pyo3(text_signature = "($self)")]
    fn collect(&mut self) {
        match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.collect(),
            Err(_) => {
                panic!("Unable to collect garbage due to borrow error");
            }
        }
    }

    #[pyo3(text_signature = "($self)")]
    fn object_count(&self) -> usize {
        match self.gc_system.borrow() {
            Ok(gc_system) => gc_system._count(),
            Err(_) => {
                panic!("Unable to get object count due to borrow error");
            }
        }
    }

    #[pyo3(text_signature = "($self, value)")]
    fn new_int(&mut self, value: i64) -> VMInt {
        VMInt::create(self, value)
    }

    #[pyo3(text_signature = "($self, value)")]
    fn new_float(&mut self, value: f64) -> VMFloat {
        VMFloat::create(self, value)
    }

    #[pyo3(text_signature = "($self, value)")]
    fn new_string(&mut self, value: String) -> VMString {
        VMString::create(self, value)
    }

    #[pyo3(text_signature = "($self)")]
    fn new_null(&mut self) -> VMNull {
        VMNull::create(self)
    }

    #[pyo3(text_signature = "($self, value)")]
    fn new_bytes(&mut self, value: Vec<u8>) -> VMBytes {
        VMBytes::create(self, value)
    }

    #[pyo3(text_signature = "($self, key, value, py)")]
    fn new_keyval(&mut self, key: PyObject, value: PyObject, py: Python) -> PyResult<VMKeyVal> {
        VMKeyVal::create(self, key, value, py)
    }

    #[pyo3(text_signature = "($self, name, value, py)")]
    fn new_named(&mut self, name: PyObject, value: PyObject, py: Python) -> PyResult<VMNamed> {
        VMNamed::create(self, name, value, py)
    }

    #[pyo3(text_signature = "($self, values, py)")]
    fn new_tuple(&mut self, values: Vec<PyObject>, py: Python) -> PyResult<VMTuple> {
        VMTuple::create(self, values, py)
    }

    #[pyo3(text_signature = "($self, value, py)")]
    fn new_wrapper(&mut self, value: PyObject, py: Python) -> PyResult<VMWrapper> {
        let mut xlang_ref =
            extract_xlang_gc_ref_with_gc_arc(value.bind(py), self.gc_system.clone())?;
        let wrapped = VMWrapper::create(self, &mut xlang_ref);
        xlang_ref.drop_ref(); // Drop the cloned ref
        Ok(wrapped)
    }

    #[pyo3(text_signature = "($self)")]
    fn new_lambda(&mut self) -> PyResult<Lambda> {
        Ok(Lambda::create(self))
    }

    #[pyo3(text_signature = "($self, start, end)")]
    fn new_range(&mut self, start: i64, end: i64) -> VMRange {
        VMRange::create(self, start, end)
    }

    #[pyo3(text_signature = "($self)")]
    fn new_pyfunction(&mut self) -> WrappedPyFunction {
        WrappedPyFunction::create(self)
    }

    /// Internal helper: Converts a Python object to its corresponding xlang VM object,
    /// wrapped as a PyObject.
    #[allow(deprecated)]
    unsafe fn _py_object_to_xlang_object(
        &mut self,
        value: &Bound<'_, PyAny>,
        py: Python,
    ) -> PyResult<PyObject> {
        if let Ok(s) = value.extract::<String>() {
            let vm_obj = self.new_string(s);
            Ok(Py::new(py, vm_obj)?.into())
        } else if let Ok(i) = value.extract::<i64>() {
            let vm_obj = self.new_int(i);
            Ok(Py::new(py, vm_obj)?.into())
        } else if let Ok(f) = value.extract::<f64>() {
            let vm_obj = self.new_float(f);
            Ok(Py::new(py, vm_obj)?.into())
        } else if value.is_none() {
            let vm_obj = self.new_null();
            Ok(Py::new(py, vm_obj)?.into())
        // 检查集合类型应在通用的字节提取之前
        } else if let Ok(dict) = value.downcast::<PyDict>() {
            // Recursively convert dictionary
            let vm_tuple_struct = self._py_dict_to_keyval_tuple(dict, py)?;
            Ok(Py::new(py, vm_tuple_struct)?.into())
        } else if let Ok(list) = value.downcast::<pyo3::types::PyList>() {
            let mut vm_elements: Vec<PyObject> = Vec::with_capacity(list.len());
            for item_any in list.iter() {
                vm_elements.push(self._py_object_to_xlang_object(&item_any, py)?);
            }
            let vm_tuple_struct = self.new_tuple(vm_elements, py)?;
            Ok(Py::new(py, vm_tuple_struct)?.into())
        } else if let Ok(py_tuple) = value.downcast::<pyo3::types::PyTuple>() {
            let mut vm_elements: Vec<PyObject> = Vec::with_capacity(py_tuple.len());
            for item_any in py_tuple.iter() {
                vm_elements.push(self._py_object_to_xlang_object(&item_any, py)?);
            }
            let vm_tuple_struct = self.new_tuple(vm_elements, py)?;
            Ok(Py::new(py, vm_tuple_struct)?.into())
        // 明确检查 PyBytes 类型
        } else if let Ok(py_bytes) = value.downcast::<pyo3::types::PyBytes>() {
            let b = py_bytes.as_bytes().to_vec(); // 从 PyBytes 获取 Vec<u8>
            let vm_obj = self.new_bytes(b);
            Ok(Py::new(py, vm_obj)?.into())
        // （可选）如果也想处理 PyByteArray
        } else if let Ok(py_byte_array) = value.downcast::<pyo3::types::PyByteArray>() {
            let b = py_byte_array.as_bytes().to_vec();
            let vm_obj = self.new_bytes(b);
            Ok(Py::new(py, vm_obj)?.into())
        } else if let Ok(py_set) = value.downcast::<pyo3::types::PySet>() {
            // Convert set to list
            let mut vm_elements: Vec<PyObject> = Vec::with_capacity(py_set.len());
            for item_any in py_set.iter() {
                vm_elements.push(self._py_object_to_xlang_object(&item_any, py)?);
            }
            let vm_tuple_struct = self.new_tuple(vm_elements, py)?;
            Ok(Py::new(py, vm_tuple_struct)?.into())
        }
        // Check if it's already one of our wrapped VM types
        else if value.is_instance_of::<VMInt>()
            || value.is_instance_of::<VMFloat>()
            || value.is_instance_of::<VMString>()
            || value.is_instance_of::<VMNull>()
            || value.is_instance_of::<VMBytes>()
            || value.is_instance_of::<VMKeyVal>()
            || value.is_instance_of::<VMNamed>()
            || value.is_instance_of::<VMTuple>()
        {
            Ok(value.to_object(py))
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "Unsupported Python type for xlang conversion: {}",
                value.get_type().name()?
            )))
        }
    }

    /// Internal helper: Converts a Python dictionary to an xlang VMTuple (Rust struct) of VMKeyVals.
    unsafe fn _py_dict_to_keyval_tuple(
        &mut self,
        dict: &Bound<'_, PyDict>,
        py: Python,
    ) -> PyResult<VMTuple> {
        let mut keyval_pyobjects: Vec<PyObject> = Vec::with_capacity(dict.len());

        for (key_any, value_any) in dict.iter() {
            let vm_key_pyobj = self._py_object_to_xlang_object(&key_any, py)?;
            let vm_value_pyobj = self._py_object_to_xlang_object(&value_any, py)?;

            let vm_keyval_struct = self.new_keyval(vm_key_pyobj, vm_value_pyobj, py)?;
            keyval_pyobjects.push(Py::new(py, vm_keyval_struct)?.into());
        }

        self.new_tuple(keyval_pyobjects, py)
    }

    /// Creates an xlang key-value tuple from a Python dictionary.
    /// The resulting VMTuple will contain VMKeyVal objects.
    #[pyo3(text_signature = "($self, pydict)")]
    pub fn new_dict(&mut self, pydict: &Bound<'_, PyDict>, py: Python) -> PyResult<VMTuple> {
        unsafe { self._py_dict_to_keyval_tuple(pydict, py) }
    }

    /// Alternative name for creating an xlang key-value tuple from a Python dictionary.
    /// Functionally identical to `new_dict`.
    #[pyo3(text_signature = "($self, pydict)")]
    pub fn from_pydict(&mut self, pydict: &Bound<'_, PyDict>, py: Python) -> PyResult<VMTuple> {
        unsafe { self._py_dict_to_keyval_tuple(pydict, py) }
    }
}

create_exception!(xlang_py, XlangSetupError, pyo3::exceptions::PyException);
create_exception!(
    xlang_py,
    XlangCompilationError,
    pyo3::exceptions::PyException
);
create_exception!(
    xlang_py,
    XlangTranslationError,
    pyo3::exceptions::PyException
);
create_exception!(xlang_py, XlangExecutionError, pyo3::exceptions::PyException);

// 修复模块导出
#[pymodule(name = "xlang_py")]
fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GCSystem>()?;
    m.add_class::<VMInt>()?;
    m.add_class::<VMFloat>()?;
    m.add_class::<VMString>()?;
    m.add_class::<VMNull>()?;
    m.add_class::<VMBytes>()?;
    m.add_class::<VMKeyVal>()?;
    m.add_class::<VMNamed>()?;
    m.add_class::<VMTuple>()?;
    m.add_class::<VMWrapper>()?;
    m.add_class::<VMRange>()?;

    m.add_class::<Lambda>()?;
    m.add_class::<WrappedPyFunction>()?;

    let py = m.py();
    // 添加异常类
    m.add("XlangSetupError", py.get_type::<XlangSetupError>())?;
    m.add(
        "XlangCompilationError",
        py.get_type::<XlangCompilationError>(),
    )?;
    m.add(
        "XlangTranslationError",
        py.get_type::<XlangTranslationError>(),
    )?;
    m.add("XlangExecutionError", py.get_type::<XlangExecutionError>())?;

    // 添加模块级函数和常量
    m.add("__doc__", "XLang-Rust for python")?;
    m.add("VERSION", "0.1.0")?;

    Ok(())
}
