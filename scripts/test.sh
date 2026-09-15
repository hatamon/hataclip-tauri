#!/usr/bin/env bash
set -euo pipefail
cd /app
npm ci
npm test
cargo test --manifest-path src-tauri/Cargo.toml
