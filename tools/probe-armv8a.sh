#!/usr/bin/env bash
# probe-armv8a.sh — run Phase 1 risk gate.
# Exit 0 on UART marker seen; non-zero otherwise.
set -euo pipefail

KERNEL="${AXIOMOS_KERNEL:-/home/utkarsh/axiomos/target/aarch64-unknown-none/debug/kernel}"
TIMEOUT="${PROBE_TIMEOUT:-90s}"
VIRTUAL_TIME="${PROBE_VIRTUAL_TIME:-0.1}"
# First canonical UART marker: the very first byte sent by boot.S is `{`,
# but that may be interleaved with control chars. Use any of these:
#   "Logging initialized" — kernel alive past early init
#   "ARM64 kernel started" — virt-mode marker
MARKER="${PROBE_MARKER:-Logging initialized}"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/.." && pwd)"
RUN_DIR="$(mktemp -d /tmp/voln-vp-phase1.XXXXXX)"
CAPTURE="$RUN_DIR/uart.log"
RENODE_LOG="$RUN_DIR/renode.log"

if [[ ! -f "$KERNEL" ]]; then
  echo "FAIL: axiomOS kernel not found at $KERNEL" >&2
  echo "  build with: cargo build --target aarch64-unknown-none --features virt --release -p kernel" >&2
  exit 2
fi

if ! command -v renode >/dev/null; then
  echo "FAIL: renode not on PATH (install via: yay -S renode-bin)" >&2
  exit 3
fi

if [[ ! "$VIRTUAL_TIME" =~ ^[0-9]+([.][0-9]+)?$ ]] || [[ ! "$VIRTUAL_TIME" =~ [1-9] ]]; then
  echo "FAIL: PROBE_VIRTUAL_TIME must be a positive number of virtual seconds" >&2
  exit 4
fi

KERNEL="$(realpath -e -- "$KERNEL")"
ln -s -- "$KERNEL" "$RUN_DIR/kernel.elf"
printf '%s\n' \
  "\$axiomos_kernel=@$RUN_DIR/kernel.elf" \
  'include @renode/probes/stock-armv8a.resc' \
  >"$RUN_DIR/probe.resc"

# Renode 1.16 can return zero for Monitor errors, so process status catches
# crashes/timeouts while the capture artifact and marker establish semantic
# success. A generated positional wrapper sets the safe kernel path before the
# main script is included; Renode runs -e commands only after positional scripts.
set +e
(
  cd -- "$REPO_ROOT"
  timeout --foreground "$TIMEOUT" \
    renode --config "$RUN_DIR/renode.config" --disable-gui --plain -P 0 \
      "$RUN_DIR/probe.resc" \
      -e "sysbus.uart0 CreateFileBackend @$CAPTURE true; emulation RunFor \"$VIRTUAL_TIME\"; quit"
) >"$RENODE_LOG" 2>&1
RENODE_STATUS=$?
set -e

if (( RENODE_STATUS != 0 )); then
  echo "FAIL: renode exited with status $RENODE_STATUS" >&2
  echo "Artifacts: $RUN_DIR" >&2
  tail -80 "$RENODE_LOG" >&2 || true
  exit 1
fi

if [[ -f "$CAPTURE" ]] && grep -Fq -- "$MARKER" "$CAPTURE"; then
  echo "PASS: UART marker '$MARKER' observed"
  echo "Artifacts: $RUN_DIR"
  exit 0
else
  echo "FAIL: UART marker '$MARKER' not observed within $VIRTUAL_TIME virtual seconds"
  echo "Artifacts: $RUN_DIR"
  echo "--- last 80 lines of UART capture ---"
  tail -80 "$CAPTURE" >&2 || true
  echo "--- last 80 lines of Renode log ---"
  tail -80 "$RENODE_LOG" >&2 || true
  exit 1
fi
