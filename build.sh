#!/bin/bash
set -e

echo "Building Rust core..."
cd rust-core
cargo build --release
cp target/release/libtradechest_core.dylib ../gui-csharp/

echo "Building C# GUI..."
cd ../gui-csharp
dotnet build --configuration Release

echo "Hybrid build complete."
echo "Run GUI: cd gui-csharp && dotnet run"

# echo "Running the Application"
# cd ../gui-csharp && dotnet run