# voln-vp

Virtual Platform for axiomOS Development.

```sh
cargo install voln-vp
voln-vp doctor
voln-vp run --board virt-pi5
voln-vp test --board virt-pi5
```

Phase 1 passed on Renode's stock Cortex-A78 platform. See the
[risk-gate evidence](docs/probes/2026-07-30-armv8a-risk-gate.md) and
[implementation checklist](TODO_NOW.md).

See the [approved design](docs/superpowers/specs/2026-07-14-voln-vp-design.md)
for architecture and scope.
