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
(pin f379261; PROVENANCE.md §2), plus 8085EXM for M3. The NEC V20/V30 user's manual, pinned by x86jit in the
same stash, is recorded for M2 in PROVENANCE.md §1. The unpublished `x80-oracle` crate runs them
offline ([commands](commands.md)); on 2026-11-12 8080PRE passed and 8080EXM passed
all 25 groups, about 16 s for 2.9 billion instructions in a release build.
M2 remains in progress: its x80-side work (beads `x80-pzp.1` to `.5`) runs from the
pinned NEC manual, and its joint witnesses wait on x86jit's V20/V30 bead (C4).

**M4: done (2026-11-13, epic `x80-bi2`).** The `z80` model executes every
documented instruction through `run`/`step` (decisions 13–15). The Zilog NMOS
undocumented track is also done (2026-11-13, decision-20): every normally fetched
encoding executes, with full eight-bit F/F', retained WZ and Q. The eleven group
executors dispatch directly. Five stored maps contain MAIN 252, CB 256, ED 256,
INDEX 85 and INDEX_CB 256 effective entries; with ignored MAIN fallbacks,
1780 logical terminals execute (dispatch slots excluded).

Decision-20 admits ED70 (**A/H**), ED71 (CPU preservation **H**, emitted zero
**L**), NEG/return aliases, index halves, SLL and indexed-CB copies/BIT aliases
(**H** observed operations; new costs/access mappings **L**). General ED holes,
IM aliases and ignored/repeated prefixes remain **L**. Chains use the last
selector, one boundary/retirement, wrapping PC/R and saturating u64 costs
outside terminal-table fields, without a prefix buffer. All ED returns,
including RETI, copy IFF2 into IFF1 (**L**); ED4D retains its peripheral signature.

Full F/F' transfers, ordinary Y/X classes, BIT S/PV and block-I/O flags have
independent literal witnesses. BIT-register Y/X retains its **L** written-key
gap. Exact WZ equations, Default=0 and reset preservation remain **L**; only
discriminated BIT projections are **H**. Continuing INxR uses the accepted
ED-address+1 replacement for the falsified BC rule. Q selects SCF/CCF pattern A
(**B8039/H**, specimen-qualified), with **L** classification, prefix clearing,
interrupt/default/reset and snapshot bookkeeping. Continuing blocks expose
PC-derived Y/X and I/O PV/H at each boundary: **H** discriminated LDxR/INxR
observations; **L** arbitrary PC, CPxR/OTxR and full I/O domain. Termination
retains ordinary flags. DAA's accepted correction and the **L** LD A,I/R
parity-note and EI/NMI mappings remain; no new anomaly is admitted.

