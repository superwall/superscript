#!/usr/bin/env bash

# Function to print section sizes for a binary
analyze_binary() {
    local file=$1
    echo "=== Analysis for: $file ==="
    echo "Basic file info:"
    file "$file"
    echo

    echo "Overall file size:"
    ls -lh "$file"
    echo

    echo "Architectures:"
    if [[ "$file" == *.a ]]; then
        # For .a files, we need to check the objects inside
        for obj in $(ar t "$file"); do
            echo "Object: $obj"
            ar x "$file" "$obj" 2>/dev/null
            if [ -f "$obj" ]; then
                lipo -info "$obj" 2>/dev/null || echo "Not a Mach-O file"
                rm "$obj"
            fi
        done
    else
        lipo -info "$file"
    fi
    echo

    echo "Section sizes:"
    size -m "$file" 2>/dev/null || echo "size command failed"
    echo

    echo "Symbol count:"
    if [[ "$file" == *.a ]]; then
        nm "$file" 2>/dev/null | wc -l
    else
        nm "$file" 2>/dev/null | wc -l
    fi
    echo

    echo "Largest symbols (top 10):"
    if [[ "$file" == *.a ]]; then
        nm --size-sort "$file" 2>/dev/null | tail -n 10
    else
        nm --size-sort "$file" 2>/dev/null | tail -n 10
    fi
    echo "----------------------------------------"
}

# Check if any files were provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <binary_files...>"
    exit 1
fi

# Analyze each provided file
for file in "$@"; do
    if [ -f "$file" ]; then
        analyze_binary "$file"
    else
        echo "File not found: $file"
    fi
done