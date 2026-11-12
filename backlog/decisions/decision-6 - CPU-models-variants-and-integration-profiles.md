---
id: decision-6
title: CPU models, variants and integration profiles
date: '2026-11-12'
status: accepted
---

Decider: maintainer. Settles [plan](../docs/plan.md) §6 Q9 except the run-time
wrapper; design in §2.6.

## Context

The 8080 family spans the Intel 8080 and 8085, the Zilog Z80 and its
derivatives, second-source and clone parts, and the 8080 emulation mode of the
NEC V20/V30. Plan §1 commits to the 8080 (M1), the 8085 (M3) and the Z80 (M4).
The models are expected to differ in register state, input lines, flag rules,
opcode-map structure and timing; this is unverified until their sources are
pinned at M3 and M4. One example from a pinned source: 9800301-04 Ch. 3 prints 8085 state counts
beside the 8080 ones, and some instructions have a single count on one part and
two on the other (p. 3-28, JNC: "10 (7 or 10 on 8085)"; the 8085 still needs
its own pinned primary source at M3).
The first API must leave room for later models without inventing their shape
before their sources are pinned (AGENTS.md: no speculative framework).

## Choice

- A model is one CPU type as described by its pinned primary source. Each model
  is a concrete Rust type in its own module: `i8080::I8080` in M1, `i8085` at M3
  and `z80` at M4.
- A model owns its register file, execution latches, opcode map, flag rules,
  timing tables, input lines and reset. There are no family-wide register,
  state or CPU types at the crate root.
- The model-neutral core is the bus trait, the budget, the report, the exit
  skeleton and the host stop. It carries no hardware fact.
- There is no family-wide model enum and no model trait. An embedder that picks
  the CPU at run time wraps the model types it needs and selects once per run
  call. Whether the crate offers such a wrapper is decided at M3.
- A variant is a part with the same state shape, inputs and opcode-map
  structure as its model that differs in a documented table entry or rule. It
  is a `#[non_exhaustive]` value fixed at construction. It exists only once a
  pinned primary source documents an observable difference, with a citation and
  a witness. M1 has no variant.
- A new model is needed when the state shape, the inputs or the opcode-map
  structure differ.
- Code moves into a shared module at its second use, never in advance. A fact
  common to two models is cited and witnessed for each.
- Each model has its own unsupported-encoding policy record; decision-3 is the
  8080's.
- An integration profile (V20/V30, plan §3) is a separate axis. It defines where
  control returns to the embedder, not what an instruction does. Whether the
  NEC emulation mode is the 8080 model, a variant or a model of its own is
  decided from the pinned NEC manual at M2 (`x80-pzp`).

## Alternatives

- A run-time model enum on one CPU type, with a superset state. Rejected: one
  state shape and one input type across models, model checks on the
  instruction path, and a one-variant enum in M1 that is a framework before its
  milestone.
- A generic CPU over a compile-time model trait. Rejected: the trait's shape
  would be guessed before any 8085 or Z80 source is pinned.

## Consequences

- Each model is monomorphic; there is no per-instruction model dispatch. A
  variant costs a field read in the instructions it affects.
- Until M3, an embedder that wants run-time selection writes its own wrapper.
- Adding the 8085 or the Z80 adds a module and extracts shared code; it does
  not change the 8080's types or witnesses.
- Parts outside plan §1 need a maintainer scope decision and a pinned primary
  source first (plan §6 Q10).

## Revisit when

M3 starts (the wrapper and the first shared extraction), the M2 contract decides
how the NEC emulation mode is modelled, or a part enters scope that does not fit
the model/variant rule.
