//! Callbacks remain rooted and execute on their originating JavaScript thread.
use napi::bindgen_prelude::{Function as JsFunction, FunctionRef, ToNapiValue, Unknown};
use napi::{Env, Error, Result, ValueType};
use std::rc::Rc;
pub(crate) enum Value {
    Number(f64),
    Other,
}
impl Value {
    pub(crate) fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            Self::Other => None,
        }
    }
    fn from_result(value: Unknown<'_>) -> Result<Self> {
        if value.get_type()? != ValueType::Number {
            return Ok(Self::Other);
        }
        // SAFETY: this local handle was checked to contain a number above.
        Ok(Self::Number(unsafe { value.cast::<f64>()? }))
    }
}
struct RootedFunction {
    env: Env,
    reference: FunctionRef<(), Unknown<'static>>,
}
#[derive(Clone)]
pub(crate) struct Function {
    root: Rc<RootedFunction>,
}
impl Function {
    pub(crate) fn new(env: Env, value: Unknown<'_>) -> Result<Self> {
        if value.get_type()? != ValueType::Function {
            return Err(Error::from_reason("callback must be callable"));
        }
        // SAFETY: the checked function is immediately rooted in this environment.
        let function = unsafe { value.cast::<JsFunction<'_, (), Unknown<'static>>>()? };
        Ok(Self {
            root: Rc::new(RootedFunction {
                env,
                reference: function.create_ref()?,
            }),
        })
    }
    pub(crate) fn call0(&self, _receiver: &()) -> Result<Value> {
        let function = self.root.reference.borrow_back(&self.root.env)?;
        Value::from_result(function.apply((), ())?)
    }
}
impl Function {
    pub(crate) fn call_report(&self, report: crate::api::IterationReport) -> Result<()> {
        let function = self
            .root
            .reference
            .borrow_back(&self.root.env)?
            .into_unknown(&self.root.env)?;
        // SAFETY: the rooted value was validated as a function in new().
        let function =
            unsafe { function.cast::<JsFunction<'_, crate::IterationReport, Unknown<'_>>>()? };
        function.apply(
            (),
            crate::IterationReport {
                inner: std::cell::RefCell::new(Some(report)),
            },
        )?;
        Ok(())
    }
}
