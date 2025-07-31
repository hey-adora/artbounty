#!/usr/bin/env sh

cargo build --package=artbounty-backend &&\
cargo build --package=artbounty-frontend --target=wasm32-unknown-unknown --features=hydrate &&\
rm -rf ./target/site/* &&\
mkdir -p ./target/site/pkg &&\
cp -r ./assets/* ./target/site &&\
tailwindcss -i style/tailwind.css -o target/site/pkg/artbounty_1.css &&\
wasm-bindgen ./target/wasm32-unknown-unknown/debug/artbounty_frontend.wasm --no-typescript --target web --out-dir ./target/site/pkg --out-name artbounty_1 &&\
RUST_LOG="artbounty=trace" ./target/debug/artbounty-backend
