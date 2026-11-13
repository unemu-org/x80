---
id: decision-15
title: z80 API contract
date: '2026-11-13'
status: accepted
---

Decider: maintainer (2026-11-13).

## Context

The Z80 extends decision-5's model-neutral bus, budget, report and exits within
decision-6's model boundary and decision-13's layout. decision-14 owns the
documented-encoding policy and the fact-free extension of `Unsupported`; this
record fixes how execution exposes those choices. It is the decision-12 analogue
for M4, bead `x80-bi2.4`. The maintainer's readings of 2026-11-13 are incorporated
below; the maintainer accepted the record on 2026-11-13, choosing the
asserted-true input convention of rule 1's preamble.

Sources are pinned in PROVENANCE.md §1. Here **UM** means Zilog Z80 CPU User Manual
UM008011-0816, revision 11, August 2016 (primary; printed p. n = PDF n+14).
**TM** means Zilog 03-0029-01, 1977 (printed p. n = PDF n+6); **ALPM** means
Zilog 03-0002-01, January 1978 (piecewise map in PROVENANCE.md); **MK** means
Mostek MK78515 V2.0, October 1978 (printed 2-n = PDF n+14).
These second witnesses share documentation lineage, not independent hardware
measurements. Every cited page below was checked as a rendered image.

**Stated** identifies explicit source text, or an explicitly stated API choice
where so qualified. **Derived** identifies an inference from cited statements.
**Unverified** identifies a specific model reading that the inspected pages do
not settle. API choices do not claim pin-level equivalence. PROVENANCE.md §1
records each labelled reading, including departures and the timing disagreement.
Implementation still requires independent witnesses derived from these citations.

READ, 2026-11-13: page images rendered at 150 dpi (OCR used only to locate pages):

| Source | Printed pages read on images |
| --- | --- |
| UM008011-0816 | 3, 6–8, 12–14, 18–20, 94–95, 97, 132, 136, 139–142, 181–183, 247, 288, 290, 292–293, 295–296, 298, 300, 302, 304, 306–307, 309, 311–313, 315–316 (PDF = printed+14) |
| 03-0029-01 | 4, 7–9, 11, 16, 18, 46, 53–57 (PDF = printed+6) |
| 03-0002-01 | 48, 49, 51, 246, 248, 267, 269 (PDF 52, 53, 55, 255, 257, 277, 279 respectively) |
| MK78515 V2.0 | 2-15 and 2-200 (PDF 29 and 214) |

## Choice

Names below are a sketch; behaviour and the field responsibilities are fixed here.
The two input methods report whether the request is asserted: `true` means the
active-low pin is Low (requesting), `false` means it is High (inactive). Both
default to `false`. This logical convention is a stated API choice by the
maintainer (2026-11-13), matching the `i8080` and `i8085` inputs, where `true`
always means the line is active; NMI's negative edge (UM Pin Functions p. 6;
TM §3 p. 8) is therefore a `false`-to-`true` transition of `nmi()`.

```rust
pub mod z80 {
    pub trait Inputs: crate::Control {
        fn nmi(&mut self) -> bool { false } // true while asserted (pin Low); false-to-true edge requests NMI
        fn int(&mut self) -> bool { false } // true while asserted (pin Low) requests INT
    }
    pub struct Latches { /* IFF1, IFF2, interrupt mode (0/1/2), EI delay,
                            halted, last sampled NMI state (initially not asserted),
                            LD A,I / LD A,R parity-note pending flag */ }
}

// Provided additions to crate::Bus; existing required methods remain.
fn input16(&mut self, port: u16) -> u8 { self.input(port as u8) }
fn output16(&mut self, port: u16, value: u8) {
    self.output(port as u8, value);
}
```

1. **Polls — stated API choice (extends decision-5 item 4).** At each boundary
   passing the budget check call `stop_requested`, `nmi`, `int`, once each, in
   that order, whatever their results, IFFs, delay or halt state. These should be
   side-effect-free level reads. Calls and returned levels belong to the
   deterministic trace. The fixed host order is not hardware timing; the hardware
   recognition context is UM Pin Functions p. 6 and Interrupt Request/Acknowledge
   pp. 12–14; TM §3 p. 8 and §4 pp. 16, 18.
