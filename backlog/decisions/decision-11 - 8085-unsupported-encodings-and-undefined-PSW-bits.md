---
id: decision-11
title: 8085 unsupported encodings and undefined PSW bits
date: '2026-11-12'
status: proposed
---

Decider: maintainer. The 8085 counterpart of decision-3 (each model has its own
unsupported-encoding record, decision-6); bead `x80-4gk.4`.

## Context

- 205775-002 Table 5-2, "8085A CPU instructions in operation code sequence" (p. 5-20,
  checked on the page image), prints a dash at 08, 10, 18, 28, 38, CB, D9, DD, ED and FD.
  20 is RIM and 30 is SIM, the two instructions the 8085 adds; the 8085AH data sheet lists
  them as "New 8085AH instructions" (handbook p. 16-30).
- The same table prints "RET Adr" at C9 and "JZ" without "Adr" at CA, the operand-marker
  slip already recorded for 9800301-04 p. A-5; the per-instruction pages give RET one byte
  and JZ three.
- 205775-002 p. 5-15, PUSH PSW: the flag word has S Z X AC X P X CY, with bits 5, 3 and 1
  marked "X: undefined". The 8080 sources fix them at 0, 0, 1 (98-153B p. 4-13). POP PSW
  (p. 5-16) loads only bits 7, 6, 4, 2 and 0.

## Choice

- A byte that the pinned 8085 sources do not document as an instruction returns a typed
  `Unsupported` exit with no architectural state change, as decision-3 for the 8080. In
  code the undocumented set is the complement of the documented one, never a list.
- Behaviour for such a byte is added only when a pinned primary source documents it.
- Where Table 5-2's operand markers disagree with the per-instruction pages, the
  per-instruction pages win and the witness pins the bytes, as for the 8080.
- PUSH PSW on the 8085 stores bits 5, 3 and 1 as 0, 0, 1, the 8080 image. This is a
  library choice, not a hardware claim: any value conforms to "undefined". Witnesses
  compare the 8085 flag byte under mask D5h, except one test that pins the library choice
  and says so.

## Alternatives

- Store zeros in the undefined bits. Not proposed: no more correct than the 8080 image,
  and it would make 8080-based programs see a different byte for no documented reason.
- Keep eight bits of flag storage. Rejected: the CPU's flag register holds five flags
  (205775-002 Ch. 2), and POP PSW loads only five bits.

## Consequences

- Software that relies on undocumented 8085 encodings stops with a typed exit.
- An exerciser comparing the full flag byte under mask D5h is unaffected by the choice.

## Revisit when

A pinned primary source documents more 8085 encodings or fixes the undefined bits.
