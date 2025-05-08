use std::{cell::RefCell, sync::Arc};

use crate::{
    extract_xlang_gc_ref, xlang_gc_ref_to_py_object, GCSystem, VMTuple, XlangCompilationError,
    XlangExecutionError,
};
use pyo3::types::{PyDict, PyTuple};
use pyo3::{exceptions::PyIOError, prelude::*};
use xlang_frontend::{compile::build_code, dir_stack::DirStack};
use xlang_vm_core::executor::vm::VMCoroutinePool;
use xlang_vm_core::gc::{GCRef, GCSystem as XlangGCSystem};
use xlang_vm_core::ir_translator::IRTranslator;

use xlang_vm_core::executor::variable::VMBytes as XLangVMBytes;
use xlang_vm_core::executor::variable::VMLambdaBody as XLangVMLambdaBody;
use xlang_vm_core::executor::variable::VMNamed as XLangVMNamed;
use xlang_vm_core::executor::variable::VMNull as XLangVMNull;
use xlang_vm_core::executor::variable::VMString as XLangVMString;
use xlang_vm_core::executor::variable::{VMInstructions, VMTuple as XLangVMTuple};
use xlang_vm_core::executor::variable::{VMLambda as XLangVMLambda, VMVariableError};

#[pyclass(unsendable)]
#[derive(Clone)]
pub struct Lambda {
    gc_system: Arc<RefCell<XlangGCSystem>>,
    lambda_object: Option<GCRef>,
}

impl Lambda {
    pub(crate) fn create(gc: &mut GCSystem) -> Self {
        Lambda {
            lambda_object: None,
            gc_system: Arc::clone(&gc.gc_system),
        }
    }
}

impl Drop for Lambda {
    fn drop(&mut self) {
        if let Some(ref mut lambda) = self.lambda_object {
            lambda.drop_ref();
        }
    }
}

#[pymethods]
impl Lambda {
    #[new]
    fn new(gc: &mut GCSystem) -> Self {
        Lambda {
            lambda_object: None,
            gc_system: Arc::clone(&gc.gc_system),
        }
    }

    // 失败时返回错误信息
    #[pyo3(signature = (code, default_args, capture=None, self_object=None, work_dir=None))]
    fn load(
        &mut self,
        code: &str,
        default_args: &mut VMTuple,
        capture: Option<PyObject>,
        self_object: Option<PyObject>,
        work_dir: Option<&str>,
        py: Python<'_>,
    ) -> PyResult<()> {
        let dir_stack = DirStack::new(Some(&work_dir.unwrap_or_else(|| ".").into()));
        if dir_stack.is_err() {
            return Err(PyIOError::new_err(format!(
                "Failed to create directory stack: {}",
                dir_stack.err().unwrap()
            )));
        }
        let mut dir_stack = dir_stack.unwrap();
        let instruction_package;
        match build_code(&code, &mut dir_stack) {
            Ok(package) => {
                let mut translator = IRTranslator::new(&package);
                let translate_result = translator.translate();
                let result = if translate_result.is_ok() {
                    translator.get_result()
                } else {
                    return Err(XlangCompilationError::new_err(format!(
                        "Failed to translate code: {:?}",
                        translate_result.err().unwrap()
                    )));
                };
                instruction_package = Some(result);
            }
            Err(e) => {
                return Err(XlangCompilationError::new_err(format!(
                    "Failed to build code: {}",
                    e
                )))
            }
        }

        let mut capture_ref_option: Option<GCRef> = if let Some(c) = capture {
            Some(extract_xlang_gc_ref(&c.into_bound(py))?)
        } else {
            None
        };

        let mut self_object_ref_option: Option<GCRef> = if let Some(s) = self_object {
            Some(extract_xlang_gc_ref(&s.into_bound(py))?)
        } else {
            None
        };

        let mut default_result = self.gc_system.borrow_mut().new_object(XLangVMNull::new());
        let mut instruction_ref = self
            .gc_system
            .borrow_mut()
            .new_object(VMInstructions::new(&instruction_package.unwrap()));
        let lambda: GCRef = self.gc_system.borrow_mut().new_object(XLangVMLambda::new(
            0,
            "__main__".to_string(),
            &mut default_args.gc_ref,
            capture_ref_option.as_mut(),
            self_object_ref_option.as_mut(),
            &mut XLangVMLambdaBody::VMInstruction(instruction_ref.clone()),
            &mut default_result,
            false,
        ));

        let mut old_ref = self.lambda_object.take();
        self.lambda_object = Some(lambda);
        if let Some(ref mut old_lambda) = old_ref {
            old_lambda.drop_ref();
        }

        default_result.drop_ref();
        instruction_ref.drop_ref();

        return Ok(());
    }

