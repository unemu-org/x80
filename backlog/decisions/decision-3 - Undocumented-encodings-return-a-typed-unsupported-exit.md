---
id: decision-3
title: Undocumented encodings return a typed unsupported exit
date: '2026-11-11'
status: accepted
---

Decider: maintainer. Settles the undocumented-encoding half of [plan](../docs/plan.md)
§6 Q7 for the 8080; the oracle-licence half stays open.
decision-5 fixes the exit contents and, for M1 only, extends the exit to
documented opcodes not yet implemented; decision-6 makes this the 8080's policy.

## Context

[PROVENANCE.md](../../PROVENANCE.md) allows behaviour only from a cited primary
source pinned with a witness. Where the pinned primary source does not document
an opcode byte as an instruction, x80 has no licensed behaviour for it.
Exerciser programs and hardware captures can show what a part does, but they
cannot license a fact, and other emulators' implementations are never a source.

## Choice

- An opcode that the pinned primary source does not document as an instruction
  returns a typed `Unsupported` exit. The API bead fixes the exit's contents.
- The exit changes no architectural state: registers, flags, PC, stack and
  memory stay as they were.
- Behaviour for such an encoding is added only when a primary source documents
  it, with a citation and a witness test.
- Silicon variants and second-source parts are out of scope until a model needs
  them. Supporting one needs its own decision and sources.

## Alternatives

- Execute commonly reported aliases. Rejected: no primary source licenses them.
- An embedder-selected profile that runs aliases. Rejected for now: it adds API
  surface for behaviour that has no source.

## Consequences

- Software that relies on undocumented encodings stops with a typed exit. The
  embedder can decide what to do outside the core.
- The opcode table and its witnesses state which encodings are documented and
  cite the source for each.

## Revisit when

A pinned primary source documents more encodings, or M3 (8085) or M4 (Z80)
starts: each model needs its own policy record or an explicit extension of this
one.
