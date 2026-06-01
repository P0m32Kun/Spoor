#!/usr/bin/env bash
# Compare Spoor vs jsluice on regression fixtures.
# Requires: jsluice (go install github.com/BishopFox/jsluice/cmd/jsluice@latest)
# Primary gate: cargo test jsluice_parity (Rust module, skips if jsluice missing)

set -euo pipefail
cd "$(dirname "$0")/.."

if ! command -v jsluice >/dev/null 2>&1 && [[ ! -x "${HOME}/go/bin/jsluice" ]]; then
  echo "jsluice not found; install with:"
  echo "  go install github.com/BishopFox/jsluice/cmd/jsluice@latest"
  exit 1
fi

export PATH="${HOME}/go/bin:${PATH}"
cargo test -p spoor-core jsluice_parity -- --nocapture
