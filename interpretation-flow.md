# Superscript Expression Interpretation Flow

This document describes the complete evaluation process for Superscript expressions in the Superscript library, including parsing, variable retrieval, normalization, and transformation steps.

## Overview

The Superscript evaluation system processes expressions through several key stages:
1. **Input Parsing** - Parse JSON execution context and Superscript expression string
2. **Variable Normalization** - Transform string values to appropriate types  
3. **AST Transformation** - Apply null-safety transformations
4. **Context Setup** - Initialize Superscript context with variables and functions
5. **Property Resolution** - Handle dynamic device/computed properties
6. **Expression Evaluation** - Execute the transformed AST

## Example Expression Analysis

Let's trace through the expression: `device.daysSince("some_event") > "3.0" && computed.some_property == "true"`

### Step 1: Input Parsing

**Input JSON:**
```json
{
  "variables": {
    "map": {
      "some_property": {"type": "string", "value": "true"}
    }
  },
  "expression": "device.daysSince(\"some_event\") > \"3.0\" && computed.some_property == \"true\"",
  "device": {
    "daysSince": [{"type": "string", "value": "event_name"}]
  },
  "computed": {
    "some_property": []
  }
}
```

Note: The `device` and `computed` sections define available functions that the host exposes to Superscript. These are function signatures, not variable values.

**Parsed Expression AST:**
```
And(
  Relation(
    FunctionCall(
      Member(Ident("device"), Attribute("daysSince")),
      None,
      [Atom(String("some_event"))]
    ),
    GreaterThan,
    Atom(String("3.0"))
  ),
  Relation(
    Member(Ident("computed"), Attribute("some_property")),
    Equals,
    Atom(String("true"))
  )
)
```

### Step 2: Variable Normalization

The `normalize_variables` function (src/lib.rs:557) processes all variables:

**Before normalization:**
- `some_property`: `{"type": "string", "value": "true"}` → becomes `Bool(true)`
- Right-side literal `"3.0"`: string → becomes `Float(3.0)` 
- Right-side literal `"true"`: string → becomes `Bool(true)`

**After normalization:**
- Variables: `{"some_property": Bool(true)}`
- AST atoms transformed by `normalize_ast_variables` (src/lib.rs:593)

### Step 3: AST Null-Safety Transformation

The `transform_expression_for_null_safety` function (src/lib.rs:654) applies two types of transformations:

#### 3a. Property Access Null-Safety
Note: `computed.some_property` is a variable access, not a function call, so it gets null-safety transformation.

**Original:**
```
computed.some_property
```

**Transformed:**
```
has(computed.some_property) ? computed.some_property : null
```

#### 3b. Function Call hasFn Wrapping
Device and computed function calls get wrapped with `hasFn` checks to ensure graceful handling of missing functions.

**Original:**
```
device.daysSince("some_event")
```

**Transformed:**
```
hasFn("device.daysSince") ? device.daysSince("some_event") : false
```

**Full transformed expression:**
```
And(
  Relation(
    Ternary(
      FunctionCall(Ident("hasFn"), None, [Atom(String("device.daysSince"))]),
      FunctionCall(
        Member(Ident("device"), Attribute("daysSince")),
        None,
        [Atom(String("some_event"))]
      ),
      Atom(Bool(false))
    ),
    GreaterThan,
    Atom(Float(3.0))
  ),
  Relation(
    Ternary(
      FunctionCall(Ident("has"), None, [Member(Ident("computed"), Attribute("some_property"))]),
      Member(Ident("computed"), Attribute("some_property")),
      Atom(Null)
    ),
    Equals,
    Atom(Bool(true))
  )
)
```

### Step 4: Context Setup

The `execute_with` function (src/lib.rs:180) sets up the Superscript evaluation context:

1. **Variables added to context:**
   - `some_property` → `Bool(true)`

2. **Utility functions registered from `SUPPORTED_FUNCTIONS` constant (src/lib.rs:32):**
   - `maybe`, `toString`, `toBool`, `toInt`, `toFloat`, `hasFn`, `has`

3. **Device function map created from host-exposed functions:**
   ```rust
   device_host_properties = {
     "daysSince": Function("daysSince", Some(List([String("event_name")])))
   }
   ```

