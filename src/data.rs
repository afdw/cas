use crate::Value;
use std::collections::HashMap;

pub struct ExecutionContext {
    pub values: HashMap<Value, Value>,
    #[allow(clippy::type_complexity)]
    pub intrinsics: HashMap<Value, fn(&mut ExecutionContext, Vec<Value>) -> Value>,
}

pub struct HoldValueInner {
    pub inner: Value,
}

pub struct NullValueInner;

pub struct SymbolValueInner {
    pub name: String,
}

pub struct ExecutionValueInner {
    pub inner: Value,
}

pub struct ExecutableSequenceValueInner {
    pub inner: Vec<Value>,
}

pub struct AssignmentValueInner {
    pub source: Value,
    pub target: Value,
}

pub struct ExecutableFunctionValueInner {
    pub arguments: Vec<Value>,
    pub body: Value,
}

pub struct FunctionApplicationValueInner {
    pub function: Value,
    pub arguments: Vec<Value>,
}

pub struct IntrinsicCallValueInner {
    pub intrinsic: Value,
    pub arguments: Vec<Value>,
}

pub struct FloatingPointNumberValueInner {
    pub value: f64,
}

pub fn evaluate(execution_context: &mut ExecutionContext, value: Value) -> Value {
    if let Some(value_inner) = value.try_downcast::<ExecutionValueInner>() {
        execute(execution_context, value_inner.inner.clone())
    } else if let Some(value_inner) = value.try_downcast::<ExecutableSequenceValueInner>() {
        let inner = value_inner.inner.iter().map(|x| evaluate(execution_context, x.clone())).collect();
        if inner == value_inner.inner {
            value
        } else {
            Value::new(ExecutableSequenceValueInner { inner })
        }
    } else if let Some(value_inner) = value.try_downcast::<AssignmentValueInner>() {
        let source = evaluate(execution_context, value_inner.source.clone());
        let target = evaluate(execution_context, value_inner.target.clone());
        if source == value_inner.source && target == value_inner.target {
            value
        } else {
            Value::new(AssignmentValueInner { source, target })
        }
    } else if let Some(value_inner) = value.try_downcast::<ExecutableFunctionValueInner>() {
        let arguments = value_inner.arguments.iter().map(|x| evaluate(execution_context, x.clone())).collect();
        let body = evaluate(execution_context, value_inner.body.clone());
        if arguments == value_inner.arguments && body == value_inner.body {
            value
        } else {
            Value::new(ExecutableFunctionValueInner { arguments, body })
        }
    } else if let Some(value_inner) = value.try_downcast::<FunctionApplicationValueInner>() {
        let function = evaluate(execution_context, value_inner.function.clone());
        let arguments = value_inner.arguments.iter().map(|x| evaluate(execution_context, x.clone())).collect();
        if function == value_inner.function && arguments == value_inner.arguments {
            value
        } else {
            Value::new(FunctionApplicationValueInner { function, arguments })
        }
    } else if let Some(value_inner) = value.try_downcast::<IntrinsicCallValueInner>() {
        let intrinsic = evaluate(execution_context, value_inner.intrinsic.clone());
        let arguments = value_inner.arguments.iter().map(|x| evaluate(execution_context, x.clone())).collect();
        if intrinsic == value_inner.intrinsic && arguments == value_inner.arguments {
            value
        } else {
            Value::new(IntrinsicCallValueInner { intrinsic, arguments })
        }
    } else {
        value
    }
}

