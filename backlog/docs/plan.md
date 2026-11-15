# Draft plan — x80

Design sketch, not an accepted API or a task board. Open work belongs in beads.

## 1. Goals and non-goals

Build a guest-agnostic, pure-Rust interpreter library for the Intel 8080 family:
8080 first, then 8085, then Zilog Z80 in the same crate family. Start with zero
dependencies. Embedders own memory, port devices, scheduling and interrupt sources.
Potential consumers include CP/M environments, Altair-class machines, arcade boards
and later Z80 machines; none supplies special cases to the core. The committed
models are the Intel 8080 (M1), the Intel 8085 (M3) and the Zilog Z80 (M4).
Other family members, second-source parts and later derivatives are not in
scope; adding one needs a maintainer scope decision and a pinned primary source.

No JIT, machine emulation, x86 execution, device models, file-format loaders,
operating-system services or bundled ROMs. V20/V30 mode switching belongs to
x86jit and its embedder; x80 executes only the emulation-mode instruction stream.

## 2. Architecture sketch

### 2.1. Crate layout

Two workspace members: `x80-core`, edition 2021, with zero normal, build or dev
dependencies, and the unpublished `x80-oracle` harness, which depends only on
`x80-core`. `tests/boundary.rs` enforces the core's boundary, including optional
and target-specific dependencies. The 8080 model executes every documented
instruction (M1).

Model-neutral types (bus, budget, report, exits, host stop) live at the crate
root. Each CPU model has its own module and type, starting with `i8080` in M1;
see §2.6. Add a separate unpublished harness crate later if oracle tooling needs
dependencies. Do not create 8085 or Z80 modules, crates, a model trait or a
generic decoder before their milestones; move code into a shared module only
when a second model uses it.

### 2.2. CPU state and borrowing

8080 state, implemented in `i8080`: A, B, C, D, E, H, L, PC, SP, flags,
interrupt-enable state, pending enable delay and halt state. The register file and the execution latches
are types of the model (§2.6), not family-wide types; a later model defines its
own. Exact flag images, reset values and timing latches require the pinned
manual, not guessed constants. Keep register-pair and flag manipulation
centralized within the model.

M1 ships an owned register file and owned execution latches inside the model
type (decision-5). A run borrows the embedder's bus; it does not own it. A
borrowed register view for the V20/V30 handoff is an M2 question (§6 Q4).
Execution latches persist across runs and budget exits. Avoid raw-pointer
overlays and a frozen C ABI at this stage.

### 2.3. Bus interface

Accepted in decision-1: a synchronous, borrowed `Bus` trait for byte memory
reads/writes and port input/output. Each access carries a kind that distinguishes
an opcode fetch, another memory access and a port access. Accesses are
infallible; M1 has no bus faults, wait states or bus-cycle stepping. This fits a
small interpreter and avoids replaying I/O side effects through an exit protocol.
An embedder implements segmentation, mapping and devices. No host allocation,
filesystem access or global memory in the core.

The memory address is 16 bits for the 8080; the 8085 and Z80 are expected to
match, to be confirmed from their sources at M3 and M4. The port address carries
what the model's primary source documents for I/O; for the 8080 that is the
8-bit device number. Every memory access carries a kind (opcode fetch, operand
fetch, data, stack, interrupt acknowledge; decision-5). An interrupt acknowledge is a bus read of its own kind
at the current PC, returning the instruction byte the device supplies (§2.5).
The M1 API decision fixes the signatures and the port width is revisited at M4.
A part with a wider address space is out of scope and would define its own bus
contract.

Bus failures, wait-state accounting and observation points need a new,
superseding decision. Do not promise restartable bus faults until partial
reads/writes and device side effects have an explicit contract. Bus-cycle
stepping is a separate requirement, not implied by instruction cycle totals.

### 2.4. Step, run, exits and timing

