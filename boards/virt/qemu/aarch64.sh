#!/usr/bin/env bash
set -euo pipefail

AXIOMOS_ROOT="${AXIOMOS_ROOT:-/home/utkarsh/axiomos}"
KERNEL="${AXIOMOS_KERNEL:-$AXIOMOS_ROOT/target/aarch64-unknown-none/debug/kernel}"
TIMEOUT="${VOLN_VP_QEMU_TIMEOUT:-600s}"

if [[ ! -f "$KERNEL" ]]; then
  echo "AArch64 kernel not found: $KERNEL" >&2
  echo "Build it with: cargo build --target aarch64-unknown-none --features virt,cloud-profile -p kernel" >&2
  exit 2
fi

exec timeout --foreground "$TIMEOUT" qemu-system-aarch64 \
  -machine virt \
  -m 1G \
  -cpu cortex-a57 \
  -nographic \
  -kernel "$KERNEL" \
  "$@"
