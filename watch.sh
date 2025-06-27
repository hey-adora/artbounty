#!/bin/sh

RUST_LOG="artbounty=trace" LEPTOS_TAILWIND_VERSION=v4.0.7 cargo-leptos watch --hot-reload -v
