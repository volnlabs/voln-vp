#!/usr/bin/env bash
set -euo pipefail

AXIOMOS_ROOT="${AXIOMOS_ROOT:-/home/utkarsh/axiomos}"

if [[ ! -f "$AXIOMOS_ROOT/Cargo.toml" ]]; then
  echo "axiomOS repository not found: $AXIOMOS_ROOT" >&2
  exit 2
fi

exec cargo run --manifest-path "$AXIOMOS_ROOT/Cargo.toml" -- --headless "$@"
