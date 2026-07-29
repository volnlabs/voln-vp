voln-vp — implementation checklist

full plans live in .hermes/plans/ — this file is the at-a-glance tracker
follow the spec's phase order. stop gates between phases are real.

phase 0 — precondition
  [x] design spec read (docs/superpowers/specs/2026-07-14-voln-vp-design.md)
  [x] axiomOS sibling repo confirmed buildable

phase 1 — risk gate (probes only, no voln-vp structure)
  [x] 1.1 confirm axiomOS produces DTB-aware AArch64 ELF
  [x] 1.2 probes-only skeleton (renode/probes/, tools/, docs/probes/)
  [x] 1.3 minimal Pi 5 DTB stub
  [x] 1.4 stock ARMv8-A boot .resc
  [x] 1.5 UART capture (Renode file backend; legacy python shim retained)
  [x] 1.6 install renode, pin stock core model (Cortex-A78, Renode 1.16.1)
  [x] 1.7 probe driver shell script
  [x] 1.8 RUN PROBE — PASS recorded in docs/probes/2026-07-30-armv8a-risk-gate.md
        -> PASS: proceed
        -> FAIL with diagnostic: iterate DTB / kernel entry
        -> HARD FAIL: spec says revisit backend choice. STOP.

phase 2 — CLI + adapter contract + virt-pi5 boot
  [x] 2.1 workspace + cli crate skeleton (cargo build, --version)
  [x] 2.2 errors module
  [ ] 2.3 manifest types + validation (TDD, 4 cases)
  [ ] 2.4 backend discovery (TDD, 3 cases)
  [ ] 2.5 backend dispatch + run/test wiring (TDD, 4 cases)
  [ ] 2.6 qemu backend wrapped
  [ ] 2.7 renode backend manifest + doctor
  [ ] 2.8 virt-pi5 board manifest
  [ ] 2.9 --dry-run flag
  [ ] 2.10 mailbox stub python peripheral
  [ ] 2.11 virt-pi5 DTB (real, not phase 1 stub)
  [ ] 2.12 virt-pi5.repl + boot script
  [ ] 2.13 verify boot-to-userspace

phase 3 — RP1 models + driver suite
  [ ] 3.1 RP2040 reuse audit (BLOCKER for 3.2-3.6)
  [ ] 3.2 PCIe RC python peripheral (5-day budget)
  [ ] 3.3 RP1 GPIO (TDD)
  [ ] 3.4 RP1 PWM with capture (TDD)
  [ ] 3.5 RP1 I²C + imu fake (TDD)
  [ ] 3.6 RP1 SPI loopback (TDD)
  [ ] 3.7 wire peripherals into virt-pi5.repl
  [ ] 3.8 robot: GPIO toggle readback
  [ ] 3.9 robot: PWM sweep capture
  [ ] 3.10 robot: I²C IMU + SPI loopback
  [ ] 3.11 robot: determinism check
  [ ] 3.12 strict-mode enforcement
  [ ] 3.13 PCIe flat-map fallback (DECISION gate, only if 3.2 stalls)
  [ ] 3.14 full driver suite green

phase 4 — sensors, injection, trace replay
  [ ] 4.1 trace format v1 spec (one page)
  [ ] 4.2 trace parser (TDD, 7 cases)
  [ ] 4.3 trace writer (TDD, 2 cases)
  [ ] 4.4 IMU device with injection (TDD)
  [ ] 4.5 sensor hub dispatcher (TDD)
  [ ] 4.6 robot keyword: inject imu.accel_x 9.81 @ t=10ms
  [ ] 4.7 robot: IMU inject scenario
  [ ] 4.8 trace replay integration (TDD)
  [ ] 4.9 robot: IMU replay + actuator capture
  [ ] 4.10 bridge interface contract (NO implementation)
  [ ] 4.11 verify full suite green

phase 5 — CI hardening, nightly, error handling
  [ ] 5.1 CI log helper
  [ ] 5.2 per-commit driver
  [ ] 5.3 nightly driver
  [ ] 5.4 verify.sh local wrapper
  [ ] 5.5 error-handling audit
  [ ] 5.6 UART dump on simulator failure (TDD)
  [ ] 5.7 GH Actions per-commit workflow
  [ ] 5.8 GH Actions nightly workflow
  [ ] 5.9 CI docs
  [ ] 5.10 verify pipeline green

notes
- commit after every task
- spec decisions log is the source of truth for "is X in scope?"
- never fudge exit codes; never report PASS without real artifact
- amber (FAIL with diagnostic) is normal and publishable; HARD FAIL is the only blocker
