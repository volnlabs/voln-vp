# Phase 2 virt-pi5 boot gate — BLOCKED

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

## Result

**BLOCKED.** The ELF, DTB, platform description, GIC, UART, and mailbox stub
load, but the userspace marker is never emitted. During execution Renode
treats axiomOS high-half virtual instruction/data addresses as physical
system-bus addresses. The accesses therefore fall outside modeled RAM,
produce repeated unmapped-access diagnostics, and leave the UART capture
empty.

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
That diagnostic run used the Phase 1 DTB, whose RAM begins at `0x40000000`;
the `+0x40000000` portion reflects that input, but it does not explain the
untranslated `0xFFFF8000...` high half reaching the system bus.

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

## Stop-gate decision

Do not start the RP1, sensor, trace, or CI scenario phases and do not report
Phase 2 as passing. Continuing requires one of these architecture decisions:

1. Fix or extend Renode ARMv8-A address-translation behavior so guest virtual
   addresses are translated before system-bus access.
2. Add a documented high-address alias in the machine model. This is a
   simulation divergence and needs explicit acceptance plus validation that
   it does not hide MMU bugs.
3. Revisit the primary backend choice, as required by the design's ARMv8-A
   risk gate.

Changing axiomOS to add a simulation-only MMU path is not an in-scope
workaround: the design requires an unmodified kernel image.
