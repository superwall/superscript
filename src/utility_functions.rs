use cel_interpreter::extractors::This;
use cel_interpreter::{ExecutionError, FunctionContext, Value};
use cel_parser::Expression;
use std::sync::Arc;

/** A method on a string type. When added to the CEL context, this function
* can be called by running. We use the [`This`] extractor give us a reference
* to the string that this method was called on.
*
* ```
* "123".to_string()
* ```
*/
pub fn to_string_i(This(s): This<i64>) -> Arc<String> {
    Arc::new(s.to_string())
}
pub fn to_string_u(This(s): This<u64>) -> Arc<String> {
    Arc::new(s.to_string())
}
pub fn to_string_f(This(s): This<f64>) -> Arc<String> {
    Arc::new(s.to_string())
}

pub fn to_string_b(This(s): This<bool>) -> Arc<String> {
    Arc::new(s.to_string())
}

/**
* A method that takes in two expressions, and if the left side fails evaluates the right one.
*/
pub fn maybe(
    ftx: &FunctionContext,
    This(_this): This<Value>,
    left: Expression,
    right: Expression,
) -> Result<Value, ExecutionError> {
    return ftx.ptx.resolve(&left).or_else(|_| ftx.ptx.resolve(&right));
}
