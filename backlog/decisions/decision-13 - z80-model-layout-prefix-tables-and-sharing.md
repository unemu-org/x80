---
id: decision-13
title: z80 model layout, prefix tables and sharing
date: '2026-11-13'
status: accepted
---

Decider: maintainer (2026-11-13). Applies decision-6 to M4 and answers decision-10's
M4 revisit; bead `x80-bi2.2`. The maintainer's scope and sharing choices below
were given on 2026-11-13; the maintainer accepted this record the same day.

Rule 7 amended by [decision-16](<decision-16 - Intel-family-shared-executors-for-the-8080-and-8085.md>)
closes the Intel no-dedup clause after M4; the no-macro rule and Z80 sharing boundary stand.

Rules 3–6 superseded in part by [decision-20](<decision-20 - Z80-undocumented-behaviour-Zilog-NMOS.md>) §7:
chained/ignored prefixes, whole-instruction cost width and the Zilog NMOS target.

## Context

decision-6 assigns state, inputs, encodings, timing and execution to concrete
model modules. decision-10 extracted the fact-free `decode` and `progress`
machinery at its second use, while keeping the 8080 and 8085 executors separate.
The Z80 is the third model. Its register configuration and interrupt flip-flops
are model-owned state (library choice: decision-6; stated source descriptions:
Zilog Z80 CPU User Manual UM008011-0816, August 2016, "CPU Register", pp. 2–3,
and Table 1, p. 18).

