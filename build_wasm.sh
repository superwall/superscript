#!/bin/bash

set -e

echo "0% - Building for WASM(browser,node):"
cargo build --lib --release --target wasm32-unknown-unknown
echo " 25% - WASM build successful ✅"
echo " 25% - Ensuring WASM wrapper is built"
cd wasm
cargo build --lib --release --target wasm32-unknown-unknown
echo "50% - WASM wrapper build successful ✅"

echo "50% - Building JS bundles"
mkdir -p ./target/browser
mkdir -p ./target/node
bun run build

echo "75% - Build done - ✅"
echo "75% - Installing to example project"
cd ../examples/browser/
bun install ../../wasm/target/browser

echo "100% - Done - ✅"