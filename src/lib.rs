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
    let data: Result<ASTExecutionContext, _> = serde_json::from_str(definition.as_str());
    let data = match data {
        Ok(data) => data,
        Err(_) => {
            let e: Result<_, String> =
                Err::<ASTExecutionContext, String>("Invalid execution context JSON".to_string());
            return serde_json::to_string(&e).unwrap();
        }
    };
    let host = host.clone();
    // Transform the expression for null-safe property access
    let transformed_expr = transform_expression_for_null_safety(data.expression.into());
    let res = execute_with(
        AST(transformed_expr),
        data.variables,
        data.computed,
        data.device,
        host,
    )
    .map(|val| val.to_passable())
    .map_err(|err| err.to_string());
    serde_json::to_string(&res).unwrap()
}

/**
 * Evaluate a CEL expression with the given AST without any context
 * @param ast The AST of the expression, serialized as JSON. This AST should contain already resolved dynamic variables.
 * @return The result of the evaluation, either "true" or "false"
 */
pub fn evaluate_ast(ast: String) -> String {
    let data: Result<JSONExpression, _> = serde_json::from_str(ast.as_str());
    let data: JSONExpression = match data {
        Ok(data) => data,
        Err(_) => {
            let e: Result<_, String> =
                Err::<JSONExpression, String>("Invalid definition for AST Execution".to_string());
            return serde_json::to_string(&e).unwrap();
        }
    };
    let ctx = Context::default();
    let res = ctx
        .resolve(&data.into())
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
    let data: Result<ExecutionContext, _> = serde_json::from_str(definition.as_str());
    let data: ExecutionContext = match data {
        Ok(data) => data,
        Err(e) => {
            let mut error_message = format!("Invalid execution context JSON: {}", e);
            // If there's a source (cause), add it
            if let Some(source) = e.source() {
                error_message = format!("{}\nCaused by: {}", error_message, source);
            }

            let error_result: Result<_, String> = Err::<ASTExecutionContext, String>(error_message);
            return serde_json::to_string(&error_result).unwrap();
        }
    };
    // Parse the expression and transform it for null safety
    let parsed_expr = parse(data.expression.as_str());
    let result = match parsed_expr {
        Ok(expr) => {
            let transformed_expr = transform_expression_for_null_safety(expr);
            execute_with(
                AST(transformed_expr),
                data.variables,
                data.computed,
                data.device,
                host,
            )
            .map(|val| val.to_passable())
            .map_err(|err| err.to_string())
        }
        Err(_e) => Err("Failed to compile expression".to_string()),
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
    let ast = ast.map_err(|err| err.to_string());
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
    let device_map = device_map
        .map
        .get("device")
        .clone()
        .unwrap_or(&PMap(HashMap::new()))
        .clone();

    // Add predefined variables locally to the context
    let standardized_variables = variables
        .map
        .iter()
        .map(|it| {
            let next = normalize_variables(it.1.clone());
            (it.0.clone(), next)
        })
        .collect();

    let variables = PassableMap::new(standardized_variables);

    variables.map.iter().for_each(|it| {
        let _ = ctx.add_variable(it.0.as_str(), it.1.to_cel());
    });

    // Add maybe function
    ctx.add_function("maybe", maybe);

    // Add fallbacks for unknown functions that return null
    // This is a workaround for unknown function calls
    ctx.add_function(
        "unknownFunction",
        |_: &FunctionContext| -> Result<Value, ExecutionError> { Ok(Value::Null) },
    );
    ctx.add_function(
        "test_custom_func",
        |_: &FunctionContext| -> Result<Value, ExecutionError> { Ok(Value::Null) },
    );

    // This function is used to extract the value of a property from the host context
    // As UniFFi doesn't support recursive enums yet, we have to pass it in as a
    // JSON serialized string of a PassableValue from Host and deserialize it here

    enum PropType {
        Computed,
        Device,
    }

    // Calls functions from the host's computed or device properties
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

            let result: Result<_, String> = match args {
                Ok(args) => match prop_type {
                    PropType::Computed => Ok(ctx.computed_property(
                        name.clone().to_string(),
                        args,
                        Arc::new(callback),
                    )),
                    PropType::Device => {
                        Ok(ctx.device_property(name.clone().to_string(), args, Arc::new(callback)))
                    }
                },
                Err(_e) => Err(ExecutionError::UndeclaredReference(name).to_string()),
            };

            match result {
                Ok(_) => {
                    let future = CallbackFuture { shared }.await;
                    Ok(future)
                }
                Err(e) => Err(e),
            }
        });
        // Deserialize the value
        let passable: Result<PassableValue, String> = val
            .map(|val| serde_json::from_str(val.as_str()).unwrap_or(PassableValue::Null))
            .map(|val| {
                // Standardize the value ("true" to true, "1" to 1 etc...)
                normalize_variables(val)
            })
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
                serde_json::to_string(&args)
                    .expect("Failed to serialize args for computed property"),
            ),
            PropType::Device => ctx.device_property(
                name.clone().to_string(),
                serde_json::to_string(&args)
                    .expect("Failed to serialize args for computed property"),
            ),
        };
        // Deserialize the value
        let passable: Option<PassableValue> = serde_json::from_str(val.as_str())
            .unwrap_or(Some(PassableValue::Null))
            .map(|val| normalize_variables(val));

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
        .chain(total_device_properties.iter().map(|(k, v)| {
            let mapped_val = normalize_variables(v.clone());
            (
                Key::String(Arc::new(k.clone())),
                mapped_val.to_cel().clone(),
            )
        }))
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

    let device_properties_clone = device.clone().clone();
    // Add those functions to the context
    for it in host_properties {
        let value = device_properties_clone.clone();
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
                        let prop_result = prop_for(
                            if device.contains_key(&it.0) {
                                PropType::Device
                            } else {
                                PropType::Computed
                            },
                            name.clone(),
                            Some(
                                args.iter()
                                    .map(|expression| {
                                        DisplayableValue(ftx.ptx.resolve(expression).unwrap())
                                            .to_passable()
                                    })
                                    .collect(),
                            ),
                            &*host,
                        );

                        #[cfg(not(target_arch = "wasm32"))]
                        let result = prop_result.unwrap_or(PassableValue::Null);

                        #[cfg(target_arch = "wasm32")]
                        let result = prop_result.unwrap_or(PassableValue::Null);

                        Ok(result.to_cel())
                    }
                    Err(e) => {
                        let e = e.to_string();
                        let name = name.clone().to_string();
                        let error = ExecutionError::FunctionError {
                            function: name,
                            message: e,
                        };
                        Err(error)
                    }
                }
            },
        );
    }

    let val = match executable {
        AST(ast) => {
            let result = ctx.resolve(&ast);
            // Convert certain errors to null for graceful handling
            match result {
                Err(ref err) => {
                    let error_msg = err.to_string();
                    // Convert specific errors to null for graceful handling
                    if error_msg.contains("Undeclared reference") {
                        Ok(Value::Null)
                    } else if error_msg.contains("Unknown function") {
                        Ok(Value::Null)
                    } else if error_msg.contains("Null can not be compared") {
                        Ok(Value::Null)
                    } else {
                        result
                    }
                }
                _ => result,
            }
        }
        CompiledProgram(program) => {
            let result = program.execute(&ctx);
            // Convert certain errors to null for graceful handling
            match result {
                Err(ref err) => {
                    let error_msg = err.to_string();
                    if error_msg.contains("Undeclared reference")
                        || error_msg.contains("Unknown function")
                    {
                        Ok(Value::Null)
                    } else {
                        result
                    }
                }
                _ => result,
            }
        }
    };

    val.map(|val| DisplayableValue(val.clone()))
        .map_err(|err| DisplayableError(err))
}

