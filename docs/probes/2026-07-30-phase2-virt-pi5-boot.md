# Phase 2 virt-pi5 boot gate — MMU resolved, userspace still blocked

Date: 2026-07-30

## Gate

Phase 2 succeeds only when the unmodified axiomOS `embedded-rpi5` image
reaches EL0 userspace on the Renode `virt-pi5` model. The required UART
marker is:

```text
=== axiomos eBPF init ===
```

A kernel-side startup message is not sufficient evidence.

## Inputs

- Renode: `v1.16.1.16973`
- CPU: Renode `CPU.ARMv8A`, `cortex-a78`
- Interrupt controller: Renode GICv2
- Kernel:
  `/home/utkarsh/Work/axiomOS/target/aarch64-unknown-none/release/kernel`
- Kernel profile: `embedded-rpi5`
- ELF entry point: `0x80000`
- ELF SHA-256:
  `136262dd0fa164e46da914d602891a0625241a60478386ff17374214f9f36e31`
- DTB load address: `0x10000000`

The model deliberately starts with one CPU. Multi-core bring-up cannot
resolve a single-core execution failure and would make the gate harder to
diagnose.

## Initial result and corrected diagnosis

The initial run entered Renode's default EL3 reset state. axiomOS's direct
kernel entry handles EL2 and EL1, but treats every non-EL2 entry as though it
were already EL1. It therefore programmed `TTBR0_EL1`, `TTBR1_EL1`,
`TCR_EL1`, and `SCTLR_EL1` while execution remained at EL3. Because
`SCTLR_EL3.M` was clear, Renode correctly sent EL3 high-half accesses to the
system bus without translation.

Representative diagnostics from the bounded probe were:

```text
[WARNING] sysbus: [cpu: 0x87B84] ReadQuadWord from non existing peripheral at 0xFFFF800001512C00.
[WARNING] sysbus: [cpu: 0xF5938] WriteQuadWord to non existing peripheral at 0xFFFF800040001000, value 0x0.
[WARNING] sysbus: [cpu: 0xF5938] WriteQuadWord to non existing peripheral at 0xFFFF800040001008, value 0x0.
```

The sequential writes continued through at least
`0xFFFF8000402CD8F8`. Renode exited zero, while the UART file was empty
(SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`).
That diagnostic run used the Phase 1 DTB, whose RAM begins at `0x40000000`.
The result did not establish a Renode MMU defect.

Renode's official direct-kernel ARMv8-A pattern is now applied before
execution:

```text
cpu SetAvailableExceptionLevels false false
gic DisabledSecurity true
```

The model also initializes the PL011 to the state normally handed off by Pi
firmware. With those corrections, the high-half warnings disappear and the
unmodified kernel reaches physical RP1/PCIe initialization. This validates
Renode's page-table translation path; no Renode fork or high-address alias is
required.

Renode process exit status alone cannot pass this gate: Renode 1.16 can
return success after Monitor errors, and a bounded run can finish without
the guest making semantic progress. The adapter consequently requires the
EL0 marker and returns non-zero when it is absent.

## Reproduction

From the repository root:

```sh
VOLN_VP_VIRTUAL_TIME=0.1 \
  cargo run --quiet -- run --board virt-pi5
```

The adapter prints the `/tmp/voln-vp-phase2.*` artifact directory. It
contains `renode.log`, `uart.log`, the generated wrapper script, and symlinks
to the exact kernel and DTB used.

## Current result

**MMU architecture resolved; userspace gate still blocked.** The corrected
run emits:

```text
PI5_BENCH_FAIL stage=rp1_irq_route error=PcieLinkDown(0)
kernel panicked ... memory allocation of 20971520 bytes failed
```

The first line is expected until the Phase 3 PCIe/RP1 model exists and proves
the kernel reached real Pi peripheral initialization after enabling the MMU.
The second line is the immediate userspace blocker: the embedded 20 MiB
rootfs allocation exceeds the current guest heap's usable contiguous
capacity, even with the specified 8 GiB DTB. That is an axiomOS heap/rootfs
contract issue, not a Renode translation issue.

RP1 model unit development and Monitor-level register tests may now proceed.
Guest userspace integration and the Phase 2 PASS remain blocked until the
axiomOS allocation failure is fixed. A high-address alias is neither needed
nor permitted in the canonical backend.

Artifacts for the corrected run:
`/tmp/voln-vp-phase2.CfjCwv`.
