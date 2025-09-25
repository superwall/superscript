#!/bin/bash

set -e

echo "=== Testing 16KB page size alignment ==="
echo

# Check if libraries exist, if not suggest building
if [ ! -d "target/android/jniLibs" ] || [ -z "$(find target/android/jniLibs -name "*.so" 2>/dev/null)" ]; then
    echo "No built libraries found. Please run: ./build_android.sh first"
    echo "Or run with --build to build automatically:"
    echo "  $0 --build"
    if [ "$1" = "--build" ]; then
        echo
        echo "Building Android libraries with 16KB page size..."
        ./build_android.sh
    else
        exit 1
    fi
fi

echo
echo "=== Checking ELF alignment of built libraries ==="

# Check the built libraries using the external script
./check_elf_alignment.sh target/android/jniLibs/

echo
echo "=== Test Complete ==="
if [ $? -eq 0 ]; then
    echo "✅ All libraries are properly aligned for 16KB page size!"
else
    echo "❌ Some libraries are not properly aligned for 16KB page size."
    exit 1
fi