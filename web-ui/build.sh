#!/bin/bash

# Build script for No-Downtime Service Web UI

echo "Building No-Downtime Service Web UI..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null
then
    echo "wasm-pack could not be found. Please install it with:"
    echo "cargo install wasm-pack"
    exit 1
fi

# Build the WebAssembly package
echo "Building WebAssembly package..."
wasm-pack build --target web --out-dir pkg

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "To serve the application, use a static file server:"
    echo "  python -m http.server 8000"
    echo "  or"
    echo "  npm install -g serve && serve ."
else
    echo "Build failed!"
    exit 1
fi