2. **NMI sampler — stated hardware edge; stated API sampler.** NMI is negative
   edge-triggered (UM Pin Functions p. 6; TM §3 p. 8). Detect an edge when the
   previous committed sample is not asserted and the new sample is asserted
   (pin High then Low), keeping that history in `Latches` across `run`/`step`.
   Holding it asserted produces no further edge; a sampled not-asserted value
   re-arms it. Initial/reset history is not asserted by library choice, so an
   initially asserted sample requests NMI. A pulse wholly between polls is
   missed; the embedder must hold the level through a poll. This is a boundary
   sampler, not the hardware's within-instruction edge capture.
   As in decision-12 rule 2, Stop or Unsupported restores all boundary state,
   including the pre-sampling history. Budget performs no sampling. Halt keeps
   samples. A pulse seen only at a restored boundary can be seen on retry only
   if the embedder still presents it; input calls and bus reads are not undone.
3. **Boundary sequence — stated API choice, with stated interrupt priority.**
   Budget; all three polls and host stop; sampled NMI edge; INT if the sampled
   `int()` reports asserted, IFF1 is set and the EI delay has expired; halt; fetch/execute.
   NMI takes priority over INT and is independent of IFF1 and the EI delay
   (UM Pin Functions p. 6, HALT Exit p. 14, EI p. 183; TM §3 p. 8, §4 p. 18,
   §8 p. 55). A successfully accepted interrupt occupies one step, with no
   handler instruction in that step. decision-5's budget overshoot, saturation,
   earlier-progress reporting and `step` definition remain. A zero/reached
   budget makes no poll or bus call; Stop changes no CPU state.
4. **Ports — stated hardware addresses; stated API extension of decision-5
   item 3.** Add the two provided 16-bit methods sketched above. Only the Z80
   uses them here; an embedder may override them to observe all address bits.
   Defaults forward the low byte to the existing 8-bit methods. For each I/O
   access form the port from the values present at that access:

   | Instruction class | A8–A15 | A0–A7 | Stated primary citation; second witness |
   | --- | --- | --- | --- |
   | IN A,(n) | A before the input | n | UM IN A,(n) p. 295; TM Table 7.0-11 p. 54 |
   | OUT (n),A | A | n | UM OUT (n),A p. 306; TM Table 7.0-11 p. 54 |
   | IN r,(C) | B before the input | C before the input | UM IN r,(C) p. 296; TM Table 7.0-11 p. 54 |
   | OUT (C),r | B | C | UM OUT (C),r p. 307; TM Table 7.0-11 p. 54 |
   | INI / INIR / IND / INDR | B before decrement | C | UM corresponding entries pp. 298, 300, 302, 304; TM Table 7.0-11 p. 54 |
   | OUTI / OTIR / OUTD / OTDR | B after decrement | C | UM corresponding entries pp. 309, 311, 313, 315; TM Table 7.0-11 p. 54; ALPM OUTI p. 267/PDF 277 and OTIR p. 269/PDF 279 explicitly confirm decremented B |

   In particular OUTI and OTIR expose the decremented byte counter, not its
   value on entry. Register-field exclusions belong to decision-14. UM p. 296
   calls r=110 undefined while TM p. 54 describes a flags-only input; this
   wording disagreement is recorded in PROVENANCE.md §1 and delegated to
   decision-14's exclusion policy, not admitted as another API operation here.
