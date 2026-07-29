#!/usr/bin/env bash
set -euo pipefail

AXIOMOS_ROOT="${AXIOMOS_ROOT:-/home/utkarsh/Work/axiomOS}"

if [[ ! -f "$AXIOMOS_ROOT/Cargo.toml" ]]; then
  echo "axiomOS repository not found: $AXIOMOS_ROOT" >&2
  exit 2
fi

cd -- "$AXIOMOS_ROOT"
exec cargo xtask run x86_64 -- "$@"