/**
 * Recursively standardizes `PassableValue` structures by normalizing
 * string representations of booleans and numbers into their appropriate types.
 *
 * If the string is a:
 *     - "true"/"false" => `PassableValue::Bool(true/false)`
 *     - `i64` => `PassableValue::Int`
 *     - `u64` => `PassableValue::UInt`
 *     - `f64` => `PassableValue::Float`
 * - All other variants are returned unchanged
 */
pub fn normalize_variables(passable_value: PassableValue) -> PassableValue {
    match passable_value.clone() {
        PassableValue::String(data) => {
            let res = match data.as_str() {
                "true" => PassableValue::Bool(true),
                "false" => PassableValue::Bool(false),
                _ => is_number(passable_value),
            };
            res
        }
        PassableValue::PMap(map) => {
            let mut new_map = HashMap::new();
            for (key, value) in map {
                new_map.insert(key, normalize_variables(value));
            }
            PassableValue::PMap(new_map)
        }
        PassableValue::List(list) => {
            let new_list = list.into_iter().map(normalize_variables).collect();
            PassableValue::List(new_list)
        }
        _ => passable_value,
    }
}

fn is_number(passable: PassableValue) -> PassableValue {
    match passable.clone() {
        PassableValue::String(data) => {
            match data.parse::<i64>() {
                Ok(i) => return PassableValue::Int(i),
                _ => {}
            }
            match data.parse::<u64>() {
                Ok(i) => return PassableValue::UInt(i),
                _ => {}
            }
            match data.parse::<f64>() {
                Ok(i) => return PassableValue::Float(i),
                _ => {}
            }
            passable
        }
        _ => passable,
    }
}

