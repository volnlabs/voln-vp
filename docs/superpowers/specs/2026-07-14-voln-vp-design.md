# voln-vp Design: Virtual Platform for axiomOS Development

**Date:** 2026-07-14 (revised 2026-07-15 after external review)
**Status:** Approved design, pre-implementation
**Context:** axiomOS boots on QEMU virt (x86_64, AArch64, RISC-V) and on real
Raspberry Pi 5 with full userspace, eBPF, GPIO/PWM/IIO drivers, and a measured
211ns single-core interrupt latency. No simulation infrastructure beyond QEMU
virt exists. Jetson is a future target; nothing Jetson-specific is built now.

## Goal

voln-vp is a **developer platform**, not another simulator: glue that makes
existing simulation/emulation infrastructure feel like one coherent
development environment. It answers, before touching hardware:

1. **Will it work?** — peripheral drivers (RP1 GPIO/PWM/I²C/SPI/UART/ADC),
   post-firmware boot path, robotics app logic against sensors.
2. **How fast is it?** — deterministic reproduction of timing-dependent bugs
   now; calibrated latency numbers via a reserved gem5 backend when latency
   work starts.

Composition over construction: QEMU, Renode, and gem5 do what they already do
well; voln-vp never reimplements a simulator and never forks QEMU.

## Non-Goals / Stated Boundaries

- **Firmware stage is not simulated.** No open tool models the Pi 5 bootrom →
  EEPROM → VideoCore chain. Simulation entry point is kernel entry with a DTB.
  Firmware-stage bugs are hardware-only.
- **GPU is not simulated** (VideoCore VII, and Jetson GPU later).
- **Jetson:** deferred entirely. Backend/board structure leaves a slot for a
  future `virt-jetson` board; no code is written for it now.
- **Physics simulation:** never built in-house. Sensor fidelity today is value
  injection and trace replay; when robotics work demands physics, a Gazebo/
  Webots **bridge backend** feeds their sensor outputs into Renode's virtual
  devices. voln-vp carries data, not dynamics.
- **Custom GUI / visualization:** not built. Future slots: perfetto trace
  export (kernel trace events → Perfetto timeline) and a VS Code extension
  wrapping the CLI. Nothing now.

## Architecture

### Backends, boards, and the adapter contract

The core knows **no backend by name**. Each backend is a self-describing
directory implementing a small adapter contract; the runner discovers backend
directories and dispatches to them.

```
                       voln-vp
              ┌─────────────────────┐
              │     cli (runner)    │   discovers backends/,
              └──────────┬──────────┘   dispatches via adapter contract
                         │
     ┌───────────┬───────┴──────┬─────────────────┐
     │           │              │                 │
   qemu/      renode/      gem5/ (reserved)   future: gazebo-bridge/,
   boot,      virt-pi5     calibrated         perfetto/, vscode/
   smoke,     machine,     latency            (slots only)
   gdb,       drivers,
   snapshots  sensors,
              determinism
     └───────────┴──────────────┴─────────────────┘
                         │
                  axiomOS ELF — same unmodified kernel everywhere
```

**Adapter contract** (per backend directory):
- `manifest.toml` — name, supported verbs, supported boards.
- Entry points for the verbs it supports: `run`, `test`, `bench`.
- Exit code discipline: simulator failure propagates; CI cannot false-green.

Adding a backend later (gazebo bridge, perfetto exporter, webots) = new
directory satisfying the contract. Zero core change. No plugin-manager
subsystem is built — directory discovery + contract is the entire mechanism;
a manager appears only if backend count ever makes discovery insufficient.

**Boards** are declarative descriptions (memory map, DTB, platform files)
consumed by backends: `boards/virt/` (generic QEMU), `boards/virt-pi5/`.

### What each backend answers

| Backend | Question | Speed | Status |
|---|---|---|---|
| `qemu` | Does it boot? Multi-arch smoke (x86_64/AArch64/RISC-V), GDB, snapshots | seconds | exists, wrapped |
| `renode` | Will it WORK on Pi 5? Drivers, sensors, deterministic execution | fast, deterministic | **primary build** |
| `gem5` | How fast? Calibrated latency | minutes–hours | **reserved slot, deferred** |

**Timing fidelity contract:** Renode gives *determinism and instruction-count
timing proxies* (catches "IRQ path grew 3×" regressions), never nanosecond
accuracy. The gem5 backend, when built, gives *calibrated cycle-level
estimates* (target ±15% of hardware). Only hardware gives true numbers.

