---
id: decision-9
title: NEC V20/V30 8080 emulation mode is the i8080 model with a V20 profile
date: '2026-11-12'
status: accepted
---

Decider: maintainer (2026-11-12). Settles decision-6's open question on the NEC
emulation mode; bead `x80-pzp.2`. Design in [plan](../docs/plan.md) §3.

Amended 2026-11-14: IE and BRK after BRKEM and CALLN (see the amendment below).
Amended 2026-11-15: option B, the view exchange and the adapter rules (see the
amendment below).

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
    performs RETEM (ED FD) or CALLN (ED ED imm8) natively. *(This second sentence is
    superseded by the 2026-11-15 amendment, option B: the native side decodes them.)*
  - The adapter never asserts INT into x80. Requests and acceptance are distinct: NMI is
    always accepted; INT is accepted when it is high at an instruction boundary with
    IE = 1 and no simultaneous NMI or hold request (p. 6-2; pp. 6-1, 8-4). When the native
    side would accept one, the adapter requests a host stop, copies state out at the clean
    boundary and services it in native mode; a masked INT is not serviced (whether the
    native EI delay applies to the 8080 EI in emulation is unconfirmed).
  - On x80's `Halt` exit the adapter owns the V20 standby: an INT with interrupts disabled
    resumes the 8080 program after the HLT without service (p. 9-1), by clearing
    `latches.halted` and running again. *(Amended 2026-11-15: halted lives in the view,
    kept by the native core; the adapter still performs this wake.)*
  - Memory accesses map by kind: `OpcodeFetch` and `OperandFetch` to PS, `Data` and `Stack`
    to DS0 (p. 8-1). Register state is copied in and out by value at BRKEM, RETEM, CALLN
    and interrupt stops. *(Amended 2026-11-15: through the view, at every hand-back.)*
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

## Amendment, 2026-11-15: option B, the view exchange and the adapter rules

Except for the BRK = 1 rule, the bullets below record maintainer decisions of
2026-11-14 during review of the C4 exit-contract draft; x86jit's C4a under decision-23
(x86jit) implements them (bead `xj-50w.7` (x86jit), `spec.md §17.10` (x86jit)). The
BRK = 1 rule is dated separately below. x80 bead: `x80-pzp.9`. These are interface
decisions, not hardware facts; only the cited pages are read. x80's own behaviour is
unchanged.

- **Option B.** x80 returns `Unsupported { opcode: 0xED, acknowledge: false }` with PC on
  the ED byte; only that `OpcodeFetch` reached its bus (decision-5 item 6). The adapter
  reads nothing more and hands back an ED escape. At its next boundary the native core
  fetches ED at PS:PC again and decodes it: ED FD is RETEM (p. 12-177) and ED ED imm8 is
  CALLN (p. 12-176), each one native step. Any other ED form is the native core's
  unknown-instruction exit, and MD stays 0. The ED byte is read twice, once by x80 and
  once by the native core. Frames and vectors are read from memory at that time. An INT
  delivered before the native decode drops the pending escape; RETI then returns to the
  ED, which x80 fetches again. This supersedes the second sentence of the first profile
  rule above.
- **The view exchange.** The state exchanged at every hand-back is A, the five 8080 flags
  (S, Z, AC, P, CY, in the byte S Z 0 AC 0 P 1 CY), BC, DE, HL, SP, PC, IE, `ei_delay`
  and `halted`. PS and DS0 are part of the view but read-only: the hand-back never changes
  them (p. 8-1). The native core keeps `halted` in its vCPU (x86jit: a standby latch).
  `ei_delay` is exchanged verbatim, and a zero-step exit never clears it. Each copy-in
  rebuilds x80's registers and `Latches` from the view alone; the adapter keeps no 8080
  latch across a hand-back. x80's `Latches` has no field the view lacks.
- **Refusals.** Any other `Unsupported` opcode is a typed refusal (x86jit:
  `Refused { opcode }`): MD stays 0, nothing is decoded natively, and the refusal is
  reported. After a refusal, later runs are answered by the native core until the next
  hand-back or an interrupt taken at that boundary. `acknowledge = true` cannot occur under the profile, because the adapter
  never asserts INT into x80.
- **BRK = 1 (2026-11-15).** x86jit's rule, agreed with x80 on 2026-11-15 and landed with
  C4a: the native core refuses BRKEM with BRK = 1, before any effect, following the
  principle of decision-23 §3 (x86jit), which literally covers undefined encodings. p. 6-3
  (PDF 28) gives the general single-step rule: when BRK is set, "an interrupt routine
  specified by vector 1 starts after the current instruction has been executed", and BRK
  and IE are reset then. That covers BRKEM literally, but no page settles how it combines
  with BRKEM's switch into emulation mode and with R2's BRK clear (ASSUMED); single-step
  in emulation mode is not described. A RETI that starts with BRK = 1 and re-enters 8080
  mode is refused the same way; service entry clears BRK (p. 6-1; R2 for CALLN) and the
  mock routine leaves it clear, so no vector reaches it. This is an interface rule, not a
  hardware fact. The contract vectors therefore start with BRK = 0, so after BRKEM IE = 0
  comes from R2 and BRK = 0 from this precondition. R2's BRK clear has no observable
  effect at BRKEM, and format 2 witnesses only its IE clear there. CALLN's BRK clear under
  R2 is not witnessed either: BRK is 0 throughout emulation, so every CALLN starts with
  BRK = 0.
- **CALLN and the EI delay, ASSUMED.** No EI delay survives a CALLN excursion, and CALLN
  itself consumes it; both parts are ASSUMED, since no page states either. x80 leaves
  `ei_delay` set on the ED refusal, because only a completed instruction clears it.
  x86jit's C4a makes the same choice in its CALLN unit and witnesses it right after the
  CALLN step. On a native core the routine's
  own RETI would also end the delay, since every completed native instruction ends it.
- **Adapter rules.** The adapter never injects INT into the native core while NMI or HOLD
  is asserted (p. 6-2). x80's `stop_requested` must also carry the host's own stop or
  cancel. 8080 stores must become visible to native translations (x86jit:
  `Vm::write_bytes`).
- **To confirm jointly**, listed in PROVENANCE.md §1:
  - Offset wrap inside the segment. x80's bus offset is `u16`, so a carry into the next
    segment would need an x80 API change. p. 10-1 (PDF 40) and its Fig. 10-1 add a 16-bit
    offset to the segment value times 16, both unsigned, but do not say what an operand
    byte past FFFFh addresses. ASSUMED: offsets wrap inside the segment, as the fixture
    selects.
  - 8080 IN/OUT high address byte. It is the adapter's alone, because x80 passes only the
    8-bit port (decision-5 item 3). The fixture uses 00, ASSUMED.
  - The EI delay across INT and NMI service (open). An INT is accepted only after the
    delay, so only an NMI right after an 8080 EI reaches this case. The fixture keeps the
    delay across the excursion; on a native core the routine's instructions would end it.
    Format 2 cannot express this case: its routines are a fixed mock payload with no
    native instructions, and it has no record for the EI delay.

The contract vectors move to format `x80-v20-contract-2`. Its `access` record counts only
accesses through x80's bus, so the native side's reads after ED are excluded, like native
frames and vector reads. Its initial PSW is F292 (BRK = 0) instead of F392. Revisit when
an NEC source or hardware run settles CALLN's effect on the EI delay, the offset wrap or
the IN/OUT high address byte, or when the EI delay across NMI service is decided. The
rest of this record is unchanged.
