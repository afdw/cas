mod data;
mod value;

use data::*;
use std::collections::HashMap;
pub use value::Value;

macro_rules! intrinsic {
    (($execution_context:ident, $($arguments:ident),*) $body:expr) => {{
        fn f(#[allow(unused_variables)] $execution_context: &mut ExecutionContext, arguments: Vec<Value>) -> Value {
            let mut arguments = arguments.into_iter();
            $(let $arguments = arguments.next().unwrap());*;
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
            let $name = Value::new(SymbolValueInner { name: stringify!($name).to_owned() });
            intrinsics.insert($name.clone(), intrinsic!($($definition)*));
        };
    }
    define_intrinsic!(floating_point_number_add_intrinsic => (execution_context, a, b) {
        Value::new(FloatingPointNumberValueInner {
            inner: a.downcast::<FloatingPointNumberValueInner>().inner + b.downcast::<FloatingPointNumberValueInner>().inner,
        })
    });
    let mut execution_context = ExecutionContext {
        intrinsics,
        values: HashMap::new(),
        stack: Vec::new(),
    };
    dbg!(
        execute(
            &mut execution_context,
            Value::new(IntrinsicCallValueInner {
                intrinsic: floating_point_number_add_intrinsic,
                arguments: vec![
                    Value::new(FloatingPointNumberValueInner { inner: 2.0 }),
                    Value::new(FloatingPointNumberValueInner { inner: 3.0 }),
                ],
            }),
        )
        .downcast::<FloatingPointNumberValueInner>()
        .inner
    );
}
