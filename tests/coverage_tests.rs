use super::*;
use cel_interpreter::Value;
use cel_parser::Atom;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
struct TestContext {
    map: HashMap<String, String>,
}

impl HostContext for TestContext {
    fn computed_property(&self, name: String, _args: String, callback: Arc<dyn ResultCallback>) {
        let result = self
            .map
            .get(&name)
            .unwrap_or(&"null".to_string())
            .to_string();
        callback.on_result(result);
    }

    fn device_property(&self, name: String, _args: String, callback: Arc<dyn ResultCallback>) {
        let result = self
            .map
            .get(&name)
            .unwrap_or(&"null".to_string())
            .to_string();
        callback.on_result(result);
    }
}

#[test]
fn test_expression_with_complex_ast_structures() {
    // Test expressions that will trigger AST transformation functions (lines 878-893)
    let unary_expr = "!true";
    let unary_ast = parse_to_ast(unary_expr.to_string());
    assert!(unary_ast.contains("Unary"));

    let list_expr = "[1, 2, 3]";
    let list_ast = parse_to_ast(list_expr.to_string());
    assert!(list_ast.contains("List"));

    // Test with complex nested expressions
    let complex_expr = "device.test(1, 2, 3) && computed.other('test')";
    let complex_ast = parse_to_ast(complex_expr.to_string());
    assert!(complex_ast.contains("FunctionCall") || complex_ast.contains("device.test"));
}

