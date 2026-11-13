# Deliberately deferred

Do not implement these unprompted. Open work is tracked in beads.

Undocumented-family work is tracked under epic `x80-mn3`, always on with no
embedder profile. All ten 8085 extensions now execute; no fetched 8085 byte
is refused. Zilog NMOS Z80 decision-20 §§1–6 ED, index-prefix, CB, full flags, WZ, Q
and continuing-block flag behavior now executes;
undocumented 8080 research is `x80-mn3.4`.
The maintainer's 2026-11-13 evidence rule is recorded in [decision-17](<../decisions/decision-17 - Two-key-evidence-rule-for-undocumented-behaviour.md>).
Source pins, two-key evidence and later per-model records precede implementation;
decision-18 supersedes decision-11 in part;
[decision-20](<../decisions/decision-20 - Z80-undocumented-behaviour-Zilog-NMOS.md>)
records the Z80 changes; the maintainer accepted it on 2026-11-13. The §1 ED
implementation admits all 256 terminals (78 operations / 178 L NOPs), with
A/H/L limits preserved. Section 2 adds H register-only index halves, A/H
ordinary-H/L indexed-memory exceptions and L ignored/repeated prefixes,
including unbounded chains with no intermediate boundary. Section 3 adds H SLL, indexed-CB copies and BIT aliases, with L costs/access
and computed-result copies even when a device ignores the write. No normally
fetched Z80 encoding refuses. Section 4 implements all eight F/F' bits,
ordinary Y/X classes, BIT S/PV and block-I/O formulas. BIT-register Y/X stays
L (independent written key missing). Section 5 supplies retained WZ and
BIT projections; exact words remain L. Section 6 implements specimen-qualified
SCF/CCF pattern A (B8039/H), L Q bookkeeping/prefix/default/reset/interrupt
and snapshot rules, and continuing-step flags (H discriminated LDxR/INxR;
L arbitrary PC, CPxR/OTxR and full I/O equation domain). Terminating repeats
keep ordinary §4 flags. `z80_q_repeat.rs` pins these rules and the retained L
LD A,I/R parity-note and NMI-during-EI mappings; no new anomaly is admitted.
Beads `x80-mn3.9` to `x80-mn3.17` carry the implementation track.

The accepted 8085 contract is [decision-18](<../decisions/decision-18 - 8085-undocumented-instructions-and-V-K-flags.md>)
and applies the maintainer's 2026-11-13 second-source
manufacturer licence. It admits all ten instructions, their operations, lengths
and path totals as **documented by second source (CA80C85B), unverified for
Intel 8085**, with independent die confirmation of each encoding. Silicon's
K=V XOR result sign wins; source defects receive explicit rulings. Physical bus
order and unstated flag effects use labelled library rules, not silicon claims.
These operations and rules are implemented in the 8085-owned `undocumented`
group with independent literal witnesses; the remaining evidence gaps below
do not reinstate fetched-byte refusals. Unsupported INTR acknowledges remain
limited by decision-12 rule 5.

