---
id: decision-9
title: NEC V20/V30 8080 emulation mode is the i8080 model with a V20 profile
date: '2026-11-12'
status: accepted
---

Decider: maintainer (2026-11-12). Settles decision-6's open question on the NEC
emulation mode; bead `x80-pzp.2`. Design in [plan](../docs/plan.md) §3.

Amended 2026-11-14: IE and BRK after BRKEM and CALLN (see the amendment below).

## Context

decision-6 left open whether the NEC V20/V30 8080 emulation mode is the 8080 model, a
variant of it, or a model of its own, to be decided from the pinned NEC manual at M2.
The manual is pinned (PROVENANCE.md §1, "NEC V20/V30 8080 emulation mode"):

- p. 8-1, Figs 8-1 and 8-2: the emulation mode has the 8080 register set and flag byte,
  mapped onto native registers.
- p. 8-1: it executes "the instruction set for the uPD8080AF and the RETEM and CALLN
  instructions"; the manual lists no 8080 instruction, flag rule or clock count, so it
  documents no behaviour difference from the Intel 8080 that x80 implements.
- Every difference it documents is a control transfer between the two modes: RETEM and
  CALLN (ED FD, ED ED imm8; pp. 12-176, 12-177), interrupts serviced natively (p. 8-4),
  HLT standby and wake-up (pp. 8-2, 9-1), RESET, and the segment used per access (p. 8-1).

decision-23 (x86jit) splits the mode between the two cores, and the maintainer decided
that the embedder owns the adapter (bead `x80-pzp`).

## Choice

- The emulation mode is the `i8080` model plus a V20 integration profile. No variant is
  added: decision-6 allows one only for a documented difference in a table entry or rule,
  and the manual documents none.
- In M2 the profile is this record and its witnesses, not a type in code. M1 behaviour
  already meets it.
- Profile rules:
  - Opcode EDh stays `Unsupported` under the V20, even if a later milestone implemented
    undocumented 8080 encodings. The adapter reads the following bytes through its bus and
    performs RETEM (ED FD) or CALLN (ED ED imm8) natively.
  - The adapter never asserts INT into x80. Requests and acceptance are distinct: NMI is
    always accepted; INT is accepted when it is high at an instruction boundary with
    IE = 1 and no simultaneous NMI or hold request (p. 6-2; pp. 6-1, 8-4). When the native
    side would accept one, the adapter requests a host stop, copies state out at the clean
    boundary and services it in native mode; a masked INT is not serviced (whether the
    native EI delay applies to the 8080 EI in emulation is unconfirmed).
  - On x80's `Halt` exit the adapter owns the V20 standby: an INT with interrupts disabled
    resumes the 8080 program after the HLT without service (p. 9-1), by clearing
    `latches.halted` and running again.
  - Memory accesses map by kind: `OpcodeFetch` and `OperandFetch` to PS, `Data` and `Stack`
    to DS0 (p. 8-1). Register state is copied in and out by value at BRKEM, RETEM, CALLN
    and interrupt stops.
- Timing: under the V20, x80 reports Intel 98-153B T-states, labelled as not V20 clocks.
  The manual gives V20 clocks only for BRKEM, RETEM, CALLN, RETI and the interrupt
  sequences; no table is invented.
- ASSUMED: the uPD8080AF behaves as the Intel 8080A, including decision-8 and the derived
  DAA carry rule. It is listed as a residual in PROVENANCE.md.
- x80 owns the contract vectors and a mock native side (bead `x80-pzp.5`); the embedder
  owns the adapter; x86jit's C4 (`xj-50w.7` (x86jit)) owns the native side.

## Alternatives

- An `i8080` variant for the NEC part. Rejected: no documented instruction-level
  difference and no clock table.
- A model of its own. Rejected: the state shape, inputs and opcode-map structure are the
  8080's (decision-6's test for a new model).
- A profile type in code now. Not needed: no x80 behaviour differs yet; revisit if a rule
  must change x80's own behaviour.

## Consequences

- Plan §3 is rewritten from the manual (bead `x80-pzp.3`); its "memory through DS" becomes
  PS for instruction bytes and DS0 for data and stack.
- Witnesses for the profile rules use the public API (bead `x80-pzp.4`).
- A joint clock that mixes Intel T-states with V20 clocks is the embedder's to reconcile.

## Revisit when

A uPD8080AF source or hardware evidence shows a flag, encoding or timing difference (then
a variant under decision-6), or C4 needs x80 itself to behave differently under the V20.

## Amendment, 2026-11-14: IE and BRK after BRKEM and CALLN

Maintainer decision of 2026-11-14 during review of the C4 exit-contract draft
(`xj-50w.7` (x86jit)), recorded on both projects; bead `x80-pzp.8`. The ruling is **ASSUMED**, not read.

The pinned manual does not settle whether BRKEM and CALLN change IE and BRK:

- p. 6-1 (PDF 26): the general interrupt-routine entry pushes PSW, PS, PC, then
  "IE <- 0, BRK <- 0, MD <- 1"; Table 6-1 and Fig. 6-1 list BRKEM and CALLN among the
  sources using vectors 32-255.
- p. 12-169 (PDF 214, BRKEM) and p. 12-176 (PDF 221, CALLN) list only MD, in both the
  operation list and the flag table (MD <- 0 and MD <- 1 respectively).
- Table 12-4, p. 12-3 (PDF 48): a blank flag-table entry means "No change".
- p. 12-167 (PDF 212): BRK imm8, by contrast, lists IE <- 0 and BRK <- 0 explicitly.
- The mode-shift prose also names only MD: BRKEM "resets the MD flag to 0" (p. 8-2,
  PDF 33); CALLN "sets the mode flag to 1" (p. 8-4, PDF 35).
- No 8080-mode hardware corpus is pinned or known.

Readings considered:

- R1: the instruction page governs; BRKEM and CALLN both preserve IE and BRK.
- R2: p. 6-1 governs; BRKEM and CALLN both clear IE and BRK.

Choice: **R2** for both instructions, labelled ASSUMED. MD still follows each
instruction page. INT and NMI entry from emulation clearing IE and BRK remains read:
p. 8-4 says their interrupt process "is performed the same as in native mode", and
p. 6-1 gives that sequence. Only the two instructions are assumed. The contract fixture
selects R2 through one ASSUMPTION constant, and PROVENANCE.md §1 keeps the question on
its to-confirm list. Revisit when an NEC source or hardware run settles either
instruction. The rest of this record is unchanged.
