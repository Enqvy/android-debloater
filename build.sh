#!/bin/bash
cargo build --release
mkdir -p release
cp target/release/android-debloater release/
echo "Built: release/android-debloater"