V/K are independent stored flags with unmasked witnesses; decision-19's four
flag-effect hooks keep their rules model-owned. The oracle host uses the
private `OUT FFh; RET` boundary-stop rendezvous for 8085 BDOS, preserving
the stack word's Data view; it adds one step / 10 cycles per call. The
8080 retains the 08h refusal rendezvous. These implemented library mappings
and the per-fact S/D/L limits are listed in
[PROVENANCE §3](../../PROVENANCE.md#residual-facts-at-the-8085-undocumented-closure-2026-11-13).

The Z80 handoff carries forward source revision/date/hash pins, origin accounting,
independent silicon confirmation and literal witnesses, explicit conflict rulings
with silicon precedence, and part-qualified labels. Decision-18's manufacturer
licence applies only to a qualifying compatible second-source data sheet, with
independent die confirmation of each admitted encoding; it does not transfer
the CA80C85B facts to Z80. The NMOS sources are pinned in PROVENANCE §1;
decision-20 supersedes decision-14's admission/refusal, F bits 5/3
and unknown-flag preservation rules, decision-13's NMOS/CMOS assumption and
whole-instruction u8 cost limit, and decision-15's hidden-state omission and
RETI rule. Its seven ordered implementation briefs cover ED forms, index
halves/prefixes, SLL/indexed-CB copies, full flags, WZ, Q/repeat-boundary
flags/interrupt quirks, then closure and exerciser hosts. The beads exist
under epic `x80-mn3`; this list is decision scope, not a second task list.

The decision chooses executable library fallbacks wherever the two keys do not
establish a subfact; this does not turn those rules into silicon claims.
Outstanding evidence includes ED71's emitted byte and alleged CMOS contrast,
general prefix timing/boundaries, RETI IFF effects, a correct independent written
BIT-register Y/X statement, exact WZ updates/reset, an independent written INxR
key and full-word evidence for the continuing-input PC rule, Q's specimen/board
dependence, and the precise LD A,I/R parity window. The continuing-INxR
BC_before_decrement ±1 fallback is **falsified**: in-memory enumeration of each
pinned `.inir_nop`/`.indr_nop` post-BIT flag stream gives 0A537B63, versus
F3B1BE2F from the selected **L** WZ=(address_of_ED+1) modulo 65536 rule and the
pinned expectations. MEMPTR explicitly states that PC form for continuing
LDIR/LDDR/CPIR/CPDR, but not INIR/INDR. The other six continuing block rules
reproduce their groups' pinned CRCs; OTIR/OTDR's terminating assignment hides
the continuing WZ, so those groups cannot discriminate a PC override. See
decision-20 §5 and PROVENANCE §1 for the audit and the retained source disagreement.
Banks' EI/NMI page describes a simulator discrepancy and admits no new anomaly.
The selected Q rule follows stable pattern A, qualified by the 8039 report;
8811's different pattern and the 8634/8644 identity ambiguity remain explicit.
CMOS and other parts need separate evidence and a decision-6 scope decision;
oracle CRCs alone cannot admit hardware facts.

Decision-20 §7 now judges all 67 ZEXDOC groups, including the nine historical
exclusions. ZEXALL has its own matched COM/source descriptors, FFh masks,
digest pins and CP/M host audit; all 67 groups can falsify full-byte flag rules.
Neither suite's unnamed-real-Z80 CRCs license semantics. Their host and
campaigns are no longer deferred; final closure remains `x80-mn3.17`. The six
z80test variants now have a Spectrum-style oracle host in `x80-oracle`, with
fixed linked/data addresses and explicit input precheck accounting; see
[commands](commands.md#z80test-v12a-spectrum-style-host). Its runs are non-final
baselines. After §6, all six variants complete 160/160 groups with 156/156
judged OK, 003–006 skipped diagnostics and zero input-check failures (worker
campaign, MEASURED 2026-11-13). Final closure remains `x80-mn3.17`.
The exact latch table remains **L**; WZ implementation is no longer deferred.
Section 6 now supplies Q and continuing-step flags; implementation is no
longer deferred, while the evidence limits above remain. A MEMPTR
CRC observes a two-bit projection, not an entire latch; post-CCF observes only
tested predecessor distinctions. The decision-20 source audit itself did not run
these campaigns. The Z80 CP/M host now uses a private
`OUT FFh; RET` boundary stop instead of an ED70 refusal, adding one step /
11 states per call while preserving the stack word's Data view;
retain the IM0 non-RST and odd-IM2 API refusals until separately extended.

| Item | Why deferred | Revisit |
| --- | --- | --- |
| NEC emulation-mode extensions | Shared mapping and handoff contract unverified | M2 with decision-23 and V20 bead (x86jit) |
| Bus-cycle stepping, pin-level timing, READY and HOLD/HLDA behavior | Instruction T-state totals do not establish bus-cycle accuracy; includes the 8085's HOLD/HLDA and temporary halt leave during HOLD (decision-12 rule 11) | An explicit embedder requirement and cited timing contract |
| Undocumented 8080 behavior | Research-only `x80-mn3.4`; no qualifying source yet; decision-3 keeps typed Unsupported exits and decision-9's V20 ED escape working | Research finds evidence satisfying decision-17 and a later per-model record admits the behavior |
| 8085 Intel silicon scope and physical bus traces | decision-18 admits all ten operations, lengths and timings under the second-source licence; Intel specimen/revision equivalence, exact temporal order/dummy activity for LHLX/SHLX/RSTV and JK/JNK operand-read traces remain unverified. Its API mappings and width/offset rules are explicitly labelled | Pinned named-Intel evidence; revise the labelled mappings if hardware differs, without treating licensed timings as blocked |
| 8085 remaining hidden-flag hardware evidence | decision-18 chooses V/K effects, arbitrary POP loads, Default/reset and explicit preservation rules; single-origin die readings and unstated preservation remain labelled library rules. DSUB P/AC details and hidden-enable discrepancies are not fully confirmed | Independent evidence for exact updates/load/preservation, DSUB P/AC and non-BCD DAA, followed by unmasked witnesses; D5h-masked 8085EXM cannot settle these questions |
| Z80 BUSREQ/BUSACK, external WAIT, pin timing and refresh addresses | decision-15 rule 14 omits these inputs/signals and their interrupt gating; automatic acknowledge waits remain in totals, external waits add no modelled time | An explicit embedder requirement and cited timing/arbitration contract |
| Z80 CMOS, other-maker and NMOS subvariant behavior | Epic `x80-mn3` targets Zilog NMOS. Accepted decision-20 replaces decision-13 rule 6's equivalence assumption and selects deterministic Q pattern A without a profile; 8811 and board/frequency-sensitive reports prevent a universal NMOS claim. ED71 CMOS FFh and part-specific parity-window contrasts remain unverified | Base scope accepted in decision-20 (2026-11-13); pinned part-qualified evidence, independent witnesses and a maintainer decision-6 variant decision for extensions |
| Z80 remaining undocumented hardware evidence | Accepted decision-20 supplies labelled library fallbacks for unconfirmed output bytes, timings/prefix edges, RETI IFFs, BIT-register written-key gap, full WZ/reset and repeat-I/O paths, Q details and interrupt windows. No qualifying second-source manufacturer licence exists in this corpus | Independent written/silicon evidence and discriminating literal/projection witnesses; preserve source disagreements and revise a fallback only with an explicit provenance ruling |
| Z80 IM 0 multi-byte/general injected instructions | decision-15 rule 9 accepts RST only; hardware's other documented instructions, including CALL, need an additional acknowledge/operand contract. Odd IM 2 vectors also remain refused without a hardware-outcome claim | An embedder requirement and an accepted extension with independent witnesses |
| Z80 sub-boundary edges and halted-time advancement | Boundary NMI sampling can miss shorter pulses; prompt Halt performs no idle fetches, cycles or R updates. EI-delay interactions and the LD A,I/R parity-note window remain unverified (decision-15 rules 2/7/10) | Pinned evidence settling the readings or an embedder requiring finer event timing |
| Cross-model helper sharing and crate run-time wrapper | M4 measures duplication without authorising extraction; executors and hardware facts stay model-owned. The oracle harness owns its model enum (decisions 10/13) | Review the closure measurements before helper extraction; a crate wrapper requires an embedder request |
| Oracle vendoring | The chosen exercisers are GPL-2.0-or-later and pinned fetch-only (PROVENANCE.md §2); nothing GPL is committed | A need for offline durability, with the maintainer's licence decision |
| Benchmark crate and optimization framework | No representative benchmark workload is established | Measured need after correctness |

JIT, x86 execution, full machines, devices, loaders and OS services are non-goals,
not waiting milestones. See [plan §1](plan.md#1-goals-and-non-goals).