## Renode Backend: `virt-pi5` Machine

### Platform (`boards/virt-pi5/virt-pi5.repl`)

| Block | Model | Source |
|---|---|---|
| CPU | 4× Cortex-A76 (closest available Renode ARMv8-A core — A75/A78 acceptable; exact core chosen at Phase 1) | built-in |
| Interrupt controller | GIC-400 (GICv2, as on Pi 5) | built-in |
| Timer | ARM generic timer @ 54 MHz | built-in |
| Debug UART | PL011 at Pi 5 address | built-in |
| RAM | Pi 5 memory map, 8 GB configuration | built-in |
| Mailbox / firmware properties | Stub returning canned firmware responses the kernel expects | custom, small |
| PCIe root complex | BCM2712-style RC | **custom** |
| RP1 peripheral block | GPIO banks, PWM channels, I²C, SPI, RP1 UARTs, ADC | **adapt existing first, write what's missing** |

### RP1 attachment: full PCIe model (decided)

RP1 is modeled as a PCIe endpoint behind a custom BCM2712-style root complex,
with registers exposed through BARs — the kernel's real PCIe enumeration and
probe path runs in simulation.

**Risk:** Renode's PCIe infrastructure is thin; the RC model is likely built
from scratch. **Fallback (logged here deliberately):** if PCIe modeling stalls
past its phase budget, flat-map RP1 registers on the system bus at
DTB-described addresses. Driver register logic stays fully tested; only PCIe
enumeration falls back to hardware-only coverage.

### RP1 peripheral models — reuse before building

RP1 peripherals largely reuse RP2040-family IP, and Renode ships RP2040
models (Raspberry Pi Pico support). Phase 3 therefore starts with an audit:
map each RP1 block the axiomOS drivers touch against Renode's RP2040 models,
adapt what matches, and write only the genuinely missing pieces.

- Custom models written as Renode **Python peripherals**; C# only where
  Python hits limits.
- **Register-accurate for registers the axiomOS drivers touch**;
  datasheet-complete where the driver needs it. Models are built in the phase
  where the driver test suite demands them — never speculatively.
- **Unmodeled-register tripwire:** any access to an unimplemented register
  logs a warning; **strict mode** (used in CI) makes it fatal. Gaps surface
  loudly, never silently.
- No generic `Device → GPIO/PWM/…` abstraction layer: the kernel image is
  unmodified, so drivers speak RP1 registers — an abstract device interface
  would require a sim-specific kernel HAL, defeating the point of testing
  real drivers.

### Sensor simulation

- **Fake bus devices** (`devices/`): IMU, ADC chips, etc., as Python
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
- The same fake-device interfaces are the attachment point for a future
  Gazebo/Webots bridge: physics engine output replaces injected values, the
  Renode side is unchanged.

## gem5 Backend (Reserved, Deferred)

Built when latency work actually starts, not in the initial rollout. The
backend directory and adapter contract slot are defined now so it lands
without core changes. Design, recorded for that day:

- ARM **full-system mode**, boots the same axiomOS image at kernel entry.
- **O3 CPU** parameterized toward Cortex-A76 (~4-wide decode, ~128-entry
  ROB-class window, A76-like branch predictor); caches matching BCM2712
  (64 KB L1I + 64 KB L1D per core, 512 KB private L2, 2 MB shared L3);
  LPDDR4X-class DRAM model. Generic devices only — no RP1.
- **Calibration:** microbenchmark suite (IRQ latency, syscall round-trip,
  context switch, memcpy bandwidth) measured on real Pi 5 → golden numbers
  (211 ns IRQ latency is the first anchor); gem5 parameters tuned until every
  benchmark is within ±15%; calibration manifest committed. Nightly runs diff
  against the manifest; drift fails the run.

Until then, latency regressions are caught by Renode instruction-count
proxies and validated on hardware.

## Repository Layout

This repository, renamed **voln-vp**:

```
voln-vp/
  cli/                     # thin Rust runner: discovers backends/, dispatches
                           #   voln-vp <run|test|bench> --backend <name> --board <name>
  backends/
    qemu/                  # manifest + adapters wrapping existing virt configs
    renode/
      peripherals/         # pcie_rc.py, mailbox.py, rp1_*.py (only what RP2040
                           #   models don't cover)
      scripts/             # .resc boot/scenario scripts
      tests/               # Robot Framework suites
    gem5/                  # reserved: manifest + design notes, no implementation
  boards/
    virt/                  # generic QEMU virt board descriptions (3 arches)
    virt-pi5/              # virt-pi5.repl, memory map, DTB
  devices/                 # fake sensors: imu.py, adc_chip.py
  traces/                  # recorded Pi 5 sensor traces (versioned format)
  ci/
  docs/
```

