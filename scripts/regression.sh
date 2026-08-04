#!/usr/bin/env bash
# Regression gate (pre-push, minutes) = THE gate. One implementation, no drift.
set -u
cd "$(dirname "$0")/.."
if ! command -v cargo >/dev/null 2>&1; then echo "regression: SKIP (no cargo on this machine)"; exit 0; fi
exec ./verify_meshwork.sh "$@"
