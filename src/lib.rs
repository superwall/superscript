#[cfg(not(target_arch = "wasm32"))]
uniffi::include_scaffolding!("cel");
mod ast;
mod models;

use crate::ast::{ASTExecutionContext, JSONExpression};
use crate::models::PassableValue::Function;
use crate::models::PassableValue::PMap;
use crate::models::{ExecutionContext, PassableMap, PassableValue};
use crate::ExecutableType::{CompiledProgram, AST};
use cel_interpreter::extractors::This;
use cel_interpreter::objects::{Key, Map, TryIntoValue};
use cel_interpreter::{Context, ExecutionError, Expression, FunctionContext, Program, Value};
use cel_parser::parse;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;


/**
 * Host context trait that defines the methods that the host context should implement,
 * i.e. iOS or Android calling code. This trait is used to resolve dynamic properties in the
 * CEL expression during evaluation, such as `computed.daysSinceEvent("event_name")` or similar.
 * Note: Since WASM async support in the browser is still not fully mature, we're using the
 * target_arch cfg to define the trait methods differently for WASM and non-WASM targets.
 */
#[cfg(target_arch = "wasm32")]
pub trait HostContext: Send + Sync {
    fn computed_property(&self, name: String, args: String) -> String;

    fn device_property(&self, name: String, args: String) -> String;
}

#[cfg(not(target_arch = "wasm32"))]
pub trait HostContext: Send + Sync {
    fn computed_property(&self, name: String, args: String, callback: Arc<dyn ResultCallback>);

    fn device_property(&self, name: String, args: String, callback: Arc<dyn ResultCallback>);
}

#[cfg(not(target_arch = "wasm32"))]
pub trait ResultCallback: Send + Sync {
    fn on_result(&self, result: String);
}

/**
 * Evaluate a CEL expression with the given AST
 * @param ast The AST Execution Context, serialized as JSON. This defines the AST, the variables, and the platform properties.
 * @param host The host context to use for resolving properties
 * @return The result of the evaluation, either "true" or "false"
 */
pub fn evaluate_ast_with_context(definition: String, host: Arc<dyn HostContext>) -> String {
    let data: Result<ASTExecutionContext,_> = serde_json::from_str(definition.as_str());
    let data = match data {
        Ok(data) => data,
        Err(_) => {
            let e : Result<_, String> = Err::<ASTExecutionContext,String>("Invalid execution context JSON".to_string());
            return serde_json::to_string(&e).unwrap()
        }
    };
    let host = host.clone();
    let res = execute_with(
        AST(data.expression.into()),
        data.variables,
        data.computed,
        data.device,
        host,
    ).map(|val| val.to_passable())
        .map_err(|err| err.to_string());
    serde_json::to_string(&res).unwrap()
}

/**
 * Evaluate a CEL expression with the given AST without any context
 * @param ast The AST of the expression, serialized as JSON. This AST should contain already resolved dynamic variables.
 * @return The result of the evaluation, either "true" or "false"
 */
pub fn evaluate_ast(ast: String) -> String {
    let data: Result<JSONExpression,_> = serde_json::from_str(ast.as_str());
    let data : JSONExpression = match data {
        Ok(data) => data,
        Err(_) => {
            let e : Result<_, String> = Err::<JSONExpression,String>("Invalid definition for AST Execution".to_string());
            return serde_json::to_string(&e).unwrap()
        }
    };
    let ctx = Context::default();
    let res = ctx.resolve(&data.into())
        .map(|val| DisplayableValue(val.clone()).to_passable())
        .map_err(|err| DisplayableError(err).to_string());
    serde_json::to_string(&res).unwrap()
}

/**
 * Evaluate a CEL expression with the given definition by compiling it first.
 * @param definition The definition of the expression, serialized as JSON. This defines the expression, the variables, and the platform properties.
 * @param host The host context to use for resolving properties
 * @return The result of the evaluation, either "true" or "false"
 */

