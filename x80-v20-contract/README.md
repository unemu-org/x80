# V20 mode-switch contract fixtures

This unpublished, test-only crate owns portable contract vectors and a mock native
side for decision-9 and its amendments and
[plan §3](../backlog/docs/plan.md#3-v20v30-integration-contract).
It is a contract fixture, not an adapter for production. The embedder owns its
adapter. The mock performs BRKEM, interrupt entry, RETI, RETEM, CALLN and RESET as
operations. The only bytes it decodes are ED FD and ED ED nn after an ED escape
hand-back (option B); it never executes x86 instructions. Its only dependency is
`x80-core`; all implementation code lives under `tests/`.

Run `cargo test -p x80-v20-contract --offline --locked`. Ordinary tests need neither
a native core nor a manual download. Tests compare the entire final native context,
every native frame and service-entry context, the full 1 MiB memory image, and
access counts and segments. Separate witnesses cover initial mapping, refusals,
the option-B escape, halted and the EI delay across hand-backs, the selected
EI-delay/I/O/wrap assumptions, and copy-out before RESET.

## Option B and the view exchange

decision-9's 2026-11-15 amendment fixes how the reference adapter in
[`tests/support/adapter.rs`](tests/support/adapter.rs) talks to the mock native
core in [`tests/support/native.rs`](tests/support/native.rs):

- Every run rebuilds x80 from the native model: A, the five flags, BC, DE, HL, SP,
  PC, IE, `ei_delay` and `halted`. Every x80 exit copies them back, the two latches
  verbatim. The adapter keeps no 8080 register or latch between runs. x80's
  `Latches` has no field the view lacks.
- On `Unsupported` at ED the adapter reads nothing more. The native model fetches
  ED at PS:PC again from its own memory: ED FD is RETEM, ED ED nn is CALLN, and any
  other ED form is the native side's unknown instruction (`Refusal::NativeUnknown`,
  MD stays 0). These native reads never reach x80's bus.
- Any other `Unsupported` opcode is the adapter's typed refusal (`Refusal::Opcode`,
  MD stays 0). An acknowledge refusal cannot occur: the adapter never asserts INT.

## Authority and limits

All NEC page references mean **NEC uPD70108/uPD70116 (V20/V30) User's Manual,
Stock No. 500350, October 1986**, pinned in [PROVENANCE §1](../PROVENANCE.md).
SHA-256: `8252fe5ddc0623380b7f28819009e6bb7d2b8e345ef347d9e6f2d299ce97865f`.
Pages were checked as rendered images. No manual images or text are distributed.

| Contract | Printed NEC page (PDF page) |
| --- | --- |
| AL=A; BW=HL, CW=BC, DW=DE (high/low bytes as drawn); BP=SP, PC=PC; native-only registers inaccessible; PS for code, DS0 for data and emulated stack | 8-1, Figs 8-1/8-2 (32) |
| PSW IE=bit 9, BRK=bit 8, MD=bit 15; low flag byte S Z 0 AC 0 P 1 CY | 3-3 (16), 8-1 (32) |
| 20-bit flat memory; little-endian words; segment times 16 plus unsigned offset; native stack SS:SP | 4-1, Fig. 4-2 (20); 10-1, Fig. 10-1 (40) |
| Vector layout PC word followed by PS word at vector times four; NMI vector 2; push PSW, PS, PC; clear IE/BRK and set MD for INT/NMI service | 6-1, Table 6-1/Fig. 6-1 (26) |
| INT acceptance at boundary requires high INT, IE=1, no simultaneous NMI or hold | 6-2 (27) |
| BRKEM saves PSW, PS, PC, sets MD=0, loads vector; native continuation PC supplied as an operation parameter | 8-2 (33), 12-169 (214) |
| ED ED nn CALLN saves emulation PSW (MD=0), PS, PC; loads native vector; returns by RETI | 8-4, Fig. 8-6 (35), 12-176 (221), 6-1 (26) |
| RETI restores PC, PS, PSW including MD during a native excursion from emulation | 12-168 (213) |
| ED FD RETEM restores original PC, PS, full PSW; emulated flags do not carry back | 8-4/8-5, Fig. 8-7 (35/36), 12-177 (222) |
| HLT wake: masked INT resumes without service, enabled INT services; NMI unconditional; return after HLT | 8-2 (33), 9-1 (38) |
| RESET aborts emulation and performs native reset | 8-4 (35), 9-1 (38), 7-1 (30) |

The CALLN page's `SP <- PS - 6` is the misprint recorded in PROVENANCE; the mock
uses the stack decrement independently specified by p. 6-1. RETI is used only for
an excursion from emulation, where p. 12-168 permits MD restoration. No nested
BRKEM operation exists (the prohibition is on p. 8-5).

