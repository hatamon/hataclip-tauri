#!/usr/bin/env bash
set -euo pipefail
cd /app
bash scripts/install-deps.sh
npm run tauri -- build --target x86_64-pc-windows-gnu --no-bundle
cargo build --release --bin hataclip --target x86_64-pc-windows-gnu --manifest-path src-tauri/Cargo.toml
mkdir -p dist/windows
release=src-tauri/target/x86_64-pc-windows-gnu/release
if ! cp "$release/hataclip-gui.exe" dist/windows/hataclip-gui.exe; then
  echo "failed to write dist/windows/hataclip-gui.exe; close hataclip-gui.exe if it is running" >&2
  exit 1
fi
rm -f dist/windows/hataclip-cli.exe
if ! cp "$release/hataclip.exe" dist/windows/hataclip.exe; then
  echo "failed to write dist/windows/hataclip.exe; close hataclip.exe if it is running" >&2
  exit 1
fi
cp src-tauri/target/x86_64-pc-windows-gnu/release/WebView2Loader.dll dist/windows/WebView2Loader.dll

copy_dll() {
  local name="$1"
  local found
  found="$(find /usr -name "$name" 2>/dev/null | head -n 1 || true)"
  if [[ -n "$found" ]]; then
    cp "$found" "dist/windows/$name"
  fi
}

copy_dll libgcc_s_seh-1.dll
copy_dll libstdc++-6.dll
copy_dll libwinpthread-1.dll

echo "wrote dist/windows/hataclip-gui.exe and dist/windows/hataclip.exe"
