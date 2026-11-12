---
id: decision-12
title: i8085 API contract
date: '2026-11-12'
status: accepted
---

Decider: maintainer (2026-11-12). The 8085 extension of decision-5 within the layout of decision-10;
bead `x80-4gk.5`. Page numbers are 205775-002 (printed 2-n = PDF 23+n, 5-n = PDF 85+n)
unless marked "DS" (the 8085AH data sheet 231718-001 in the 1988 handbook, printed 16-n =
PDF 947+n) or 9800301-04. Values marked "to confirm" are checked on the page image by
the bead that encodes them.

## Context

decision-5 fixes the model-neutral contract (bus, access kinds, budget, report, exits,
host stop, boundary precedence) and the 8080's inputs. The 8085 keeps the model-neutral
part and changes the inputs and interrupt structure:

- Interrupt lines (p. 2-5; DS p. 16-13 Table 2): TRAP, priority 1, vector 24H, "rising
  edge AND high level until sampled"; RST 7.5, priority 2, 3CH, rising edge, latched;
  RST 6.5, priority 3, 34H, level; RST 5.5, priority 4, 2CH, level; INTR, priority 5,
  level, with an acknowledge cycle.
- TRAP, RST 7.5, RST 6.5 and RST 5.5 are vectored internally: the CPU runs an opcode-fetch
  machine cycle without RD and inserts the restart itself, a bus-idle cycle (p. 2-15,
  Fig. 2-19; DS Table 3 p. 16-20). There is no acknowledge read.
- INTR (pp. 2-13, 2-14, Figs 2-17 and 2-18): INTE is reset, then an INA cycle "identical
  to an OF cycle" with INTA instead of RD; for CALL the PC increment is inhibited during the
  three INA cycles, then two memory writes. "Any instruction could be used (except EI or
  DI)"; placing EI or DI on the bus during INA "is prohibited" (p. 5-17).
- Masks and enables (p. 5-18 SIM; p. 5-17 RIM): RST 5.5, 6.5, 7.5 are masked
  individually; "the servicing of any interrupt (TRAP, RST 7.5, RST 6.5, RST 5.5, INTR)
  disables all future interrupts (except TRAPs) until an EI instruction is executed" (DS
  p. 16-15). TRAP "disables interrupts, but preserves the previous interrupt enable status";
  the first RIM after a TRAP returns that status (DS p. 16-15; p. 5-17).
- Halt is left by RESET or a valid interrupt, "enabled and unmasked (masking only applies
  for RST 5.5, 6.5, 7.5)" (p. 2-16; Fig. 2-11 p. 2-8); with interrupts disabled only RESET
  or TRAP restarts a halted 8085 (9800301-04 p. 3-24).
- RESET IN (p. 2-18 table) resets PC, INTE, the RST 7.5 flip-flop, TRAP and SOD and sets
  the three masks; it "does not explicitly change" the registers and flags, whose results
  "are indeterminate" (DS p. 16-13: "may be altered by RESET with unpredictable results").
- SID is read by RIM; SOD is written by SIM (pp. 5-17, 5-18; p. 2-19).

## Choice

Sketch below; the bead `x80-4gk.8` fixes the exact names. Behaviour is fixed here.

```rust
pub mod i8085 {
    pub trait Inputs: crate::Control {        // polled per boundary, fixed order
        fn trap(&mut self) -> bool { false }      // TRAP line level
        fn rst7_5(&mut self) -> bool { false }    // RST 7.5 line level
        fn rst6_5(&mut self) -> bool { false }
        fn rst5_5(&mut self) -> bool { false }
        fn intr(&mut self) -> bool { false }
        fn sid(&mut self) -> bool { false }       // read only by RIM, inside the instruction
        fn sod_changed(&mut self, _level: bool) {} // notified by SIM, in program order
    }
    pub struct Latches { /* interrupt enable, EI delay, halted, masks 5.5/6.5/7.5,
                            RST 7.5 flip-flop, TRAP edge state, enable status saved
                            by TRAP and its "first RIM" flag, SOD, last sampled levels */ }
}
```

1. Polls (extends decision-5 item 4): at each boundary that passes the budget check the
   core calls `stop_requested`, `trap`, `rst7_5`, `rst6_5`, `rst5_5` and `intr`, once each,
   in that order, whatever their results and the state. `sid` is read only by RIM. The
   call pattern is part of the deterministic trace.
