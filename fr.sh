#!/usr/bin/env sh

set -e

cargo build --package=artbounty --lib --target=wasm32-unknown-unknown --features=csr,testing --profile wasm-debug
# cargo build --package=artbounty --lib --target=wasm32-unknown-unknown --features=csr,testing -Z build-std=panic_abort,std --profile wasm-debug
rm -rf ./target/site/*
mkdir -p ./target/site/pkg
cp -r ./assets/* ./target/site
cp index.html ./target/site/index.html

tailwindcss -i style/tailwind.css -o target/site/pkg/artbounty_1.css
wasm-bindgen ./target/wasm32-unknown-unknown/wasm-debug/artbounty.wasm --no-typescript --target no-modules --out-dir ./target/site/pkg --out-name artbounty_1

RUST_LOG="artbounty=trace" LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:./target/debug/deps/" ./target/debug/artbounty
