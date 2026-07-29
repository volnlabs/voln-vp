#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "$SCRIPT_DIR/../../.." && pwd)"
BOOT_SCRIPT="$REPO_ROOT/backends/renode/scripts/boot-virt-pi5.resc"
KERNEL="${AXIOMOS_KERNEL:-/home/utkarsh/Work/axiomOS/target/aarch64-unknown-none/release/kernel}"
DTB="${VOLN_VP_DTB:-$REPO_ROOT/boards/virt-pi5/virt-pi5.dtb}"
TIMEOUT="${VOLN_VP_TIMEOUT:-90s}"
VIRTUAL_TIME="${VOLN_VP_VIRTUAL_TIME:-0.1}"
MARKER="${VOLN_VP_BOOT_MARKER:-=== axiomos eBPF init ===}"

if [[ ! -f "$BOOT_SCRIPT" ]]; then
  echo "Renode boot script not found: $BOOT_SCRIPT" >&2
  exit 2
fi

if [[ ! -f "$KERNEL" ]]; then
  echo "axiomOS embedded-rpi5 kernel not found: $KERNEL" >&2
  exit 2
fi

if [[ ! -f "$DTB" ]]; then
  echo "virt-pi5 DTB not found: $DTB" >&2
  exit 2
fi

if ! command -v renode >/dev/null; then
  echo "renode is not on PATH" >&2
  exit 3
fi

if [[ ! "$VIRTUAL_TIME" =~ ^[0-9]+([.][0-9]+)?$ ]] || [[ ! "$VIRTUAL_TIME" =~ [1-9] ]]; then
  echo "VOLN_VP_VIRTUAL_TIME must be a positive number of virtual seconds" >&2
  exit 4
fi

RUN_DIR="${VOLN_VP_ARTIFACT_DIR:-$(mktemp -d /tmp/voln-vp-phase2.XXXXXX)}"
mkdir -p -- "$RUN_DIR"
CAPTURE="$RUN_DIR/uart.log"
RENODE_LOG="$RUN_DIR/renode.log"
WRAPPER="$RUN_DIR/boot.resc"

KERNEL="$(realpath -e -- "$KERNEL")"
DTB="$(realpath -e -- "$DTB")"
ln -sf -- "$KERNEL" "$RUN_DIR/kernel.elf"
ln -sf -- "$DTB" "$RUN_DIR/virt-pi5.dtb"
printf '%s\n' \
  "\$axiomos_kernel=@$RUN_DIR/kernel.elf" \
  "\$virt_pi5_dtb=@$RUN_DIR/virt-pi5.dtb" \
  'include @backends/renode/scripts/boot-virt-pi5.resc' \
  >"$WRAPPER"

# Renode 1.16 may return zero after a Monitor error. Process status proves that
# the simulator ran; the userspace UART marker is the semantic success gate.
set +e
(
  cd -- "$REPO_ROOT"
  timeout --foreground "$TIMEOUT" \
    renode --config "$RUN_DIR/renode.config" --disable-gui --plain -P 0 \
      "$@" "$WRAPPER" \
      -e "sysbus.uart0 CreateFileBackend @$CAPTURE true; emulation RunFor \"$VIRTUAL_TIME\"; quit"
) >"$RENODE_LOG" 2>&1
RENODE_STATUS=$?
set -e

if (( RENODE_STATUS != 0 )); then
  echo "FAIL: Renode exited with status $RENODE_STATUS" >&2
  echo "Artifacts: $RUN_DIR" >&2
  tail -80 "$RENODE_LOG" >&2 || true
  exit "$RENODE_STATUS"
fi

if [[ -f "$CAPTURE" ]] && grep -Fq -- "$MARKER" "$CAPTURE"; then
  echo "PASS: userspace UART marker '$MARKER' observed"
  echo "Artifacts: $RUN_DIR"
  exit 0
fi

echo "FAIL: userspace UART marker '$MARKER' not observed" >&2
echo "Artifacts: $RUN_DIR" >&2
echo "--- UART tail ---" >&2
tail -80 "$CAPTURE" >&2 || true
echo "--- Renode log tail ---" >&2
tail -80 "$RENODE_LOG" >&2 || true
exit 1
