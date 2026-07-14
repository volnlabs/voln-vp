#!/usr/bin/env bash
# probe-armv8a.sh — run Phase 1 risk gate.
# Exit 0 on UART marker seen; non-zero otherwise.
set -euo pipefail

CAPTURE="/tmp/uart-capture-phase1.log"
KERNEL="${AXIOMOS_KERNEL:-/home/utkarsh/Work/axiomOS/target/aarch64-unknown-none/release/kernel}"
TIMEOUT="${PROBE_TIMEOUT:-90s}"
# First canonical UART marker: the very first byte sent by boot.S is `{`,
# but that may be interleaved with control chars. Use any of these:
#   "Logging initialized" — kernel alive past early init
#   "ARM64 kernel started" — virt-mode marker
MARKER="${PROBE_MARKER:-Logging initialized}"

rm -f "$CAPTURE"
export UART_CAPTURE_PATH="$CAPTURE"

if [[ ! -f "$KERNEL" ]]; then
  echo "FAIL: axiomOS kernel not found at $KERNEL" >&2
  echo "  build with: cargo build --target aarch64-unknown-none --features virt --release -p kernel" >&2
  exit 2
fi

if ! command -v renode >/dev/null; then
  echo "FAIL: renode not on PATH (install via: yay -S renode-bin)" >&2
  exit 3
fi

# Renode's exit code discipline: simulator failure must propagate.
timeout --foreground "$TIMEOUT" \
  renode renode/probes/stock-armv8a.resc \
  || { echo "FAIL: renode exited non-zero"; exit 1; }

if grep -q "$MARKER" "$CAPTURE" 2>/dev/null; then
  echo "PASS: UART marker '$MARKER' observed"
  exit 0
else
  echo "FAIL: UART marker '$MARKER' not observed within $TIMEOUT"
  echo "--- last 80 lines of UART capture ---"
  tail -80 "$CAPTURE" >&2 || true
  exit 1
fi