M4 implements documented Z80 behaviour only. decision-14 owns the documented
encoding boundary, the `Unsupported` payload and undefined flag-bit choices;
decision-15 owns the API and boundary behaviour. This record fixes layout and
sharing, not their contracts. The primary and second-witness pins and their
shared documentation lineage are in [PROVENANCE.md §1](../../PROVENANCE.md#1-authority-and-citations).
Below, **stated** identifies a source statement, **derived** a conclusion from
the cited pages or code, and **unverified** an assumption. **Library choice**
identifies a software design rule, not a hardware claim. UM page numbers are
printed pages; PDF = printed + 14. For 03-0029-01 (1977), PDF = printed + 6.

## Choice

1. **Model and public names — library choice (decision-6).** Add
   `z80::{Z80, Registers, Flags, Latches, Inputs}` beside `i8080` and `i8085`.
   `Z80` owns the model's registers and latches. These names do not fix field
   shapes, method signatures or reset values: decision-15 and the core-types
   bead fix those, with decision-14 governing the flag image. There is no
   family-wide CPU, state, register or input type or model trait.

2. **Instruction-group modules — library choice from stated headings.** Use
   one executor/encoding module for each of the eleven instruction-description
   sections below, with names drawn from UM008011-0816's own headings. Each
   module owns its encoding lists, lengths, costs, executor and source citations;
   the model owns its ALU and flag rules. The headings and starting pages are
   stated on the rendered pages listed here. Inclusive ranges are derived from
   the next section's start; the final section ends on p. 316. These are the
   detailed instruction sections, not the earlier opcode-summary headings.

   | Module under `z80/` | Printed instruction-description heading | Printed pages (PDF pages) |
   | --- | --- | --- |
   | `load8.rs` | 8-Bit Load Group | 70–97 (84–111) |
   | `load16.rs` | 16-Bit Load Group | 98–122 (112–136) |
   | `exchange_block_search.rs` | Exchange, Block Transfer, and Search Group | 123–143 (137–157) |
   | `arithmetic8.rs` | 8-Bit Arithmetic Group | 144–171 (158–185) |
   | `general_arithmetic_cpu_control.rs` | General-Purpose Arithmetic and CPU Control Groups | 172–186 (186–200) |
   | `arithmetic16.rs` | 16-Bit Arithmetic Group | 187–203 (201–217) |
   | `rotate_shift.rs` | Rotate and Shift Group | 204–241 (218–255) |
   | `bit_set_reset_test.rs` | Bit Set, Reset, and Test Group | 242–260 (256–274) |
   | `jump.rs` | Jump Group | 261–279 (275–293) |
   | `call_return.rs` | Call and Return Group | 280–293 (294–307) |
   | `input_output.rs` | Input and Output Group | 294–316 (308–330) |

   In particular, p. 172 combines general-purpose arithmetic and CPU control
   into one section; splitting its plural heading would create a twelfth
   module. The earlier second witness has eleven tables too (stated:
   03-0029-01 §7, p. 43, Tables 7.0-1–7.0-11, pp. 44–54), although its heading
   wording is not always the same as the primary's.

3. **Five opcode tables — library choice, using `decode::build` unchanged.**
   `z80/opcodes.rs` assembles five private 256-entry arrays, generic over the
   Z80's own group enum. Prefix selection and the selected IX/IY register are
   model-owned context, not shared matcher data.

   | Table | Byte used as the array index | Context retained by `z80/cpu.rs` |
   | --- | --- | --- |
   | Main | First instruction byte | Instruction start |
   | CB | Byte after CB | CB selected |
   | ED | Byte after ED | ED selected |
   | DD/FD index | Byte after DD or FD, except the CB dispatch case | IX for DD, IY for FD |
   | DD CB/FD CB | Fourth byte in `DD CB d op` or `FD CB d op` | IX/IY selection and displacement `d` |

   The byte layouts are stated in UM008011-0816 Table 6, p. 42 (main, ED and
   corresponding DD/FD load forms), and "RL m", p. 221 (CB and both indexed
   CB forms); second witnesses: 03-0029-01 Tables 7.0-1 and 7.0-7, pp. 44, 50.
   Five arrays, including one common index array and one common indexed-CB
   array, are a derived software organisation of those encodings, not five
   hardware tables asserted by the manual.

   **READ, current API:** `decode::build<G: Copy>` accepts
   `&[(G, &[Encoding])]` and `&[FieldRestriction]`, returning
   `[Option<Instruction<G>>; 256]`. `Encoding::new` takes an eight-bit pattern,
   a `u8` length and `States`; the builder matches each candidate byte and
   copies group, length and costs. It does not fetch a byte, know its stream
   position or interpret its length. Thus a caller indexing by the byte after
   a prefix, or by byte four after saving byte three as `d`, fits the API
   exactly. No fact-free addition is needed. Patterns describe the selected
   key byte; model-supplied lengths and costs describe the entire instruction,
   including its prefixes and displacement where present. Table-specific
   lists and restrictions keep patterns disjoint; `build` already rejects
   overlaps and leaves unmatched keys `None`.

4. **Prefix dispatch — library choice.** `z80/cpu.rs` owns prefix reads,
   table selection, the instruction-start address, IX/IY and displacement
   context, group dispatch and retirement. It recognises prefix dispatch before
   treating a table's `None` as a refused encoding; prefixes need no fake
   instruction entries. A group executor receives the selected table entry,
   key byte and necessary Z80 context. A prefixed instruction retires once,
   with its complete cost, not once per prefix. The layouts are those cited in
   rule 3; access kinds, R updates and interrupt/boundary semantics belong to
   decision-15. Unsupported prefix sequences follow decision-14, without a
   fallback to the main table or a general prefix-chaining executor.

5. **Two `States` slots and `u8` costs suffice — derived; reuse `decode` and
   `progress` unchanged.** The largest documented instruction total found is
   **23 T-states**: EX (SP),IX and EX (SP),IY state 23 (stated:
   UM008011-0816 pp. 128–129; 03-0029-01 Table 7.0-3, p. 46). Indexed rotates
   also reach 23 (stated: UM "RL m", p. 222; 03-0029-01 Table 7.0-7, p. 50).
   The whole-set second-witness timing tables, 03-0029-01 Tables 7.0-1–7.0-11,
   pp. 44–54, contain no larger total and no encoding needing more than two
   path totals. This conclusion is for the documented, no-wait-state M4 scope.

   A mnemonic can have more than two totals: "RL m" explicitly lists 8 for
   registers, 15 for (HL), and 23 for (IX+d)/(IY+d) (stated: UM pp. 221–222;
   second witness p. 50). These are different encodings in different entries
   or tables, each with one cost; they are not three paths of one entry.
   Register and memory patterns with different costs must therefore be split
   in the model's encoding lists, with model-owned field exclusions where
   needed. `States::single` fills both slots for a fixed-cost entry.

   For an entry with two paths, use the existing `Path::Untaken`/`Path::Taken`
   selectors for the model-supplied costs. Examples are DJNZ's 8/13 (stated:
   UM p. 278; second witness Table 7.0-9, p. 52) and RET cc's 5/11 (stated:
   UM p. 287; second witness Table 7.0-10, p. 53). For a repeating block entry,
   use Untaken for termination and Taken for repetition: LDIR states 16/21
   per transfer (stated: UM p. 132; second witness Table 7.0-3, p. 46). decision-15
   owns the one-iteration-per-step contract. This keeps an unbounded block
   operation out of one `u8` cost. `progress::Progress::retire(u8)` already
   counts one completed step and adds the supplied cost to its saturating
   `u64` cycle counter; neither shared module needs a Z80 rule or a new slot.
   Per-encoding timing witnesses remain required during implementation.

6. **One model for NMOS and CMOS parts — unverified assumption, library
   scope choice; no variant.** Stated: UM008011-0816's Architectural Overview
   calls this the Z80 CPU family and gives 6–20 MHz offerings (p. 1). Its
   power-down description discusses stopping and restarting the clock under
   the name Z80 CPU (pp. 15–16). Those pages do not state instruction-level NMOS/CMOS
   equivalence or a Z80/Z80A distinction; no such equivalence was established
   by this reading. Stated: the historical witness explicitly names Z80A at
   4 MHz (03-0029-01 §1, p. 1), which is not evidence of CMOS equivalence either.
   The single model implements the documented scope; it does not claim all
   silicon revisions behave identically. A variant needs a pinned source for
   an observable difference and a witness, as decision-6 requires.