    #[pyo3(signature = (args = None, kwargs=None))]
    fn __call__(
        &mut self,
        args: Option<Vec<PyObject>>,
        kwargs: Option<Bound<'_, PyDict>>,
        py: Python<'_>,
    ) -> PyResult<PyObject> {
        if self.lambda_object.is_none() {
            return Err(XlangExecutionError::new_err(format!(
                "Lambda object is not initialized"
            )));
        }
        // 使用空向量作为默认值
        let args_vec_ref = args.unwrap_or_else(|| vec![]);
        let mut args_vec = Vec::with_capacity(args_vec_ref.len());

        for arg in args_vec_ref.iter() {
            args_vec.push(extract_xlang_gc_ref(
                &arg.extract::<PyObject>(py).unwrap().into_bound(py),
            )?);
        }

        let mut arg_tuple = self
            .gc_system
            .borrow_mut()
            .new_object(XLangVMTuple::new(&mut args_vec.iter_mut().collect()));

        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs.iter() {
                let mut key_str = self
                    .gc_system
                    .borrow_mut()
                    .new_object(XLangVMString::new(&key.extract::<String>().unwrap()));
                let mut value_ref =
                    extract_xlang_gc_ref(&value.extract::<PyObject>().unwrap().into_bound(py))?;

                let mut keyval = self
                    .gc_system
                    .borrow_mut()
                    .new_object(XLangVMNamed::new(&mut key_str, &mut value_ref));
                key_str.drop_ref();
                value_ref.drop_ref();

                if arg_tuple
                    .as_type::<XLangVMTuple>()
                    .append(&mut keyval)
                    .is_err()
                {
                    for arg in args_vec.iter_mut() {
                        arg.drop_ref();
                    }
                    arg_tuple.drop_ref();
                    keyval.drop_ref();
                    return Err(XlangExecutionError::new_err(format!(
                        "Failed to append keyval to tuple"
                    )));
                }
                keyval.drop_ref();
            }
        }

        let mut coroutine_pool = VMCoroutinePool::new(true);

        let coro_id = coroutine_pool.new_coroutine(
            &mut self.lambda_object.as_mut().unwrap().clone_ref(),
            &mut arg_tuple,
            &mut self.gc_system.borrow_mut(),
        );
        if coro_id.is_err() {
            return Err(XlangExecutionError::new_err(format!(
                "Failed to create coroutine: {}",
                {
                    let mut e = coro_id.err().unwrap();
                    e.consume_ref();
                    for arg in args_vec.iter_mut() {
                        arg.drop_ref();
                    }
                    e.to_string()
                }
            )));
        }
        let _coro_id = coro_id.unwrap();

        let result = coroutine_pool.run_until_finished(&mut self.gc_system.borrow_mut());
        if result.is_err() {
            return Err(XlangExecutionError::new_err(format!(
                "Failed to run coroutine: {}",
                {
                    let mut e = result.err().unwrap();
                    e.consume_ref();
                    for arg in args_vec.iter_mut() {
                        arg.drop_ref();
                    }
                    e.to_string()
                }
            )));
        }

        let result = self
            .lambda_object
            .as_mut()
            .unwrap()
            .as_type::<XLangVMLambda>()
            .get_value();

        for arg in args_vec.iter_mut() {
            arg.drop_ref();
        }

        let py_object = xlang_gc_ref_to_py_object(result, self.gc_system.clone(), py)?;

        Ok(py_object)
    }

    fn __repr__(&self, _py: Python<'_>) -> PyResult<String> {
        if self.lambda_object.is_none() {
            return Err(XlangExecutionError::new_err(format!(
                "Lambda object is not initialized"
            )));
        }
        let lambda = self.lambda_object.as_ref().unwrap();
        let repr = format!("<xlang lambda object at {:p}>", lambda);
        Ok(repr)
    }
}

#[pyclass(unsendable)]
#[derive(Clone)]
pub struct WrappedPyFunction {
    pub(crate) gc_system: Arc<RefCell<XlangGCSystem>>,
    pub(crate) function_object: Option<GCRef>,
}

impl WrappedPyFunction {
    pub(crate) fn create(gc: &mut GCSystem) -> Self {
        WrappedPyFunction {
            function_object: None,
            gc_system: Arc::clone(&gc.gc_system),
        }
    }
}

