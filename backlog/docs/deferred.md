# Deliberately deferred

Do not implement these unprompted. Open work is tracked in beads.

| Item | Why deferred | Revisit |
| --- | --- | --- |
| NEC emulation-mode extensions | Shared mapping and handoff contract unverified | M2 with decision-23 and V20 bead (x86jit) |
| Z80 model | Milestone order (plan §5) | M4 |
| Bus-cycle stepping, pin-level timing, READY and HOLD/HLDA behavior | Instruction T-state totals do not establish bus-cycle accuracy; includes the 8085's HOLD/HLDA and temporary halt leave during HOLD (decision-12 rule 11) | An explicit embedder requirement and cited timing contract |
| Undocumented/silicon-specific behavior | Typed unsupported exit until a primary source documents an encoding (decision-3) | A pinned primary source, or a model that needs a silicon variant (M3/M4) |
| Oracle vendoring | The chosen exercisers are GPL-2.0-or-later and pinned fetch-only (PROVENANCE.md §2); nothing GPL is committed | A need for offline durability, with the maintainer's licence decision |
| Benchmark crate and optimization framework | No representative benchmark workload is established | Measured need after correctness |

JIT, x86 execution, full machines, devices, loaders and OS services are non-goals,
not waiting milestones. See [plan §1](plan.md#1-goals-and-non-goals).
