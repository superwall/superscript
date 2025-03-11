#!/bin/bash

set -e

echo "Building for Android x86_64, armv7, aarch64:"

export ANDROID_NDK=r25b ANDROID_SDK=26 ANDROID_VERSION=11.0.0_r48

build_targets=(
    "x86_64-linux-android"
    "armv7-linux-androideabi"
    "aarch64-linux-android"
    "i686-linux-android"
)

export CROSS_NO_WARNINGS=0
for target in "${build_targets[@]}"; do
    rustup target add "$target"
    echo "Building for $target"
    cross build --target "$target" --release --lib
done

echo "Copying results to target/android/jniLibs"

target_dir="target/android"
jniLibs_dir="${target_dir}/jniLibs"

mkdir -p "${jniLibs_dir}"/{arm64-v8a,armeabi-v7a,x86_64,x86,aarch64}

cp target/aarch64-linux-android/release/libcel_eval.so "${jniLibs_dir}/arm64-v8a/libuniffi_cel.so"
cp target/aarch64-linux-android/release/libcel_eval.so "${jniLibs_dir}/aarch64/libuniffi_cel.so"
cp target/armv7-linux-androideabi/release/libcel_eval.so "${jniLibs_dir}/armeabi-v7a/libuniffi_cel.so"
cp target/x86_64-linux-android/release/libcel_eval.so "${jniLibs_dir}/x86_64/libuniffi_cel.so"
cp target/i686-linux-android/release/libcel_eval.so "${jniLibs_dir}/x86/libuniffi_cel.so"

echo "Running UniFFI to generate Kotlin bindings"
mkdir -p "${target_dir}/java/uniffi/cel"
cargo run --features=uniffi/cli \
    --bin uniffi-bindgen \
    generate src/cel.udl \
    --language kotlin \
    --out-dir ./target/android/java/uniffi/cel


echo "Build done --- ✅"
printf "\e]8;;file://%s/%s\aFind your output in ./%s\n" "$(pwd)" "$target_dir" "$target_dir"
