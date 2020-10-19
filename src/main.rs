mod data;
mod value;

use data::*;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
};
pub use value::Value;

macro_rules! symbol {
    ($name:ident) => {
        let $name = Value::new(SymbolValueInner {
            name: stringify!($name).to_owned(),
        });
    };
}

macro_rules! intrinsic {
    (($execution_context:ident$(,)? $($arguments:ident),*) $body:expr) => {{
        fn f(#[allow(unused_variables)] $execution_context: &mut ExecutionContext, arguments: Vec<Value>) -> Value {
            let mut arguments = arguments.into_iter();
            $(let $arguments = arguments.next().unwrap();)*
            assert!(arguments.next().is_none());
            $body
        }
        f as fn(&mut ExecutionContext, Vec<Value>) -> Value
    }}
}

fn main() {
    let mut intrinsics = HashMap::new();
    macro_rules! define_intrinsic {
        ($name:ident => $($definition:tt)*) => {
            symbol!($name);
            intrinsics.insert($name.clone(), intrinsic!($($definition)*));
        };
    }
    define_intrinsic!(intrinsic_push => (execution_context, a) {
        execution_context.stack.push(a);
        Value::new(NullValueInner)
    });
    define_intrinsic!(intrinsic_pop => (execution_context) {
        execution_context.stack.pop().unwrap()
    });
    define_intrinsic!(intrinsic_print_hash => (execution_context, a) {
        let mut hasher = DefaultHasher::new();
        a.hash(&mut hasher);
        println!("{}", hasher.finish());
        Value::new(NullValueInner)
    });
    define_intrinsic!(intrinsic_floating_point_number_add => (execution_context, a, b) {
        Value::new(FloatingPointNumberValueInner {
            inner: a.downcast::<FloatingPointNumberValueInner>().inner + b.downcast::<FloatingPointNumberValueInner>().inner,
        })
    });
    let mut execution_context = ExecutionContext {
        intrinsics,
        values: HashMap::new(),
        stack: Vec::new(),
    };
    symbol!(function_dynamic_scope);
    symbol!(argument_symbol);
    symbol!(argument_inner);
    evaluate(
        &mut execution_context,
        Value::new(ExecutableSequenceValueInner {
            inner: vec![Value::new(ExecutionValueInner {
                inner: Value::new(AssignmentValueInner {
                    source: Value::new(ExecutableFunctionValueInner {
                        arguments: vec![argument_symbol.clone(), argument_inner.clone()],
                        body: Value::new(HoldValueInner {
                            inner: Value::new(ExecutableSequenceValueInner {
                                inner: vec![
                                    Value::new(ExecutionValueInner {
                                        inner: Value::new(IntrinsicCallValueInner {
                                            intrinsic: intrinsic_push,
                                            arguments: vec![Value::new(AssignmentValueInner {
                                                source: Value::new(ExecutionValueInner {
                                                    inner: argument_symbol.clone(),
                                                }),
                                                target: argument_symbol,
                                            })],
                                        }),
                                    }),
                                    Value::new(ExecutionValueInner { inner: argument_inner }),
                                    Value::new(ExecutionValueInner {
                                        inner: Value::new(ExecutionValueInner {
                                            inner: Value::new(IntrinsicCallValueInner {
                                                intrinsic: intrinsic_pop,
                                                arguments: vec![],
                                            }),
                                        }),
                                    }),
                                    Value::new(NullValueInner),
                                ],
                            }),
                        }),
                    }),
                    target: function_dynamic_scope.clone(),
                }),
            })],
        }),
    );
    symbol!(variable_a);
    evaluate(
        &mut execution_context,
        Value::new(ExecutableSequenceValueInner {
            inner: vec![
                Value::new(ExecutionValueInner {
                    inner: Value::new(AssignmentValueInner {
                        source: Value::new(FloatingPointNumberValueInner { inner: 1.0 }),
                        target: variable_a.clone(),
                    }),
                }),
                Value::new(ExecutionValueInner {
                    inner: Value::new(IntrinsicCallValueInner {
                        intrinsic: intrinsic_print_hash.clone(),
                        arguments: vec![Value::new(ExecutionValueInner { inner: variable_a.clone() })],
                    }),
                }),
                Value::new(ExecutionValueInner {
                    inner: Value::new(FunctionApplicationValueInner {
                        function: Value::new(ExecutionValueInner { inner: function_dynamic_scope }),
                        arguments: vec![
                            variable_a.clone(),
                            Value::new(HoldValueInner {
                                inner: Value::new(ExecutionValueInner {
                                    inner: Value::new(ExecutableSequenceValueInner {
                                        inner: vec![
                                            Value::new(ExecutionValueInner {
                                                inner: Value::new(AssignmentValueInner {
                                                    source: Value::new(FloatingPointNumberValueInner { inner: 2.0 }),
                                                    target: variable_a.clone(),
                                                }),
                                            }),
                                            Value::new(ExecutionValueInner {
                                                inner: Value::new(IntrinsicCallValueInner {
                                                    intrinsic: intrinsic_print_hash.clone(),
                                                    arguments: vec![Value::new(ExecutionValueInner { inner: variable_a.clone() })],
                                                }),
                                            }),
                                        ],
                                    }),
                                }),
                            }),
                        ],
                    }),
                }),
                Value::new(ExecutionValueInner {
                    inner: Value::new(IntrinsicCallValueInner {
                        intrinsic: intrinsic_print_hash,
                        arguments: vec![Value::new(ExecutionValueInner { inner: variable_a })],
                    }),
                }),
            ],
        }),
    );
    println!(
        "{:?}",
        evaluate(
            &mut execution_context,
            Value::new(ExecutionValueInner {
                inner: Value::new(IntrinsicCallValueInner {
                    intrinsic: intrinsic_floating_point_number_add,
                    arguments: vec![
                        Value::new(FloatingPointNumberValueInner { inner: 2.0 }),
                        Value::new(FloatingPointNumberValueInner { inner: 3.0 }),
                    ],
                })
            }),
        )
        .downcast::<FloatingPointNumberValueInner>()
        .inner
    );
}