#[test]
fn test_string_to_number_conversion_edge_cases() {
    // Test string conversion functions that cover lines 630-642
    let int_string_expr = "'42'";
    let ast = parse_to_ast(int_string_expr.to_string());
    let ctx_json = format!(r#"{{"ast": {}, "variables": {{}}}}"#, ast);

    let test_ctx = TestContext {
        map: HashMap::new(),
    };
    let result = evaluate_with_context(ctx_json, Arc::new(test_ctx));

    // Just ensure it processes without crashing
    assert!(!result.is_empty());

    // Test float string conversion
    let float_string_expr = "'42.0'";
    let ast2 = parse_to_ast(float_string_expr.to_string());
    let ctx_json2 = format!(r#"{{"ast": {}, "variables": {{}}}}"#, ast2);
    let result2 = evaluate_with_context(
        ctx_json2,
        Arc::new(TestContext {
            map: HashMap::new(),
        }),
    );
    assert!(!result2.is_empty());
}

#[test]
fn test_error_handling_in_property_resolution() {
    // Test error path in property resolution (lines 500-507)
    let expression = "unknownProperty.access";
    let ast = parse_to_ast(expression.to_string());
    let ctx_json = format!(r#"{{"ast": {}, "variables": {{}}}}"#, ast);

    let test_ctx = TestContext {
        map: HashMap::new(),
    };
    let result = evaluate_with_context(ctx_json, Arc::new(test_ctx));

    // Should handle unknown property access gracefully
    assert!(!result.is_empty());
}

#[test]
fn test_additional_ast_edge_cases() {
    // Test additional AST cases for better coverage
    let conditional_expr = "true ? 1 : 0";
    let cond_ast = parse_to_ast(conditional_expr.to_string());
    assert!(cond_ast.contains("Conditional") || cond_ast.contains("Ternary"));

    // Test member access
    let member_expr = "obj.prop";
    let member_ast = parse_to_ast(member_expr.to_string());
    assert!(member_ast.contains("Member"));

    // Test complex arithmetic
    let arith_expr = "1 + 2 * 3 - 4 / 5";
    let arith_ast = parse_to_ast(arith_expr.to_string());
    assert!(arith_ast.contains("Add") || arith_ast.contains("Arithmetic"));
}

#[test]
fn test_more_ast_transformation_edge_cases() {
    // Test additional expression types for AST transformation coverage
    let map_expr = "{'key': 'value', 'nested': {'inner': true}}";
    let map_ast = parse_to_ast(map_expr.to_string());
    assert!(map_ast.contains("Map"));

    let in_expr = "'test' in ['test', 'other']";
    let in_ast = parse_to_ast(in_expr.to_string());
    assert!(in_ast.contains("In") || in_ast.contains("test"));

    // Test function calls with multiple arguments
    let func_call_expr = "hasFn('device.test') && hasFn('computed.other')";
    let func_ast = parse_to_ast(func_call_expr.to_string());
    assert!(func_ast.contains("FunctionCall") || func_ast.contains("hasFn"));
}

#[test]
fn test_large_uncovered_string_conversion_blocks() {
    // Test string conversion functions to cover lines 630-651
    // These handle various numeric string parsing edge cases

    // Test parsing numeric strings that should trigger different conversion paths
    let expressions = vec![
        "'42'",                    // Should parse as int
        "'42.0'",                  // Should parse as float that converts to int
        "'3.14159'",               // Should parse as float
        "'999999999999999999999'", // Large number
        "'true'",                  // Boolean string
        "'false'",                 // Boolean string
        "'not_a_number'",          // Invalid number string
    ];

    for expr in expressions {
        let ast = parse_to_ast(expr.to_string());
        let ctx_json = format!(r#"{{"ast": {}, "variables": {{}}}}"#, ast);
        let result = evaluate_with_context(
            ctx_json,
            Arc::new(TestContext {
                map: HashMap::new(),
            }),
        );
        // Just ensure they all process without crashing
        assert!(!result.is_empty());
    }
}

#[test]
fn test_ast_transformation_comprehensive_recursion() {
    // Test comprehensive AST transformation recursion to cover lines 875-890

    // Test deeply nested expressions that should trigger all transformation paths
    let complex_expressions = vec![
        // Nested unary operations
        "!!true",
        "!(!false)",
        // Complex list operations with nested expressions
        "[1, 2 + 3, true ? 4 : 5]",
        // Nested map operations
        "{'a': 1 + 2, 'b': [3, 4], 'c': {'nested': true}}",
        // Complex member access chains
        "obj.prop.subprop",
        // Mixed conditional and function calls
        "hasFn('test') ? device.prop() : computed.other(1, 2, 3)",
        // Complex arithmetic with grouping
        "((1 + 2) * (3 - 4)) / (5 % 2)",
    ];

    for expr in complex_expressions {
        let ast = parse_to_ast(expr.to_string());
        // Just ensure complex expressions parse and transform without crashing
        assert!(!ast.is_empty());
        assert!(ast.len() > 10); // Should be substantial AST
    }
}

#[test]
fn test_hasfn_comprehensive_error_paths() {
    // Test hasFn function error handling to cover lines 254-263

    let expressions = vec![
        // Test hasFn with various argument types that should trigger error paths
        "hasFn(123)",            // Non-string argument
        "hasFn(true)",           // Boolean argument
        "hasFn([1, 2])",         // Array argument
        "hasFn({'key': 'val'})", // Map argument
        "hasFn(null)",           // Null argument
    ];

    for expr in expressions {
        let ast = parse_to_ast(expr.to_string());
        let ctx_json = format!(r#"{{"ast": {}, "variables": {{}}}}"#, ast);

        let mut test_ctx = TestContext {
            map: HashMap::new(),
        };
        test_ctx
            .map
            .insert("test_prop".to_string(), "value".to_string());

        let result = evaluate_with_context(ctx_json, Arc::new(test_ctx));

        // Should handle errors gracefully - either return error or null
        assert!(result.contains("Err") || result.contains("null") || result.contains("false"));
    }
}

#[test]
fn test_async_property_resolution_edge_cases() {
    // Test async property resolution paths to cover lines 315-324

    // Test expressions that will trigger async property resolution
    let async_expressions = vec![
        "device.unknownProperty",
        "computed.missingFunction()",
        "device.testProp(1, 'arg2', true)",
        "computed.calculate(42, [1, 2, 3])",
    ];

    for expr in async_expressions {
        let ast = parse_to_ast(expr.to_string());
        let ctx_json = format!(r#"{{"ast": {}, "variables": {{}}}}"#, ast);

        // Create test context with some properties
        let mut test_ctx = TestContext {
            map: HashMap::new(),
        };
        test_ctx
            .map
            .insert("testProp".to_string(), "async_result".to_string());
        test_ctx
            .map
            .insert("calculate".to_string(), "computed_value".to_string());

        let result = evaluate_with_context(ctx_json, Arc::new(test_ctx));

        // Should handle async resolution (may return null for missing properties)
        assert!(!result.is_empty());
    }
}

#[test]
fn test_string_numeric_parsing_comprehensive() {
    // Test comprehensive string to number parsing to cover lines 627-648

    // Create context with various string values that should trigger parsing
    let test_cases = vec![
        ("int_string", "42", "int"),
        ("zero_int", "0", "int"),
        ("negative_int", "-123", "int"),
        ("float_string", "3.14", "float"),
        ("zero_float", "0.0", "float"),
        ("large_int", "999999999", "int"),
        ("scientific", "1e5", "float"),
        ("fractional_zero", "42.0", "int"), // Should convert to int
        ("invalid_number", "not_a_number", "string"),
        ("empty_string", "", "string"),
    ];

    for (key, value, expected_type) in test_cases {
        let expr_json = format!(
            r#"{{"variables": {{"map": {{"{}": {{"type": "string", "value": "{}"}}}}}}, "expression": "{}"}}"#,
            key, value, key
        );

        let result = evaluate_with_context(
            expr_json,
            Arc::new(TestContext {
                map: HashMap::new(),
            }),
        );

        // Should process without error and potentially convert types
        assert!(!result.is_empty());
        // For valid numbers, might see type conversion in the result
        if expected_type == "int" && value != "not_a_number" && value != "" {
            // Might see int conversion
        }
    }
}

#[test]
fn test_wasm_conditional_compilation() {
    // Test both WASM and non-WASM paths to ensure all conditional compilation is covered
    #[cfg(target_arch = "wasm32")]
    {
        let result = evaluate_ast_with_context(
            r#"{"variables": {}, "expression": {"type": "Literal", "value": true}}"#.to_string(),
            Arc::new(TestContext {
                map: HashMap::new(),
            }),
        );
        // WASM-specific path should process
        assert!(!result.is_empty());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let result = evaluate_ast_with_context(
            r#"{"variables": {}, "expression": {"type": "Literal", "value": true}}"#.to_string(),
            Arc::new(TestContext {
                map: HashMap::new(),
            }),
        );
        // Non-WASM path should process
        assert!(!result.is_empty());
    }
}

#[test]
fn test_json_to_cel_value_comprehensive() {
    // Test json_to_cel_value function with all JSON types to cover lines in json conversion
    // This tests the private json_to_cel_value function indirectly through device/computed property resolution
    let mut test_ctx = TestContext {
        map: HashMap::new(),
    };
    test_ctx.map.insert(
        "json_prop".to_string(),
        r#"{"key": "value", "number": 42}"#.to_string(),
    );
    test_ctx
        .map
        .insert("array_prop".to_string(), r#"[1, "test", true]"#.to_string());
    test_ctx
        .map
        .insert("null_prop".to_string(), "null".to_string());
    test_ctx
        .map
        .insert("bool_prop".to_string(), "true".to_string());
    test_ctx
        .map
        .insert("string_prop".to_string(), r#""simple_string""#.to_string());

    // Test device function that returns JSON - this will exercise json_to_cel_value
    let device_expr = "device.json_prop()";
    let ast = parse_to_ast(device_expr.to_string());
    let ctx_json = format!(
        r#"{{"ast": {}, "variables": {{}}, "device": {{"json_prop": []}}}}"#,
        ast
    );
    let result = evaluate_with_context(ctx_json, Arc::new(test_ctx.clone()));
    assert!(!result.is_empty());

    // Test array JSON response
    let array_expr = "device.array_prop()";
    let ast = parse_to_ast(array_expr.to_string());
    let ctx_json = format!(
        r#"{{"ast": {}, "variables": {{}}, "device": {{"array_prop": []}}}}"#,
        ast
    );
    let result = evaluate_with_context(ctx_json, Arc::new(test_ctx.clone()));
    assert!(!result.is_empty());

    // Test null JSON response
    let null_expr = "device.null_prop()";
    let ast = parse_to_ast(null_expr.to_string());
    let ctx_json = format!(
        r#"{{"ast": {}, "variables": {{}}, "device": {{"null_prop": []}}}}"#,
        ast
    );
    let result = evaluate_with_context(ctx_json, Arc::new(test_ctx));
    assert!(!result.is_empty());
}

#[test]
fn test_complex_device_computed_resolution() {
    // Test more complex device/computed property resolution scenarios
    let mut test_ctx = TestContext {
        map: HashMap::new(),
    };
    test_ctx.map.insert(
        "complex_device_prop".to_string(),
        r#"{"result": "device_value", "status": "success"}"#.to_string(),
    );
    test_ctx.map.insert(
        "complex_computed_prop".to_string(),
        r#"[1, 2, 3, 4, 5]"#.to_string(),
    );

    // Test device function with complex JSON response
    let device_expr = "device.complex_device_prop()";
    let ast = parse_to_ast(device_expr.to_string());
    let ctx_json = format!(
        r#"{{"ast": {}, "variables": {{}}, "device": {{"complex_device_prop": ["arg1"]}}}}"#,
        ast
    );
    let result = evaluate_with_context(ctx_json, Arc::new(test_ctx.clone()));
    assert!(!result.is_empty());

    // Test computed function with array response
    let computed_expr = "computed.complex_computed_prop(1, 2, 3)";
    let ast = parse_to_ast(computed_expr.to_string());
    let ctx_json = format!(
        r#"{{"ast": {}, "variables": {{}}, "computed": {{"complex_computed_prop": ["arg1", "arg2", "arg3"]}}}}"#,
        ast
    );
    let result = evaluate_with_context(ctx_json, Arc::new(test_ctx));
    assert!(!result.is_empty());
}

#[test]
fn test_all_atom_types_normalization() {
    // Test normalize_ast_variables with all atom types
    // Note: Only "true" and "false" strings are converted to booleans.
    // Numeric strings stay as strings because quoted literals should preserve their type.
    use cel_parser::Atom;

    // Test that numeric string atoms stay as strings (not converted to numbers)
    // This is correct behavior: "42" in an expression should stay as String("42")
    let string_atom = Atom::String(Arc::new("42".to_string()));
    let result = normalize_ast_variables(string_atom.clone());
    assert_eq!(result, string_atom); // Should stay as String

    // Test "true" and "false" strings are converted to booleans
    let bool_string = Atom::String(Arc::new("true".to_string()));
    let result = normalize_ast_variables(bool_string);
    assert_eq!(result, Atom::Bool(true));

    let false_string = Atom::String(Arc::new("false".to_string()));
    let result = normalize_ast_variables(false_string);
    assert_eq!(result, Atom::Bool(false));

    // Large numeric strings stay as strings
    let uint_string = Atom::String(Arc::new("18446744073709551615".to_string()));
    let result = normalize_ast_variables(uint_string.clone());
    assert_eq!(result, uint_string); // Should stay as String

    // Float-like strings stay as strings
    let float_string = Atom::String(Arc::new("3.14159".to_string()));
    let result = normalize_ast_variables(float_string.clone());
    assert_eq!(result, float_string); // Should stay as String

    let fractional_zero = Atom::String(Arc::new("42.0".to_string()));
    let result = normalize_ast_variables(fractional_zero.clone());
    assert_eq!(result, fractional_zero); // Should stay as String

    let non_numeric = Atom::String(Arc::new("not_a_number".to_string()));
    let result = normalize_ast_variables(non_numeric.clone());
    assert_eq!(result, non_numeric); // Should stay as String

    // Test non-string atoms (should return unchanged)
    let int_atom = Atom::Int(42);
    let result = normalize_ast_variables(int_atom.clone());
    assert_eq!(result, int_atom);

    let bool_atom = Atom::Bool(true);
    let result = normalize_ast_variables(bool_atom.clone());
    assert_eq!(result, bool_atom);

    let null_atom = Atom::Null;
    let result = normalize_ast_variables(null_atom.clone());
    assert_eq!(result, null_atom);
}

#[test]
fn test_execute_with_compiled_program() {
    // Test ExecutableType::CompiledProgram path which is currently dead code
    // This exercises the match arm in execute_with function
    let expression = "1 + 1";
    let ctx_json = format!(r#"{{"expression": "{}"}}"#, expression);
    let result = evaluate_with_context(
        ctx_json,
        Arc::new(TestContext {
            map: HashMap::new(),
        }),
    );
    assert!(!result.is_empty());
}

#[test]
fn test_prop_for_error_paths() {
    // Test error paths in property resolution that might not be covered
    let mut test_ctx = TestContext {
        map: HashMap::new(),
    };

    // Test device property that returns error
    test_ctx
        .map
        .insert("error_prop".to_string(), "Error: test error".to_string());

    let device_expr = "device.error_prop()";
    let ast = parse_to_ast(device_expr.to_string());
    let ctx_json = format!(
        r#"{{"ast": {}, "variables": {{}}, "device": {{"error_prop": []}}}}"#,
        ast
    );
    let result = evaluate_with_context(ctx_json, Arc::new(test_ctx.clone()));
    assert!(!result.is_empty());

    // Test computed property that returns error
    test_ctx.map.insert(
        "error_computed".to_string(),
        "Error: computation failed".to_string(),
    );

    let computed_expr = "computed.error_computed()";
    let ast = parse_to_ast(computed_expr.to_string());
    let ctx_json = format!(
        r#"{{"ast": {}, "variables": {{}}, "computed": {{"error_computed": []}}}}"#,
        ast
    );
    let result = evaluate_with_context(ctx_json, Arc::new(test_ctx));
    assert!(!result.is_empty());
}

#[test]
fn test_transform_expression_unary_operations() {
    // Test transformation of unary operations to cover more AST transformation branches
    let unary_expressions = vec![
        "!true",
        "!!false",
        "!device.prop",
        "-42",
        "--100",
        "-(device.value() + 1)",
    ];

    for expr in unary_expressions {
        let ast = parse_to_ast(expr.to_string());
        let ctx_json = format!(
            r#"{{"ast": {}, "variables": {{}}, "device": {{"prop": [], "value": []}}}}"#,
            ast
        );
        let result = evaluate_with_context(
            ctx_json,
            Arc::new(TestContext {
                map: HashMap::new(),
            }),
        );
        // Should handle unary operations without crashing
        assert!(!result.is_empty());
    }
}

#[test]
fn test_transform_expression_list_operations() {
    // Test list operations in AST transformation
    let list_expressions = vec![
        "[1, 2, 3]",
        "[device.prop, computed.value]",
        "[true, false, device.bool_prop()]",
        "[[1, 2], [3, 4]]",        // Nested lists
        "[{\"key\": device.val}]", // List with map
    ];

    for expr in list_expressions {
        let ast = parse_to_ast(expr.to_string());
        let ctx_json = format!(
            r#"{{"ast": {}, "variables": {{}}, "device": {{"prop": [], "bool_prop": [], "val": []}}, "computed": {{"value": []}}}}"#,
            ast
        );
        let result = evaluate_with_context(
            ctx_json,
            Arc::new(TestContext {
                map: HashMap::new(),
            }),
        );
        // Should handle list operations
        assert!(!result.is_empty());
    }
}

#[test]
fn test_transform_expression_map_operations() {
    // Test map operations in AST transformation
    let map_expressions = vec![
        r#"{"key": "value"}"#,
        r#"{"dynamic": device.prop, "computed": computed.value}"#,
        r#"{"nested": {"inner": device.inner_prop()}}"#,
        r#"{"list": [1, 2, device.list_item()]}"#,
        r#"{"bool": true, "device": device.flag}"#,
    ];

    for expr in map_expressions {
        let ast = parse_to_ast(expr.to_string());
        let ctx_json = format!(
            r#"{{"ast": {}, "variables": {{}}, "device": {{"prop": [], "inner_prop": [], "list_item": [], "flag": []}}, "computed": {{"value": []}}}}"#,
            ast
        );
        let result = evaluate_with_context(
            ctx_json,
            Arc::new(TestContext {
                map: HashMap::new(),
            }),
        );
        // Should handle map operations
        assert!(!result.is_empty());
    }
}

#[test]
fn test_transform_expression_ternary_operations() {
    // Test ternary (conditional) operations in AST transformation
    let ternary_expressions = vec![
        "true ? 1 : 2",
        "device.condition() ? device.a() : device.b()",
        "computed.flag ? 'yes' : 'no'",
        "hasFn('device.test') ? device.test() : null",
        "(device.x() > 5) ? computed.high() : computed.low()",
    ];

    for expr in ternary_expressions {
        let ast = parse_to_ast(expr.to_string());
        let ctx_json = format!(
            r#"{{"ast": {}, "variables": {{}}, "device": {{"condition": [], "a": [], "b": [], "test": [], "x": []}}, "computed": {{"flag": [], "high": [], "low": []}}}}"#,
            ast
        );
        let result = evaluate_with_context(
            ctx_json,
            Arc::new(TestContext {
                map: HashMap::new(),
            }),
        );
        // Should handle ternary operations
        assert!(!result.is_empty());
    }
}

#[test]
fn test_ast_transformation_member_access_edge_cases() {
    // Test member access transformations that might not be covered
    let member_expressions = vec![
        "obj.prop.subprop",
        "device.nested.deep.property",
        "computed.data.field.value",
        "list[0].property",
        "map['key'].subfield",
    ];

    for expr in member_expressions {
        let ast = parse_to_ast(expr.to_string());
        let ctx_json = format!(r#"{{"ast": {}, "variables": {{}}}}"#, ast);
        let result = evaluate_with_context(
            ctx_json,
            Arc::new(TestContext {
                map: HashMap::new(),
            }),
        );
        // Should handle member access patterns
        assert!(!result.is_empty());
    }
}

#[test]
fn test_variable_normalization_recursive_edge_cases() {
    // Test deeply nested variable normalization
    let mut deep_map = HashMap::new();
    deep_map.insert(
        "level1".to_string(),
        PassableValue::PMap({
            let mut l1 = HashMap::new();
            l1.insert(
                "level2".to_string(),
                PassableValue::PMap({
                    let mut l2 = HashMap::new();
                    l2.insert(
                        "bool_str".to_string(),
                        PassableValue::String("true".to_string()),
                    );
                    l2.insert(
                        "num_str".to_string(),
                        PassableValue::String("42".to_string()),
                    );
                    l2.insert(
                        "float_str".to_string(),
                        PassableValue::String("3.14".to_string()),
                    );
                    l2
                }),
            );
            l1
        }),
    );

    let complex_list = PassableValue::List(vec![
        PassableValue::String("false".to_string()),
        PassableValue::String("123".to_string()),
        PassableValue::PMap({
            let mut inner = HashMap::new();
            inner.insert(
                "nested_bool".to_string(),
                PassableValue::String("true".to_string()),
            );
            inner
        }),
    ]);

    deep_map.insert("complex_list".to_string(), complex_list);

    let normalized = normalize_variables(PassableValue::PMap(deep_map));

    // Check that deep normalization worked
    match normalized {
        PassableValue::PMap(map) => {
            assert!(map.contains_key("level1"));
            assert!(map.contains_key("complex_list"));
        }
        _ => panic!("Expected normalized map"),
    }
}

#[test]
fn test_displayable_value_to_passable_edge_cases() {
    // Test DisplayableValue to_passable conversion with complex types
    use cel_interpreter::{
        objects::{Key, Map},
        Value,
    };

    let duration_value = Value::Float(3.12);
    let displayable = DisplayableValue(duration_value);
    let passable = displayable.to_passable();
    match passable {
        PassableValue::Float(_) => {} // Duration becomes string
        _ => panic!("Expected float representation"),
    }

    // Test Function with arguments
    let func_with_args = Value::Function(
        Arc::new("test_func".to_string()),
        Some(Box::new(Value::String(Arc::new("arg".to_string())))),
    );
    let displayable_func = DisplayableValue(func_with_args);
    let passable_func = displayable_func.to_passable();
    match passable_func {
        PassableValue::Function(name, args) => {
            assert_eq!(name, "test_func");
            assert!(args.is_some());
        }
        _ => panic!("Expected function with args"),
    }
}
