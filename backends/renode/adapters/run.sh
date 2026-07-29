#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../../.." && pwd)"
BOOT_SCRIPT="$REPO_ROOT/backends/renode/scripts/boot-virt-pi5.resc"

if [[ ! -f "$BOOT_SCRIPT" ]]; then
  echo "Renode boot script not found: $BOOT_SCRIPT" >&2
  exit 2
fi

exec renode --disable-gui "$BOOT_SCRIPT" "$@"
