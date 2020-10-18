mod data;
mod value;

use data::*;
use std::collections::HashMap;
pub use value::Value;

fn main() {
    let floating_point_number_add = Value::new(SymbolValueInner {
        name: "floating_point_number_add".into(),
    });
    let mut execution_context = ExecutionContext {
        values: HashMap::new(),
        intrinsics: {
            let mut intrinsics = HashMap::new();
            macro_rules! intrinsic {
                (($execution_context:ident, $($args:ident),*) $body:expr) => {{
                    fn f(#[allow(unused_variables)] $execution_context: &mut ExecutionContext, arguments: Vec<Value>) -> Value {
                        let mut arguments = arguments.into_iter();
                        $(let $args = arguments.next().unwrap());*;
                        assert!(arguments.next().is_none());
                        $body
                    }
                    f as fn(&mut ExecutionContext, Vec<Value>) -> Value
                }}
            }
            intrinsics.insert(
                floating_point_number_add.clone(),
                intrinsic!((execution_context, a, b) {
                    Value::new(FloatingPointNumberValueInner {
                        inner: a.downcast::<FloatingPointNumberValueInner>().inner + b.downcast::<FloatingPointNumberValueInner>().inner,
                    })
                }),
            );
            intrinsics
        },
    };
    dbg!(
        execute(
            &mut execution_context,
            Value::new(IntrinsicCallValueInner {
                intrinsic: floating_point_number_add,
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
