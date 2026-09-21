#!/usr/bin/env bash
set -euo pipefail
cd /app
bash scripts/install-deps.sh
npm run tauri -- build --no-bundle
mkdir -p dist/linux
if ! cp src-tauri/target/release/hataclip dist/linux/hataclip; then
  echo "failed to write dist/linux/hataclip; close hataclip if it is running" >&2
  exit 1
fi
echo "wrote dist/linux/hataclip"