The 8080 payloads follow Intel 98-153B, September 1975, Ch. 4 pp. 4-4/4-5,
4-11, 4-13/4-14; their stack accesses follow Ch. 2 pp. 2-18 to 2-20. Equivalence
of the uPD8080AF to x80's Intel model remains ASSUMED (PROVENANCE §1,
decision-9), including decision-8 and the derived DAA carry rule. Any x80 cycle
counts are Intel T-states, **not V20 clocks**; these vectors assert no timing.

## ASSUMPTIONS and fixture policies

All six unresolved hardware choices are centralized in
[`tests/support/assumptions.rs`](tests/support/assumptions.rs):

| Label | Selected value | Unsettled source |
| --- | --- | --- |
| ASSUMPTION `BRKEM_CALLN_CLEAR_IE_BRK` | true (ruling R2, decision-9 amendment): BRKEM and CALLN clear IE and BRK, as NEC 6-1's general sequence does | PROVENANCE §1 to-confirm list; NEC 6-1 versus 12-169 and 12-176, which list only MD (blank means "No change", Table 12-4, 12-3) |
| ASSUMPTION `CALLN_PC_ADVANCE` | 3 bytes, so saved PC is after ED ED nn | PROVENANCE §1 to-confirm list; NEC 12-176 |
| ASSUMPTION `EMULATION_EI_DELAY` | true: honor x80's following-instruction delay | PROVENANCE §1 to-confirm list; native EI applicability unresolved |
| ASSUMPTION `CALLN_CONSUMES_EI_DELAY` | true (decision-9 amendment, 2026-11-15): no EI delay survives a CALLN excursion; CALLN itself consumes it (on a native core the routine's RETI would also end it) | PROVENANCE §1 to-confirm list; no page states it |
| ASSUMPTION `IO_HIGH_BYTE` | 00: the 8080 IN/OUT high address byte, which the adapter alone supplies | PROVENANCE §1 to-confirm list; native I/O does not settle emulation high byte |
| ASSUMPTION `WRAP_OFFSETS` | true: offset wrap inside the segment, FFFF -> 0000 | PROVENANCE §1 to-confirm list; NEC 10-1 gives a 16-bit offset but does not state the wrap |

These are fixture selections for joint confirmation, not silicon claims. The
portable vectors select these values; changing them requires different expected
data. Additional tests exercise EI delay, IN/OUT high byte and instruction-offset
wrap; the nine required round trips do not depend on I/O or boundary wrapping.

INT and NMI entry clearing IE and BRK is read (6-1, 8-4), not assumed; only BRKEM
and CALLN depend on the R2 constant. The emulation PSW starts with IE = 0 (R2) and
BRK = 0 (precondition: the initial PSW has BRK = 0), 7092 after BRKEM and 7093 after
the common prefix. Cases that need
IE = 1 therefore execute an 8080 EI; where an INT must be acceptable, a NOP or HLT
completes after the EI so the EI delay (`EMULATION_EI_DELAY`) has expired.
`brkem_retem` checks the emulation PSW after BRKEM and the prefix (7093); the exact
7092 BRKEM leaves is checked in `contract.rs`. `calln_reti_retem` executes EI so that
CALLN's IE clear is visible. BRK is always 0 in emulation, so no vector can observe
R2's BRK clear at BRKEM or CALLN, or a BRK clear at service entry or RESET.

Other fixture policies, collected in that same file: INT and NMI service keep
the native model's EI delay across the excursion. That case is open (PROVENANCE
§1, decision-9 amendment 2026-11-15): only an NMI right after an 8080 EI reaches
it, and on a native core the routine's instructions would end the delay. Format 2
cannot express it, because its routines are a fixed mock payload and it has no
record for the EI delay. Service and wake clear halted; RETEM and RESET drop both
8080 latches; scripts explicitly order events. The mock
native payload changes AL to C7, CW to ABCD, IX to DEAD, and temporarily inverts
CY, P, AC, Z, S, IE, DIR and V relative to the saved PSW, with MD=1 and
reserved bits retained. BRK stays 0, because the native core refuses a RETI that
starts with BRK = 1 and re-enters 8080 mode. Thus every writable PSW bit except
BRK differs from its saved value before RETI. Literal `routine` records check this
live PSW (FC46 for saved 7293, FE46 for saved 7093). RETI restores the entire saved PSW,
while general-register edits carry back.
Every service vector checks the restored emulation PSW (including MD=0) and
8080 flag byte immediately after RETI, before RETEM restores the native PSW.
This payload is arbitrary test data and implies no native instruction semantics.

