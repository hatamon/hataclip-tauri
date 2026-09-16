#!/usr/bin/env bash
# npm ci は毎回 node_modules を作り直すので、lockfile が変わったときだけ走らせる。
set -euo pipefail
cd /app

stamp=node_modules/.hataclip-lock
if [[ -f "$stamp" ]] && cmp -s package-lock.json "$stamp"; then
  exit 0
fi

npm ci
cp package-lock.json "$stamp"
