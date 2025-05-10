use std::sync::Arc;

use crate::arc_unsafe_refcell::Inner;
use crate::{
    extract_xlang_gc_ref_with_gc, extract_xlang_gc_ref_with_gc_arc, xlang_gc_ref_to_py_object,
    ArcUnsafeRefCellWrapper, GCSystem, VMTuple, XlangCompilationError, XlangExecutionError,
};
use pyo3::types::{PyDict, PyTuple};
use pyo3::{exceptions::PyIOError, prelude::*};
use xlang_frontend::{compile::build_code, dir_stack::DirStack};
use xlang_vm_core::executor::vm::{VMCoroutinePool, VMError};
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
    gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
    lambda_object: Option<GCRef>,
    run_condition: Option<Arc<PyObject>>,
}

impl Lambda {
    pub(crate) fn create(gc: &mut GCSystem) -> Self {
        Lambda {
            lambda_object: None,
            gc_system: gc.gc_system.clone(),
            run_condition: None,
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
            gc_system: gc.gc_system.clone(),
            run_condition: None,
        }
    }

    // 失败时返回错误信息
    #[pyo3(signature = (code, default_args, capture=None, self_object=None, work_dir=None, run_condition=None))]
    fn load(
        &mut self,
        code: &str,
        default_args: &mut VMTuple,
        capture: Option<PyObject>,
        self_object: Option<PyObject>,
        work_dir: Option<&str>,
        run_condition: Option<PyObject>,
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
            Some(extract_xlang_gc_ref_with_gc_arc(
                &c.into_bound(py),
                self.gc_system.clone(),
            )?)
        } else {
            None
        };

        let mut self_object_ref_option: Option<GCRef> = if let Some(s) = self_object {
            Some(extract_xlang_gc_ref_with_gc_arc(
                &s.into_bound(py),
                self.gc_system.clone(),
            )?)
        } else {
            None
        };

