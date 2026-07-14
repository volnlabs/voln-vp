# voln-sim Design: Pre-Hardware Simulation Stack for axiomOS

**Date:** 2026-07-14
**Status:** Approved design, pre-implementation
**Context:** axiomOS boots on QEMU virt (x86_64, AArch64, RISC-V) and on real
Raspberry Pi 5 with full userspace, eBPF, GPIO/PWM/IIO drivers, and a measured
211ns single-core interrupt latency. No simulation infrastructure beyond QEMU
virt exists. Jetson is a future target; nothing Jetson-specific is built now.

## Goal

A complete simulated machine that answers, before touching hardware:

1. **Will it work?** — peripheral drivers (RP1 GPIO/PWM/I²C/SPI/UART/ADC),
   post-firmware boot path, robotics app logic against sensors.
2. **How fast is it?** — latency numbers calibrated against the real Pi 5,
   plus deterministic reproduction of timing-dependent bugs.

## Non-Goals / Stated Boundaries

- **Firmware stage is not simulated.** No open tool models the Pi 5 bootrom →
  EEPROM → VideoCore chain. Simulation entry point is kernel entry with a DTB.
  Firmware-stage bugs are hardware-only.
- **GPU is not simulated** (VideoCore VII, and Jetson GPU later).
- **Jetson:** deferred entirely. The tier structure leaves a slot for a future
  `virt-jetson` platform; no code is written for it now.
- **Physics simulation:** out of scope. Sensor fidelity is value injection and
  trace replay, not closed-loop physics (no motor/IMU feedback models).

## Architecture: Three Tiers

One kernel image, three execution environments, each answering one question:

```
                    ┌──────────────────────────────┐
                    │  axiomOS kernel image (ELF)  │
                    └──────┬───────┬───────┬───────┘
                           │       │       │
        ┌──────────────────▼┐  ┌───▼────────────┐  ┌──▼──────────────────┐
        │ TIER 1: QEMU virt │  │ TIER 2: Renode │  │ TIER 3: gem5        │
        │ (exists today)    │  │ "virt-pi5"     │  │ A76 O3 model        │
        │                   │  │                │  │                     │
        │ Q: does it boot?  │  │ Q: will it     │  │ Q: how fast?        │
        │ multi-arch smoke  │  │ WORK on Pi5?   │  │ calibrated latency  │
        │ seconds/run       │  │ drivers+sensors│  │ minutes–hours/run   │
        │                   │  │ deterministic  │  │                     │
        └───────────────────┘  └────────────────┘  └─────────────────────┘
                  all feed CI · real Pi 5 hardware remains final truth
```

- **Tier 1 — QEMU virt (unchanged).** Existing multi-arch boot smoke tests
  (x86_64, AArch64, RISC-V). Wrapped by the runner, otherwise untouched.
- **Tier 2 — Renode `virt-pi5` (primary build).** Functionally complete Pi 5
  machine. Daily driver for driver/app development. Deterministic virtual
  time: race conditions and IRQ-ordering bugs reproduce identically every run.
- **Tier 3 — gem5 (latency).** Cortex-A76-approximating out-of-order CPU +
  BCM2712-like cache/DRAM hierarchy, calibrated against measured Pi 5 numbers.
  Answers performance questions; run on demand and nightly, not per-commit.

**Timing fidelity contract:** Tier 2 gives *determinism and instruction-count
timing proxies* (catches "IRQ path grew 3×" regressions), never nanosecond
accuracy. Tier 3 gives *calibrated cycle-level estimates* (target ±15% of
hardware). Only hardware gives true numbers.

## Tier 2: Renode `virt-pi5` Machine

### Platform (`renode/platforms/virt-pi5.repl`)

