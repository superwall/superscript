#!/bin/bash

# Function to set up xcconfig environment variables
setup_xcconfig_env() {
    # Common settings
    export ONLY_ACTIVE_ARCH="YES"
    export DEFINES_MODULE="YES"
    export VERSIONING_SYSTEM="apple-generic"
    export TARGETED_DEVICE_FAMILY="1,2"
    export IPHONEOS_DEPLOYMENT_TARGET="12"
    export MACOSX_DEPLOYMENT_TARGET="10.2"
    export MACOS_DEPLOYMENT_TARGET="10.2"
    export DYLIB_COMPATIBILITY_VERSION="1"
    export DYLIB_CURRENT_VERSION="1"
    export DYLIB_INSTALL_NAME_BASE="@rpath"
    export LD_RUNPATH_SEARCH_PATHS="@executable_path/Frameworks @loader_path/Frameworks"
    export SWIFT_VERSION="5.0"
    export ENABLE_BITCODE="NO"

    # Release-specific settings
    export SWIFT_OPTIMIZATION_LEVEL="-O"
    export SWIFT_ACTIVE_COMPILATION_CONDITIONS=""
    export SWIFT_COMPILATION_MODE="wholemodule"
    export ENABLE_TESTABILITY="NO"
    export GCC_OPTIMIZATION_LEVEL="s"
    export GCC_PREPROCESSOR_DEFINITIONS=""

    # Common compiler settings
    export GCC_DYNAMIC_NO_PIC="NO"
    export GCC_NO_COMMON_BLOCKS="YES"
    export GCC_C_LANGUAGE_STANDARD="gnu11"
    export CLANG_CXX_LANGUAGE_STANDARD="gnu++14"
    export CLANG_CXX_LIBRARY="libc++"
    export CLANG_ENABLE_OBJC_ARC="YES"
    export CLANG_ENABLE_OBJC_WEAK="YES"
    export ENABLE_STRICT_OBJC_MSGSEND="YES"

    # Warning flags
    export CLANG_WARN_BLOCK_CAPTURE_AUTORELEASING="YES"
    export CLANG_WARN_EMPTY_BODY="YES"
    export CLANG_WARN_BOOL_CONVERSION="YES"
    export CLANG_WARN_CONSTANT_CONVERSION="YES"
    export GCC_WARN_64_TO_32_BIT_CONVERSION="YES"
    export CLANG_WARN_ENUM_CONVERSION="YES"
    export CLANG_WARN_INT_CONVERSION="YES"
    export CLANG_WARN_NON_LITERAL_NULL_CONVERSION="YES"
    export CLANG_WARN_INFINITE_RECURSION="YES"
    export GCC_WARN_ABOUT_RETURN_TYPE="YES"
    export CLANG_WARN_STRICT_PROTOTYPES="YES"
    export CLANG_WARN_COMMA="YES"
    export GCC_WARN_UNINITIALIZED_AUTOS="YES"
    export CLANG_WARN_UNREACHABLE_CODE="YES"
    export GCC_WARN_UNUSED_FUNCTION="YES"
    export GCC_WARN_UNUSED_VARIABLE="YES"
    export CLANG_WARN_RANGE_LOOP_ANALYSIS="YES"
    export CLANG_WARN_SUSPICIOUS_MOVE="YES"
    export CLANG_WARN__DUPLICATE_METHOD_MATCH="YES"
    export CLANG_WARN_OBJC_LITERAL_CONVERSION="YES"
    export CLANG_WARN_DEPRECATED_OBJC_IMPLEMENTATIONS="YES"
    export GCC_WARN_UNDECLARED_SELECTOR="YES"
    export CLANG_WARN_OBJC_IMPLICIT_RETAIN_SELF="YES"
}

# Set up xcconfig environment
setup_xcconfig_env

# Set base RUSTFLAGS
export RUSTFLAGS="-C opt-level=3 -C debuginfo=0 -C strip=symbols"

set -e
echo "Building the rust library for binding generation"
cargo build --lib

echo "Generating Swift bindings in .target/ios/"
# Generate bindings
cargo run --features=uniffi/cli \
     --bin uniffi-bindgen \
     generate src/cel.udl \
     --language swift \
     --out-dir ./target/ios/
# Add the iOS targets and build
for TARGET in \
        aarch64-apple-darwin \
        aarch64-apple-ios \
        aarch64-apple-ios-sim \
        aarch64-apple-ios-macabi \
        x86_64-apple-darwin \
        x86_64-apple-ios \
        x86_64-apple-ios-macabi