pub fn evaluate_with_context(definition: String, host: Arc<dyn HostContext>) -> String {
    let data: Result<ExecutionContext,_> = serde_json::from_str(definition.as_str());
    let data: ExecutionContext = match data {
        Ok(data) => data,
        Err(e) => {
            let mut error_message = format!("Invalid execution context JSON: {}", e);
            // If there's a source (cause), add it
            if let Some(source) = e.source() {
                error_message = format!("{}\nCaused by: {}", error_message, source);
            }

            let error_result: Result<_, String> = Err::<ASTExecutionContext,String>(error_message);
            return serde_json::to_string(&error_result).unwrap()
        }
    };
    let compiled = Program::compile(data.expression.as_str())
        .map(|program| CompiledProgram(program));
    let result = match compiled {
        Ok(compiled) => {
            execute_with(
                compiled,
                data.variables,
                data.computed,
                data.device,
                host,
            ).map(|val| val.to_passable())
                .map_err(|err| err.to_string())

        }
        Err(e) =>
            Err("Failed to compile expression".to_string())
    };
    serde_json::to_string(&result).unwrap()
}

/**
 * Transforms a given CEL expression into a CEL AST, serialized as JSON.
 * @param expression The CEL expression to parse
 * @return The AST of the expression, serialized as JSON
 */
pub fn parse_to_ast(expression: String) -> String {
    let ast: Result<JSONExpression, _> = parse(expression.as_str()).map(|expr| expr.into());
    let ast = ast
        .map_err(|err| err.to_string());
    serde_json::to_string(&ast.unwrap()).unwrap()
}

/**
Type of expression to be executed, either a compiled program or an AST.
 */
enum ExecutableType {
    AST(Expression),
    CompiledProgram(Program),
}

/**
 * Execute a CEL expression, either compiled or pure AST; with the given context.
 * @param executable The executable type, either an AST or a compiled program
 * @param variables The variables to use in the expression
 * @param platform The platform properties or functions to use in the expression
 * @param host The host context to use for resolving properties
 */