| Block | Model | Source |
|---|---|---|
| CPU | 4× Cortex-A76 (closest available Renode ARMv8-A core — A75/A78 acceptable; exact core chosen at Phase 1) | built-in |
| Interrupt controller | GIC-400 (GICv2, as on Pi 5) | built-in |
| Timer | ARM generic timer @ 54 MHz | built-in |
| Debug UART | PL011 at Pi 5 address | built-in |
| RAM | Pi 5 memory map, 8 GB configuration | built-in |
| Mailbox / firmware properties | Stub returning canned firmware responses the kernel expects | custom, small |
| PCIe root complex | BCM2712-style RC | **custom** |
| RP1 peripheral block | GPIO banks, PWM channels, I²C, SPI, RP1 UARTs, ADC | **custom — main build effort** |

### RP1 attachment: full PCIe model (decided)

RP1 is modeled as a PCIe endpoint behind a custom BCM2712-style root complex,
with registers exposed through BARs — the kernel's real PCIe enumeration and
probe path runs in simulation.

**Risk:** Renode's PCIe infrastructure is thin; the RC model is likely built
from scratch. **Fallback (logged here deliberately):** if PCIe modeling stalls
past its phase budget, flat-map RP1 registers on the system bus at
DTB-described addresses. Driver register logic stays fully tested; only PCIe
enumeration falls back to hardware-only coverage.

### RP1 peripheral models

- Written as Renode **Python peripherals**; C# only where Python hits limits.
- **Register-accurate for registers the axiomOS drivers touch**;
  datasheet-complete where the driver needs it.
- **Unmodeled-register tripwire:** any access to an unimplemented register
  logs a warning; **strict mode** (used in CI) makes it fatal. Gaps surface
  loudly, never silently.

### Sensor simulation

- **Fake bus devices** (`renode/devices/`): IMU, ADC chips, etc., as Python
  peripherals on the modeled I²C/SPI buses. Drivers perform real bus
  transactions; devices answer with injected values.
- **Value injection:** Renode monitor commands + Robot Framework keywords,
  e.g. `inject imu.accel_x 9.81 @ t=10ms`.
- **Trace replay:** timestamped CSV recorded from real Pi 5 runs, delivered
  at virtual-time timestamps. Deterministic: same trace → identical execution
  every run. Trace files carry a format version field.
- **Actuator capture:** PWM model logs duty/period changes with virtual
  timestamps, enabling assertions like "PWM reached 50% within 5 ms (virtual)
  of IMU spike".

## Tier 3: gem5 Latency Model

### Configuration (`gem5/configs/virt-pi5-a76.py`)

- ARM **full-system mode**, boots the same axiomOS image at kernel entry.
- **O3 CPU** parameterized toward Cortex-A76: ~4-wide decode, ~128-entry
  ROB-class instruction window, A76-like branch predictor.
- **Cache hierarchy matching BCM2712:** 64 KB L1I + 64 KB L1D per core,
  512 KB private L2, 2 MB shared L3.
- **LPDDR4X-class DRAM timing model.**
- **Generic devices only** (GIC, timer, UART). No RP1 in gem5 — this tier
  measures CPU/memory/IRQ-path latency; peripheral logic is Tier 2's job.

### Calibration workflow

1. Microbenchmark suite in axiomOS (kernel-side code lives in the axiomos
   repo; definitions in `gem5/benchmarks/`): IRQ latency, syscall round-trip,
   context switch, memcpy bandwidth. Identical on hardware and in gem5.
2. Measure on real Pi 5 → golden numbers (211 ns IRQ latency is the first
   anchor).
3. Run in gem5; tune CPU/cache/DRAM parameters until every benchmark is
   within the tolerance band (**target ±15%**).
4. Commit `gem5/calibration/manifest.toml`: parameters, golden numbers,
   achieved deltas, date. Recalibrate when measurement methodology or
   hardware revision changes.

### Usage

On-demand (`voln-sim bench --tier gem5 <benchmark>`) plus a nightly CI job.
Output: latency report diffed against the previous run; regression beyond
threshold fails the nightly build.

## Repository Layout

New `voln-sim` repository (this repo):

