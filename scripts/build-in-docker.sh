#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="/work"
APP_DIR="${TAURI_APP_DIR:-/work/app}"

# If the caller didn't mount/create an app folder, fall back to repo root.
if [ ! -f "${APP_DIR}/package.json" ]; then
  APP_DIR="${REPO_ROOT}"
fi

cd "${APP_DIR}"

build_with_manager() {
  if [ -f pnpm-lock.yaml ]; then
    # Node 20 includes corepack; enable it if needed.
    corepack enable >/dev/null 2>&1 || true
    pnpm install --frozen-lockfile
  elif [ -f yarn.lock ]; then
    yarn install --frozen-lockfile
  elif [ -f package-lock.json ]; then
    npm ci
  else
    npm install
  fi
}

build_with_manager

# Use `--no-bundle` so we can compile Windows binaries from Linux without Wix installers.
TOOL_ARGS=()
if [ "${TAURI_NO_BUNDLE:-1}" = "1" ]; then
  TOOL_ARGS+=(--no-bundle)
fi

TARGET="${TAURI_TARGET:-x86_64-unknown-linux-gnu}"
echo "tauri build target: ${TARGET}"

# Assumes the project has `tauri` commands wired in `package.json` (we'll add that in the next step).
npm run tauri build -- --target "${TARGET}" "${TOOL_ARGS[@]}"

