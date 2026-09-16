#!/usr/bin/env bash
set -euo pipefail
cd /app
bash scripts/install-deps.sh
npm test
cargo test --manifest-path src-tauri/Cargo.toml
