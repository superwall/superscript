# CHANGELOG

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
