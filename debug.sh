#!/usr/bin/env sh

# cargo build --package=artbounty-backend &&\
# cargo build --package=artbounty-frontend --target=wasm32-unknown-unknown --features=hydrate &&\
# rm -rf ./target/site/* &&\
# mkdir -p ./target/site/pkg &&\
# cp -r ./assets/* ./target/site &&\
# tailwindcss -i style/tailwind.css -o target/site/pkg/artbounty_1.css &&\
# wasm-bindgen ./target/wasm32-unknown-unknown/debug/artbounty_frontend.wasm --no-typescript --target web --out-dir ./target/site/pkg --out-name artbounty_1 &&\
# RUST_LOG="artbounty=trace" ./target/debug/artbounty-backend
cargo build --package=artbounty --features=ssr &&\
cargo build --package=artbounty --lib --target=wasm32-unknown-unknown --features=hydrate --profile wasm-debug &&\
rm -rf ./target/site/* &&\
mkdir -p ./target/site/pkg &&\
cp -r ./assets/* ./target/site &&\
# cp leptos.toml ./target/debug/leptos.toml
# cp artbounty.toml ./target/debug/artbounty.toml
tailwindcss -i style/tailwind.css -o target/site/pkg/artbounty_1.css &&\
wasm-bindgen ./target/wasm32-unknown-unknown/wasm-debug/artbounty.wasm --no-typescript --target web --out-dir ./target/site/pkg --out-name artbounty_1 &&\
# cd ./target/debug
# RUST_LOG="artbounty=trace" cargo run --package=artbounty --features=ssr
RUST_LOG="artbounty=trace" LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:./target/debug/deps/" ./target/debug/artbounty
# RUST_LOG="artbounty=trace" ./artbounty 