7. **Sharing and closure measurement — library choice, maintainer direction
   (2026-11-13).** Share the existing fact-free `decode`, `progress` and test
   support; extend the recording bus additively for the contracts in
   decisions 14–15. The model supplies all patterns, restrictions, lengths,
   costs and execution paths. Shared modules continue to cite decisions,
   never hardware manuals, under the existing boundary test.

   Do not share Z80 executors, ALU/flag semantics, register/latch types,
   interrupts or reset with `i8080` or `i8085`. Do not deduplicate the existing
   8080/8085 executors before or during M4. There is no register-access trait
   and no macro standing in for one. Matching hardware facts still need
   separate model citations and independent witnesses. decision-10's
   embedder-owned run-time wrapper policy stands.

   **MEASURED, bead x80-4gk.22 notes**, also recorded on `x80-bi2`
   (quoted from those bead notes by the supervisor for this record): the M3 closure
   worker compared the parent tree of `fa58066` on 2026-11-13
   using the longest common subsequence (LCS) of comment-stripped source
   lines between `x80-core/src/i8080/` and `x80-core/src/i8085/`, then repeated
   it after normalising `I8080`/`I8085` and their module paths. Group executor
   modules had 570/556 lines, with 488 exact and 498 type-normalised matching
   lines. The `execute` bodies had 284 matching lines out of 319/310. Private
   operand-fetch, register/M and pair-access, and condition-evaluation helpers
   were fully identical; restart matched on 7 of 9 lines. These are recorded
   measurements, not measurements rerun for this decision.

   The supplied closure note identifies model-owned facts in state totals,
   untaken conditional jump/call operand reads, AND auxiliary carry, undefined PSW
   bits, EI/DI inhibition, interrupt sources and RIM/SIM (MEASURED, bead
   x80-4gk.22 notes; design context: decisions 10–12). The notes are bead
   evidence, not a checked-in source document. Similarity supports revisiting
   helper sharing later; it does not override decision-10's rejection of a
   register-access model trait or the maintainer's M4 sharing boundary.
   At M4 closure (`x80-bi2.27`), repeat the same comment-stripped, exact and
   type-normalised LCS method, extended to `z80`, reporting per-group executor
   and private-helper counts and the facts that must remain model-owned.
   Keep the i8080/i8085 baseline comparable; identify Z80 groups/helpers with
   no counterpart explicitly. Record the results on `x80-bi2` before deciding
   whether any later helper extraction is justified.

## Alternatives

- A family-wide CPU or a generic executor over register access, including a
  macro with the same role. Rejected by the maintainer and decisions 6/10:
  model state and execution facts remain coupled through that abstraction.
- Separate DD and FD arrays, and separate DD CB and FD CB arrays. Not proposed:
  the model can retain IX/IY selection while reusing the corresponding key-byte
  map (rule 3); this is internal Z80 layout, not cross-model executor sharing.
- Add prefix parsing or multi-byte matching to `decode`. Not proposed: its
  current byte-indexed API already fits all five tables; prefix meaning stays
  in the model.
- One cost pair per mnemonic, or a third cost slot. Not proposed: different
  encodings need separate entries, while the documented paths of an individual
  entry fit the existing two slots (rule 5).
- Deduplicate i8080/i8085 helpers now, or fork the fact-free machinery for the
  Z80. Rejected: the former contradicts the maintainer's M4 direction; the
  latter duplicates machinery whose current API already suffices.

## Consequences

- The Z80 adds a concrete model and its own facts and witnesses. Group-module
  boundaries follow the primary's eleven detailed sections; opcode-table
  boundaries follow prefix context, so a group may contribute to several tables.
- No change to `decode.rs` or `progress.rs` is planned. Root bus/exit additions
  and test-support extensions are governed by decisions 14–15 and retain the
  existing models' witness expectations.
- NMOS/CMOS equivalence remains an explicit assumption. The two-slot conclusion
  is a documented-scope derivation, to be checked by independent encoding and
  timing witnesses, not a substitute for them.
- The cited byte layouts and timing totals agree with the second-witness tables
  checked here; no disagreement on those facts requires a PROVENANCE §1 row.
  Labelled API readings are recorded by decision-15's owner.
- Executor duplication remains deliberate through M4. Closure measures it;
  measurement alone does not authorise a sharing refactor.

## Revisit when

M4 closes with the rule 7 measurements, a pinned primary source documents an
observable part difference, a documented encoding does not fit the table/cost
layout, or an embedder requests a crate run-time wrapper. Any broader encoding
scope first needs the maintainer's decision under decision-14.
