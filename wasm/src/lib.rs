use std::sync::Arc;

pub use cel_eval::HostContext;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

/**
 * This is the definition for the JS Host module contracts.
 * It is used to define the methods that are exposed to our Rust code
 * from the JS Host.
 */
#[wasm_bindgen]
extern "C" {

    // Enable this when logging support is needed
    //#[wasm_bindgen(js_namespace = console)]
    //fn log(s: &str);

    /**
    Defines the Rust type and method signatures of the JS Host context.
    */
    #[wasm_bindgen(typescript_type = "WasmHostContext")]
    pub type JsHostContext;

    #[wasm_bindgen(method, catch)]
    fn computed_property(
        this: &JsHostContext,
        name: String,
        args: String,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    fn device_property(
        this: &JsHostContext,
        name: String,
        args: String,
    ) -> Result<JsValue, JsValue>;

}

/**
* Sets up a panic hook to log panics to the console.
* This method is a nice-to-have for debugging purposes.
* It should be called once on initialization of the WASM module.
*/
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/**
 * This is the adapter that is used to convert the JS Host context into a Supercel Rust Host Context.
 */
struct HostContextAdapter {
    context: Arc<JsHostContext>,
}

impl HostContextAdapter {
    fn new(context: JsHostContext) -> Self {
        Self {
            context: Arc::new(context),
        }
    }
}

impl HostContext for HostContextAdapter {
    /**
     * This method is used to call the computed property method on the JS Host context.
     * It proxies evaluator calls for `platform.something(arg)` to the JS Host context itself.
     */
    fn computed_property(&self, name: String, args: String) -> String {
        let context = Arc::clone(&self.context);
        let promise = context.computed_property(name.clone(), args.clone());
        let result = promise
            .expect("Did not receive the proper result from computed")
            .as_string();
        result.clone().expect(
            format!(
                "Could not deserialize the result from computed property - Is some: {}",
                result.is_some()
            )
            .as_str(),
        )
    }

    fn device_property(&self, name: String, args: String) -> String {
        let context = Arc::clone(&self.context);
        let promise = context.device_property(name.clone(), args.clone());
        let result = promise
            .expect("Did not receive the proper result from computed")
            .as_string();
        result.clone().expect(
            format!(
                "Could not deserialize the result from computed property - Is some: {}",
                result.is_some()
            )
            .as_str(),
        )
    }
}

unsafe impl Send for HostContextAdapter {}

unsafe impl Sync for HostContextAdapter {}

#[wasm_bindgen]
pub async fn evaluate_with_context(
    definition: String,
    context: JsHostContext,
) -> Result<String, JsValue> {
    let adapter = Arc::new(HostContextAdapter::new(context));
    Ok(cel_eval::evaluate_with_context(definition, adapter))
}

#[wasm_bindgen]
pub async fn evaluate_ast_with_context(
    definition: String,
    context: JsHostContext,
) -> Result<String, JsValue> {
    let adapter = Arc::new(HostContextAdapter::new(context));
    Ok(cel_eval::evaluate_ast_with_context(definition, adapter))
}

#[wasm_bindgen]
pub async fn evaluate_ast(ast: String) -> Result<String, JsValue> {
    Ok(cel_eval::evaluate_ast(ast))
}

#[wasm_bindgen]
pub async fn parse_into_ast(expression: String) -> Result<String, JsValue> {
    Ok(cel_eval::parse_to_ast(expression))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
