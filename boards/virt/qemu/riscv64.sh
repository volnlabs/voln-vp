#!/usr/bin/env bash
set -euo pipefail

AXIOMOS_ROOT="${AXIOMOS_ROOT:-/home/utkarsh/axiomos}"
KERNEL="${AXIOMOS_KERNEL:-$AXIOMOS_ROOT/kernel/demos/riscv/target/riscv64gc-unknown-none-elf/debug/riscv-kernel-demo}"

if [[ ! -f "$KERNEL" ]]; then
  echo "RISC-V kernel not found: $KERNEL" >&2
  echo "Build it with: $AXIOMOS_ROOT/scripts/build-riscv.sh" >&2
  exit 2
fi

exec qemu-system-riscv64 \
  -machine virt \
  -bios default \
  -kernel "$KERNEL" \
  -nographic \
  -serial mon:stdio \
  "$@"
