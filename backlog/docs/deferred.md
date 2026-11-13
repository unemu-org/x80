# Deliberately deferred

Do not implement these unprompted. Open work is tracked in beads.

Undocumented 8085 behavior, then Zilog NMOS Z80 behavior, is planned under epic
`x80-mn3`, always on with no embedder profile. The maintainer's 2026-11-13 direction
is drafted in [decision-17](<../decisions/decision-17 - Two-key-evidence-rule-for-undocumented-behaviour.md>).
Source pins, two-key evidence and later per-model records precede implementation;
those records will supersede decisions 11 and 14 in part.

| Item | Why deferred | Revisit |
| --- | --- | --- |
| NEC emulation-mode extensions | Shared mapping and handoff contract unverified | M2 with decision-23 and V20 bead (x86jit) |
| Bus-cycle stepping, pin-level timing, READY and HOLD/HLDA behavior | Instruction T-state totals do not establish bus-cycle accuracy; includes the 8085's HOLD/HLDA and temporary halt leave during HOLD (decision-12 rule 11) | An explicit embedder requirement and cited timing contract |
| Undocumented 8080 behavior | No qualifying source yet; decision-3 keeps typed Unsupported exits and decision-9's V20 ED escape working | Research finds evidence satisfying decision-17 and a later per-model record admits the behavior |
| Z80 BUSREQ/BUSACK, external WAIT, pin timing and refresh addresses | decision-15 rule 14 omits these inputs/signals and their interrupt gating; automatic acknowledge waits remain in totals, external waits add no modelled time | An explicit embedder requirement and cited timing/arbitration contract |
| Z80 CMOS and other-part undocumented behavior | Epic `x80-mn3` targets Zilog NMOS Z80; the earlier documented-model NMOS/CMOS equivalence remains unverified (decision-13 rule 6) | Pinned evidence of an observable part difference, a witness and maintainer variant decision |
| Z80 IM 0 multi-byte/general injected instructions | decision-15 rule 9 accepts RST only; hardware's other documented instructions, including CALL, need an additional acknowledge/operand contract. Odd IM 2 vectors also remain refused without a hardware-outcome claim | An embedder requirement and an accepted extension with independent witnesses |
| Z80 sub-boundary edges and halted-time advancement | Boundary NMI sampling can miss shorter pulses; prompt Halt performs no idle fetches, cycles or R updates. EI-delay interactions and the LD A,I/R parity-note window remain unverified (decision-15 rules 2/7/10) | Pinned evidence settling the readings or an embedder requiring finer event timing |
| Cross-model helper sharing and crate run-time wrapper | M4 measures duplication without authorising extraction; executors and hardware facts stay model-owned. The oracle harness owns its model enum (decisions 10/13) | Review the closure measurements before helper extraction; a crate wrapper requires an embedder request |
| Oracle vendoring | The chosen exercisers are GPL-2.0-or-later and pinned fetch-only (PROVENANCE.md §2); nothing GPL is committed | A need for offline durability, with the maintainer's licence decision |
| Benchmark crate and optimization framework | No representative benchmark workload is established | Measured need after correctness |

JIT, x86 execution, full machines, devices, loaders and OS services are non-goals,
not waiting milestones. See [plan §1](plan.md#1-goals-and-non-goals).
