---
id: decision-16
title: Intel-family shared executors for the 8080 and 8085
date: '2026-11-13'
status: accepted
---

Decider: maintainer (2026-11-13). Bead `x80-8sg.1`. The maintainer accepted the
sharing design on 2026-11-13, with a checkpoint after the Stack/IO extraction;
this record awaits acceptance. It amends decision-10 and decision-13 rule 7
as specified below. decision-6 is unchanged.

## Context

M4 is complete. The closure measurement in [architecture](../docs/architecture.md)
(`x80-bi2.27`, base `37571c9`) found 488 exact / 498 type-normalised matching
lines out of 570/556 in the five 8080/8085 group modules, and 279/284 matching
lines out of 319/310 in complete `execute` functions, signatures included.
**MEASURED again for this record (2026-11-13, base `fac28c7`):** the same
comment-stripped, nonblank, trimmed-line LCS method reproduces those numbers,
normalising only CPU type/module names for the second comparison.

**READ, independently re-checked for this record:** the current implementations
compare as follows. Locations are under `x80-core/src/`, at this record's base;
these are code observations, not a new hardware measurement or a fresh manual
audit. Identical behaviour does not imply equal source authority.

| Area | Comparison and code evidence |
| --- | --- |
| Registers | Both have `a: u8`, `flags: Flags`, `b/c/d/e/h/l: u8`, `sp/pc: u16`, in that order, and matching BC/DE/HL/PSW getters and setters (`i8080/state.rs:18`, `:45`, `:84`; `i8085/state.rs:15`, `:42`, `:74`). Their public types remain distinct. |
| Flags and PSW | Both store `sign`, `zero`, `auxiliary_carry`, `parity`, `carry` as booleans (`i8080/flags.rs:44`; `i8085/flags.rs:12`). `to_byte` produces the same byte, with bit 1 set and bits 3/5 clear, despite different spellings (`:60` / `:28`); `from_byte` extracts the same five bits (`:85` / `:38`). The 8085 filler bits are decision-11's library choice, not a hardware fact (205775-002 §5.6.5 p. 5-15 marks them undefined; contrast 98-153B Ch. 4 p. 4-13's fixed image). |
| Latches | Three versus fourteen fields (`i8080/state.rs:111`; `i8085/state.rs:94`). Both have interrupt enable, EI delay and halt. Only the 8085 also has `mask5_5`, `mask6_5`, `mask7_5`, `rst7_5_pending`, `trap_pending`, `trap_armed`, `trap_saved_interrupt_enable`, `trap_rim_pending`, `sod`, `last_trap`, `last_rst7_5`. The delay's meaning differs too. |
| ALU | `result_flags` is identical (`i8080/alu.rs:21`; `i8085/alu.rs:23`). Addition uses equivalent names, masks and thresholds (`:32` / `:36`). Subtraction uses unsigned two's-complement addition versus signed differences (`:43` / `:47`), but computes the same byte, AC as no low-nibble borrow, and CY as borrow. |
| Data Transfer | After CPU type normalisation, `execute` and its four helpers (`operand`, `operand_word`, `read_location`, `write_location`) are identical (`i8080/data_transfer.rs:60`, `:142`; `i8085/data_transfer.rs:61`, `:146`). Only the MOV register/register state total differs, 5 versus 4 (`:22` / `:25`). |
| Arithmetic | Apart from totals and an equivalent DAA rewrite, the code matches (`i8080/arithmetic.rs:16`, `:61`; `i8085/arithmetic.rs:19`, `:63`). DAA's 8080 helper returns `before || adjustment_carry` (`:174`); the 8085 spells the same OR inline. Both retain incoming/adjustment carry and preserve the low adjustment's AC in the final flags. Carry retention remains labelled derived for each model. |
| Logical | Only AND's AC rule differs: bit 3 of `A | operand` versus always set (`i8080/logical.rs:64`, `:154`; `i8085/logical.rs:63`, `:147`). The 8080 rule is decision-8's resolution, not agreement between the two primaries. |
| Branch | Totals differ (`i8080/branch.rs:14`; `i8085/branch.rs:24`). Untaken Jcond/Ccond read both operands on the 8080 (`:60`, `:149`), only byte 2 on the 8085 (`:67`, `:162`). PC advances incrementally versus base-plus-offset, with the same external addresses/results for otherwise matching paths. Condition evaluation has no bus effect (`:132` / `:136`). Restart takes an opcode and return PC versus a return PC and vector (`:164` / `:175`); the 8080's `push_and_jump` (`:173`) supplies the matching stack operation. |
| Stack/IO/control | Totals differ (`i8080/stack_io_control.rs:21`; `i8085/stack_io_control.rs:20`). The Stack/IO bodies match; the 8085 combines EI/DI and arms inhibition for either (`:137`), while 8080 DI clears its delay (`:148`). Single-total entries return Taken versus Untaken (`:165` / `:156`), selecting equal cost slots. NOP is a named zero constant versus a zero literal. Matching XTHL code does not remove the 8085's unverified address-order qualification (`i8085/stack_io_control.rs:100`). |
| Model-only execution | The 8085 has RIM/SIM (`i8085/rim_sim.rs:28`), vectored acceptance (`vectored.rs:27`), separate INTR acknowledge (`acknowledge.rs:20`), edge sampling (`cpu.rs:191`) and six boundary polls including host stop (`cpu.rs:118`). The 8080 acknowledges inside `i8080/cpu.rs:201`; its run loop (`:132`) differs structurally from `i8085/cpu.rs:112`. These loops stay separate. |