5. **Access kinds — stated M1 fact; stated API mapping; unverified indexed-CB
   exception.** Extend decision-5 item 3's first-byte wording: on the Z80 every
   memory opcode-fetch (M1) cycle is an `OpcodeFetch`, prefixes included.
   UM Pin Functions p. 6 identifies M1+MREQ as opcode fetch; TM §3 p. 7 explicitly
   says M1 is generated for each byte of two-byte opcodes beginning CB, DD, ED
   or FD. Thus the ordinary byte following any of those prefixes is also
   `OpcodeFetch`, not `OperandFetch`. No boundary poll occurs between bytes.
   Immediate operands and displacements are `OperandFetch` by the existing
   instruction-stream API distinction; displacement is an operand (UM Registers
   p. 3 and BIT b,(IX+d) p. 247/PDF 261; TM §2 p. 4).
   For DD CB d op (and FD CB d op), classify DD/FD and CB as `OpcodeFetch`, d
   and the fourth byte op as `OperandFetch`. The fourth byte is an operation
   byte, not displacement or memory data. This non-M1 reading is **unverified**:
   UM BIT b,(IX+d) p. 247 supplies the byte order and 20 (4,4,3,5,4) timing but
   does not label the fourth byte's cycle; TM §3 p. 7's two-byte wording does
   not expressly settle the four-byte case. The chosen kind keeps an ordinary
   instruction-stream read visible without claiming a third M1. Do not infer
   cycle identity merely from a T-state count. Other memory data is `Data`,
   SP-based push/pop accesses are `Stack`, and INT's special M1 read is
   `InterruptAcknowledge` (rules 8/9), not an additional `OpcodeFetch`.
6. **Refresh — stated register rule; derived fetch count and ordering; stated
   API omission.** UM Registers p. 3 and TM §2 p. 4 state that seven R bits
   increment after each instruction fetch and bit 7 stays as programmed by
   LD R,A. Use `(R & 80h) | ((R + 1) & 7Fh)` once per opcode-fetch cycle,
   including prefixes. This per-M1 interpretation is **derived** from those
   pages, TM §3 p. 7 and UM LDIR p. 132's two refresh cycles per iteration.
   Increment once on NMI's discarded fetch and once on INT's special M1
   acknowledge (**derived**, UM Figs 9/10 pp. 13–14; TM §4 pp. 16, 18).
   Under rule 5's unverified classification DD CB d op increments R twice.
   LD R,A loads all eight bits (UM LD R,A p. 97; ALPM p. 51/PDF 55);
   execute that load after its fetch increments, replacing their result.
   LD A,R observes the increments from its own two fetches (UM LD A,R p. 95;
   ALPM p. 49/PDF 53). This ordering is **derived** from fetching before
   execution, not an explicit microcycle statement on those instruction pages.
   There is no refresh bus call: UM Instruction Fetch p. 8 states RD is absent
   during refresh. Omitting refresh addresses/signals from `Bus` is a stated API
   choice. While halted there is no R advance (rule 7, a stated departure).
7. **EI, DI, HALT, RETI and RETN.** **Stated:** EI sets IFF1 and IFF2; maskable
   interrupts remain disabled through the following instruction (UM Table 1
   p. 18 and EI p. 183; TM §8 p. 55). DI clears both immediately (UM DI p. 182,
   Table 1 p. 18; TM Fig. 8.0-1 p. 56). Neither prevents NMI (UM p. 6; TM p. 8).
   **Unverified edge-case model:** EI after EI re-arms the one-instruction
   delay; DI cancels it; NMI acceptance leaves a pending delay for the next
   completed ordinary instruction/iteration to consume. The cited EI pages do
   not specify those delay-latch interactions. Stop, Budget and Unsupported do
   not consume it.
   HALT retires its entry instruction in 4 states (**stated**, UM HALT p. 181;
   TM §4 p. 18 describes the following fetch loop). Keep the resume PC after
   HALT (**derived** from instruction completion and the restart return PC,
   UM pp. 14, 181, 290). At a halted boundary an NMI edge always wakes;
   INT wakes when IFF1 is set and eligible under rule 3 (**stated**, UM HALT
   Exit p. 14; TM §4 p. 18). Reset also leaves halt (UM HALT p. 181).
   decision-2's prompt Halt exit is kept: no idle steps, cycles, fetches or R
   increments. This is a **stated departure** from the hardware NOP-fetch loop
   and refresh on UM pp. 14, 181 and TM p. 18. The normal budget precedence
   still applies after HALT entry; `step` may report Budget with `halted` set.
   RETN pops low then high using `Stack`, advances SP twice and copies IFF2 to
   IFF1, leaving IFF2 (**stated**, UM RETN p. 290 and Table 1 p. 19;
   ALPM p. 248/PDF 257). RETI performs the same PC pop but leaves both IFFs:
   UM RETI p. 288 says it does not re-enable entry-disabled interrupts;
   MK Table 2-1 p. 2-15/PDF 29 and RETI p. 2-200/PDF 214 explicitly state
   both unchanged. That is the **stated documented reading**, not a RETN alias.
   Both take 14 states (UM entries pp. 288, 290; TM Table 7.0-10 p. 53).
   RETI's ED 4D fetches are observable to the embedder (rule 5); no new
   daisy-chain callback or device model is added (stated API choice).