`tests/z80_undocumented_{ed,index,cb,flags}.rs`, `tests/z80_wz.rs` and
`tests/z80_q_repeat.rs` pin literal maps, flags, latch rows, traces, long/wrapping
chains, repeat boundaries, reset and complete snapshots. `tests/z80_coverage.rs`
checks every logical terminal; `tests/z80_timing.rs` independently checks costs
and conditional/repeat paths, NMI and interrupt modes. Interrupt refusals retain
zero contributed progress and complete CPU restoration, including WZ/Q.
The [closure residual inventory](../../PROVENANCE.md#residual-facts-at-the-z80-undocumented-closure-2026-11-13)
collects every decision-20 label, witness pointer and remaining evidence gap.

NMI and INT acceptance execute, including HALT wakeup, RETN/RETI, EI delay,
IM 0's RST-only restriction, IM 1's fixed vector and IM 2's even-vector table
lookup (decision-15 rules 7–10). Refused interrupt acknowledges restore the
complete boundary state. `tests/z80_interrupts_nmi.rs` and
`tests/z80_interrupts_int.rs` pin acceptance and refusal traces. The boundary
witnesses in `tests/z80_run.rs` use IM0 non-RST acknowledges for rollback,
and `tests/z80_interrupts_nmi.rs` uses odd IM2 after NMI/RETN (decision-20 §3
fetched-admission update; decision-15 rule 9 restrictions retained);
`tests/z80_undocumented_ed.rs` pins the full ED map, aliases, ports, unequal
IFFs, NOP traces, wrapping PC/R and one retirement (decision-20 §1).
`tests/z80_determinism.rs` pins a 40-step, 410-state mixed program with CB, ED,
DD, FD and DD CB, an NMI between LDIR iterations, full-width port I/O and
scripted IM 1/IM 2 interrupts after HALT. Unlimited execution, individual steps,
fixed and varying cycle budgets produce identical final registers, latches,
memory, bus traces, poll sequences and actual progress totals.

The model exposes `Z80`, `Registers`, `Flags`, `Latches`, `InterruptMode` and
`Inputs`, with both register sets, pair accessors and reset. Construction
values, independent eight-bit F/F' representation (`Flags.y`/`Flags.x`),
reset preservation and sampler bookkeeping remain library choices. Source pins, page maps, reconciliations and the shared Zilog lineage
caveat remain in PROVENANCE.md §1. INDEX_CB's fourth-byte access kind, EI-delay
edge cases and parity-note boundary timing retain decision-15's unverified
qualifications; per-M1 R counting is derived. The complete residual list is in
[PROVENANCE.md §3](../../PROVENANCE.md#residual-unverified-or-derived-facts-at-the-m4-closure-2026-11-13).

Final closure campaigns, **MEASURED 2026-11-13**, release on `44c81c5`:
Z80 prelim PASS (899 steps / 8,714 cycles); ZEXDOC and independently audited
ZEXALL each **67/67 judged OK**, 5,764,169,747 steps / 46,734,978,642 cycles.
All six z80test variants complete **160/160 groups**, **156/156 judged OK**,
four skipped NEC/ST diagnostics and zero input-check failures. The 8080PRE
(on both Intel models), 8080EXM and 8085EXM regressions pass; 8085EXM's long
ALU CRC remains reported, not judged. [PROVENANCE §2](../../PROVENANCE.md#final-z80-undocumented-closure-campaigns-2026-11-13)
records counts, steps, cycles and host costs. No ZEX descriptor is filtered.
The Z80 CP/M output/RET host adds one step / 11 cycles per BDOS call;
z80test service RETs add one step / 10 cycles each. CRCs license no new
hardware facts: unidentified ZEX CRCs only falsify, and z80test confirms only
independently stated facts discriminated within its identified specimen scope.
Fmt, clippy, 1,246 workspace tests and 15 doctests pass. The sandbox rejects
`nice`'s priority change; commands still complete. Supervisor gates, review
and landing remain separate.

Closure duplication measurements retain decision-13's sharing boundary; see
[architecture](architecture.md) and [plan §5](plan.md#5-milestones).

**M3: done (2026-11-13, epic `x80-4gk`).** The `i8085` model executes every
documented instruction, including RIM/SIM, and all five interrupt sources with
8085 state totals; INTR supports RST/CALL, while other
acknowledge bytes return `Unsupported` with boundary state preserved. All 256
fetched bytes now execute under decision-18, superseding decision-11 in part
(decisions 12/18; `tests/i8085_coverage.rs`, `tests/i8085_timing.rs`).
The six-poll boundary sequence, persistent TRAP/RST 7.5 sampling, serial
notifications and budget splits have replay witnesses (`tests/i8085_run.rs`,
`tests/i8085_determinism.rs`; decision-12).
[PROVENANCE.md §2](../../PROVENANCE.md#2-oracles) records the 8080PRE and
8085EXM passes, including the unjudged long-ALU CRC;
[§3](../../PROVENANCE.md#residual-unverified-or-derived-facts-at-the-m3-closure-2026-11-13)
lists the original residual readings; the
[8085 undocumented closure list](../../PROVENANCE.md#residual-facts-at-the-8085-undocumented-closure-2026-11-13)
maps the implemented decision-18/19 facts and their S/D/L limits to named witnesses.

The 8085 now stores V/K independently and implements the documented-instruction
hidden-flag rules of decision-18 §4, including its D/S/L qualifications
([architecture](architecture.md)). The PSW image is `S Z K AC 0 P V CY`;
POP loads V/K independently and discards bit 3. Default clears V/K and reset
preserves them by library rule. Unmasked literal witnesses live in
`tests/i8085_vk_flags.rs`; the documented five-flag expectations remain
separate from these qualified hidden-flag rules. The ten admitted undocumented
instructions now execute in the 8085-owned `undocumented` group (decision-18
§§2–4): all 256 fetched bytes complete, with 27 conditional second paths.
Their operations/lengths/totals are documented by second source (CA80C85B),
unverified for Intel 8085; each encoding has independent die confirmation
(exact 8085 part unverified). Explicit library rules cover bus mappings,
unsigned offsets/wrapping, DSUB's high-byte flags and ARHL's V clear.
`tests/i8085_undocumented.rs` witnesses every instruction/path with literal
flags and traces, including width edges and RSTV's ordinary retirement.
The 8080 instruction behavior and five-flag model are unchanged. 8085EXM's D5h mask
excludes V/K and cannot validate them (decision-18 §5).

The oracle host now uses a private `OUT FFh; RET` fetch stub at 0005h for
8085 BDOS calls, since 08h executes DSUB. Its boundary-stop rendezvous keeps
Data reads of the stack word at 0006h intact and adds one step / 10 cycles
per call. The 8080 retains its 08h refusal rendezvous; these are host rules,
outside core semantics. The closure release reruns on `766906e` reproduce
the §2 verdicts and totals. Intel specimen/revision confirmation, physical bus
traces, exact POP loads and the L hidden-flag effects remain unverified.

The Zilog NMOS Z80 track is complete under decision-20, as recorded above.
Undocumented 8080 behavior was researched (`x80-mn3.4`): one hedged second-source table (Mitsubishi M58710S, 1977) lists the folklore NOP/JMP/RET/CALL aliases, AMD's Am9080A handbook calls the response unpredictable, and no silicon key exists. It stays
refused, preserving
decision-3's refusals and decision-9's V20 ED escape.

**M5: done (2026-11-13, epic `x80-8sg`).** The private `intel` tier shares
8080/8085 register access and Data Transfer, Arithmetic (including DAA), Logical,
Branch and Stack/I/O executor bodies under decision-16. Concrete borrowed
register views and the closed `FlagRules` contract preserve separate public
model types. Encoding tables, state totals, flag/ALU facts, qualified DAA carry
and XTHL order, conditional operand reads, run loops, reset, interrupts and
machine control remain model-owned. Z80's sharing boundary is unchanged.
The existing tests and hardware witnesses remain intact; this extraction
admits no additional encodings or hardware claims. The closure measurement
reproduces the baseline and checkpoint, reducing the two models' five group
modules from 570/556 to 226/220 stripped lines, with 448 lines in the shared
tier; [architecture](architecture.md) records the method and per-file counts.
The optional release closure campaign passed 8080EXM's 25 judged groups and
8085EXM's 24 judged groups, retaining the long-ALU qualification. Alternating
runtime measurements and an unsuccessful, reverted inline-hint experiment are
recorded there; the checkpoint's 8085 slowdown was absent from the final
baseline comparison within the observed timing variation.

This is a capability snapshot, not the task board. Consult beads for claims and
priorities. Update this file when a milestone advances.