RESET sets PC, SS, DS0, DS1 to 0000, PS to FFFF, MD=1, DIR=IE=BRK=0 (NEC 7-1).
All other register values are undefined by that page. The mock retains those
values solely to make replay deterministic; portable hardware checks must mask
them. The full literal final RESET state describes this fixture policy. The
pre-reset copied-out boundary is also checked. No reset-vector instruction runs.

## Language-neutral vector format, version 2

[`vectors/mode-switch.txt`](vectors/mode-switch.txt) is the canonical data.
It uses UTF-8 lines, whitespace-delimited tokens, `#` comments, hexadecimal
numbers without prefixes, and case-sensitive record names. No Rust expressions,
external libraries, inheritance chains or generated expected values are needed.

Start with `format x80-v20-contract-2`; the reader rejects any other format,
including format 1. Records before the first `case NAME` are
common to every case. A case ends with `end`. Each case starts independently.
Memory starts as **CC in every byte of a flat 1 MiB array**. Common memory patches
then case memory patches are applied in file order before BRKEM. Logical addresses
use `(segment * 16 + offset) modulo 1048576`, and multi-byte patches wrap their
offset modulo 65536. Code/data/native-stack segments share this one array, so
aliases are real, even though these round trips use disjoint ranges.

| Record | Meaning |
| --- | --- |
| `initial AW BW CW DW BP SP IX IY PS SS DS0 DS1 PC PSW` | Fourteen native 16-bit words, including full PSW. PC is the already advanced native continuation, not the address of encoded BRKEM. |
| `brkem NN` | Perform BRKEM operation with vector NN after loading memory. The native side refuses BRKEM with BRK = 1, so the `initial` PSW has BRK = 0. |
| `memory SEG OFF BYTE...` | Initial byte patch; contains vector table and 8080 code as plain data. |
| `run N` | Run N 8080 instructions with no external request; require Budget. |
| `until halt` / `until retem` | Run until the named exit, with a 64-iteration safety cap; CALLN service/RETI is automatic. |
| `until serviced` | Run until a CALLN service/RETI completes, with the same safety cap. |
| `emulation PSW` | Expected live emulation PSW and low 8080 flag byte at this script point; used immediately after RETI to witness restored flags and MD=0, and in `brkem_retem` to witness the PSW after BRKEM and the prefix. |
| `signal INT NMI HOLD RESET NN RESULT` | Deliver a boundary event (line bits 0/1), NN supplies INT vector; perform one adapter run. RESULT is budget/halt/retem/serviced/wake/reset. NMI means an already latched rising edge, always vector 02. |
| `final AW ... PSW` | Complete expected final native context, same word order as initial. |
| `frame SOURCE SS SP PC PS PSW` | Expected historical frame, in write order among frames. SOURCE is brkem/calln/int/nmi. SP points to its lowest address. Words at SP, SP+2, SP+4 are PC, PS, PSW; writes occurred PSW then PS then PC. |
| `routine PSW` | Expected full live PSW after each mock payload, immediately before RETI, in service order. |
| `service AW ... PSW` | Complete expected native context on entry to each mock routine, before its arbitrary edits and RETI. |
| `changed SEG OFF BYTE...` | Expected final 8080 data/stack writes, overlaid on initial memory. Expected frame bytes are also overlaid, oldest frame first. No other memory byte may change. |
| `access PS DS0 OF PF DR DW SR SW` | Expected segment values and six hexadecimal byte-access counts: opcode fetch, operand fetch, data read/write, emulated stack read/write. Counts only accesses through x80's bus. Excludes the native side's re-read of ED and its reads of the bytes after ED, native frames and vector reads. |

The reference reader also compares both 8080 latches at each `service` and
`final` point: neither the EI delay nor halted may be pending there.

Changes from format 1:

- The format line names `x80-v20-contract-2`.
- `access` excludes the bytes after ED, which the native side now reads. Every
  operand-fetch count drops by one per RETEM and two per CALLN; all fifteen cases
  count 08.
- New case `calln_consumes_ei_delay`.
- The initial PSW is F292 (BRK = 0), not F392. The native core refuses BRKEM with
  BRK = 1 (decision-9 amendment, 2026-11-15, following the principle of
  decision-23 §3 (x86jit)), so format 1's vectors could not be ported verbatim.
  NEC p. 6-3's general single-step rule covers BRKEM literally, but no page settles
  how it combines with BRKEM's switch into emulation mode and with R2's BRK clear
  (ASSUMED); single-step in emulation mode is not described. The BRKEM frame PSW
  and the native PSW that RETEM restores are F292 too. The 8080-mode PSWs (7092
  after BRKEM), service and RESET values are unchanged.
