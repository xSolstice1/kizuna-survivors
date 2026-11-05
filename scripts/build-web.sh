#!/usr/bin/env bash
# Builds the browser version into dist/ (static files, ready for `vercel deploy`).
set -euo pipefail
cd "$(dirname "$0")/.."
# Build output stays on the same drive as the source: the Windows drive clock can drift from
# WSL's, and cargo's freshness check compares timestamps across the two.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-target}"
# --allow-undefined: quad-snd (macroquad audio) declares its JS imports without a module name.
RUSTFLAGS="-C target-feature=+simd128 -C link-arg=--allow-undefined" cargo build -p client --release --target wasm32-unknown-unknown

rm -rf dist && mkdir -p dist
cp web/index.html dist/
cp "$(ls -d ~/.cargo/registry/src/*/macroquad-0.4.*/js | tail -1)/mq_js_bundle.js" dist/
WASM="$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/client.wasm"
if command -v wasm-opt >/dev/null; then
  wasm-opt -Oz --enable-simd --enable-bulk-memory "$WASM" -o dist/game.wasm
else
  cp "$WASM" dist/game.wasm
fi
cp web/vercel.json dist/
# Keep the Vercel project link inside dist/ so `vercel deploy` from dist targets the right project.
if [ -d .vercel ]; then cp -r .vercel dist/; fi
ls -la dist