do
    rustup target add $TARGET
    echo "Building for $TARGET"
    cargo build --target=$TARGET --lib --release
done

# For visionOS device
SDKROOT="$(xcrun --sdk xros --show-sdk-path)" \
cargo +nightly build -Zbuild-std=std,core,alloc,panic_abort --target=aarch64-apple-visionos --lib --release
# For visionOS simulator
SDKROOT="$(xcrun --sdk xrsimulator --show-sdk-path)" \
cargo +nightly build -Zbuild-std=std,core,alloc,panic_abort --target=aarch64-apple-visionos-sim --lib --release

# For watchOS device
SDKROOT="$(xcrun --sdk watchos --show-sdk-path)" \
cargo +nightly build -Zbuild-std=std,core,alloc,panic_abort --target=arm64_32-apple-watchos --lib --release
# For watchOS simulator
SDKROOT="$(xcrun --sdk watchsimulator --show-sdk-path)" \
cargo +nightly build -Zbuild-std=std,core,alloc,panic_abort --target=aarch64-apple-watchos-sim --lib --release
SDKROOT="$(xcrun --sdk watchsimulator --show-sdk-path)" \
cargo +nightly build -Zbuild-std=std,core,alloc,panic_abort --target=x86_64-apple-watchos-sim --lib --release


# Rename *.modulemap to module.modulemap
mv ./target/ios/celFFI.modulemap ./target/ios/module.modulemap

rm -rf target/ios/ios.xcframework
rm -rf target/ios/macos.xcframework

mkdir -p ./target/ios-sim/release/
mkdir -p ./target/ios/release/
mkdir -p ./target/macos/release/
mkdir -p ./target/visionos/release/
mkdir -p ./target/watchos/release/

# Clean up previous build artifacts
rm -rf ./target/xcframeworks

# Create directories for headers
mkdir -p ./target/xcframeworks/headers/ios-simulator
mkdir -p ./target/xcframeworks/headers/ios-device
mkdir -p ./target/xcframeworks/headers/macos
mkdir -p ./target/xcframeworks/headers/catalyst
mkdir -p ./target/xcframeworks/headers/visionos-simulator
mkdir -p ./target/xcframeworks/headers/visionos-device
mkdir -p ./target/xcframeworks/headers/watchos-simulator
mkdir -p ./target/xcframeworks/headers/watchos-device