One thin CLI crate only. Rejected twice now: the research report's five-crate
simulator framework, and a plugin-manager subsystem — both reimplement or
over-wrap what the backends already are.

## Interfaces

- **CLI** is the single entry point for humans and CI:
  `voln-vp <run|test|bench> --backend <name> --board <name> [args]`.
  Backends are discovered from `backends/`, never hardcoded. Simulator exit
  codes propagate.
- **Kernel ↔ backend:** the unmodified axiomOS image. No sim-specific kernel
  code paths; the machine is made to fit the kernel, not vice versa.
- **Backend ↔ core:** the adapter contract (manifest + verb entry points +
  exit-code discipline).
- **Tests ↔ renode backend:** Robot Framework keywords (UART expectations,
  sensor injection, PWM capture assertions).

## Testing & CI

**Per-commit (fast, deterministic, minutes total):**
- qemu backend: boot smoke on x86_64, AArch64, RISC-V.
- renode backend: `virt-pi5` boot-to-userspace; driver suite — GPIO toggle
  readback, PWM sweep capture, I²C IMU read, SPI transfer; one determinism
  check (same scenario twice → byte-identical UART output). Strict mode on.

**Nightly:**
- Full Robot scenario suites and trace replays.
- (When gem5 backend exists: benchmark suite with regression thresholds.)

**Hardware:** manual for now; HIL automation is future work.

## Error Handling

- Unmodeled RP1 register access: warn in dev, fatal in CI (strict mode).
- CLI: propagates non-zero simulator exits; timeout per test with diagnostic
  UART dump on failure.
- Trace replay: version-checked trace format; mismatch is a hard error.
- Backend manifest invalid or verb unsupported: hard error at dispatch, named
  clearly.

## Build Order (each phase a usable milestone)

1. **Risk gate:** boot axiomOS to UART on a stock Renode ARMv8-A board.
   Proves Renode CPU viability before any custom modeling. If Renode's
   ARMv8-A support proves inadequate here, revisit backend choice before
   proceeding.
2. CLI skeleton + adapter contract; qemu backend wrapped (existing flow);
   `virt-pi5.repl` + mailbox stub → boot to userspace on the Pi 5 memory map.
3. **RP2040-model audit**, then PCIe RC + remaining RP1 models → per-commit
   driver suite green. *(Largest phase; the flat-map fallback decision point
   lives here.)*
4. Sensor devices, injection API, trace replay.
5. CI hardening throughout (partial wiring begins in Phase 2).

*(gem5 backend: built when latency work starts — slot and design ready.)*

## Decisions Log

| Decision | Choice | Alternative rejected |
|---|---|---|
| Project framing | Developer platform (voln-vp): compose existing tools behind one CLI | Building/owning another simulator |
| Overall approach | Hybrid: QEMU smoke + Renode complete machine + gem5 latency slot | Renode-only; QEMU fork with raspi5 machine |
| Machine completeness | Full — all RP1 peripherals drivers touch, 4 cores, full memory map | Trimmed minimal machine |
| Latency backend | gem5 **deferred to reserved slot** (revised 2026-07-15; was "day one") | Day-one calibrated gem5 build |
| RP1 attachment | Full PCIe model (RC + endpoint + BARs) | Flat-mapped registers (kept as fallback) |
| RP1 model sourcing | Adapt Renode RP2040 models first, write only gaps (added 2026-07-15) | Writing all models from scratch |
| Device abstraction | None — drivers speak RP1 registers, kernel unmodified | Generic Device→GPIO/PWM interface layer |
| Extensibility | Backend directories + adapter contract, runner discovery | Plugin-manager subsystem; hardcoded `--tier` CLI |
| Sensor fidelity | Value injection + trace replay; future Gazebo bridge feeds same interfaces | In-house physics |
| Visualization | Future slots: perfetto export, VS Code extension | Custom GUI/timeline tooling |
| Jetson | Deferred, board slot reserved | CPU-arch-level support now |
| Repo shape | One thin CLI crate + backend/board dirs | Five-crate simulator framework |