Shapes and exit precedence are fixed in decision-5: `step` executes at most
one instruction; `run` repeats it under a budget. Both are methods of the model
type and take the embedder's bus and input lines. Budgets carry
instruction steps and T-states; reports include actual progress and a typed return
reason (budget, halt, stop request or unsupported encoding, decision-3; handoff
exits for an integration profile come with M2). `step` and `run` are implemented
for the 8080 model.
Budget, report and exit reasons are model-neutral. The exit enum and the
unsupported-encoding payload are non-exhaustive so that later models and the M2
handoff can extend them. Each model has its own unsupported-encoding policy
record (decision-3 is the 8080's).

Budget policy, accepted in decision-2: zero budget executes nothing; stop when
either limit is reached. The step unit is one retired instruction. Finish an
instruction already started and report actual steps and cycles, including any
overshoot of the cycle limit. Halt returns promptly with a halt exit rather than
spin in `run`. Cycles spent accepting an interrupt count against the cycle limit;
their exact accounting comes with the interrupt implementation. Counters are
`u64` and saturate.

M1 targets documented per-instruction T-state totals, including taken/untaken
paths, from the manual's state tables. Do not advertise cycle-exact bus waveforms,
READY sampling or pin-level timing on that basis alone.

### 2.5. Interrupts, determinism and no_std

The embedder supplies interrupt requests and acknowledgement data explicitly.
The core polls the model's input lines and the host stop at each instruction
boundary, so a device can raise a request during a run. Each model defines its
input lines: the 8080 has one request line (INT), and the inputs of later
models are settled at their milestones. On acceptance the core performs the
acknowledge bus read (§2.3), so the device sees the acknowledge when it happens.
Separate a cooperative host stop from an architectural interrupt. The standalone
8080 path will accept eligible interrupts according to cited enable/delay/halt
rules; an integration policy must hand a V20 interrupt request back to the
embedder before x80 services it, including when 8080 interrupts are disabled.
Settle priority and precise observation boundaries with the NEC contract.

Identical initial state, bus responses and input-poll responses must produce
identical state, accesses, exits and cycle counts. No wall clock, host randomness
or hidden worker threads. Timing and events advance only through explicit inputs.
Accepted in decision-4: the library stays `#![no_std]`, forbids unsafe code and
does not allocate. MSRV stays 1.95 to match x86jit; the `msrv` dev shell
tests it ([commands](commands.md#msrv)). Host-side integration tests may use
`std` without adding crate dependencies.

### 2.6. CPU models and variants

A model is one CPU type as described by its pinned primary source. Each model is
a concrete Rust type in its own module (`i8080::I8080` in M1) and owns its
register file, execution latches, opcode map, flag rules, timing tables, input
lines and reset. There is no family-wide model enum and no model trait: an
embedder that chooses the CPU at run time wraps the model types it needs and
selects once per run call. The crate may offer such a wrapper once two models
exist (M3).

A variant is a part with the same state shape, inputs and opcode-map structure
as its model that differs in a documented table entry or rule. It is a value
fixed at construction, added only when a pinned primary source documents an
observable difference, with a citation and a witness. A part with no documented
difference is not represented. A new model is needed when the state shape, the
inputs or the structure of the opcode map differ.

Models share code by moving it into a shared module at its second use, never in
advance. Shared modules carry no hardware fact. Timing is table data per model;
an entry may hold separate untaken and taken totals for any instruction.

An integration profile (§3) is a separate axis: it defines where control returns
to the embedder, not what an instruction does.

## 3. V20/V30 integration contract

First consumer: the NEC V20/V30 models of x86jit (decision-23 (x86jit) §3, bead C4
`xj-50w.7` (x86jit)), embedded by a machine. decision-9 settles x80's
side: the emulation mode is the `i8080` model plus a V20 integration profile. Facts are
from the pinned NEC manual (stock no. 500350; PROVENANCE.md §1, "NEC V20/V30 8080
emulation mode"); page numbers below are its printed pages.

Ownership: x86jit decodes BRKEM in the native stream and returns a typed exit; the
embedder's adapter (not x80) copies state in and out and runs `I8080::run`; x80 owns the
contract vectors and a mock native side (bead `x80-pzp.5`). Neither core depends on the
other.

| 8080 state or access | V20/V30 backing | Source |
| --- | --- | --- |
| A | low byte of AW (AL) | p. 8-1, Fig. 8-1 |
| BC, DE, HL | CW, DW, BW (high and low bytes as drawn in Fig. 8-1) | p. 8-1, Fig. 8-1 |
| SP | BP | p. 8-1 |
| PC | PC | Fig. 8-1 |
| Flags | low byte of the PSW, S Z 0 AC 0 P 1 CY | Fig. 8-2 |
| Interrupt enable | PSW.IE (bit 9) | Fig. 8-2; p. 9-1 |
| `OpcodeFetch`, `OperandFetch` | PS:address | p. 8-1 |
| `Data`, `Stack` | DS0:address | p. 8-1 |
| Not addressable in emulation | AH, native SP, IX, IY, PS, SS, DS0, DS1 | p. 8-1 |

Flow:

- Entry: BRKEM imm8 (0F FF imm8, p. 12-169) pushes PSW, PS, PC on the native stack and
  enters emulation at PS:PC from the vector (IE and BRK cleared: ASSUMED R2, decision-9
  amendment). BRK = 0 is a precondition: the native core refuses BRKEM with BRK = 1
  (decision-9 amendment 2026-11-15). The adapter copies the mapped registers into
  `Registers` and runs x80.
- Return (option B, decision-9 amendment 2026-11-15): x80 returns `Unsupported` at opcode
  EDh with PC at that byte; only the ED fetch reached its bus. The adapter hands back an
  ED escape, and the native core fetches ED at PS:PC again and decodes it: ED FD is RETEM
  (pops PC, PS, PSW; p. 12-177), ED ED imm8 is CALLN (pushes PSW with MD = 0, PS, PC;
  native routine returns with RETI; p. 12-176; IE and BRK cleared: ASSUMED R2, decision-9
  amendment 2026-11-14; consumes a pending 8080 EI delay: ASSUMED). Any other ED form is
  the native core's unknown instruction; any other Unsupported byte is the adapter's typed
  refusal. MD stays 0 in both.
- Interrupts: serviced in native mode (p. 8-4, Fig. 8-6). A request is not an acceptance:
  NMI is always accepted; INT is accepted when it is high at an instruction boundary with
  IE = 1 and no simultaneous NMI or hold request (p. 6-2; pp. 6-1, 8-4). When the native side
  would accept one, the adapter requests a host stop, copies state out at the clean boundary
  and services it natively; RETI returns to emulation. A masked INT is not serviced, except
  that it ends HLT standby (below). The adapter never asserts INT into x80.
- State exchange: at every hand-back the view carries A, the five flags, BC, DE, HL, SP,
  PC, IE, the EI delay (verbatim) and halted; PS and DS0 are read-only. x80 is rebuilt
  from the view at each copy-in (decision-9 amendment 2026-11-15).
- HLT: x80 returns `Halt`; halted lives in the view, kept by the native core. An INT with
  interrupts disabled resumes the 8080 program after the HLT without service (pp. 8-2,
  9-1): the adapter clears halted and runs again. NMI is unconditional; RESET ends
  emulation.
- Timing: x80 reports Intel 98-153B T-states, not V20 clocks; NEC gives V20 clocks only for
  BRKEM, RETEM, CALLN, RETI and interrupts. A joint clock is the embedder's to reconcile.

Answered from the manual: native-only state (AH, native SP, IX, IY, segment registers) is
not addressable in emulation and is untouched by 8080 code (p. 8-1); RETEM restores the full
PSW from the stack, so the native flags and MD come back and the 8080 flags do not carry
over (p. 12-177); the program runs within the 64 KiB of PS, and data and stack within DS0,
with 16-bit offsets (pp. 8-1, 8-2).

To confirm with x86jit (the manual is unclear): IE and BRK after BRKEM and CALLN (ruled R2, both
clear, ASSUMED: decision-9 amendment 2026-11-14); whether the native EI delay
applies to the 8080 EI in emulation, and CALLN consuming it (ASSUMED); the EI delay across
NMI service (open); the PC that CALLN pushes; the offset wrap inside the segment at FFFFh
within PS and DS0 (ASSUMED); the 8080 IN/OUT high address byte (the adapter's; ASSUMED 00);
the uPD8080AF's equivalence to the Intel 8080A (ASSUMED, PROVENANCE §1). The 2026-11-15
items are in decision-9's amendment.

Open for the joint contract (C4 and the adapter; decision-5 gives only x80's local rules):
joint event and budget precedence across the two cores; exact cycle accounting of each
handoff. Settled by decision-9's 2026-11-15 amendment: step accounting (RETEM and CALLN
are one native step each), cancellation (x80's `stop_requested` carries the host's own
stop or cancel) and latch ownership (the State exchange bullet above).

Joint witnesses (beads `x80-pzp.5`, `x80-pzp.6`): the same contract vectors run on both
sides. Maintainer decision of 2026-11-15: shared vectors satisfy M2's joint witnesses.
x80 runs `x80-v20-contract/vectors/mode-switch.txt` (format 2) against its mock native
side; x86jit embeds the same file byte for byte and runs it against its 8080 fixture
(commit 6614d2e0 (x86jit), bead `xj-50w.18` (x86jit)). Twelve of the fifteen cases pass on
both the NecV20 and NecV30 models there, including full entry/exit round trips
(`brkem_retem`, `calln_reti_retem`) and interrupted ones (`int_enabled_running`,
`hlt_int_enabled`). The three NMI cases wait for a NEC NMI input in x86jit (`xj-50w.19`
(x86jit)). A round trip with x80-core and x86jit running together on shared memory
through a real adapter belongs to the embedder ([deferred](deferred.md)).

## 4. Testing strategy and provenance

Pin sources first using [PROVENANCE.md](../../PROVENANCE.md). For each opcode,
derive independent expected values and cycle totals from document, revision,
section/table and printed page. Witness flags that change and flags that remain,
carry/auxiliary carry boundaries, decimal adjustment, register/memory variants,
stack and address wrapping, taken/untaken paths, halt and delayed interrupt enable.
Exercise all documented encodings and record the chosen unsupported/undocumented
opcode policy; never infer one from another emulator's implementation.

Use exhaustive small input spaces where practical for byte ALU operations, plus
small instruction sequences for stateful behavior. Add budget/event boundary
tests, deterministic bus traces, borrowed-versus-owned state comparisons and
instruction-cycle totals checked against the manual's state tables. Expected
values must not merely call the implementation's own flag/timing helpers.
Witnesses are per model and name it. A fact that looks common to two models is
cited and witnessed for each.

Candidate oracles: 8080 exercisers/diagnostics (for example CPU diagnostic and
8080EXM-family programs), Z80 ZEX-family exercisers, and hardware-captured
single-step corpora where available. Exact artifacts, origin, licenses, pins and
redistribution permissions are **to check before vendoring**. Oracles alone never
license a fact. For undocumented behavior, an identified-hardware oracle may
confirm a written fact under decision-17's two-key rule; unidentified-hardware
CRCs remain falsifiers only. Record measured divergences and source rulings.
Manuals and downloaded corpora must never be required for ordinary
offline witness tests. The two Intel 8080 manuals are pinned (PROVENANCE.md);
no 8080 corpus is pinned yet.

## 5. Milestones

| Milestone | Completion criterion |
| --- | --- |
| M0 — skeleton and tooling | Review scaffold, initialize tracking, verify the pinned shell and local gates, settle source pins and initial API decisions. Tracked as epic `x80-2g7`. |
| M1 — 8080 | Complete documented instruction behavior, interrupts and halt, manual-derived cycle counts, per-opcode witnesses, and a licensed diagnostic run where a licence permits one (decision-5). Done 2026-11-12 (epic `x80-ecy`). |
| M2 — V20 integration | Agree the shared contract with x86jit; borrowed-state/memory entry, RETEM/CALLN and interrupt handoffs pass joint witnesses. |
| M3 — 8085 | Pin 8085 sources; record its unsupported-encoding policy; add the `i8085` model, extracting what it shares with the 8080; implement and test model-specific instructions, flags, interrupts and timing without changing 8080 witnesses. Done 2026-11-13 (epic `x80-4gk`). |
| M4 — Z80 | Pin Zilog sources; record its unsupported-encoding policy; add the `z80` model with its own state, prefix decoding, input lines and timing; revisit the port address in the bus contract; model-specific witnesses and licensed oracles. Done 2026-11-13 (epic `x80-bi2`). |
| M5 — shared Intel executors | Extract the private 8080/8085 register view, access helpers and five shared executor groups under decision-16; preserve public types, model-owned facts, encodings and witnesses; re-measure duplication and release oracle runtime. Done 2026-11-13 (epic `x80-8sg`; [closure measurements](architecture.md)). |

These are planning landmarks; after setup, beads holds priorities, dependencies
and claims. Do not turn this draft into a second task tracker.

After M4, epic `x80-mn3` extends undocumented behavior under accepted
[decision-17](<../decisions/decision-17 - Two-key-evidence-rule-for-undocumented-behaviour.md>)
as amended by decision-18, always on with no embedder profile. The 8085 now
executes every byte and stores V/K under decisions 18/19, superseding decision-11
in part with explicit S/D/L qualifications. The Zilog NMOS Z80 undocumented
track is done (2026-11-13, decision-20): every normally fetched encoding,
full F/F', WZ/Q and final six-variant z80test / 67-group ZEXDOC/ZEXALL campaigns,
with explicit **A**, **H**, **B8039/H** and **L** evidence limits and remaining gaps in PROVENANCE §3.
Undocumented 8080 research (`x80-mn3.4`) found no qualifying source: one hedged
second-source table, contradicted by AMD, and no silicon key; decision-3 and
decision-9's V20 ED escape stand.

## 6. Questions for the maintainer

Decided items cite their record; the rest stay open.

1. Decided for the 8080: 98-153B (primary) and 9800301-04 (second witness), pinned
   fetch-only in the shared `unemu-org/oracles` repository (PROVENANCE.md). Decided
   for the 8085 (2026-11-12): 205775-002 (primary) and the 8085AH data sheet
   231718-001, with 9800301-04 as second witness for states. Decided for the Z80
   (2026-11-13): M4 covers documented behaviour only; UM008011-0816 (August 2016)
   is primary, with 03-0029-01 (1977), 03-0002-01 (January 1978 printing),
   03-0027-02 (March 1978) and Mostek MK78515 (V2.0, October 1978) as second
   witnesses. All five are pinned fetch-only and cite-only; page maps, the
   archived vendor-file pin and the shared Zilog lineage caveat are in
   PROVENANCE.md §1. The NEC V20/V30 source for M2 is also pinned there.
2. Decided in decision-1: synchronous, infallible bus; no wait states;
   instruction-boundary timing for M1. Settled for M4 in decision-15: provided
   16-bit port methods retain the 8-bit defaults; Z80 instructions report complete
   T-state totals, without BUSREQ/BUSACK, external WAIT or pin timing.
3. Decided in decision-2: overshoot at completed instruction boundaries; prompt
   halt exit; interrupt-acceptance cycles count; saturating `u64` counters.
   Exit precedence and the step count of an accepted interrupt: decision-5.
   M4 retains these choices (decision-15), retires one block iteration per step,
   and keeps prompt Halt without idle fetches or refresh.
4. Is a safe borrowed-register trait sufficient for V20, and which execution
   latches must the embedder retain across native/emulation transitions? x86jit's
   view (2026-11-12): by-value copy-in/copy-out through `I8080::new`,
   `registers_mut` and `latches_mut` suffices, since copies happen only at
   BRKEM/RETEM/CALLN and interrupt stops. Decided by the maintainer: the V20/V30
   adapter belongs to the embedder (bead `x80-pzp`); decision-9 records the copy-in/out
   contract. Superseded in part by decision-9's 2026-11-15 amendment: the view is
   exchanged at every hand-back, the native core keeps `ei_delay` and `halted`, and the
   adapter keeps no 8080 latch.
5. The joint register/flag/PC/segment/stack and native-context mapping is settled in
   x86jit's C4 bead (xj-50w.7) against the pinned NEC manual; x80's side must be
   reviewed in the same change. Who owns the shared test fixture? Decided (decision-9):
   x80 owns the contract vectors and a mock native side; the mapping x80 relies on is
   in §3.
6. Who recognizes RETEM/CALLN, and what exact bytes, state, timing and event
   precedence apply to each native handoff, including masked external interrupts?
   Decided in decision-9's 2026-11-15 amendment (option B): x80 stops on EDh, the adapter
   hands back an ED escape and the native core decodes RETEM and CALLN; the view
   exchange and adapter rules are there too. Interrupts are served natively through the
   host stop (§3). Still open with C4: handoff timing, event precedence and the items
   listed at the end of §3.
7. Decided for the 8080 in decision-3: undocumented encodings return a typed
   unsupported exit; decision-9's V20 ED escape remains working. Undocumented
   8080 research (`x80-mn3.4`) found no qualifying source; admission needs
   qualifying evidence and a later record. Decided and implemented for 8085: decision-18 supersedes decision-11
   in part; all 256 fetched bytes execute (documented set plus ten second-source
   instructions), with operations/lengths/totals labelled S (CA80C85B,
   Intel-unverified) and independent D encoding confirmation (exact 8085 part
   unverified). Seven flags pack as `S Z K AC 0 P V CY`; arbitrary POP loads,
   Default/reset, physical bus mappings and selected hidden-flag effects retain
   their L qualifications. Decision-19's four flag-effect hooks preserve
   model ownership. The 8085 oracle host uses `OUT FFh; RET` at BDOS 0005h,
   retaining the stack word's Data view, for one extra step / 10 cycles per
   call; 8080 keeps its 08h refusal rendezvous. Release reruns on `766906e`
   reproduce the recorded verdicts/totals; 8085EXM judges 24 groups and reports
   the long ALU group without judging it. D5h excludes V/K; these runs cannot
   settle the remaining Intel/hidden-flag evidence gaps. The
   [closure list](../../PROVENANCE.md#residual-facts-at-the-8085-undocumented-closure-2026-11-13)
   names every implemented rule's label and witness.

   The Zilog **NMOS Z80** track is complete under decision-20, always on
   without a profile: every normally fetched encoding, full F/F', WZ/Q and
   final six-variant z80test / 67-group ZEXDOC/ZEXALL campaigns. The
   [Z80 closure inventory](../../PROVENANCE.md#residual-facts-at-the-z80-undocumented-closure-2026-11-13)
   records each label, witness and remaining two-key gap. CMOS, other makers
   and NMOS subvariants require independent evidence and a decision-6 scope
   decision. IM0 non-RST and odd IM2 remain API refusals; a CRC alone cannot
   admit a hardware fact.
8. Decided in decision-4: keep no_std and no allocation; MSRV 1.95, tested by
   the `msrv` dev shell gate ([commands](commands.md#msrv)).
9. Decided by the maintainer, recorded in decision-6: per-model types with a
   model-neutral core, variants as construction-time values, integration
   profiles as a separate axis (§2.6). Decided for M3 in decision-10: no crate run-time
   wrapper; the oracle harness keeps its own enum. M4 decision-13 retains
   model-owned state, executors, ALU/flags, interrupts and reset, sharing only
   fact-free decode/progress and test support. The closure measurement is
   summarised in architecture.md; helper extraction remains a later decision.
   NMOS/CMOS equivalence is unverified, with no Z80 variant yet. A crate run-time
   wrapper remains deferred until an embedder requests it.
10. Which parts beyond the 8080, 8085 and Z80, if any, enter scope, and with
    which primary sources?