4. **Computed object created:**
   The `computed` object contains both:
   - Host-exposed functions: `some_property: Function("some_property", None)`  
   - Variable properties: `some_property` → `Bool(true)` (from variables map)

5. **Objects added to context:**
   - `device` → `Map(device_host_properties)`
   - `computed` → `Map(computed_host_properties + variables)`

### Step 5: Property Resolution

**For `device.daysSince("some_event")` (wrapped with hasFn):**
1. `hasFn("device.daysSince")` is evaluated first - checks if function is available
2. If available: Function `daysSince` is called with args `["some_event"]`
3. `prop_for(PropType::Device, "daysSince", Some([String("some_event")]), host)` is invoked
4. Host context's `device_property("daysSince", "[\"some_event\"]", callback)` is called
5. Host returns serialized result (e.g., `{"type": "uint", "value": 5}`)
6. Result is deserialized and normalized to `UInt(5)`
7. If not available: Returns `false` instead of throwing an error

**For `computed.some_property`:**
1. Null-safety check: `has(computed.some_property)` evaluates first
2. Since `some_property` is a variable (not a host function), it resolves to `Bool(true)` from the variables context
3. No host call is made - this is a direct variable lookup

### Step 6: Expression Evaluation

**Evaluation proceeds step by step:**

1. **Left side:** `hasFn("device.daysSince") ? device.daysSince("some_event") : false > 3.0`
   - `hasFn("device.daysSince")` → `Bool(true)` (function is available)
   - `device.daysSince("some_event")` → `UInt(5)` (from host function call)
   - `UInt(5) > Float(3.0)` → `Bool(true)` (implicit type conversion)

2. **Right side:** `has(computed.some_property) ? computed.some_property : null == true`
   - `has(computed.some_property)` → `Bool(true)` (variable exists in context)
   - `computed.some_property` → `Bool(true)` (from variables map)
   - `Bool(true) == Bool(true)` → `Bool(true)`

3. **Final result:** `Bool(true) && Bool(true)` → `Bool(true)`

## Key Components

### Built-in Functions

- **`SUPPORTED_FUNCTIONS` constant** (src/lib.rs:32): Defines all built-in functions available in Superscript
  - Single source of truth for function availability
  - Used by both AST transformation and runtime evaluation
  - Functions: `"maybe"`, `"toString"`, `"toBool"`, `"toFloat"`, `"toInt"`, `"hasFn"`, `"has"`

### Normalization Functions

- **`normalize_variables`** (src/lib.rs:557): Recursively converts string representations of primitives
  - `"true"/"false"` → `Bool`
  - Numeric strings → `Int`/`UInt`/`Float`
  - Nested maps/lists are processed recursively

- **`normalize_ast_variables`** (src/lib.rs:593): Similar normalization for AST atoms

### Transformation Functions

- **`transform_expression_for_null_safety`** (src/lib.rs:654): Applies two types of safety transformations:
  - **Property access**: Wraps with `has()` checks to prevent `UndeclaredReference` errors
  - **Function calls**: Wraps device/computed function calls with `hasFn()` checks using the host-exposed function lists
  - Returns `null` for missing properties or `false` for unavailable functions instead of throwing errors

### Property Resolution

- **`prop_for`** function handles async property resolution from host context
- Supports both device and computed host-exposed functions
- Results are JSON-serialized for transport and deserialized back
- Variables are resolved directly from context without host calls

### Error Handling

The system gracefully handles errors by converting them to `null`:
- `UndeclaredReference` → `null`
- `Unknown function` → `null`  
- `Null can not be compared` → `null`

## Data Flow Diagram

```
Input JSON
    ↓
[Parse] → ExecutionContext
    ↓
[Normalize Variables] → Standardized types
    ↓
[Parse Expression] → AST
    ↓
[Transform AST] → Null-safe AST
    ↓
[Setup Context] → Superscript Context + Variables + Functions
    ↓
[Register Host Functions] → Device/Computed host-exposed functions
    ↓
[Evaluate AST] → Property resolution (host calls for functions, direct lookup for variables)
    ↓
[Return Result] → Serialized PassableValue
```

This flow ensures robust evaluation of Superscript expressions with proper type handling, null safety, and dynamic property resolution.