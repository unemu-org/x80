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

**M4: done (2026-11-13, epic `x80-bi2`).** The `z80` model executes every documented
instruction through `run`/`step`, including all prefixed forms and repeating
blocks (decisions 13–15). The eleven group executors dispatch directly; the
temporary documented-but-unimplemented path and all group staging switches
are removed. `tests/z80_timing.rs` independently checks every encoding and
conditional/repeat path total, plus NMI and all interrupt modes. Documented
totals use the primary and Mostek second witness; decision-20 §1 supplies the
labelled L alias/hole and §3 CB totals. The five tables contain MAIN 252,
CB 256, ED 256, INDEX 85 and INDEX_CB 256 effective entries. With ignored MAIN
fallbacks, 1780 logical terminal encodings execute (dispatch slots excluded).
Decision-20 §1 admits ED70 flags-only input (**A/H**), ED71 zero output
(**L**, CPU preservation **H**), NEG/return aliases (**H**, alias costs **L**),
IM aliases (**L**) and 178 two-byte, 8-state NOP holes (**L**). All ED returns,
including RETI, copy IFF2 into IFF1 (**L**); ED4D alone retains the peripheral
RETI signature without a callback. The Z80 CP/M host uses a private
`OUT FFh; RET` boundary stop, adding one step / 11 states per BDOS call and
preserving Data reads of the stack word. The nine historically excluded ZEXDOC
groups are now runnable under §§2–3; the existing host still judges its audited
58-group subset. Section 4 now stores full F/F' images and implements the
class rules below. ED63/6B remain four-byte pair loads. Decision-20 §2 now executes
register-only index halves (**H** operation/flags, **L** costs), retains
ordinary H/L beside indexed memory (**A/H**), and admits ignored/repeated
DD/FD (**L**). Chains use the last selector, ED cancels it, CB selects indexed-CB,
and prefixed HALT has no displacement. Arbitrary chains have one boundary and
retirement, wrapping PC/R and saturating u64 costs without a prefix buffer.
Decision-20 §3 adds SLL (**H** operation/flags, **L** costs), indexed-CB
rotate/shift/SET/RES register copies and BIT aliases (**H**), with **L**
access/cost/no-write and computed-result copy rules. Ordinary H/L receive copies;
a write-ignoring device still leaves the computed result in the register. No
normally fetched encoding refuses. `tests/z80_undocumented_index.rs` pins the
full main map, half flags, exceptions, chains, interrupts and long/wrapping traces;
`tests/z80_undocumented_cb.rs` pins all CB/DDCB/FDCB terminals, flags, copies,
BIT aliases, device behavior, access/R/cost traces and a prefix chain.

Decision-20 §4 now implements full F/F' input/packing/exchange and all ordinary
Y/X class rules, BIT S/PV and complete block-I/O flags. Transfer and class
rules retain their **H** observed-vector limits; BIT-register Y/X remains
**L** with an explicit written-key gap. DAA's accepted correction is unchanged.
`tests/z80_undocumented_flags.rs` supplies independent literal full-byte
witnesses; existing six-bit/unknown-preservation assertions are replaced.
The changed-assertion inventory and measured campaigns are in PROVENANCE §1,
“Decision-20 §4 implementation”. Decision-20 §5 now supplies public
`Latches.wz`: Z80-owned 16-bit state, Default=0000h, preserved by reset,
included in supplied/cloned boundary snapshots, deterministic continuation,
Budget preservation and Stop/Unsupported rollback. Every exact update rule
is **L**, including continuing INIR/INDR's accepted ED-address+1 replacement
for the falsified BC rule. BIT (HL) observes retained WZ13/11; indexed BIT
first sets WZ to its effective address. The projected observations retain
their **H** vector limits; they do not establish whole-word hardware state.
`tests/z80_wz.rs` has 21 literal row/state witnesses, with carry/no-carry,
wrapping, both block paths, ignored prefixes and BIT projection probes.
Existing full-state expectations now include WZ without added bus accesses.
**Interim L:** SCF/CCF uses unconditional A until §6 Q (`x80-mn3.16`). Continuing repeats use
ordinary §4 flags until §6's boundary overrides; terminating iterations use
§4 in full. These staged results do not claim final Z80 conformance.

`tests/z80_coverage.rs` independently enumerates literal admitted and empty
refused key sets for every table, distinguishing dispatch slots. Every admitted
encoding completes one step from neutral state. Interrupt API refusals retain
zero contributed progress and full CPU restoration, including R, alternate
registers, EI delay, parity-note, NMI history and WZ. Per-group witnesses pin semantics,
flags, timing and bus accesses with the A/H/L limits of the accepted contracts.

The §5 worker campaign is **MEASURED**, offline, 2026-11-13: fmt, clippy,
workspace tests (1,234 tests plus 15 doctests) and separate doctests PASS;
release prelim PASS, ZEXDOC 58/58 judged PASS, 8080PRE/8080EXM/8085EXM PASS.
All six z80test variants complete 160/160 groups with zero input-check
failures: memptr/doc/docflags 156 judged OK, full/flags 150 OK / 6 FAILED,
ccf 89 OK / 67 FAILED. Memptr groups 000–002 and 007–159 are OK; diagnostics
003–006 are skipped. Full/flags now pass both BIT (HL) groups; remaining
failures are SCF/CCF and the four continuing LDxR/INxR self-overwrite groups,
consistent with the deferred §6 flags. No final conformance claim follows.
Full campaign totals and changed-assertion inventory are in PROVENANCE §1,
“Decision-20 §5 implementation”. Supervisor gates/review/landing are separate.

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

The licensed, fetch-only Z80 oracles passed the decision-20 §1 release
campaign on 2026-11-13 ([PROVENANCE.md §2](../../PROVENANCE.md#2-oracles);
[commands](commands.md#oracle-runs)), **MEASURED**: prelim PASS, 899 steps /
8,714 cycles; ZEXDOC **58/58 judged groups PASS**, 4,432,085,110 steps /
35,946,551,658 cycles. Relative to the historical refusal rendezvous, the
increases are 1 step / 11 cycles for prelim and 118 steps / 1,298 cycles for
ZEXDOC: exactly one step / 11 cycles per BDOS call.
Nine undocumented-encoding groups were **not run (decision-14)**. Expected CRCs
are attributed to a real Z80 by the source, without a part/revision or per-group
capture; passing does not establish silicon equivalence or license semantics.
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

Next in the undocumented track is Zilog **NMOS Z80** (`x80-mn3.5`): pin sources
and origins, qualify facts under decisions 17/18, and accept a record
superseding the relevant parts of decision-14 before implementation.
Undocumented 8080 behavior remains research-only (`x80-mn3.4`), preserving
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
