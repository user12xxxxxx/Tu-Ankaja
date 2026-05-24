#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "== Firmware simulator =="
make -C "$ROOT_DIR/firmware" run

echo
echo "== Entropy engine tests =="
cargo test --manifest-path "$ROOT_DIR/entropy-engine/Cargo.toml"

echo
echo "== Frontend config =="
npm --prefix "$ROOT_DIR/frontend" run lint
npm --prefix "$ROOT_DIR/frontend" run build

echo
echo "== Tauri desktop check =="
cargo check --manifest-path "$ROOT_DIR/tauri/src-tauri/Cargo.toml"

echo
echo "All local checks passed."
