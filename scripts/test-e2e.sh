#!/usr/bin/env bash
set -euo pipefail
cd /app
npm ci
npm run test:e2e
