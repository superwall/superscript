# CHANGELOG

## 1.0.12

- Bump version

## 1.0.11

## Fixes
- Fixes comparison issue of padded numbers by skipping expression conversion and normalization in special cases

## 1.0.10

## Fixes
- Removes modulemap from outputs

## 1.0.9

## Fixes
- Ensures XC projects also work with module SPM setup
## 1.0.8

## Fixes
- Fix generic module name

## 1.0.7

## Fixes
- Fix iOS build script

## 1.0.6

## Fixes
- Ensure new module headers point to right places

## 1.0.5

## Fixes
- Fixes namespaces for SPM module headers

## 1.0.4

## Enhancements
- Disable Android Cleaner in UniFFI builds

## 1.0.3

## Enhancements
- Ensure that previously set compilation flags do not affect Android compilation
- Add flags to ensure common page size is being passed to Cross

## 1.0.2.

## Enhancements
- Removes log print, reduces binary size

## 1.0.1

## Enhancements
- Adds `hasFn` function that checks for the existance of a function or returns `false`
- Enhance `hasFn` and `has` checks to do the following:
  - If a `device.` or `computed.` function is used, or a variable is accessed in an expression
  - Wrap the accessor in `has` or `hasFn`
  - Wrap the evaluation to:
    - Evaluate `has/hasFn` first
    - if true, run the expression.
    - if false and the right side is atomic, we evaluate the default fallback value (`0` for `int/float`, `""` for `String`)
    - if false and the right side is not atomic, we wrap the whole expression to return `false` (to avoid error due to comparing different types)
- Removes `string.toBool()`,`string.toInt()`, `string.toFloat()` functions as every possible valid atom conversion is done in the AST

## General
- Adds more tests, improves test coverage, adds displaying coverage badge
- Improves `README.MD` and adds an `interpretation-flow.md` to serve as a guide for how things are interpreted

## 1.0.0

## Enhancements
- Adds truthiness and string normalization so value such as "true", "false", "1.1" etc are treated as true, false, 1.1. This occurs on both left and right side of an expression.
- Adds conversion methods `bool.toString()`, `float.toString()`, `int.toString()`, `bool.toString()` and `string.toBool()`,`string.toInt()`, `string.toFloat()` to enable typecasting in CEL

## Truthiness
- Fixes issues with undeclared references for properties and functions by wrapping them in a has(x)? x : Null tertiary expression

## 0.2.8

### Enhancements

- Pass linker flags for 16kb architecture on Android

## 0.2.7

- Version bump

## 0.2.6

### Enhancements

- Removes string return requirement from HostContext methods

## 0.2.5

### Enhancements
- Moves the HostContext to a Sync version with callback
- Updates Android NDK to support 16kb page sizes
- Updates Uniffi version

## 0.2.4
- Fix aarch64 build for Android

## 0.2.3
- Fix aarch64 build for Android

## 0.2.2
- Version bump for deployment purposes

## 0.2.1

- Readme and example updates

## 0.2.0

- Add typescript wrapper for the JS library
- Add aarch64 support for Android

## 0.1.16

- Bumped version for iOS cocoapods fix.

## 0.1.15

- Add watchOS, visionOS and Catalyst targets

## 0.1.12

### Fixes

- Adds Result types to fix issues when building for iOS.

## 0.1.11

### Enhancements

- Adds new Android target
- Ensures JSON deserialization is done in a safe manner

## 0.1.10

### Enhancements

- Updates github workflow for the renaming of the iOS repository.

## 0.1.9

### Enhancements

- Added returning of a JSON encoded `Result<PassableValue,String>` from the exposed methods instead of relying on panics.
  Example JSON:
  - Error: `{"Err":"No such key: should_display"}`
  - Ok: `{"Ok":{"type":"bool","value":true}}`

### Fixes

- Fixed a bug where getting properties from `device` would panic when `device` functions were defined