8. **NMI acceptance — stated effects, derived 11 states and access mapping.**
   Clear IFF1, keep IFF2; leave halt (UM Table 1 p. 19, HALT Exit p. 14;
   TM Fig. 8.0-1 p. 56 and §4 p. 18). Read once at the boundary PC with
   `OpcodeFetch` and discard the value, without advancing PC. UM Nonmaskable
   Interrupt Response p. 13 and Fig. 10 p. 14 show PC, M1, MREQ and RD, and
   CPU Response p. 19 says the fetched instruction is ignored; TM §4 p. 18
   and §8 p. 56 agree. The one-read API mapping is **derived** from that cycle,
   adopted explicitly instead of treating it as a bus-idle cycle.
   Increment R once (rule 6), then push the boundary PC: decrement SP and
   write PCH as `Stack`, decrement SP and write PCL as `Stack`, then set
   PC = 0066h. Push order and unchanged return PC are **stated** by the RETN
   example (UM p. 290; ALPM p. 248/PDF 257); kind names are the API mapping.
   Count one step and **11 states, derived** from CPU Response p. 19's restart
   equivalence and RST p. 293's 11 (5,3,3); TM §8 p. 56 and Table 7.0-10
   p. 53 support the same derivation. The inspected response pages do not
   explicitly print an NMI total. Apply rule 10's pending P/V correction on
   successful acceptance. No `InterruptAcknowledge` read is made for NMI.
9. **INT acceptance by mode — stated mode rules; derived acknowledge mapping
   and totals; stated restrictions.** On successful acceptance clear both
   IFFs and leave halt (UM p. 18, p. 14; TM §8 p. 55, §4 p. 18).
   In every mode read one byte at the unchanged boundary PC using
   `InterruptAcknowledge`, then increment R once. This common mapping is
   **derived** from UM Interrupt Request/Acknowledge pp. 12–13, Fig. 9
   (PC on the address bus), and TM §4 p. 16; neither cycle description is
   restricted to a mode. Acknowledge does not advance PC in this API.
   Validate any refusal condition before stack writes, vector reads or other
   committed changes. A successful acceptance also applies rule 10.

   - IM 0 executes RST n only, pushing the boundary PC in rule 8's order and
     loading the selected restart address. This is a **stated library scope
     restriction**, not the full hardware repertoire: UM CPU Response p. 19
     and TM §8 p. 56 allow other instructions, expressly including CALL.
     Other acknowledge bytes give Unsupported with `acknowledge = true` and
     the whole boundary state restored, including R and sampler history.
     Only the acknowledge read has reached the bus. The accepted RST costs
     **13 states, derived**: UM p. 19's normal instruction cost plus two,
     with RST's 11 on p. 293; TM pp. 56, 53 agree.
   - IM 1 ignores the acknowledge byte, pushes the boundary PC in rule 8's
     order and sets PC = 0038h (**stated** vector, UM p. 19; TM p. 56).
     Cost is **13 states, derived** from the same pages' restart-plus-two rule
     and RST total. No byte is refused in this mode.
   - IM 2 forms `v = (I << 8) | acknowledge_byte`; bit 0 must be zero
     (**stated**, UM p. 20, Fig. 17; TM §8 p. 57). An odd byte gives
     Unsupported with `acknowledge = true`, that byte as `opcode`, empty
     preceding bytes and the boundary state restored. Refusing the unspecified
     odd case is a **stated library restriction**, not a hardware outcome.
     For an even byte, push PC first, then read `Data` at v (low target byte)
     and v+1 (high target byte), and set PC to that target. UM p. 20 explicitly
     orders acknowledge, PC save, then table lookup, and identifies the first
     table byte as low; TM p. 57 agrees. Kind names are the API mapping.
     **19 states is stated**, not inferred: both pages give 7 acknowledge,
     6 PC-save and 6 table-read clocks. All successful modes count one step.
