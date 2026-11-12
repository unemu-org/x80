---
id: decision-7
title: Interim auxiliary carry after 8080 AND
date: '2026-11-12'
status: accepted
---

Decider: maintainer (2026-11-12). Interim reading for the open disagreement in
[PROVENANCE.md](../../PROVENANCE.md) §1; part (a) of bead `x80-ecy.2`. Needed by
the logical-group bead `x80-ecy.10`.

## Context

The two pinned Intel sources disagree on the auxiliary carry (AC) after the 8080
logical AND instructions. Checked against the page images:

- 98-153B p. 4-8, "Logical Group": "Unless indicated otherwise, all
  instructions in this group affect the Zero, Sign, Parity, Auxiliary Carry, and
  Carry flags according to the standard rules." ANA r and ANA M say only "The
  CY flag is cleared"; their flag lists include AC.
- 98-153B p. 4-9: ANI says "The CY and AC flags are cleared."
- 9800301-04 p. 1-12: "The 8080 logical AND instructions set the flag to reflect
  the logical OR of bit 3 of the values involved in the AND operation", in a
  passage that contrasts this with the 8085, whose AND instructions always set
  AC.

98-153B is not uniform between ANA and ANI. Its condition-flag definition
(p. 4-3) describes AC as a carry out of bit 3 and lists logical operations among
those that affect it, without saying what that carry is for a logical AND. 9800301-04 is later and states a rule for
all 8080 AND instructions explicitly. Both are Intel documents, so agreement or
disagreement between them is not a hardware witness. M1 cannot wait for an
oracle: an exerciser needs nearly the whole instruction set, including AND.

## Choice

- Until `x80-ecy.2` closes, ANA r, ANA M and ANI set AC to bit 3 of
  (A OR operand), per 9800301-04 p. 1-12. CY is cleared, as both sources state
  (98-153B pp. 4-8 and 4-9; 9800301-04 pp. 3-6 and 3-7: "the carry flag is
  reset to zero").
- The rule lives in one named function in the `i8080` module, cites both
  sources and is marked unverified.
- Its witnesses are labelled provisional and reference `x80-ecy.2`.
- PROVENANCE.md records the interim reading as unverified.
- `x80-ecy.2` (b) closes the question: run the licensed oracle from `x80-ecy.17`
  under both readings, or find a further primary source. A falsified reading is
  replaced in a superseding record.

## Alternatives

- 98-153B's ANI wording (AC cleared) for all AND instructions. Rejected:
  the earlier manual is not uniform, and the later one states the 8080 rule
  explicitly while contrasting it with the 8085.
- Hold the AND instructions until `x80-ecy.2` closes. Rejected: the oracle run
  needs AND, so this creates a cycle.

## Consequences

- Software that depends on AC after AND may see a difference if the oracle
  falsifies this reading; the change is confined to one function and its
  witnesses.

## Revisit when

`x80-ecy.2` produces oracle evidence or a further primary source, such as a
pinned later edition of the 8080 user's manual (PROVENANCE.md discovery
inventory).