**MEASURED, same stripped-line method:**

| Group module | 8080 lines | 8085 lines |
| --- | ---: | ---: |
| `data_transfer.rs` | 116 | 116 |
| `arithmetic.rs` | 143 | 142 |
| `logical.rs` | 97 | 94 |
| `branch.rs` | 121 | 117 |
| `stack_io_control.rs` | 93 | 87 |
| Total | 570 | 556 |

The five `ENCODINGS` declarations occupy 109 stripped lines per model,
including their declaration and closing lines. They remain deliberate
per-model tables. Z80's different executor contract, flags, prefix context,
refresh accounting and small helper overlap do not justify extending the tier
to it (decision-13 rule 7; architecture's M4 measurement).

## Choice

1. **Private Intel-family tier — library choice.** Plan
   `x80-core/src/intel/{mod,access,data_transfer,arithmetic,logical,branch,stack_io}.rs`.
   It contains the listed common 8080/8085 executor bodies and access helpers,
   without depending on concrete model modules. Each model retains a thin
   `execute(cpu, system, opcode, instruction)` wrapper: model-owned arms run
   first, then the shared fallback. A model may take an opcode back into its
   wrapper without changing the other model. This record introduces no module
   or executor implementation yet.

2. **Concrete borrowed register view.** A private `View<'a, F>` holds mutable
   references to A, the model's flag value, B/C/D/E/H/L, SP and PC. Each model
   constructs it from its own `Registers`; register access is concrete, never
   a register-access trait. Public register, flag and CPU type identities stay
   separate. Shared entry points take the view, `S: Bus`, opcode and length
   primitives, plus applicable per-model facts, and return `decode::Path`.
   They never take model CPU/state types or model `Instruction`/`Group` types.

3. **Closed flag/ALU fact table.** `FlagRules: Copy` is private to the Intel
   family and implemented by each model's own `Flags`, in its `flags.rs` and
   `alu.rs`, with that model's citations. Its allowed method set is:

   ```rust
   pub(crate) trait FlagRules: Copy {
       fn sign(self) -> bool;
       fn zero(self) -> bool;
       fn parity(self) -> bool;
       fn carry(self) -> bool;
       fn auxiliary_carry(self) -> bool;
       fn set_carry(&mut self, carry: bool);
       fn to_byte(self) -> u8;
       fn from_byte(byte: u8) -> Self;
       fn result(value: u8, auxiliary_carry: bool, carry: bool) -> Self;
       fn add(left: u8, right: u8, carry: bool) -> (u8, Self);
       fn subtract(left: u8, right: u8, borrow: bool) -> (u8, Self);
       fn and(a: u8, operand: u8) -> (u8, Self);
   }
   ```

   Implementation beads introduce only the subset needed at that step.
   [decision-19](<decision-19 - FlagRules-hooks-for-8085-V-K-effects.md>) adds four
   flag-effect hooks (INX/DCX, rotates, DAD, DAA) for the 8085's V/K flags.
   Adding a method beyond this set requires a decision amendment, with its
   citation and witness; it is not routine trait growth. The trait is capped
   to flag access, PSW image and ALU facts. It has no latch, interrupt, timing
   or bus members. This is an ALU fact table, not a CPU model trait.

4. **Model-owned bus facts.** A private `BranchFacts` holds
   `untaken_operand_reads: u8`, supplied by a per-model constant: 2 for the
   8080, 1 for the 8085. The constants retain their separate citations
   (98-153B Ch. 2 pp. 2-18/2-19, notes 12/13 p. 2-20;
   205775-002 Ch. 2 §2.3.1 p. 2-7 and Table 5-1 p. 5-19).
   The shared branch body consumes this value; it does not identify a model
   or choose state totals. Other facts lacking both-primary support likewise
   enter through per-model constants rather than expanding `FlagRules` to
   bus behaviour.

5. **Moves and stays.** Move into `intel`:

   - `View`, `FlagRules`, `BranchFacts`.
   - Operand and word fetch, location/operand reads and writes, pair reads
     and writes, condition evaluation, `read_address`, and `push_and_jump`.
     The last helper replaces both restart signatures and is also used by
     model-owned interrupt paths with explicit return PC and target.
   - All Data Transfer; Arithmetic including DAA; Logical, with AND through
     `FlagRules::and`; Branch, with untaken reads through `BranchFacts`;
     Stack/IO's PUSH/POP pairs and PSW, XTHL, SPHL, IN and OUT. PSW byte
     conversion goes through `FlagRules::to_byte`/`from_byte`.

   Keep in each model:

   - Public `Registers`, `Flags`, `Latches`, `Inputs` and CPU; reset, run loop,
     boundary sequence and interrupt acceptance. This includes the 8080
     acknowledge and the 8085 vectored/INTR paths and edge sampler.
   - `ENCODINGS`, state totals and per-row citations, `Group`, field
     restrictions, opcode-map assembly and independent map witnesses.
   - `alu.rs`, `flags.rs` and their `FlagRules` implementations, including
     the 8085 ALU unit tests against its independent reference.
   - EI, DI, HLT and NOP arms, RIM/SIM, and Unsupported handling.

6. **Two citation tiers.** Root sources, `decode.rs` and `progress.rs` stay
   fact-free and cite decisions only. A hardware fact may be a literal in
   `src/intel` only when **both models' pinned primaries state it**, each
   cited at that fact by document/revision, section/table and page:
   98-153B (September 1975) and 205775-002 (January 1983). Retain the second
   witnesses where the models have them, and each model's evidential status.
   A fact that differs, or that only one manual states, goes through
   `FlagRules` or a per-model constant. Equal code, second-witness agreement
   alone and a passing oracle cannot license a shared hardware literal.

   In particular, the equal PSW image remains model-owned. DAA carry
   retention remains derived for each model (8080 `arithmetic.rs:161`;
   8085 `arithmetic.rs:83`); shared DAA must consume model-owned rules or
   constants for that derivation rather than present it as stated by both
   primaries. Likewise XTHL's 8085 address order remains unverified and
   model-supplied. Moving a body does not upgrade its evidence. A needed
   flag/ALU hook beyond rule 3 requires amendment before implementation.

   `tests/boundary.rs` rejects the existing Intel manual identifiers and
   `205775`, `231718`, `UM008011`, `03-0029`, `03-0002` in the seven
   fact-free files. For every `src/intel/*.rs`, it requires `98-153B` and
   `205775-002`, rejects `UM008011` and `03-0029`, and rejects paths to
   `crate::i8080`, `crate::i8085`, `crate::z80`. With no family files yet,
   this test passes vacuously. These are literal file-level tripwires;
   review still checks both citations at each individual fact.

7. **Amendments and preserved boundaries.** This amends decision-10's
   "Not shared: group executors" for the executors listed in rule 5.
   Its rejection of a register-access model trait stands: the register view
   is concrete and `FlagRules` is limited to flag/ALU facts. This also amends
   decision-13 rule 7: its 8080/8085 no-dedup clause is closed after the M4
   closure measurement satisfied its revisit condition. The no-macro rule
   and "Z80 shares nothing beyond decode/progress/test support" stand.
   decision-6 is unchanged: concrete per-model public types, no family-wide
   root state/CPU types or model trait, monomorphic execution with no `dyn`
   or per-instruction model dispatch, and facts cited/witnessed per model.
   The embedder-owned run-time wrapper policy also stands.

## Alternatives

- **Shared public Registers/Flags, re-exported by both models.** Rejected:
  changes public type identity, conflicts with decision-6 and treats the
  8085 PSW image's decision-11 library choice as shared state. A later pinned
  source establishing extra 8085 flag bits would force another type split.
- **Generic executor over a register-access/model trait.** Rejected as in
  decision-10: it couples model state and execution through a model trait
  and leaves register-code mapping duplicated in each implementation. The
  concrete view shares that mapping without this trait.
- **Declarative macros generating model executors.** Rejected: an implicit
  trait in place of typed parameters, bodies outside ordinary formatting,
  poorer error locations and hardware citations buried in macro text.
  decision-13's no-macro boundary stands.
- **Share nothing further.** Rejected by the accepted design direction:
  the measured Intel overlap justifies the limited private tier, while
  separate public types and model-owned facts preserve the existing split.

## Consequences

- No public API or per-model hardware witness change. `tests/i8080_*.rs`, `tests/i8085_*.rs`,
  both Intel `opcodes/tests.rs` files, the independent arithmetic references
  and `lib.rs` doctests stay unchanged through extraction. No shared
  arithmetic reference replaces the two model witnesses.
- An 8085 executor change can now touch an 8080 path. Both models' witnesses
  and gates must guard each extraction; untaken-branch trace witnesses are
  especially important. The optional 8080PRE/8080EXM/8085EXM verdicts must
  remain unchanged where artifacts and execution are available.
- Shared comments require two citation sets and preserve source asymmetry;
  similarity reduces code but does not reduce provenance review.
- Implement access helpers, Data Transfer, Branch and Stack/IO before the
  flag-dependent Logical and Arithmetic extractions. **Checkpoint after
  Stack/IO:** re-measure interim LCS and oracle runtime against the recorded
  roughly 16-second 8080EXM baseline. The maintainer decides go/no-go for
  Logical/Arithmetic in light of undocumented-8085 research. View inlining
  and the eventual line-count reduction remain assumptions until measured.
- Undocumented-opcode admission and canonical-operation decode changes need
  their own policy/decision amendments. This record adds neither. Model-first
  wrappers leave room for those later model-owned executors without widening
  the shared contract speculatively.

## Revisit when

A pinned source makes a shared fact differ or invalidates its authority,
undocumented instructions need a flag/ALU hook beyond the closed method set,
the Stack/IO checkpoint counsels stopping, or re-measurement changes the
benefit or runtime assessment. Z80 sharing needs a new measurement and a
separate decision; this record does not admit it to the Intel tier.