        let mut default_result = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XLangVMNull::new()),
            Err(e) => {
                return Err(XlangExecutionError::new_err(format!(
                    "Failed to create default result: {}",
                    e
                )))
            }
        };
        let mut instruction_ref = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => {
                gc_system.new_object(VMInstructions::new(&instruction_package.unwrap()))
            }
            Err(e) => {
                return Err(XlangExecutionError::new_err(format!(
                    "Failed to create instruction reference: {}",
                    e
                )))
            }
        };

        let lambda: GCRef = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XLangVMLambda::new(
                0,
                "__main__".to_string(),
                &mut default_args.gc_ref,
                capture_ref_option.as_mut(),
                self_object_ref_option.as_mut(),
                &mut XLangVMLambdaBody::VMInstruction(instruction_ref.clone()),
                &mut default_result,
                false,
            )),
            Err(e) => {
                return Err(XlangExecutionError::new_err(format!(
                    "Failed to create lambda object: {}",
                    e
                )))
            }
        };

        let mut old_ref = self.lambda_object.take();
        self.lambda_object = Some(lambda);
        if let Some(ref mut old_lambda) = old_ref {
            old_lambda.drop_ref();
        }

        default_result.drop_ref();
        instruction_ref.drop_ref();

        self.run_condition = match run_condition {
            Some(condition_func) => Some(Arc::new(condition_func)),
            None => None,
        };
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
            args_vec.push(extract_xlang_gc_ref_with_gc_arc(
                &arg.extract::<PyObject>(py).unwrap().into_bound(py),
                self.gc_system.clone(),
            )?);
        }

        let mut arg_tuple = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => {
                gc_system.new_object(XLangVMTuple::new(&mut args_vec.iter_mut().collect()))
            }
            Err(e) => {
                for arg in args_vec.iter_mut() {
                    arg.drop_ref();
                }
                return Err(XlangExecutionError::new_err(format!(
                    "Failed to create argument tuple: {}",
                    e
                )));
            }
        };

        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs.iter() {
                let mut key_str = match self.gc_system.borrow_mut() {
                    Ok(mut gc_system) => {
                        gc_system.new_object(XLangVMString::new(&key.extract::<String>().unwrap()))
                    }
                    Err(e) => {
                        for arg in args_vec.iter_mut() {
                            arg.drop_ref();
                        }
                        arg_tuple.drop_ref();
                        return Err(XlangExecutionError::new_err(format!(
                            "Failed to create key string: {}",
                            e
                        )));
                    }
                };

                let mut value_ref = extract_xlang_gc_ref_with_gc_arc(
                    &value.extract::<PyObject>().unwrap().into_bound(py),
                    self.gc_system.clone(),
                )?;

                let mut keyval = match self.gc_system.borrow_mut() {
                    Ok(mut gc_system) => {
                        gc_system.new_object(XLangVMNamed::new(&mut key_str, &mut value_ref))
                    }
                    Err(e) => {
                        for arg in args_vec.iter_mut() {
                            arg.drop_ref();
                        }
                        arg_tuple.drop_ref();
                        key_str.drop_ref();
                        value_ref.drop_ref();
                        return Err(XlangExecutionError::new_err(format!(
                            "Failed to create named value: {}",
                            e
                        )));
                    }
                };

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

        let assgined = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => self
                .lambda_object
                .as_mut()
                .unwrap()
                .as_type::<XLangVMLambda>()
                .default_args_tuple
                .as_type::<XLangVMTuple>()
                .clone_and_assign_members(&mut arg_tuple, &mut gc_system),
            Err(e) => {
                for arg in args_vec.iter_mut() {
                    arg.drop_ref();
                }
                arg_tuple.drop_ref();
                return Err(XlangExecutionError::new_err(format!(
                    "Failed to borrow GC system for arguments assignment: {}",
                    e
                )));
            }
        };

        arg_tuple.drop_ref();
        if assgined.is_err() {
            for arg in args_vec.iter_mut() {
                arg.drop_ref();
            }
            return Err(XlangExecutionError::new_err(format!(
                "Failed to assign arguments: {}",
                {
                    let mut e = assgined.err().unwrap();
                    e.consume_ref();
                    e.to_string()
                }
            )));
        }
        let mut assgined = assgined.unwrap();

        let coro_id = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => coroutine_pool.new_coroutine(
                &mut self.lambda_object.as_mut().unwrap().clone_ref(),
                &mut assgined,
                &mut gc_system,
            ),
            Err(e) => {
                for arg in args_vec.iter_mut() {
                    arg.drop_ref();
                }
                return Err(XlangExecutionError::new_err(format!(
                    "Failed to borrow GC system for coroutine creation: {}",
                    e
                )));
            }
        };

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

        let result = unsafe {
            coroutine_pool.run_while(self.gc_system.get_mut(), |_| {
                // 使用 run_condition 函数检查
                if let Some(ref condition) = self.run_condition {
                    match condition.call1(py, ()) {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(VMError::DetailedError(
                                format!("Run condition function failed: {}", e),
                            ));
                        }
                    }
                }
                Ok(())
            })
        };

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
    pub(crate) gc_system: ArcUnsafeRefCellWrapper<XlangGCSystem>,
    pub(crate) function_object: Option<GCRef>,
    pub(crate) callable_ref: Option<Arc<PyObject>>,
}

