# Deliberately deferred

Do not implement these unprompted. Open work is tracked in beads.

| Item | Why deferred | Revisit |
| --- | --- | --- |
| Instruction implementations and concrete public APIs | M0 delivered skeleton, tooling and decisions only | M1: the API and model decisions are accepted (decisions 5-7); beads x80-ecy.5 onward implement them |
| NEC emulation-mode extensions | Shared mapping and handoff contract unverified | M2 with decision-23 and V20 bead (x86jit) |
| 8085 and Z80 models | Preserve a narrow first implementation | M3 and M4 respectively |
| Bus-cycle stepping, pin-level timing and READY behavior | Instruction T-state totals do not establish bus-cycle accuracy | An explicit embedder requirement and cited timing contract |
| Undocumented/silicon-specific behavior | Typed unsupported exit until a primary source documents an encoding (decision-3) | A pinned primary source, or a model that needs a silicon variant (M3/M4) |
| Oracle vendoring | Licences of exerciser programs and captures not settled; storage decided: the shared oracles repository as x80's own `oracles/` submodule | Bead x80-ecy.3 (M1) |
| Benchmark crate and optimization framework | No interpreter or representative workload exists | Measured need after correctness |

JIT, x86 execution, full machines, devices, loaders and OS services are non-goals,
not waiting milestones. See [plan §1](plan.md#1-goals-and-non-goals).
