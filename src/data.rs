use crate::Value;
use std::collections::HashMap;

pub struct ExecutionContext {
    pub values: HashMap<Value, Value>,
    #[allow(clippy::type_complexity)]
    pub intrinsics: HashMap<Value, fn(&mut ExecutionContext, Vec<Value>) -> Value>,
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

pub struct HeldArgument {
    pub inner: Value,
}

pub struct EvaluatedArgument {
    pub inner: Value,
}

pub struct HoldableFunctionApplicationValueInner {
    pub function: Value,
    pub arguments: Vec<Value>,
}

pub struct FunctionApplicationValueInner {
    pub function: Value,
    pub arguments: Vec<Value>,
}

pub struct HoldableIntrinsicCallValueInner {
    pub intrinsic: Value,
    pub arguments: Vec<Value>,
}

pub struct FloatingPointNumberValueInner {
    pub value: f64,
}

pub fn evaluate(execution_context: &mut ExecutionContext, value: Value) -> Value {
    fn evaluate_arguments(execution_context: &mut ExecutionContext, arguments: &[Value]) -> Vec<Value> {
        arguments
            .iter()
            .map(|x| {
                if x.is::<HeldArgument>() {
                    x.clone()
                } else if let Some(y) = x.try_downcast::<EvaluatedArgument>() {
                    let inner = evaluate(execution_context, y.inner.clone());
                    if inner == y.inner {
                        x.clone()
                    } else {
                        Value::new(EvaluatedArgument { inner })
                    }
                } else {
                    unreachable!()
                }
            })
            .collect()
    }

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
        let arguments = evaluate_arguments(execution_context, &value_inner.arguments);
        let body = evaluate(execution_context, value_inner.body.clone());
        if arguments == value_inner.arguments && body == value_inner.body {
            value
        } else {
            Value::new(ExecutableFunctionValueInner { arguments, body })
        }
    } else if let Some(value_inner) = value.try_downcast::<HoldableFunctionApplicationValueInner>() {
        let function = evaluate(execution_context, value_inner.function.clone());
        let arguments = evaluate_arguments(execution_context, &value_inner.arguments);
        if function == value_inner.function && arguments == value_inner.arguments {
            value
        } else {
            Value::new(HoldableFunctionApplicationValueInner { function, arguments })
        }
    } else if let Some(value_inner) = value.try_downcast::<FunctionApplicationValueInner>() {
        let function = evaluate(execution_context, value_inner.function.clone());
        let arguments = value_inner.arguments.iter().map(|x| evaluate(execution_context, x.clone())).collect();
        if function == value_inner.function && arguments == value_inner.arguments {
            value
        } else {
            Value::new(FunctionApplicationValueInner { function, arguments })
        }
    } else if let Some(value_inner) = value.try_downcast::<HoldableIntrinsicCallValueInner>() {
        let intrinsic = evaluate(execution_context, value_inner.intrinsic.clone());
        let arguments = evaluate_arguments(execution_context, &value_inner.arguments);
        if intrinsic == value_inner.intrinsic && arguments == value_inner.arguments {
            value
        } else {
            Value::new(HoldableIntrinsicCallValueInner { intrinsic, arguments })
        }
    } else {
        value
    }
}

pub fn execute(execution_context: &mut ExecutionContext, value: Value) -> Value {
    let value = evaluate(execution_context, value);
    if value.is::<SymbolValueInner>() {
        execution_context.values.get(&value).cloned().unwrap_or(value)
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
    } else if let Some(value_inner) = value.try_downcast::<HoldableFunctionApplicationValueInner>() {
        let function = value_inner.function.downcast::<ExecutableFunctionValueInner>();
        let arguments = value_inner
            .arguments
            .iter()
            .map(|x| {
                if let Some(y) = x.try_downcast::<EvaluatedArgument>() {
                    y.inner.clone()
                } else if let Some(y) = x.try_downcast::<EvaluatedArgument>() {
                    y.inner.clone()
                } else {
                    unreachable!()
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(function.arguments.len(), arguments.len());
        let mut result = function.body.clone();
        for (from, to) in function.arguments.iter().cloned().zip(arguments) {
            assert!(from.is::<SymbolValueInner>());
            result = replace(result, from, to);
        }
        result
    } else if let Some(value_inner) = value.try_downcast::<FunctionApplicationValueInner>() {
        execute(
            execution_context,
            Value::new(FunctionApplicationValueInner {
                function: value_inner.function.clone(),
                arguments: value_inner
                    .arguments
                    .iter()
                    .map(|x| Value::new(EvaluatedArgument { inner: x.clone() }))
                    .collect(),
            }),
        )
    } else if let Some(value_inner) = value.try_downcast::<HoldableIntrinsicCallValueInner>() {
        (execution_context.intrinsics.get(&value_inner.intrinsic).unwrap())(execution_context, value_inner.arguments.clone())
    } else {
        unreachable!()
    }
}

pub fn replace(value: Value, from: Value, to: Value) -> Value {
    if value == from {
        to
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
    } else if let Some(value_inner) = value.try_downcast::<HoldableFunctionApplicationValueInner>() {
        let function = replace(value_inner.function.clone(), from.clone(), to.clone());
        let arguments = value_inner.arguments.iter().map(|x| replace(x.clone(), from.clone(), to.clone())).collect();
        if function == value_inner.function && arguments == value_inner.arguments {
            value
        } else {
            Value::new(HoldableFunctionApplicationValueInner { function, arguments })
        }
    } else if let Some(value_inner) = value.try_downcast::<FunctionApplicationValueInner>() {
        let function = replace(value_inner.function.clone(), from.clone(), to.clone());
        let arguments = value_inner.arguments.iter().map(|x| replace(x.clone(), from.clone(), to.clone())).collect();
        if function == value_inner.function && arguments == value_inner.arguments {
            value
        } else {
            Value::new(FunctionApplicationValueInner { function, arguments })
        }
    } else if let Some(value_inner) = value.try_downcast::<HoldableIntrinsicCallValueInner>() {
        let intrinsic = replace(value_inner.intrinsic.clone(), from.clone(), to.clone());
        let arguments = value_inner.arguments.iter().map(|x| replace(x.clone(), from.clone(), to.clone())).collect();
        if intrinsic == value_inner.intrinsic && arguments == value_inner.arguments {
            value
        } else {
            Value::new(HoldableIntrinsicCallValueInner { intrinsic, arguments })
        }
    } else {
        value
    }
}