fn execute_with(
    executable: ExecutableType,
    variables: PassableMap,
    computed: Option<HashMap<String, Vec<PassableValue>>>,
    device: Option<HashMap<String, Vec<PassableValue>>>,
    host: Arc<dyn HostContext + 'static>,
) -> Result<DisplayableValue, DisplayableError> {
    let host = host.clone();
    let host = Arc::new(Mutex::new(host));
    let mut ctx = Context::default();
    // Isolate device to re-bind later
    let device_map = variables.clone();
    let device_map = device_map.map.get("device").clone().unwrap_or(&PMap(HashMap::new())).clone();

    // Add predefined variables locally to the context
    variables
        .map
        .iter()
        .for_each(|it| {
            let _ = ctx.add_variable(it.0.as_str(), it.1.to_cel());
        });
    // Add maybe function
    ctx.add_function("maybe", maybe);

    // This function is used to extract the value of a property from the host context
    // As UniFFi doesn't support recursive enums yet, we have to pass it in as a
    // JSON serialized string of a PassableValue from Host and deserialize it here

    enum PropType {
        Computed,
        Device,
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn prop_for(
        prop_type: PropType,
        name: Arc<String>,
        args: Option<Vec<PassableValue>>,
        ctx: &Arc<dyn HostContext>,
    ) -> Result<PassableValue, String> {
        // Get computed property
        let val = futures_lite::future::block_on(async move {
            let ctx = ctx.clone();
            let args = if let Some(args) = args {
                serde_json::to_string(&args)
            } else {
                serde_json::to_string::<Vec<PassableValue>>(&vec![])
            };
            let shared = Arc::new(Mutex::new(SharedState {
                result: None,
                waker: None,
            }));
            let callback = CallbackFuture {
                shared: shared.clone(),
            };

            let result: Result<_,String> = match args {
                Ok(args) => {
                    match prop_type {
                        PropType::Computed => Ok(ctx.computed_property(
                            name.clone().to_string(),
                            args,
                            Arc::new(callback),
                        )),
                        PropType::Device => Ok(ctx.device_property(
                            name.clone().to_string(),
                            args,
                            Arc::new(callback),
                        )),
                    }
                }
                Err(e) => {
                    Err(ExecutionError::UndeclaredReference(name).to_string())
                }
            };

            match result {
                Ok(_) => {
                    let future = CallbackFuture { shared  }.await;
                    Ok(future)
                }
                Err(e) => {
                    Err(e)
                }
            }

        });
        // Deserialize the value
        let passable: Result<PassableValue, String> =
            val.map(|val| serde_json::from_str(val.as_str()).unwrap_or(PassableValue::Null))
                .map_err(|err| err.to_string());

        passable
    }

    #[cfg(target_arch = "wasm32")]
    fn prop_for(
        prop_type: PropType,
        name: Arc<String>,
        args: Option<Vec<PassableValue>>,
        ctx: &Arc<dyn HostContext>,
    ) -> Option<PassableValue> {
        let ctx = ctx.clone();

        let val = match prop_type {
            PropType::Computed => ctx.computed_property(
                name.clone().to_string(),
                serde_json::to_string(&args).expect("Failed to serialize args for computed property"),
            ),
            PropType::Device => ctx.device_property(
                name.clone().to_string(),
                serde_json::to_string(&args).expect("Failed to serialize args for computed property"),
            ),
        };
        // Deserialize the value
        let passable: Option<PassableValue> = serde_json::from_str(val.as_str()).unwrap_or(Some(PassableValue::Null));

        passable
    }

    let computed = computed.unwrap_or(HashMap::new()).clone();

    // Create computed properties as a map of keys and function names
    let computed_host_properties: HashMap<Key, Value> = computed
        .iter()
        .map(|it| {
            let args = it.1.clone();
            let args = if args.is_empty() {
                None
            } else {
                Some(Box::new(PassableValue::List(args)))
            };
            let name = it.0.clone();
            (
                Key::String(Arc::new(name.clone())),
                Function(name, args).to_cel(),
            )
        })
        .collect();

    let device = device.unwrap_or(HashMap::new()).clone();


    // From defined properties the device properties
    let total_device_properties = if let PMap(map) = device_map {
        map
    } else {
        HashMap::new()
    };

    // Create device properties as a map of keys and function names
    let device_host_properties: HashMap<Key, Value> = device
        .iter()
        .map(|it| {
            let args = it.1.clone();
            let args = if args.is_empty() {
                None
            } else {
                Some(Box::new(PassableValue::List(args)))
            };
            let name = it.0.clone();
            (
                Key::String(Arc::new(name.clone())),
                Function(name, args).to_cel(),
            )
        })
        .chain(total_device_properties.iter().map(|(k, v)| (Key::String(Arc::new(k.clone())), v.to_cel().clone())))
        .collect();

    // Add the map to the `computed` object
    let _ = ctx.add_variable(
        "computed",
        Value::Map(Map {
            map: Arc::new(computed_host_properties),
        }),
    );

    // Add the map to the `device` object
    let _ = ctx.add_variable(
        "device",
        Value::Map(Map {
            map: Arc::new(device_host_properties),
        }),
    );


    let binding = device.clone();
    // Combine the device and computed properties
    let host_properties = binding
        .iter()
        .chain(computed.iter())
        .map(|(k, v)| (k.clone(), v.clone()))
        .into_iter();

    let mut device_properties_clone = device.clone().clone();
    // Add those functions to the context
    for it in host_properties {
        let mut value = device_properties_clone.clone();
        let key = it.0.clone();
        let host_clone = Arc::clone(&host); // Clone the Arc to pass into the closure
        let key_str = key.clone(); // Clone key for usage in the closure
        ctx.add_function(
            key_str.as_str(),
            move |ftx: &FunctionContext| -> Result<Value, ExecutionError> {
                let device = value.clone();
                let fx = ftx.clone();
                let name = fx.name.clone(); // Move the name into the closure
                let args = fx.args.clone(); // Clone the arguments
                let host = host_clone.lock(); // Lock the host for safe access
                match host {
                    Ok(host) => {
                        prop_for(
                            if device.contains_key(&it.0)
                            { PropType::Device } else { PropType::Computed },
                            name.clone(),
                            Some(
                                args.iter()
                                    .map(|expression| {
                                        DisplayableValue(ftx.ptx.resolve(expression).unwrap()).to_passable()
                                    })
                                    .collect(),
                            ),
                            &*host,
                        )
                            .map_or(Err(ExecutionError::UndeclaredReference(name)), |v| {
                                Ok(v.to_cel())
                            })
                    }
                    Err(e) => {
                        let e = e.to_string();
                        let name = name.clone().to_string();
                        let error = ExecutionError::FunctionError { function: name, message: e };
                        Err(error)
                    }
                }
            },
        );
    }

    let val = match executable {
        AST(ast) => &ctx.resolve(&ast),
        CompiledProgram(program) => &program.execute(&ctx),
    };

    val.clone().map(|val| DisplayableValue(val.clone()))
        .map_err(|err| DisplayableError(err))
}

pub fn maybe(
    ftx: &FunctionContext,
    This(_this): This<Value>,
    left: Expression,
    right: Expression,
) -> Result<Value, ExecutionError> {
    return ftx.ptx.resolve(&left).or_else(|_| ftx.ptx.resolve(&right));
}

// Wrappers around CEL values used so that we can create extensions on them
pub struct DisplayableValue(Value);

pub struct DisplayableError(ExecutionError);

impl fmt::Display for DisplayableValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.0 {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(x) => write!(f, "{}", x),
            Value::String(s) => write!(f, "{}", s),
            // Add more variants as needed
            Value::UInt(i) => write!(f, "{}", i),
            Value::Bytes(_) => {
                write!(f, "{}", "bytes go here")
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Duration(d) => write!(f, "{}", d),
            Value::Timestamp(t) => write!(f, "{}", t),
            Value::Null => write!(f, "{}", "null"),
            Value::Function(name, _) => write!(f, "{}", name),
            Value::Map(map) => {
                let res: HashMap<String, String> = map
                    .map
                    .iter()
                    .map(|(k, v)| {
                        let key = DisplayableValue(k.try_into_value().unwrap().clone()).to_string();
                        let value = DisplayableValue(v.clone()).to_string().replace("\\", "");
                        (key, value)
                    })
                    .collect();
                let map = serde_json::to_string(&res).unwrap();
                write!(f, "{}", map)
            }
            Value::List(list) => write!(
                f,
                "{}",
                list.iter()
                    .map(|v| {
                        let key = DisplayableValue(v.clone());
                        return key.to_string();
                    })
                    .collect::<Vec<_>>()
                    .join(",\n ")
            ),
        }
    }
}

