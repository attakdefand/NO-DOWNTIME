@echo off
REM Build script for No-Downtime Service Web UI

echo Building No-Downtime Service Web UI...

REM Check if wasm-pack is installed
where wasm-pack >nul 2>&1
if %errorlevel% neq 0 (
    echo wasm-pack could not be found. Please install it with:
    echo cargo install wasm-pack
    exit /b 1
)

REM Build the WebAssembly package
echo Building WebAssembly package...
wasm-pack build --target web --out-dir pkg

REM Check if build was successful
if %errorlevel% equ 0 (
    echo Build successful!
    echo To serve the application, use a static file server:
    echo   python -m http.server 8000
    echo   or
    echo   npm install -g serve && serve .
) else (
    echo Build failed!
    exit /b 1
)