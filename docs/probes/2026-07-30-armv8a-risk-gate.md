# Phase 1 Risk Gate — Renode ARMv8-A Boot Results

**Date:** 2026-07-30

**Renode version:** 1.16.1.16973 (`d66b0c2a-202602160923`)

**ARMv8-A core:** stock `platforms/cpus/cortex-a78.repl`

**Outcome:** PASS for the Phase 1 CPU-viability gate

## Acceptance criterion

The approved design defines Phase 1 as proving that axiomOS can reach UART on a
stock Renode ARMv8-A platform before any custom Pi 5 modeling begins.

The probe reached the canonical `Logging initialized` marker at approximately
1.0 ms of Renode virtual time. It then continued through AArch64 exception,
physical-memory, page-table, address-space, and heap initialization. This is
sufficient to show that Renode's stock ARMv8-A CPU can execute the current
axiomOS entry path. Phase 2 is therefore no longer blocked on CPU viability.

This result does **not** claim boot-to-userspace on the stock platform. That is
the Phase 2 `virt-pi5` machine milestone.

## Inputs

- axiomOS ELF:
  `/home/utkarsh/axiomos/target/aarch64-unknown-none/debug/kernel`
- ELF type: ELF64, AArch64, statically linked
- ELF entry point: `0x40080000`
- ELF SHA-256:
  `af2ae5eb99acfd5310fe7108ea4108d1c693a8439595e8b08e946c47b67abc02`
- axiomOS commit: `d6561788bdc41f3cd186633274d5897603da7764`
- axiomOS worktree caveat: `kernel/linker-virt.ld` was modified during the
  run, so the ELF is identified by its hash rather than claimed to be a clean
  commit build.
- DTB: `renode/probes/minimal-virt-stub.dtb`
- DTB SHA-256:
  `131f673fe663a86db522c65e835f4c1b9eca6110f57b4dae9282df84a0dec478`

The AArch64 entry contract was also confirmed from the axiomOS source:
`boot.S` saves the firmware-provided DTB address from `x0`, restores it before
calling Rust `_start`, and `_start` stores and parses that address.

## Reproduction

The cleaned probe driver uses Renode's built-in UART file backend and a bounded
virtual-time run:

```sh
tools/probe-armv8a.sh
```

The driver creates a unique artifact directory under `/tmp`, passes the
selected kernel through a generated safe symlink and positional wrapper, uses
`CreateFileBackend` with immediate flush, and runs exactly 0.1 virtual seconds.
An independent 90-second wall timeout catches simulator startup failures and
hangs. Success requires both a zero process status and the expected marker in
the UART artifact because Renode 1.16 can return zero for some Monitor errors.

## UART evidence

The durable, relevant portion of the UART capture was:

```text
=== axiom-ebpf on QEMU virt ===
Platform initialized
INFO  boot [kernel] Logging initialized
INFO  boot [kernel] Initializing architecture...
INFO  boot [kernel::arch::aarch64::exceptions] Initializing AArch64 exception vector table at 0x40136000
INFO  boot [kernel::mem] Starting memory initialization...
INFO  boot [kernel::arch::aarch64::mm] Initializing ARM64 memory management...
INFO  boot [kernel::mem::phys] usable RAM: ~1024 MiB
INFO  boot [kernel::arch::aarch64::phys] Physical memory stage 1 initialized: 1024 MB available
INFO  boot [kernel::arch::aarch64::phys] Reserved kernel region: 0x40080000 - 0x40230000
INFO  boot [kernel::arch::aarch64::mm] Bootstrap page tables configured, mapped 4GB
INFO  boot [kernel::arch::aarch64::mm] ARM64 memory management initialized
INFO  boot [kernel::mem] Address space initialized
INFO  boot [kernel::mem::heap] initializing heap at 0xffffc00000000000
```

The raw temporary UART capture was 1,601 bytes over 20 lines with SHA-256
`0eb1136b43de070a9477eb9d11e16021ed359993c905871c55cd6b7dc42699d5`.

## Observations and follow-ups

- The first three early debug writes target the Pi 5 UART address
  `0x10_7d00_1000`, which is intentionally absent from the stock A78 platform.
  The virt UART at `0x0900_0000` then works and produces the acceptance marker.
- Renode logged one write while the PL011 `UARTEN` bit was disabled, followed
  by normal UART output.
- After heap initialization, the kernel performs many accesses through
  high-half virtual addresses that the stock platform reports as unmapped.
  This is not an ARMv8-A viability failure. It is evidence for Phase 2's real
  board/DTB/memory-map work and should be retained as a regression target.
- The DTB round-trip is valid but warns that the timer and UART nodes omit
  `interrupt-parent`. Add it when creating the Phase 2 board DTB; it is not
  required to reopen this gate.
- `tools/probe-armv8a.sh` now implements the reproducible gate and preserves
  per-run UART and Renode logs for both passing and failing runs.

## Decision

Proceed to Phase 2. The Phase 1 harness cleanup and tracker reconciliation are
complete, and the backend-choice stop condition was not triggered.