2. Edges: the core derives rising edges of TRAP and RST 7.5 from consecutive level samples
   kept in `Latches`. A pulse shorter than the gap between two boundaries is missed unless
   the embedder holds it until the next poll; this sampler is an API choice, labelled so.
   - TRAP: a rising edge sets a pending-TRAP latch. TRAP is accepted at the first boundary
     where it is eligible and the line is still high ("must go high and remain high until it
     is acknowledged", DS p. 16-15); if the line is low there, the pending latch is cleared.
     After acceptance it is not recognized again until the line goes low and then high again
     (DS p. 16-15). The pending latch persists across `run`/`step` calls and through the
     boundary that follows EI or DI (rule 6).
   - Clarification (x80-4gk.9 review, 2026-11-12; it applies decision-5 item 6 and the
     rule above, no change of substance): a boundary that exits with Stop or Unsupported
     leaves `Latches` as they were before its samples (the `Control` contract), so a pulse
     seen only there is sampled again next run only if the embedder still holds it. A Halt
     exit keeps its samples. During the EI/DI inhibition (rule 6) a low TRAP re-arms but
     does not clear the pending latch, since TRAP is not eligible there.
   - RST 7.5: a rising edge sets the RST 7.5 flip-flop whatever the mask and the interrupt
     enable (Fig. 2-6; p. 5-18, to confirm). The flip-flop stays set until RST 7.5 is
     accepted, SIM with bit 4 set clears it, or RESET (p. 5-18; p. 2-18). The embedder may
     also set it through `latches_mut` between calls.
3. Boundary sequence (extends decision-5 item 5): budget; polls and stop; the highest
   eligible interrupt by priority TRAP, RST 7.5, RST 6.5, RST 5.5, INTR; halt; fetch and
   execute. Eligibility: TRAP when pending and still high (rule 2); RST lines with interrupts
   enabled, the EI delay expired and the mask clear (RST 7.5 through its flip-flop, RST 6.5
   and 5.5 as levels); INTR with interrupts enabled and the EI delay expired.
4. Vectored acceptance: interrupts disabled (TRAP saves the previous status for the first
   RIM), halt left, the RST 7.5 flip-flop cleared on its own acceptance, PC pushed (high
   byte first, Stack writes) and loaded with the vector. No acknowledge read; the bus-idle
   cycle makes no bus call. It counts as one step; its states come from Fig. 2-19 and the
   RST entry (to confirm, expected 12), cited as derived.
5. INTR acceptance: interrupts disabled, halt left, `InterruptAcknowledge` reads at the
   unchanged PC. RST n executes as on acceptance above. CALL (CD) reads two more
   acknowledge bytes at the unchanged PC and pushes the boundary PC (Figs 2-17, 2-18; to
   confirm, expected 18 states). Any other byte, EI and DI included, returns `Unsupported`
   with `acknowledge = true` and the boundary state restored, as decision-5 item 6.
6. EI and DI: as the 8080 (one-instruction EI delay; DI immediate). No interrupt, TRAP
   included, is accepted at the boundary right after EI or DI (p. 5-17; 9800301-04
   p. 3-23); this reading of "not recognized during" is labelled unverified.
7. Halt: left by an eligible interrupt (rule 3) or reset; with interrupts disabled only
   TRAP wakes it. `run` on a halted CPU with nothing eligible returns `Halt` at once
   (decision-2).
8. RIM and SIM: RIM reads `sid`, `rst6_5` and `rst5_5` once each, in that order, inside
   the instruction (p. 2-19 transfers the pin status during RIM), and builds A from SID, the
   RST 7.5 flip-flop, those two levels, the interrupt-enable status (or the status saved by
   TRAP, once) and the masks (p. 5-17). These calls are part of the trace.
   SIM loads the masks when bit 3 is set, clears the RST 7.5 flip-flop when bit 4 is set,
   and when bit 6 is set loads bit 7 into SOD and calls `sod_changed` (p. 5-18). The SOD
   timing of p. 2-19 (the instruction after SIM) is simplified to SIM itself, stated.
9. Reset: clears PC, interrupt enable, the EI delay (unverified, as for the 8080), halt,
   the RST 7.5 flip-flop, the TRAP state and SOD; sets the three masks (p. 2-18). Registers,
   flags and SP are unchanged: a library choice, since the sources call them indeterminate.
   Reset has no bus parameter, so it does not call `sod_changed`.
10. The exit enum and `Unsupported` payload are unchanged (decision-5).
11. HOLD and HLDA are not modelled, like READY and wait states (deferred.md): the
    instruction-level API has no bus-cycle resolution. The temporary leave from halt
    during HOLD (p. 2-16; 9800301-04 p. 3-24) is out of scope.

## Alternatives

- The embedder reports edges itself (an "edge since last poll" method). Not proposed:
  polls stop being side-effect-free level reads, and replay needs embedder state.
- INTR acknowledge with RST n only, as the 8080. Not proposed: CALL is documented cycle by
  cycle (Figs 2-17, 2-18) and is the path the manual's interrupt controller uses.
- INTR acknowledge with any instruction except EI and DI. Rejected: one sentence, without
  PC or timing rules for other instructions.
- SOD as a latch only, read by the embedder between calls. Not proposed: a device would see
  SOD changes late and out of program order.

## Consequences

- `i8085::Inputs` and `Latches` are fixed before the group beads (bead `x80-4gk.8`), so
  traces and fixtures do not move when the interrupt beads land.
- PROVENANCE.md §1 gains rows for the labelled readings: the edge sampler, the EI/DI
  recognition reading, the SOD timing simplification, the derived acceptance states and
  the reset library choice.

## Revisit when

A pinned source settles a labelled reading, an embedder needs edge reporting it cannot
express with levels, or the INTR acknowledge set must grow.
