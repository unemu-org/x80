---
id: decision-19
title: FlagRules hooks for the 8085 V/K effects (amends decision-16 rule 3)
date: '2026-11-13'
status: accepted
---

Decider: maintainer (2026-11-13). Bead `x80-mn3.6`. Amends
[decision-16](<decision-16 - Intel-family-shared-executors-for-the-8080-and-8085.md>)
rule 3, which closes the `FlagRules` method set and requires an amendment for
any new method.

## Context

[decision-18](<decision-18 - 8085-undocumented-instructions-and-V-K-flags.md>)
§4 gives the 8085's V and K flags effects on documented instructions that the
shared Intel executors (decision-16) run: K on a 16-bit wrap of INX/DCX, V/K on
the rotates, V on DAD, and V/K on DAA. Those executors reach flags only through
`FlagRules`. The rule 3 method set has no place for these effects: INX/DCX,
the rotates and DAD change no flag through `result`, `add`, `subtract` or
`and`, and DAA's V/K depend on the combined correction.

## Choice

Allow four further `FlagRules` methods, flag/ALU facts only, each a hook that
the shared executor calls at the point the instruction's flag effect occurs:

- a 16-bit pair increment/decrement hook, given the old and new pair value and
  the direction (INX/DCX);
- a rotate hook, given the operation, the old and new accumulator and carry
  (RLC, RRC, RAL, RAR);
- a 16-bit add hook, given the operands and the result (DAD);
- a decimal-adjust hook, given the old accumulator, the selected correction and
  the result (DAA).

Exact names and signatures are the implementation's. The 8080 implementation of
each hook keeps the 8080's behaviour exactly as before this record: it changes
nothing it did not change already, so no 8080 witness changes. The 8085
implementation applies decision-18's §4 rows, with their S/D/L labels, in
`i8085/flags.rs` or `alu.rs`. The cap of rule 3 otherwise stands: no latch,
interrupt, timing or bus member, and any further method needs another
amendment.

## Alternatives

- Keep the trait closed and let the 8085 wrapper intercept INX/DCX, the rotates,
  DAD and DAA with 8085-only copies of those arms. Rejected by the maintainer:
  it brings back the duplication decision-16 removed.

## Consequences

The shared Arithmetic and Logical executors stay shared. The 8080 gains four
trivial hook implementations that preserve its behaviour.

## Revisit when

Another model-specific flag effect cannot be expressed through these hooks.
