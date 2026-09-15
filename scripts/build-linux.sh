#!/usr/bin/env bash
set -euo pipefail
cd /app
npm ci
npm run tauri -- build --no-bundle
mkdir -p dist/linux
cp src-tauri/target/release/hataclip dist/linux/hataclip
chmod +x dist/linux/hataclip
echo "wrote dist/linux/hataclip"
