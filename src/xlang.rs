use std::{cell::RefCell, sync::Arc};

use crate::{
    extract_xlang_gc_ref, xlang_gc_ref_to_py_object, GCSystem, VMTuple, XlangCompilationError,
    XlangExecutionError,
};
use pyo3::types::PyDict;
use pyo3::{exceptions::PyIOError, prelude::*};
use xlang_frontend::{compile::build_code, dir_stack::DirStack};
use xlang_vm_core::executor::vm::VMCoroutinePool;
use xlang_vm_core::gc::{GCRef, GCSystem as XlangGCSystem};
use xlang_vm_core::ir_translator::IRTranslator;

use xlang_vm_core::executor::variable::VMLambda as XLangVMLambda;
use xlang_vm_core::executor::variable::VMLambdaBody as XLangVMLambdaBody;
use xlang_vm_core::executor::variable::VMNamed as XLangVMNamed;
use xlang_vm_core::executor::variable::VMNull as XLangVMNull;
use xlang_vm_core::executor::variable::VMString as XLangVMString;
use xlang_vm_core::executor::variable::{VMInstructions, VMTuple as XLangVMTuple};

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
}