pub fn execute(execution_context: &mut ExecutionContext, value: Value) -> Value {
    let value = evaluate(execution_context, value);
    if value.is::<SymbolValueInner>() {
        execution_context.values.get(&value).cloned().unwrap_or(value)
    } else if let Some(value_inner) = value.try_downcast::<HoldValueInner>() {
        evaluate(execution_context, value_inner.inner.clone())
    } else if let Some(value_inner) = value.try_downcast::<ExecutableSequenceValueInner>() {
        value_inner
            .inner
            .iter()
            .map(|x| execute(execution_context, x.clone()))
            .last()
            .unwrap_or_else(|| Value::new(NullValueInner))
    } else if let Some(value_inner) = value.try_downcast::<AssignmentValueInner>() {
        assert!(value_inner.source.is::<SymbolValueInner>());
        execution_context.values.insert(value_inner.target.clone(), value_inner.source.clone());
        Value::new(NullValueInner)
    } else if let Some(value_inner) = value.try_downcast::<FunctionApplicationValueInner>() {
        let function = value_inner.function.downcast::<ExecutableFunctionValueInner>();
        assert_eq!(function.arguments.len(), value_inner.arguments.len());
        let mut result = function.body.clone();
        for (from, to) in function.arguments.iter().cloned().zip(value_inner.arguments.clone()) {
            assert!(from.is::<SymbolValueInner>());
            result = replace(result, from, to);
        }
        assert!(result.is::<HoldValueInner>());
        execute(execution_context, result)
    } else if let Some(value_inner) = value.try_downcast::<IntrinsicCallValueInner>() {
        (execution_context.intrinsics.get(&value_inner.intrinsic).unwrap())(execution_context, value_inner.arguments.clone())
    } else {
        unreachable!()
    }
}

pub fn replace(value: Value, from: Value, to: Value) -> Value {
    if value == from {
        to
    } else if let Some(value_inner) = value.try_downcast::<HoldValueInner>() {
        let inner = replace(value_inner.inner.clone(), from, to);
        if inner == value_inner.inner {
            value
        } else {
            Value::new(HoldValueInner { inner })
        }
    } else if let Some(value_inner) = value.try_downcast::<ExecutionValueInner>() {
        let inner = replace(value_inner.inner.clone(), from, to);
        if inner == value_inner.inner {
            value
        } else {
            Value::new(ExecutionValueInner { inner })
        }
    } else if let Some(value_inner) = value.try_downcast::<ExecutableSequenceValueInner>() {
        let inner = value_inner.inner.iter().map(|x| replace(x.clone(), from.clone(), to.clone())).collect();
        if inner == value_inner.inner {
            value
        } else {
            Value::new(ExecutableSequenceValueInner { inner })
        }
    } else if let Some(value_inner) = value.try_downcast::<AssignmentValueInner>() {
        let source = replace(value_inner.source.clone(), from.clone(), to.clone());
        let target = replace(value_inner.target.clone(), from, to);
        if source == value_inner.source && target == value_inner.target {
            value
        } else {
            Value::new(AssignmentValueInner { source, target })
        }
    } else if let Some(value_inner) = value.try_downcast::<ExecutableFunctionValueInner>() {
        let arguments = value_inner.arguments.iter().map(|x| replace(x.clone(), from.clone(), to.clone())).collect();
        let body = replace(value_inner.body.clone(), from, to);
        if arguments == value_inner.arguments && body == value_inner.body {
            value
        } else {
            Value::new(ExecutableFunctionValueInner { arguments, body })
        }
    } else if let Some(value_inner) = value.try_downcast::<FunctionApplicationValueInner>() {
        let function = replace(value_inner.function.clone(), from.clone(), to.clone());
        let arguments = value_inner.arguments.iter().map(|x| replace(x.clone(), from.clone(), to.clone())).collect();
        if function == value_inner.function && arguments == value_inner.arguments {
            value
        } else {
            Value::new(FunctionApplicationValueInner { function, arguments })
        }
    } else if let Some(value_inner) = value.try_downcast::<IntrinsicCallValueInner>() {
        let intrinsic = replace(value_inner.intrinsic.clone(), from.clone(), to.clone());
        let arguments = value_inner.arguments.iter().map(|x| replace(x.clone(), from.clone(), to.clone())).collect();
        if intrinsic == value_inner.intrinsic && arguments == value_inner.arguments {
            value
        } else {
            Value::new(IntrinsicCallValueInner { intrinsic, arguments })
        }
    } else {
        value
    }
}