# Copy headers
cp -R ./target/ios/* ./target/xcframeworks/headers/ios-simulator/
cp -R ./target/ios/* ./target/xcframeworks/headers/ios-device/
cp -R ./target/ios/* ./target/xcframeworks/headers/macos/
cp -R ./target/ios/* ./target/xcframeworks/headers/catalyst/
cp -R ./target/ios/* ./target/xcframeworks/headers/visionos-simulator/
cp -R ./target/ios/* ./target/xcframeworks/headers/visionos-device/
cp -R ./target/ios/* ./target/xcframeworks/headers/watchos-simulator/
cp -R ./target/ios/* ./target/xcframeworks/headers/watchos-device/

# iOS Simulator (combined arm64 and x86_64)
echo "Preparing iOS Simulator library (universal binary)"
mkdir -p ./target/xcframeworks/ios-simulator
lipo -create ./target/aarch64-apple-ios-sim/release/libcel_eval.a \
    ./target/x86_64-apple-ios/release/libcel_eval.a \
    -output ./target/xcframeworks/ios-simulator/libcel.a
strip -x ./target/xcframeworks/ios-simulator/libcel.a

# iOS Device (arm64)
echo "Preparing iOS Device (arm64) library"
mkdir -p ./target/xcframeworks/ios-device
cp ./target/aarch64-apple-ios/release/libcel_eval.a ./target/xcframeworks/ios-device/libcel.a
strip -x ./target/xcframeworks/ios-device/libcel.a

# macOS (combined arm64 and x86_64)
echo "Preparing macOS library (universal binary)"
mkdir -p ./target/xcframeworks/macos
lipo -create ./target/aarch64-apple-darwin/release/libcel_eval.a \
    ./target/x86_64-apple-darwin/release/libcel_eval.a \
    -output ./target/xcframeworks/macos/libcel.a
strip -x ./target/xcframeworks/macos/libcel.a

# Mac Catalyst (combined arm64 and x86_64)
echo "Preparing Mac Catalyst library (universal binary)"
mkdir -p ./target/xcframeworks/catalyst
lipo -create ./target/aarch64-apple-ios-macabi/release/libcel_eval.a \
    ./target/x86_64-apple-ios-macabi/release/libcel_eval.a \
    -output ./target/xcframeworks/catalyst/libcel.a
strip -x ./target/xcframeworks/catalyst/libcel.a

# visionOS Simulator (arm64)
echo "Preparing visionOS Simulator library"
mkdir -p ./target/xcframeworks/visionos-simulator
cp ./target/aarch64-apple-visionos-sim/release/libcel_eval.a ./target/xcframeworks/visionos-simulator/libcel.a
strip -x ./target/xcframeworks/visionos-simulator/libcel.a

# visionOS Device (arm64)
echo "Preparing visionOS Device library"
mkdir -p ./target/xcframeworks/visionos-device
cp ./target/aarch64-apple-visionos/release/libcel_eval.a ./target/xcframeworks/visionos-device/libcel.a
strip -x ./target/xcframeworks/visionos-device/libcel.a

# watchOS Simulator (combined arm64 and x86_64)
echo "Preparing watchOS Simulator library (universal binary)"
mkdir -p ./target/xcframeworks/watchos-simulator
lipo -create ./target/aarch64-apple-watchos-sim/release/libcel_eval.a \
    ./target/x86_64-apple-watchos-sim/release/libcel_eval.a \
    -output ./target/xcframeworks/watchos-simulator/libcel.a
strip -x ./target/xcframeworks/watchos-simulator/libcel.a

# watchOS Device (arm64)
echo "Preparing watchOS Device library"
mkdir -p ./target/xcframeworks/watchos-device
cp ./target/arm64_32-apple-watchos/release/libcel_eval.a ./target/xcframeworks/watchos-device/libcel.a
strip -x ./target/xcframeworks/watchos-device/libcel.a

echo "Building XCFramework"
xcodebuild -create-xcframework \
    -library ./target/xcframeworks/ios-simulator/libcel.a -headers ./target/xcframeworks/headers/ios-simulator \
    -library ./target/xcframeworks/ios-device/libcel.a -headers ./target/xcframeworks/headers/ios-device \
    -library ./target/xcframeworks/macos/libcel.a -headers ./target/xcframeworks/headers/macos \
    -library ./target/xcframeworks/catalyst/libcel.a -headers ./target/xcframeworks/headers/catalyst \
    -library ./target/xcframeworks/visionos-simulator/libcel.a -headers ./target/xcframeworks/headers/visionos-simulator \
    -library ./target/xcframeworks/visionos-device/libcel.a -headers ./target/xcframeworks/headers/visionos-device \
    -library ./target/xcframeworks/watchos-simulator/libcel.a -headers ./target/xcframeworks/headers/watchos-simulator \
    -library ./target/xcframeworks/watchos-device/libcel.a -headers ./target/xcframeworks/headers/watchos-device \
    -output ./target/xcframeworks/libcel.xcframework

<<<<<<< Updated upstream
=======
<<<<<<< Updated upstream
echo "XCFramework built at ./target/xcframeworks/libcel.xcframework"
=======
>>>>>>> Stashed changes
echo "XCFramework built at ./target/xcframeworks/libcel.xcframework"

# Restructure module maps to avoid SPM conflicts
echo "Restructuring module maps to avoid SPM conflicts"
XCFW_PATH="./target/xcframeworks/libcel.xcframework"

# For each slice in the xcframework
for slice_dir in "$XCFW_PATH"/*/; do
    if [ -f "$slice_dir/Headers/module.modulemap" ]; then
        slice_name=$(basename "$slice_dir")
        echo "  Processing $slice_name"

        # Create Modules directory
        mkdir -p "$slice_dir/Modules"

        # Move module.modulemap from Headers to Modules
        mv "$slice_dir/Headers/module.modulemap" "$slice_dir/Modules/module.modulemap"
<<<<<<< Updated upstream
    fi
done

echo "Module map restructuring complete - SPM compatible"
=======

        # Update header paths in modulemap to point to ../Headers/
        sed -i '' 's|header "\([^"]*\)"|header "../Headers/\1"|g' "$slice_dir/Modules/module.modulemap"
    fi
done

echo "Module map restructuring complete - SPM compatible"
>>>>>>> Stashed changes
>>>>>>> Stashed changes