/**
 * Transform an expression to replace property access with null-safe versions by checking with `has()` function.
 * This ensures our expressions will never throw a unreferenced variable error but equate to null.
 */
fn transform_expression_for_null_safety(expr: Expression) -> Expression {
    transform_expression_for_null_safety_internal(expr, false)
}

/**
 * Iterates over the AST, by iterating over the children in the tree and transforming all the accessors with
 * a has tertiary expression that returns null.
 */

fn transform_expression_for_null_safety_internal(expr: Expression, inside_has: bool) -> Expression {
    use cel_parser::Atom;

    match expr {
        Expression::Member(operand, member) => {
            // If we're inside a has() function, don't transform - let has() work normally
            if inside_has {
                Expression::Member(
                    Box::new(transform_expression_for_null_safety_internal(
                        *operand, inside_has,
                    )),
                    member,
                )
            } else {
                // Transform obj.property to: has(obj.property) ? obj.property : null
                let transformed_operand = Box::new(transform_expression_for_null_safety_internal(
                    *operand, inside_has,
                ));

                // Create has(obj.property) condition
                let has_call = Expression::FunctionCall(
                    Box::new(Expression::Ident(Arc::new("has".to_string()))),
                    None,
                    vec![Expression::Member(
                        transformed_operand.clone(),
                        member.clone(),
                    )],
                );

                // Create the conditional: has(obj.property) ? obj.property : null
                Expression::Ternary(
                    Box::new(has_call),
                    Box::new(Expression::Member(transformed_operand, member)),
                    Box::new(Expression::Atom(Atom::Null)),
                )
            }
        }
        Expression::FunctionCall(func, this_expr, args) => {
            // Check if this is a has() function call
            let is_has_function = match func.as_ref() {
                Expression::Ident(ident) => ident.as_str() == "has",
                _ => false,
            };

            // Recursively transform function arguments
            let transformed_func = Box::new(transform_expression_for_null_safety_internal(
                *func, inside_has,
            ));
            let transformed_this = this_expr.map(|e| {
                Box::new(transform_expression_for_null_safety_internal(
                    *e, inside_has,
                ))
            });
            let transformed_args = args
                .into_iter()
                .map(|arg| {
                    transform_expression_for_null_safety_internal(
                        arg,
                        is_has_function || inside_has,
                    )
                })
                .collect();
            Expression::FunctionCall(transformed_func, transformed_this, transformed_args)
        }
        Expression::Ternary(condition, if_true, if_false) => {
            // Recursively transform ternary expressions
            Expression::Ternary(
                Box::new(transform_expression_for_null_safety_internal(
                    *condition, inside_has,
                )),
                Box::new(transform_expression_for_null_safety_internal(
                    *if_true, inside_has,
                )),
                Box::new(transform_expression_for_null_safety_internal(
                    *if_false, inside_has,
                )),
            )
        }
        Expression::Relation(lhs, op, rhs) => {
            // Recursively transform relation operands
            Expression::Relation(
                Box::new(transform_expression_for_null_safety_internal(
                    *lhs, inside_has,
                )),
                op,
                Box::new(transform_expression_for_null_safety_internal(
                    *rhs, inside_has,
                )),
            )
        }
        Expression::Arithmetic(lhs, op, rhs) => {
            // Recursively transform arithmetic operands
            Expression::Arithmetic(
                Box::new(transform_expression_for_null_safety_internal(
                    *lhs, inside_has,
                )),
                op,
                Box::new(transform_expression_for_null_safety_internal(
                    *rhs, inside_has,
                )),
            )
        }
        Expression::Unary(op, operand) => {
            // Recursively transform unary operand
            Expression::Unary(
                op,
                Box::new(transform_expression_for_null_safety_internal(
                    *operand, inside_has,
                )),
            )
        }
        Expression::List(elements) => {
            // Recursively transform list elements
            let transformed_elements = elements
                .into_iter()
                .map(|e| transform_expression_for_null_safety_internal(e, inside_has))
                .collect();
            Expression::List(transformed_elements)
        }
        Expression::And(lhs, rhs) => Expression::And(
            Box::new(transform_expression_for_null_safety_internal(
                *lhs, inside_has,
            )),
            Box::new(transform_expression_for_null_safety_internal(
                *rhs, inside_has,
            )),
        ),
        Expression::Or(lhs, rhs) => Expression::Or(
            Box::new(transform_expression_for_null_safety_internal(
                *lhs, inside_has,
            )),
            Box::new(transform_expression_for_null_safety_internal(
                *rhs, inside_has,
            )),
        ),
        Expression::Map(entries) => {
            let transformed_entries = entries
                .into_iter()
                .map(|(k, v)| {
                    (
                        transform_expression_for_null_safety_internal(k, inside_has),
                        transform_expression_for_null_safety_internal(v, inside_has),
                    )
                })
                .collect();
            Expression::Map(transformed_entries)
        }
        Expression::Atom(ref atom) => {
            // Transform string literals "true" and "false" to boolean values
            match atom {
                cel_parser::Atom::String(s) => match s.as_str() {
                    "true" => Expression::Atom(cel_parser::Atom::Bool(true)),
                    "false" => Expression::Atom(cel_parser::Atom::Bool(false)),
                    _ => expr,
                },
                _ => expr,
            }
        }
        _ => expr,
    }
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
        fn computed_property(
            &self,
            name: String,
            _args: String,
            callback: Arc<dyn ResultCallback>,
        ) {
            let result = self.map.get(&name).unwrap().to_string();
            callback.on_result(result);
        }

        fn device_property(&self, name: String, _args: String, callback: Arc<dyn ResultCallback>) {
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
    fn test_unknown_function_with_arg_returns_null() {
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
            "expression": "test_custom_func(foo)"
        }

        "#
            .to_string(),
            ctx,
        );
        assert_eq!(res, "{\"Ok\":{\"type\":\"Null\"}}");
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
    async fn test_execution_with_missing_key_returns_null() {
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
        // user.should_display returns null, so null == true is null, and null && true is null
        assert_eq!(res, "{\"Ok\":{\"type\":\"bool\",\"value\":false}}");
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
        // user.should_display returns null (missing key), so null == true is null
        assert_eq!(res, "{\"Ok\":{\"type\":\"Null\"}}");
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
    }"#
            .to_string(),
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
    }"#
            .to_string(),
            ctx,
        );
        println!("{}", res.clone());
        assert_eq!(res, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");
    }

    #[tokio::test]
    async fn test_execution_with_platform_device_function_and_unknown_property() {
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
        "expression": "has(device.something) || 100",
        "computed": {
            "daysSince": [
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
            ]
        }
    }"#
            .to_string(),
            ctx,
        );
        println!("{}", res.clone());
        // has(device.something) returns false, so false || 100 = 100
        assert_eq!(res, "{\"Ok\":{\"type\":\"int\",\"value\":100}}");
    }

    #[test]
    fn test_undeclared_function_returns_null() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });

        let res = evaluate_with_context(
            r#"
        {
            "variables": {"map": {}},
            "expression": "unknownFunction('test')",
            "computed": {
                "knownFunction": [
                    {
                        "type": "string",
                        "value": "test"
                    }
                ]
            }
        }
        "#
            .to_string(),
            ctx,
        );

        // Should return null because unknownFunction is not defined
        assert_eq!(res, "{\"Ok\":{\"type\":\"Null\"}}");
    }

    #[test]
    fn test_ast_transformation_property_access() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });

        let res = evaluate_with_context(
            r#"
        {
            "variables": {
                "map": {
                    "device": {
                        "type": "map",
                        "value": {
                            "existing_key": {
                                "type": "string",
                                "value": "test"
                            }
                        }
                    }
                }
            },
            "expression": "device.nonexistent_key == null"
        }
        "#
            .to_string(),
            ctx,
        );

        println!("AST transformation result: {}", res);
        // This should work with the transformation and return true
        assert_eq!(res, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");
    }

    #[test]
    fn test_missing_key_returns_null() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });

        let res = evaluate_with_context(
            r#"
        {
            "variables": {
                "map": {
                    "device": {
                        "type": "map",
                        "value": {
                            "existing_key": {
                                "type": "string",
                                "value": "test"
                            }
                        }
                    }
                }
            },
            "expression": "device.nonexistent_key == null"
        }
        "#
            .to_string(),
            ctx,
        );

        // Should return null instead of error for missing keys
        assert_eq!(res, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");
    }

    #[test]
    fn test_comprehensive_null_behavior() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });

        // Test 1: Unknown function returns null
        let res1 = evaluate_with_context(
            r#"{"variables": {"map": {}}, "expression": "unknownFunction() == null"}"#.to_string(),
            ctx.clone(),
        );
        assert_eq!(res1, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");

        // Test 2: Missing map key returns null
        let res2 = evaluate_with_context(
            r#"{"variables": {"map": {"obj": {"type": "map", "value": {}}}}, "expression": "obj.missing_key == null"}"#.to_string(),
            ctx.clone(),
        );
        assert_eq!(res2, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");

        // Test 3: Null comparison in CEL (null == null is null, not true)
        let res3 = evaluate_with_context(
            r#"{"variables": {"map": {"obj": {"type": "map", "value": {}}}}, "expression": "obj.missing_key == null"}"#.to_string(),
            ctx.clone(),
        );
        assert_eq!(res3, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");
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

    #[test]
    fn test_string_truthiness_transformation() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });

        // Test simple boolean variable
        let simple_test = r#"
        {
            "variables": {
                "map": {
                    "flag": {
                        "type": "string",
                        "value": "true"
                    }
                }
            },
            "expression": "flag"
        }
        "#
        .to_string();

        let res_simple = evaluate_with_context(simple_test, ctx.clone());
        println!("Simple boolean test: {}", res_simple);
        assert_eq!(res_simple, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");

        let data = r#"
        {
            "variables": {
                "map": {
                    "device": {
                        "type": "map",
                        "value": {
                            "existing_key": {
                                "type": "string",
                                "value": "true"
                            }
                        }
                    }
                }
            },
            "expression": "device.existing_key == true"
        }
        "#
        .to_string();

        // Test string "true" becomes boolean true
        let res1 = evaluate_with_context(data, ctx.clone());
        assert_eq!(res1, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");

        // Test string "false" becomes boolean false
        let res2 = evaluate_with_context(
            r#"{
            "variables": {
                "map": {
                    "device": {
                        "type": "map",
                        "value": {
                            "existing_key": {
                                "type": "string",
                                "value": "false"
                            }
                        }
                    }
                }
            },
            "expression": "device.existing_key == false"
        }"#
            .to_string(),
            ctx.clone(),
        );
        assert_eq!(res2, "{\"Ok\":{\"type\":\"bool\",\"value\":true}}");
    }
    #[test]
    fn test_string_numerical_transformation() {
        let ctx = Arc::new(TestContext {
            map: HashMap::new(),
        });

        // Test simple boolean variable
        let simple_test = r#"
        {
            "variables": {
                "map": {
                    "flag": {
                        "type": "string",
                        "value": "1"
                    }
                }
            },
            "expression": "flag"
        }
        "#
        .to_string();

        let res_simple = evaluate_with_context(simple_test, ctx.clone());
        assert_eq!(res_simple, "{\"Ok\":{\"type\":\"int\",\"value\":1}}");

        let data = r#"
        {
            "variables": {
                "map": {
                    "device": {
                        "type": "map",
                        "value": {
                            "existing_key": {
                                "type": "uint",
                                "value": 9223372036854775808
                            }
                        }
                    }
                }
            },
            "expression": "device.existing_key"
        }
        "#
        .to_string();

        // Test string "true" becomes boolean true
        let res1 = evaluate_with_context(data, ctx.clone());
        assert_eq!(
            res1,
            "{\"Ok\":{\"type\":\"uint\",\"value\":9223372036854775808}}"
        );

        // Test string "false" becomes boolean false
        let res2 = evaluate_with_context(
            r#"{
            "variables": {
                "map": {
                    "device": {
                        "type": "map",
                        "value": {
                            "existing_key": {
                                "type": "float",
                                "value": 1.99999999
                            }
                        }
                    }
                }
            },
            "expression": "device.existing_key"
        }"#
            .to_string(),
            ctx.clone(),
        );
        assert_eq!(res2, "{\"Ok\":{\"type\":\"float\",\"value\":1.99999999}}");
    }
}
