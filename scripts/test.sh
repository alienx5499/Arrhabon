#!/usr/bin/env bash
# Offline tests: rebuild the program first so the LiteSVM tests never run a stale .so.
set -euo pipefail
cd "$(dirname "$0")/.."
anchor build
cargo test --workspace "$@"
