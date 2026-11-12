# Status

**M0: done.** Tracking, the pinned shell, hooks, local gates and the first commit
are in place. Decisions 1-5 settle the M1 API, decision-6 the CPU model layout and
decision-7 an interim flag reading. The 8080 primary sources are pinned (see [PROVENANCE.md](../../PROVENANCE.md)).

**M1: in progress.** `x80-core` has the types of decision-5: the
model-neutral `Bus`, `MemoryAccess`, `Control`, `Budget`, `Report`, `Exit` and
`Unsupported` at the crate root, and the `i8080` model with `Registers`, `Flags`,
`Latches`, `Inputs` and `I8080`. The BC, DE and HL pair and PSW accessors and the
PSW flag-byte layout are cited and witnessed.

`I8080::run` and `I8080::step` implement the decision-2 budget and the
decision-5 boundary sequence. The opcode map holds the 244 opcodes that 98-153B
Ch. 4 documents, with their lengths and groups, witnessed against the summaries
on 98-153B p. 4-15 and 9800301-04 p. A-5. The other 12 (08, 10, 18, 20, 28, 30,
38, CB, D9, DD, ED, FD) return `Unsupported` (decision-3).

Implemented instruction groups (decision-5 item 7), each with cited state
totals and bus traces:

- Data Transfer: all (98-153B pp. 4-4, 4-5).
- Arithmetic: all except DAA (pp. 4-6 to 4-8); DAA's state total is cited,
  its execution is `x80-ecy.9`.
- Branch: all (pp. 4-11, 4-12).
- Stack, I/O and Machine Control: PUSH, POP, XTHL, SPHL, IN, OUT and NOP
  (pp. 4-13, 4-14); EI, DI and HLT are `x80-ecy.13`.
- Logical: none yet (`x80-ecy.10`).

Documented opcodes outside these return `Unsupported` until their bead lands.

Interim gaps:

- Interrupts are polled at every boundary but never accepted until
  `x80-ecy.14`; a halted CPU with an eligible interrupt still returns `Halt`.
- The EI delay latch is not advanced by executed instructions yet.
- There is no `reset` yet (`x80-ecy.13`).

Two Intel 8080 manuals are pinned as fetch-only sources in the shared `oracles/`
submodule (pin ddef3b6). No ROMs or oracle corpora are pinned yet.
M2–M4 have not started. See [plan §5](plan.md#5-milestones).

This is a capability snapshot, not the task board. Consult beads for claims and
priorities. Update this file when a milestone advances.
