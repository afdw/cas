mod data;
mod value;

use std::collections::HashMap;
pub use value::Value;

fn main() {
    let execution_context = data::ExecutionContext {
        values: HashMap::new(),
        intrinsics: {
            let mut intrinsics = HashMap::new();
            macro_rules! intrinsic {
                (($execution_context:ident, $($args:ident),*) $body:tt) => {{
                    fn f($execution_context: &mut data::ExecutionContext, arguments: Vec<Value>) -> Value {
                        let mut arguments = arguments.into_iter();
                        $(let $args = arguments.next().unwrap());*;
                        assert!(arguments.next().is_none());
                        $body
                    }
                    f as fn(&mut data::ExecutionContext, Vec<Value>) -> Value
                }}
            }
            intrinsics.insert(
                Value::new(data::SymbolValueInner { name: "test".into() }),
                intrinsic!((execution_context, a, b) {
                    Value::new(data::NullValueInner)
                }),
            );
            intrinsics
        },
    };
}
