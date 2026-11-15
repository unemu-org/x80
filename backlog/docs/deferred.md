# Deliberately deferred

Do not implement these unprompted. Open work is tracked in beads.

The 8085 undocumented track (decisions 18/19) and Zilog NMOS Z80 track
(decision-20) are complete, always on without an embedder profile. Every
normally fetched encoding executes on those models; full flags and Z80 WZ/Q
are implemented with independent literal witnesses. Their remaining evidence
gaps do not reinstate fetched-byte refusals. Per-fact labels and witness
pointers are in [PROVENANCE §3](../../PROVENANCE.md#3-witnesses-and-current-scope).

Final six-variant z80test and 67-group ZEXDOC/ZEXALL campaigns pass
([PROVENANCE §2](../../PROVENANCE.md#final-z80-undocumented-closure-campaigns-2026-11-13)).
The hosts and campaigns are complete. Projected CRCs cannot confirm whole WZ
words, unobserved output data, physical timing or injected-interrupt paths;
remaining hardware questions below require the two keys of decision-17 and
an explicit provenance ruling before changing a library fallback.

| Item | Why deferred | Revisit |
| --- | --- | --- |
| NEC emulation-mode extensions | Shared mapping and handoff contract unverified | M2 with decision-23 and V20 bead (x86jit) |
| V20 round trip through a real adapter | M2's joint witnesses are the shared contract vectors, which each side runs against a fixture of the other (maintainer decision, 2026-11-15; plan §3). Running x80-core and x86jit together on shared memory needs the embedder's adapter, which neither library hosts. The three NMI contract cases also wait for a NEC NMI input in x86jit (`xj-50w.19` (x86jit)) | An embedder adapter exists: run the format-2 contract vectors through it; port the NMI cases when x86jit has the input |
| Bus-cycle stepping, pin-level timing, READY and HOLD/HLDA behavior | Instruction T-state totals do not establish bus-cycle accuracy; includes the 8085's HOLD/HLDA and temporary halt leave during HOLD (decision-12 rule 11) | An explicit embedder requirement and cited timing contract |
| Undocumented 8080 behavior | Researched in `x80-mn3.4` (2026-11-13): one hedged second-source table (Mitsubishi M58710S, 1977) lists the folklore NOP/JMP/RET/CALL aliases, AMD's Am9080A handbook calls the response unpredictable, and no silicon key exists (PROVENANCE discovery inventory); no qualifying source under decision-17; decision-3 keeps typed Unsupported exits and decision-9's V20 ED escape working | Research finds evidence satisfying decision-17 and a later per-model record admits the behavior |
| 8085 Intel silicon scope and physical bus traces | decision-18 admits all ten operations, lengths and timings under the second-source licence; Intel specimen/revision equivalence, exact temporal order/dummy activity for LHLX/SHLX/RSTV and JK/JNK operand-read traces remain unverified. Its API mappings and width/offset rules are explicitly labelled | Pinned named-Intel evidence; revise the labelled mappings if hardware differs, without treating licensed timings as blocked |
| 8085 remaining hidden-flag hardware evidence | decision-18 chooses V/K effects, arbitrary POP loads, Default/reset and explicit preservation rules; single-origin die readings and unstated preservation remain labelled library rules. DSUB P/AC details and hidden-enable discrepancies are not fully confirmed | Independent evidence for exact updates/load/preservation, DSUB P/AC and non-BCD DAA, followed by unmasked witnesses; D5h-masked 8085EXM cannot settle these questions |
| Z80 BUSREQ/BUSACK, external WAIT, pin timing and refresh addresses | decision-15 rule 14 omits these inputs/signals and their interrupt gating; automatic acknowledge waits remain in totals, external waits add no modelled time | An explicit embedder requirement and cited timing/arbitration contract |
| Z80 CMOS, other-maker and NMOS subvariant behavior | Epic `x80-mn3` targets Zilog NMOS. Accepted decision-20 replaces decision-13 rule 6's equivalence assumption and selects deterministic Q pattern A without a profile; 8811 and board/frequency-sensitive reports prevent a universal NMOS claim. ED71 CMOS FFh and part-specific parity-window contrasts remain unverified | Base scope accepted in decision-20 (2026-11-13); pinned part-qualified evidence, independent witnesses and a maintainer decision-6 variant decision for extensions |
| Z80 remaining undocumented hardware evidence | Accepted decision-20 supplies labelled library fallbacks for ED71 output bytes/CMOS contrast, prefix timing/boundaries, RETI IFFs, the BIT-register written-key gap, full WZ/reset, continuing-INxR written key and repeat-I/O paths, Q specimen/board dependence and the LD A,I/R window. No qualifying second-source manufacturer licence exists in this corpus | Independent written/silicon evidence and discriminating literal/projection witnesses; preserve source disagreements and revise a fallback only with an explicit provenance ruling |
| Z80 IM 0 multi-byte/general injected instructions | decision-15 rule 9 accepts RST only; hardware's other documented instructions, including CALL, need an additional acknowledge/operand contract. Odd IM 2 vectors also remain refused without a hardware-outcome claim | An embedder requirement and an accepted extension with independent witnesses |
| Z80 sub-boundary edges and halted-time advancement | Boundary NMI sampling can miss shorter pulses; prompt Halt performs no idle fetches, cycles or R updates. EI-delay interactions and the LD A,I/R parity-note window remain unverified (decision-15 rules 2/7/10) | Pinned evidence settling the readings or an embedder requiring finer event timing |
| Cross-model helper sharing and crate run-time wrapper | M4 measures duplication without authorising extraction; executors and hardware facts stay model-owned. The oracle harness owns its model enum (decisions 10/13) | Review the closure measurements before helper extraction; a crate wrapper requires an embedder request |
| Oracle vendoring | The chosen exercisers are GPL-2.0-or-later and pinned fetch-only (PROVENANCE.md §2); nothing GPL is committed | A need for offline durability, with the maintainer's licence decision |
| Benchmark crate and optimization framework | No representative benchmark workload is established | Measured need after correctness |

JIT, x86 execution, full machines, devices, loaders and OS services are non-goals,
not waiting milestones. See [plan §1](plan.md#1-goals-and-non-goals).