```
voln-sim/
  runner/                  # single Rust CLI crate:
                           #   voln-sim run|test|bench --tier <qemu|renode|gem5>
  renode/
    platforms/virt-pi5.repl
    peripherals/           # pcie_rc.py, rp1_gpio.py, rp1_pwm.py, rp1_i2c.py,
                           #   rp1_spi.py, rp1_uart.py, rp1_adc.py, mailbox.py
    devices/               # fake sensors: imu.py, adc_chip.py
    scripts/               # .resc boot/scenario scripts
    tests/                 # Robot Framework suites
    traces/                # recorded Pi 5 sensor traces (versioned format)
  gem5/
    configs/virt-pi5-a76.py
    calibration/manifest.toml
    benchmarks/
  qemu/                    # existing virt launch configs, wrapped
  ci/
  docs/
```

One runner crate only. The multi-crate layout from the research report
(`simulator/`, `devices/`, `platform/` crates) is rejected: it would
reimplement what Renode and gem5 already are.

## Interfaces

- **Runner CLI** is the single entry point for humans and CI:
  `voln-sim run|test|bench --tier <qemu|renode|gem5> [args]`. It launches the
  right tool with the right config and **propagates simulator exit codes**
  so CI cannot false-green.
- **Kernel ↔ sim:** the unmodified axiomOS image. No sim-specific kernel
  code paths; the machine is made to fit the kernel, not vice versa.
- **Tests ↔ Tier 2:** Robot Framework keywords (UART expectations, sensor
  injection, PWM capture assertions).
- **Benchmarks ↔ Tier 3:** benchmark definitions in `gem5/benchmarks/`,
  kernel-side implementations in the axiomos repo, results in the
  calibration manifest and nightly reports.

## Testing & CI

**Per-commit (fast, deterministic, minutes total):**
- Tier 1: QEMU boot smoke on x86_64, AArch64, RISC-V.
- Tier 2: `virt-pi5` boot-to-userspace; driver suite — GPIO toggle readback,
  PWM sweep capture, I²C IMU read, SPI transfer; one determinism check
  (same scenario twice → byte-identical UART output). Strict mode on.

**Nightly:**
- Full Robot scenario suites and trace replays.
- gem5 benchmark suite with regression thresholds.

**Hardware:** manual for now; HIL automation is future work.

## Error Handling

- Unmodeled RP1 register access: warn in dev, fatal in CI (strict mode).
- Runner: propagates non-zero simulator exits; timeout per test with
  diagnostic UART dump on failure.
- Trace replay: version-checked trace format; mismatch is a hard error.
- gem5 calibration drift: nightly compares against manifest; out-of-band
  delta fails the run and demands recalibration, not silent acceptance.

## Build Order (each phase a usable milestone)

1. **Risk gate:** boot axiomOS to UART on a stock Renode ARMv8-A board.
   Proves Renode CPU viability before any custom modeling. If Renode's
   ARMv8-A support proves inadequate here, revisit tier design before
   proceeding.
2. `virt-pi5.repl` + mailbox stub → boot to userspace on the Pi 5 memory map.
3. PCIe RC + RP1 models → per-commit driver suite green. *(Largest phase;
   the flat-map fallback decision point lives here.)*
4. Sensor devices, injection API, trace replay.
5. gem5 config + calibration against real Pi 5 + nightly job.
6. CI hardening throughout (partial wiring begins in Phase 2).

## Decisions Log

| Decision | Choice | Alternative rejected |
|---|---|---|
| Overall approach | Hybrid: QEMU smoke + Renode complete machine + gem5 latency | Renode-only; QEMU fork with raspi5 machine |
| Machine completeness | Full — all RP1 peripherals, 4 cores, full memory map | Trimmed minimal machine |
| Latency tier | gem5 from day one, calibrated | Instruction-count proxy only |
| RP1 attachment | Full PCIe model (RC + endpoint + BARs) | Flat-mapped registers (kept as fallback) |
| Sensor fidelity | Value injection + trace replay | Closed-loop physics |
| Jetson | Deferred, slot reserved | CPU-arch-level support now |
| Repo shape | One runner crate + tool configs | Five-crate simulator framework |