impl WrappedPyFunction {
    pub(crate) fn create(gc: &mut GCSystem) -> Self {
        WrappedPyFunction {
            function_object: None,
            gc_system: gc.gc_system.clone(),
            callable_ref: None,
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
    callable_ref: usize, // 使用整数代替裸指针
    gc_arc: usize,       // 使用整数代替裸指针
}

#[pymethods]
impl WrappedPyFunction {
    #[new]
    fn new(gc: &mut GCSystem) -> Self {
        WrappedPyFunction {
            function_object: None,
            gc_system: gc.gc_system.clone(),
            callable_ref: None,
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
        self.callable_ref = Some(Arc::new(py_callable));
        // 创建上下文并序列化为字节
        let context = PackedCallableContext {
            callable_ref: Arc::as_ptr(self.callable_ref.as_ref().unwrap()) as usize, // 将Arc的裸指针存储为整数
            gc_arc: self.gc_system.get_inner() as usize,
        };
        let serialized_context = bincode::serialize(&context).unwrap();

        // 定义静态函数
        #[allow(deprecated)]
        fn py_function_static(
            _self_object: Option<&mut GCRef>,
            capture: Option<&mut GCRef>,
            args: &mut GCRef,
            gc_system: &mut XlangGCSystem,
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
            std::mem::forget(callable_ref.clone()); // 防止调用时释放

            let gc_system_arc =
                ArcUnsafeRefCellWrapper::from_inner(context.gc_arc as *mut Inner<XlangGCSystem>);

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

                let py_kwargs = PyDict::new(py);

                for arg_ref in &mut args_tuple.values {
                    if arg_ref.isinstance::<XLangVMNamed>()
                        && arg_ref
                            .as_const_type::<XLangVMNamed>()
                            .key
                            .isinstance::<XLangVMString>()
                    {
                        // 解包 VMNamed 为键值对
                        let named = arg_ref.as_type::<XLangVMNamed>();
                        let key = named.key.as_type::<XLangVMString>().value.clone();
                        let py_value = match xlang_gc_ref_to_py_object(
                            &mut named.value,
                            gc_system_arc.clone(),
                            py,
                        ) {
                            Ok(obj) => obj,
                            Err(e) => {
                                return Err(VMVariableError::DetailedError(format!(
                                    "Failed to convert VMNamed value to Python: {}",
                                    e
                                )));
                            }
                        };
                        // 将命名参数添加到kwargs字典中
                        if let Err(e) = py_kwargs.set_item(key, py_value) {
                            return Err(VMVariableError::DetailedError(format!(
                                "Failed to set keyword argument: {}",
                                e
                            )));
                        }
                    } else {
                        // 普通位置参数保持不变
                        let py_arg =
                            match xlang_gc_ref_to_py_object(arg_ref, gc_system_arc.clone(), py) {
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
                }

                // 创建位置参数元组
                let py_tuple = match PyTuple::new(py, &py_args) {
                    Ok(tuple) => tuple,
                    Err(e) => {
                        return Err(VMVariableError::DetailedError(format!(
                            "Failed to create Python tuple: {}",
                            e
                        )));
                    }
                };
                match callable_ref.call(py, py_tuple, Some(&py_kwargs)) {
                    Ok(py_result) => {
                        let bound_result = py_result.into_bound(py);
                        extract_xlang_gc_ref_with_gc(&bound_result, gc_system).map_err(|e| {
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
        let mut packed_context = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XLangVMBytes::new(&serialized_context)),
            Err(e) => {
                return Err(XlangExecutionError::new_err(format!(
                    "Failed to borrow GC system for context creation: {}",
                    e
                )));
            }
        };

        let mut default_result = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XLangVMNull::new()),
            Err(e) => {
                packed_context.drop_ref();
                return Err(XlangExecutionError::new_err(format!(
                    "Failed to borrow GC system for default result creation: {}",
                    e
                )));
            }
        };

        let function_object = match self.gc_system.borrow_mut() {
            Ok(mut gc_system) => gc_system.new_object(XLangVMLambda::new(
                0,
                "<python>".to_string(),
                &mut default_args.gc_ref,
                Some(&mut packed_context),
                None,
                &mut XLangVMLambdaBody::VMNativeFunction(py_function_static),
                &mut default_result,
                false,
            )),
            Err(e) => {
                packed_context.drop_ref();
                default_result.drop_ref();
                return Err(XlangExecutionError::new_err(format!(
                    "Failed to borrow GC system for function creation: {}",
                    e
                )));
            }
        };

        default_result.drop_ref();
        packed_context.drop_ref();

        self.function_object = Some(function_object);

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