impl Drop for WrappedPyFunction {
    fn drop(&mut self) {
        if let Some(ref mut function) = self.function_object {
            function.drop_ref();
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
struct PackedCallableContext {
    callable_ref: usize,  // 使用整数代替裸指针
    cloned_gc_arc: usize, // 同样使用整数代替裸指针
}

#[pymethods]
impl WrappedPyFunction {
    #[new]
    fn new(gc: &mut GCSystem) -> Self {
        WrappedPyFunction {
            function_object: None,
            gc_system: Arc::clone(&gc.gc_system),
        }
    }

    fn wrap(
        &mut self,
        py_callable: PyObject,
        default_args: &mut VMTuple,
        _py: Python<'_>,
    ) -> PyResult<()> {
        // 释放旧引用(如果有的话)
        if let Some(ref mut old_function) = self.function_object {
            old_function.drop_ref();
            self.function_object = None;
        }

        // 将Python可调用对象存储在Arc中以安全地在多个地方共享
        let callable_ref = Arc::new(PyObject::from(py_callable));
        let gc_system_arc = Arc::clone(&self.gc_system);

        // 创建上下文并序列化为字节
        let context = PackedCallableContext {
            callable_ref: Arc::as_ptr(&callable_ref) as usize, // 将Arc的裸指针存储为整数
            cloned_gc_arc: Arc::as_ptr(&gc_system_arc) as usize, // 同样处理gc_system_arc
        };
        let serialized_context = bincode::serialize(&context).unwrap();

        // 确保Arc不会被提前释放
        let _ = Arc::clone(&callable_ref);
        let _ = Arc::clone(&gc_system_arc);

        // 定义静态函数
        fn py_function_static(
            _self_object: Option<&mut GCRef>,
            capture: Option<&mut GCRef>,
            args: &mut GCRef,
            _gc_system: &mut XlangGCSystem,
        ) -> Result<GCRef, VMVariableError> {
            if capture.is_none() {
                return Err(VMVariableError::TypeError(
                    args.clone_ref(),
                    "missing captured context".to_string(),
                ));
            }
            let capture_ref = capture.unwrap();
            if !capture_ref.isinstance::<XLangVMBytes>() {
                return Err(VMVariableError::TypeError(
                    args.clone_ref(),
                    "expected bytes for captured context".to_string(),
                ));
            }
            let capture_bytes = capture_ref.as_type::<XLangVMBytes>();

            // 解包上下文
            let context: PackedCallableContext =
                bincode::deserialize(&capture_bytes.value).unwrap();

            // 重新构造Arc
            let callable_ref = unsafe { Arc::from_raw(context.callable_ref as *const PyObject) };
            let gc_system_arc =
                unsafe { Arc::from_raw(context.cloned_gc_arc as *const RefCell<XlangGCSystem>) };

            // 确保Arc不会被提前释放
            let callable_ref_clone = Arc::clone(&callable_ref);
            let gc_system_arc_clone = Arc::clone(&gc_system_arc);

            Python::with_gil(|py| {
                // 确保参数是一个元组
                if !args.isinstance::<XLangVMTuple>() {
                    return Err(VMVariableError::TypeError(
                        args.clone_ref(),
                        "expected tuple for arguments".to_string(),
                    ));
                }

                // 获取传递给函数的参数列表
                let args_tuple = args.as_type::<XLangVMTuple>();
                let mut py_args = Vec::new();

                // 将XLang参数转换为Python对象
                for arg_ref in &mut args_tuple.values {
                    let py_arg =
                        match xlang_gc_ref_to_py_object(arg_ref, gc_system_arc_clone.clone(), py) {
                            Ok(obj) => obj,
                            Err(e) => {
                                return Err(VMVariableError::DetailedError(format!(
                                    "Failed to convert argument to Python: {}",
                                    e
                                )));
                            }
                        };
                    py_args.push(py_arg);
                }

                // 调用Python函数
                let py_tuple = match PyTuple::new(py, &py_args) {
                    Ok(tuple) => tuple,
                    Err(e) => {
                        return Err(VMVariableError::DetailedError(format!(
                            "Failed to create Python tuple: {}",
                            e
                        )));
                    }
                };
                match callable_ref_clone.call1(py, py_tuple) {
                    Ok(py_result) => {
                        let bound_result = py_result.into_bound(py);
                        extract_xlang_gc_ref(&bound_result).map_err(|e| {
                            VMVariableError::DetailedError(format!(
                                "Failed to convert Python result to XLang: {}",
                                e
                            ))
                        })
                    }
                    Err(e) => Err(VMVariableError::DetailedError(format!(
                        "Python function call failed: {}",
                        e
                    ))),
                }
            })
        }

        // 创建一个新的XLang函数对象
        let mut packed_context = self
            .gc_system
            .borrow_mut()
            .new_object(XLangVMBytes::new(&serialized_context));

        let mut default_result = self.gc_system.borrow_mut().new_object(XLangVMNull::new());
        let function_object = self.gc_system.borrow_mut().new_object(XLangVMLambda::new(
            0,
            "<python>".to_string(),
            &mut default_args.gc_ref,
            Some(&mut packed_context),
            None,
            &mut XLangVMLambdaBody::VMNativeFunction(py_function_static),
            &mut default_result,
            false,
        ));
        default_result.drop_ref();
        packed_context.drop_ref();

        self.function_object = Some(function_object);

        // 确保Arc不会被提前释放
        std::mem::forget(callable_ref);
        std::mem::forget(gc_system_arc);

        Ok(())
    }
    fn __repr__(&self, _py: Python<'_>) -> PyResult<String> {
        if self.function_object.is_none() {
            return Err(XlangExecutionError::new_err(format!(
                "Function object is not initialized"
            )));
        }
        let function = self.function_object.as_ref().unwrap();
        let repr = format!("<xlang wrapped function object at {:p}>", function);
        Ok(repr)
    }
}
