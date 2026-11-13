---
id: decision-10
title: i8085 model layout and fact-free sharing
date: '2026-11-12'
status: accepted
---

Decider: maintainer (2026-11-12). Applies decision-6 to the second model (M3, bead `x80-4gk.3`);
answers [plan](../docs/plan.md) §6 Q9 for M3.

## Context

decision-6 makes each CPU model a concrete type in its own module and moves code into a
shared module at its second use, with the rule that shared modules carry no hardware
fact. The 8085 is the second model. Its pinned sources (PROVENANCE.md §1) describe the
same five instruction groups as the 8080 (205775-002 Ch. 5), with differences in state
totals, the untaken-branch bus trace, AC after AND, the PSW image, RIM/SIM and the whole
interrupt structure (TRAP, RST 5.5/6.5/7.5, INTR; SID/SOD). The maintainer chose
(2026-11-12) to share only fact-free machinery.

## Choice

- The 8085 is a new model, not a variant of the 8080: its inputs (five interrupt lines,
  SID/SOD) and execution latches differ (decision-6's test for a model). One model covers
  the 8085A and 8085AH (PROVENANCE.md §1 records that as an assumption).
- Layout: `i8085::{I8085, Registers, Flags, Latches, Inputs}` beside `i8080`, with its own
  executor module per instruction group, its own ALU and flag rules, and modules for
  RIM/SIM and the two interrupt paths. Every fact is cited from the 8085 sources and
  witnessed for this model, even where it matches the 8080.
- Shared, fact-free machinery moves out of `i8080` into private crate-level modules: the
  opcode-map builder (encoding patterns, lengths, state-total slots, path selection) made
  generic over the model's group type, and the run accounting (step and cycle counters,
  saturation, budget check). Shared test support (the recording bus) is extended, not
  forked. Shared modules cite decisions only, never a manual (checked by
  `tests/boundary.rs`). The shared matcher carries no field rule: restrictions such as
  "110 is not a register code in DDD or SSS" (98-153B p. 4-3) are model-owned data passed
  to it from each model's encoding list.
- Not shared: group executors, ALU, flag rules, register and latch types, reset and
  interrupt logic. A private generic over a register-access trait was rejected: it is the
  model trait decision-6 rejected.
- Run-time wrapper (plan §6 Q9): none in the crate in M3. The two models have different
  `Inputs` traits and state types, so a wrapper could only forward `run`/`step`. The oracle
  harness keeps its own small enum, which documents the pattern for embedders.

## Alternatives

- Share the executors through a generic over a register-access trait. Rejected by the
  maintainer: a model trait shaped by two models, coupling 8080 code to the 8085.
- Share nothing. Rejected: duplicating fact-free machinery adds risk without benefit.
- A crate run-time wrapper now. Deferred: it would forward only `run`/`step`.

## Consequences

- The extraction (bead `x80-4gk.7`) changes no public API and no 8080 witness.
- Near-duplicate executor code per model; the duplication is measured at the M3 closure
  (`x80-4gk.22`) before the Z80 (M4) is planned.

## Revisit when

The M4 layout and sharing revisit is recorded in [decision-13](<decision-13 - z80-model-layout-prefix-tables-and-sharing.md>) (accepted 2026-11-13).

M4 starts, an embedder asks for a run-time wrapper, or the duplication measured after M3
argues for a different split.
