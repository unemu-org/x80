# Status

**M0: done.** Tracking, the pinned shell, hooks, local gates and the first commit
are in place. Decisions 1-5 settle the M1 API, decision-6 the CPU model layout and
decision-8 the auxiliary carry after AND (superseding decision-7). The 8080 primary
sources are pinned (see [PROVENANCE.md](../../PROVENANCE.md)).

**M1: done (2026-11-12, epic `x80-ecy`).** Every documented 8080 instruction and
RST interrupt acceptance run with manual-derived T-states and per-opcode
witnesses; halt and `reset` are implemented and witnessed. The licensed oracles
pass in a measured, optional run outside the ordinary gates (8080PRE, and 8080EXM
in all 25 groups; [commands](commands.md#oracle-runs)). Residual unverified or
derived facts are listed in PROVENANCE.md §3.

`x80-core` has the types of decision-5: the
model-neutral `Bus`, `MemoryAccess`, `Control`, `Budget`, `Report`, `Exit` and
`Unsupported` at the crate root, and the `i8080` model with `Registers`, `Flags`,
`Latches`, `Inputs` and `I8080`. The BC, DE and HL pair and PSW accessors and the
PSW flag-byte layout are cited and witnessed.

`I8080::run` and `I8080::step` implement the decision-2 budget and the
decision-5 boundary sequence. The opcode map holds the 244 opcodes that 98-153B
Ch. 4 documents, with their lengths and groups, witnessed against the summaries
on 98-153B p. 4-15 and 9800301-04 p. A-5. The other 12 (08, 10, 18, 20, 28, 30,
38, CB, D9, DD, ED, FD) return `Unsupported` (decision-3).

Implemented instruction groups, each with cited state
totals and bus traces:

- Data Transfer: all (98-153B pp. 4-4, 4-5).
- Arithmetic: all, including DAA (pp. 4-6 to 4-8); DAA's carry rule is
  derived, not stated (PROVENANCE.md §1).
- Branch: all (pp. 4-11, 4-12).
- Stack, I/O and Machine Control: all (pp. 4-13, 4-14), including EI with its
  one-instruction delay, DI and HLT. `I8080::reset` clears PC, the
  interrupt-enable latch and halt (pp. 2-13, 5-14). The EI-delay edge cases
  are an unverified model (PROVENANCE.md §1).
- Logical: all (pp. 4-8 to 4-11); AC after ANA and ANI follows decision-8
  (9800301-04 p. 1-12, confirmed against 8080EXM).

Every documented 8080 opcode has required state totals and a completing group
executor; the interim documented-but-unimplemented path of decision-5 item 7
is removed. `tests/i8080_coverage.rs` independently enumerates all 244 documented
bytes and the 12 undocumented bytes: every documented byte completes a step,
and every undocumented byte returns `Unsupported` without state change
(decision-3). Non-RST interrupt acknowledges are also unsupported, as below.

`tests/i8080_determinism.rs` runs a mixed program with a scripted interrupt
twice, comparing registers, latches, memory, reports and the full bus/poll trace.
The same program produces identical state, actual step/cycle totals and trace
across step and cycle budget splits, including one-step slices and overshoot.
`tests/boundary.rs` checks that the root contract cites no manual, and
`compile_fail` doctests keep the model's types off the crate root (decision-6).

Timing: each instruction reports its T-state total, with separate untaken and
taken totals for conditional instructions. The totals are witnessed three ways:
the Ch. 4 entries, the p. 4-15 summary, and an independent transcription of the
Ch. 2 state table (pp. 2-16 to 2-20) cross-read with 9800301-04 Ch. 3. No
bus-waveform, wait-state or READY timing is claimed (deferred.md).

Interrupts: an eligible request (INT high, interrupts enabled, EI delay
expired) is accepted at an instruction boundary. The acknowledge is a bus read
at PC without incrementing it; an acknowledged RST n executes as one step of 11
states (98-153B p. 2-11 Figure 2-8, pp. 2-18, 2-19) and wakes a halted CPU. Any
other acknowledged byte returns `Unsupported` with no state change
(decision-5).

Two Intel 8080 manuals and, for M3, two 8085 sources (205775-002 and the 8085AH data
sheet) are pinned as fetch-only sources in the shared `oracles/` submodule, with two 8080 CPU exercisers, 8080PRE and 8080EXM, as fetch-only oracles
(pin 1431bbc; PROVENANCE.md §2). The NEC V20/V30 user's manual, pinned by x86jit in the
same stash, is recorded for M2 in PROVENANCE.md §1. The unpublished `x80-oracle` crate runs them
offline ([commands](commands.md)); on 2026-11-12 8080PRE passed and 8080EXM passed
all 25 groups, about 16 s for 2.9 billion instructions in a release build.
M2 and M3 are in progress: M2's x80-side work (beads `x80-pzp.1` to `.5`) runs from the
pinned NEC manual, and its joint witnesses wait on x86jit's V20/V30 bead (C4); M3 (8085)
has its sources pinned. M4 has not started. See [plan §5](plan.md#5-milestones).

This is a capability snapshot, not the task board. Consult beads for claims and
priorities. Update this file when a milestone advances.