10. **LD A,I / LD A,R parity note — stated flag source; unverified boundary
    interpretation.** P/V receives IFF2, not data parity (UM entries pp. 94–95
    and Table 1 p. 18; ALPM pp. 48–49/PDF 52–53). The UM entries both say
    that an interrupt during execution makes the parity flag zero. ALPM p. 48
    has the note for LD A,I; p. 49 omits it for LD A,R. Follow the primary for
    both; omission supplies no contrary result.
    Model the note with one pending latch: completion of either instruction
    sets it; if NMI or INT is successfully accepted at the next boundary,
    clear P/V before service and clear the latch. An ineligible request does
    not clear P/V. If an ordinary instruction/iteration executes instead,
    expire the old latch before that instruction; another LD A,I/R sets it
    anew. Budget splits preserve it; Stop and Unsupported restore it with all
    boundary state. The mapping of "during execution" to the immediately
    following acceptance, and its application to both interrupt kinds, are
    **unverified readings**, explicitly adopted; no sub-instruction poll is
    introduced.
11. **Block instructions — stated repeat mechanics and costs; stated API
    retirement choice.** One iteration is one completed step. On repeat,
    rewind PC by two before retirement, so the next boundary polls normally
    and refetches both bytes. The PC rewind, recognition of interrupts and two
    refresh cycles per iteration are **stated** in UM LDIR p. 132, LDDR
    p. 136, CPIR p. 139, CPDR p. 142, INIR p. 300, INDR p. 304, OTIR
    p. 311 and OTDR p. 315. ALPM OTIR p. 269/PDF 279 agrees; TM Tables
    7.0-3 p. 46 and 7.0-11 p. 54 independently list the cost paths.
    Use `States.untaken` = terminating 16 and `States.taken` = repeating 21
    for those repeat forms; both totals are **stated** on the cited pages
    (OTIR/OTDR terminating totals continue on UM pp. 312/316). The two-slot
    assignment and per-iteration budget accounting are API choices using the
    existing fact-free `decode::States` and `progress` machinery (decision-13).
    A non-repeating block instruction retires its one iteration normally.
    For CPIR/CPDR termination is counter exhaustion **or** a match: UM
    pp. 139/142's descriptions and TM Table 7.0-3 p. 46 agree. The UM's
    16-state headings print **and**; use the descriptions and the second
    witness, recording the disagreement in PROVENANCE.md §1. A witness must
    cover early match with count remaining and exhaustion without a match.
12. **Reset — stated reset set; stated library preservation and bookkeeping.**
    Clear IFF1/IFF2, PC, I and R, and select IM 0 (UM RESET p. 6 and
    Table 1 p. 18; TM RESET p. 9 and Fig. 8.0-1 p. 56; MK Table 2-1
    p. 2-15/PDF 29 for both IFFs). Leave halt (UM HALT p. 181).
    Preserve SP, both general/AF register sets, IX, IY and flags by library
    choice: the cited reset descriptions do not specify their values. Clear
    EI delay and the parity-note latch, and set NMI sample history to not asserted by
    library choice; no hardware delay-latch reset claim is made. `reset`
    takes no bus, makes no polls or accesses and reports no elapsed cycles.
    `new` owns supplied state; `Default` uses zero register values, disabled
    IFFs, IM 0, no delay/halt/parity-note pending state and a not-asserted NMI history. Defaults are
    library construction values, not a hardware power-up claim.