- The mock routine no longer inverts BRK, so `routine` records read FC46 and FE46
  instead of FD46 and FF46. The native core refuses a RETI that starts with
  BRK = 1 and re-enters 8080 mode (decision-9 amendment, 2026-11-15).
- R2's BRK clear now has no observable effect at BRKEM, so format 2 witnesses only
  its IE clear there. After BRKEM, IE = 0 comes from R2 and BRK = 0 from the
  precondition. CALLN's BRK clear was not observable in format 1 either, since
  every CALLN starts with BRK = 0.

Signal tokens represent level observations at the scripted boundary, then are
deasserted. An accepted event host-stops x80 before fetching any opcode, copies
out, pushes a native frame, invokes the mock payload, performs RETI and copies
back in. A masked running INT is unserviced and one ordinary instruction executes.
A masked halted INT spans the wake **and execution of the following instruction**,
as required by NEC 9-1's note about holding INT high. The adapter clears halted and
executes that instruction before returning `wake`. No 15-clock claim is tested.
If that instruction produces another exit (including RETEM or CALLN service),
the adapter returns that outcome instead; refusals also propagate normally.
HOLD here only inhibits INT acceptance; bus hold timing is outside the fixture.
Simultaneous RESET/interrupt precedence and shared-core budgets are not claimed.

The common prefix executes nine instructions ending at 2000:0111. It changes AL
to A6, BW and CW to 1234, DW to 5678, sets carry, writes A6 at DS0:4000, and
pushes/pops 1234 through DS0:8FFE/8FFF. It deliberately distinguishes all register
pairs, flags, native versus emulated stack, and PS versus DS0. Untouched memory is
poisoned CC, making segment-routing mistakes observable. BRKEM's frame is always
at 3000:7FFA with PC=2345, PS=1000, PSW=F292. Native service frames are at
3000:7FF4; the complete per-case expected words remain literal in the text file.

## Scenarios and source map

All scenarios also rely on register/segment mapping (8-1), BRKEM (8-2, 12-169)
and native frame/vector layout (6-1, 4-1, 10-1). Every case except RESET ends by
RETEM (8-4/8-5, 12-177). This table adds the scenario-specific sources.

| Case | Event script after common prefix / extra NEC pages |
| --- | --- |
| brkem_retem | Emulation PSW after BRKEM (IE=0, ASSUMPTION R2; BRK=0, precondition), RETEM; 12-177 |
| calln_reti_retem | EI, CALLN 40 (IE clear observable, ASSUMPTION R2; BRK already 0, so its clear is not), mock RETI, RETEM; 6-1, 8-4, 12-176, 12-168 |
| int_enabled_running | EI, NOP, INT high with IE=1; 6-2, 8-4, 12-168 |
| int_disabled_running | DI, masked INT, no service; 6-2 |
| nmi_running | DI, NMI, mock RETI; 6-1, 8-4, 12-168 |
| hlt_int_disabled | DI, HLT, INT wake without service; 8-2, 9-1 |
| hlt_int_disabled_retem | DI, HLT, masked INT wake directly into RETEM; 9-1, 12-177 |
| hlt_int_disabled_calln | DI, HLT, masked INT wake directly into CALLN 40, RETI, RETEM; 9-1, 12-176, 12-168 |
| calln_consumes_ei_delay | EI, CALLN 40, RETI, INT serviced at the first boundary after the return. Witnesses `CALLN_CONSUMES_EI_DELAY` (ASSUMPTION) and that the adapter carries no 8080 latch across the excursion; on a native core the routine's RETI would also end the delay; 6-2, 12-176, 12-168 |
| hlt_int_enabled | EI, HLT, INT service, RETI after HLT; 8-2, 9-1, 12-168 |
| hlt_nmi | DI, HLT, unconditional NMI service; 8-2, 9-1, 12-168 |
| reset_running | EI, then RESET clears IE; 7-1, 8-4 |
| int_low_running | EI, NOP, low INT with IE=1, no service; 6-2 |
| int_held_running | EI, NOP, INT high simultaneous with hold, no service; 6-2 |
| nmi_over_int_and_hold | EI, NOP, NMI wins over high INT even with hold; 6-1/6-2, 8-4, 12-168 |

A non-ED `Unsupported` is the adapter's typed refusal; ED followed by anything but
FD or ED nn is the native side's unknown instruction. No speculative native
semantics are assigned. The current x80 public API remains
unchanged. A future native core and the embedder can port the text records and
replace mock operations while retaining these independent expectations and
explicit assumptions.