impl fmt::Display for DisplayableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_string().as_str())
    }
}

// We use this to turn the ResultCallback into a future we can await
#[cfg(not(target_arch = "wasm32"))]
impl ResultCallback for CallbackFuture {
    fn on_result(&self, result: String) {
        let mut shared = self.shared.lock().unwrap(); // Now valid
        shared.result = Some(result);
        if let Some(waker) = shared.waker.take() {
            waker.wake();
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub struct CallbackFuture {
    shared: Arc<Mutex<SharedState>>,
}

#[cfg(not(target_arch = "wasm32"))]
struct SharedState {
    result: Option<String>,
    waker: Option<Waker>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Future for CallbackFuture {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut shared = self.shared.lock().unwrap();
        if let Some(result) = shared.result.take() {
            Poll::Ready(result)
        } else {
            shared.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext {
        map: HashMap<String, String>,
    }

    impl HostContext for TestContext {
        fn computed_property(&self, name: String, args: String, callback: Arc<dyn ResultCallback>) {
            let result = self.map.get(&name).unwrap().to_string();
            callback.on_result(result);
        }

        fn device_property(&self, name: String, args: String, callback: Arc<dyn ResultCallback>) {
            let result = self.map.get(&name).unwrap().to_string();
            callback.on_result(result);
        }
    }

    #[tokio::test]
    async fn test_variables() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });
        let res = evaluate_with_context(
            r#"
        {
            "variables": {
             "map" : {
                    "foo": {"type": "int", "value": 100}
            }},
            "expression": "foo == 100"
        }

        "#
                .to_string(),
            ctx,
        );
        assert_eq!(res, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");
    }

    #[tokio::test]
    async fn test_execution_with_ctx() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });
        let res = evaluate_with_context(
            r#"
        {
            "variables": {
             "map" : {
                    "foo": {"type": "int", "value": 100},
                    "bar": {"type": "int", "value": 42}
            }},
            "expression": "foo + bar == 142"
        }

        "#
                .to_string(),
            ctx,
        );
        assert_eq!(res, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");
    }

    #[test]
    fn test_unknown_function_with_arg_fails_with_undeclared_ref() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });

        let res = evaluate_with_context(
            r#"
        {
            "variables": {
             "map" : {
                    "foo": {"type": "int", "value": 100}
            }},
            "expression": "test_custom_func(foo) == 101"
        }

        "#
                .to_string(),
            ctx,
        );
        assert_eq!(res, "{\"Err\":\"Undeclared reference to 'test_custom_func'\"}");
    }

    #[test]
    fn test_list_contains() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });
        let res = evaluate_with_context(
            r#"
        {
            "variables": {
                 "map" : {
                    "numbers": {
                        "type" : "list",
                        "value" : [
                            {"type": "int", "value": 1},
                            {"type": "int", "value": 2},
                            {"type": "int", "value": 3}
                             ]
                       }
                 }
            },
            "expression": "numbers.contains(2)"
        }

        "#
                .to_string(),
            ctx,
        );
        assert_eq!(res, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");
    }

    #[tokio::test]
    async fn test_execution_with_map() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });
        let res = evaluate_with_context(
            r#"
        {
                    "variables": {
                        "map": {
                            "user": {
                                "type": "map",
                                "value": {
                                    "should_display": {
                                        "type": "bool",
                                        "value": true
                                    },
                                    "some_value": {
                                        "type": "uint",
                                        "value": 13
                                    }
                                }
                            }
                        }
                    },
                    "expression": "user.should_display == true && user.some_value > 12"
       }

        "#
                .to_string(),
            ctx,
        );
        println!("{}", res.clone());
        assert_eq!(res, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");
    }

    #[tokio::test]
    async fn test_execution_with_failure() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });
        let res = evaluate_with_context(
            r#"
        {
                    "variables": {
                        "map": {
                            "user": {
                                "type": "map",
                                "value": {
                                    "some_value": {
                                        "type": "uint",
                                        "value": 13
                                    }
                                }
                            }
                        }
                    },
                    "expression": "user.should_display == true && user.some_value > 12"
       }

        "#
                .to_string(),
            ctx,
        );
        println!("{}", res.clone());
        assert_eq!(res, "{\"Err\":\"No such key: should_display\"}");
    }

    #[tokio::test]
    async fn test_execution_with_null() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });
        let res = evaluate_with_context(
            r#"
        {
                    "variables": {
                        "map": {
                            "user": {
                                "type": "map",
                                "value": {
                                    "some_value": {
                                        "type": "Null",
                                        "value": null
                                    }
                                }
                            }
                        }
                    },
                    "expression": "user.should_display == true && user.some_value > 12"
       }

        "#
                .to_string(),
            ctx,
        );
        println!("{}", res.clone());
        assert_eq!(res, "{\"Err\":\"No such key: should_display\"}");
    }
    #[tokio::test]
    async fn test_execution_with_platform_computed_reference() {
        let days_since = PassableValue::UInt(7);
        let days_since = serde_json::to_string(&days_since).unwrap();
        let ctx = Arc::new(TestContext {
            map: [("minutesSince".to_string(), days_since)]
                .iter()
                .cloned()
                .collect(),
        });
        let res = evaluate_with_context(
            r#"
    {
        "variables": {
            "map": {}
        },
        "expression": "device.minutesSince('app_launch') == computed.minutesSince('app_install')",
        "computed": {
            "daysSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ],
            "minutesSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ],
            "hoursSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ],
            "monthsSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ]
        },
        "device": {
            "daysSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ],
            "minutesSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ],
            "hoursSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ],
            "monthsSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ]
        }
    }"#.to_string(),
            ctx,
        );
        println!("{}", res.clone());
        assert_eq!(res, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");
    }

    #[tokio::test]
    async fn test_execution_with_platform_device_function_and_property() {
        let days_since = PassableValue::UInt(7);
        let days_since = serde_json::to_string(&days_since).unwrap();
        let ctx = Arc::new(TestContext {
            map: [("minutesSince".to_string(), days_since)]
                .iter()
                .cloned()
                .collect(),
        });
        let res = evaluate_with_context(
            r#"
    {
        "variables": {
            "map": {
                "device": {
                    "type": "map",
                    "value": {
                        "trial_days": {
                            "type": "uint",
                            "value": 7
                        }
                    }
                }
            }
        },
        "expression": "computed.minutesSince('app_launch') == device.trial_days",
        "computed": {
            "daysSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ],
            "minutesSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ],
            "hoursSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ],
            "monthsSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ]
        },
        "device": {
            "daysSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ],
            "minutesSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ],
            "hoursSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ],
            "monthsSince": [
                {
                    "type": "string",
                    "value": "event_name"
                }
            ]
        }
    }"#.to_string(),
            ctx,
        );
        println!("{}", res.clone());
        assert_eq!(res, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");
    }


    #[test]
    fn test_parse_to_ast() {
        let expression = "device.daysSince(app_install) == 3";
        let ast_json = parse_to_ast(expression.to_string());
        println!("\nSerialized AST:");
        println!("{}", ast_json);
        // Deserialize back to JSONExpression
        let deserialized_json_expr: JSONExpression = serde_json::from_str(&ast_json).unwrap();

        // Convert back to original Expression
        let deserialized_expr: Expression = deserialized_json_expr.into();

        println!("\nDeserialized Expression:");
        println!("{:?}", deserialized_expr);

        let parsed_expression = parse(expression).unwrap();
        assert_eq!(parsed_expression, deserialized_expr);
        println!("\nOriginal and deserialized expressions are equal!");
    }
}