13. **Exits and Unsupported — stated API extension of decision-5 items 6/10.**
    Keep the non-exhaustive `Exit::{Budget, Stop, Halt, Unsupported}` and
    decision-5 accounting. Apply decision-14's fact-free payload extension:
    retain `pc`, `opcode`, `acknowledge`; add a fixed `[u8; 3]` holding bytes
    read before the refused byte and a length for its populated prefix.
    Exact field names and representation invariants belong to decision-14.
    `pc` remains the first instruction-byte address, or the boundary PC for
    an acknowledge; `opcode` is the refused byte. Preceding bytes are stream
    bytes without root-level prefix semantics, so they can include d in
    DD CB d op. Unprefixed/acknowledge refusals have zero preceding length.
    Replace item 6's one-read wording for multi-byte refusal with "only the
    reads of the reported bytes have reached the bus"; operand/data/stack
    side effects beyond those reads must not occur. Zero steps/cycles are
    contributed; all registers and latches, including R, EI delay, NMI history
    and the parity-note latch, are restored to the boundary. Reads and device
    side effects from those reads cannot be rolled back. This is a stated
    library transactional contract, including interrupt refusals; it is not
    hardware interrupt rollback (UM p. 18 describes IFF clearing on acceptance).
    decision-5 receives pointers only; its 8080 rules retain their substance.
14. **Excluded pins and timing — stated API scope.** BUSREQ/BUSACK, WAIT, pin
    timing and refresh addresses on the bus are out of scope. UM Pin Functions
    pp. 6–7, Instruction Fetch p. 8 and Bus Request/Interrupt Acknowledge
    p. 12 describe these hardware features; TM §3 pp. 8–9 and §4 p. 16
    provide second witnesses. In particular the INT-not-accepted-during-BUSREQ
    condition on UM p. 12 (TM p. 16), and bus requests inhibiting either
    interrupt, have no input or extra gating in this contract. The two automatic
    acknowledge wait states remain included in the documented totals (rule 9);
    externally requested waits add no modelled time. Refresh is exposed through
    R only, within the HALT departure of rule 7.

## Alternatives

- Edge events supplied by the embedder: rejected for this contract; level reads
  and persistent sampler state keep the trace reproducible. Sub-poll pulses
  require an embedder-side holding mechanism or a future API.
- Replace the required 8-bit port methods, or add a Z80-only bus trait: rejected;
  provided methods preserve existing implementations and keep I/O on `Bus`.
- Suppress NMI's discarded memory read: rejected; the cited response diagram
  explicitly shows a read cycle whose data is ignored.
- General IM 0 injected instructions or odd IM 2 extrapolation: not proposed;
  the adopted scope and typed refusal leave those contracts explicit.
- Advance halted time internally, or ignore the LD A,I/R note: rejected in
  favour of the accepted prompt exit and the labelled boundary interpretation.
- Treat RETI as RETN: rejected for the documented model; the Mostek statement
  explicitly distinguishes their IFF effects.

## Consequences

- Inputs, latch responsibilities, trace kinds and interrupt paths are fixed
  before implementation. decision-13 owns layout and sharing; decision-14 owns
  admitted encodings and payload representation. Neither is redrafted here.
- decision-5's port width, OpcodeFetch wording and refusal-read wording gain
  additive extensions here. Existing 8080/8085 port calls and behaviour stand.
- Witnesses must pin literal poll/access order, state restoration on retries,
  R counts, both block cost slots, mode-specific acceptance totals and every
  labelled choice. A second-source transcription catches reading errors, not
  shared documentation errors. No oracle or hardware measurement is claimed.
- Residual unverified readings are the indexed-CB fourth-byte cycle, EI delay
  edge cases and the parity-note acceptance window. NMI's 11 and IM 0/1's 13
  are derived totals, not stated timing entries. The CPIR/CPDR condition-heading
  disagreement is resolved as rule 11 specifies.

## Revisit when

A pinned source explicitly identifies the indexed-CB fourth-byte cycle or
settles a labelled timing/interrupt reading; independent evidence falsifies one;
or an embedder requires sub-boundary edges, halted-time advancement, additional
IM 0 instructions, bus arbitration or externally inserted wait states.
