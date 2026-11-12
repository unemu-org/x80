# V20 mode-switch contract fixtures

This unpublished, test-only crate owns portable contract vectors and a mock native
side for decision-9 and [plan §3](../backlog/docs/plan.md#3-v20v30-integration-contract).
It is a contract fixture, not an adapter for production. The embedder owns its
adapter. The mock performs BRKEM, interrupt entry, RETI, RETEM and RESET as
operations; it never decodes or executes x86 instructions. Its only dependency is
`x80-core`; all implementation code lives under `tests/`.

Run `cargo test -p x80-v20-contract --offline --locked`. Ordinary tests need neither
a native core nor a manual download. Tests compare the entire final native context,
every native frame and service-entry context, the full 1 MiB memory image, and
access counts and segments. Separate witnesses cover initial mapping, refusals,
the selected EI-delay/I/O/wrap assumptions, and copy-out before RESET.

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
| Vector layout PC word followed by PS word at vector times four; NMI vector 2; push PSW, PS, PC; clear IE/BRK and set MD for service | 6-1, Table 6-1/Fig. 6-1 (26) |
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
| ASSUMPTION `BRKEM_PRESERVES_IE` | true | PROVENANCE §1 to-confirm list; NEC 6-1 versus 12-169 |
| ASSUMPTION `BRKEM_PRESERVES_BRK` | true | NEC 6-1 clears BRK in the general interrupt sequence; 12-169 lists only MD for BRKEM |
| ASSUMPTION `CALLN_PC_ADVANCE` | 3 bytes, so saved PC is after ED ED nn | PROVENANCE §1 to-confirm list; NEC 12-176 |
| ASSUMPTION `EMULATION_EI_DELAY` | true: honor x80's following-instruction delay | PROVENANCE §1 to-confirm list; native EI applicability unresolved |
| ASSUMPTION `IO_HIGH_BYTE` | 00 | PROVENANCE §1 to-confirm list; native I/O does not settle emulation high byte |
| ASSUMPTION `WRAP_OFFSETS` | true: FFFF -> 0000 within the same segment | PROVENANCE §3's unverified wrap; plan §3 joint questions (absent from §1's to-confirm list) |

These are fixture selections for joint confirmation, not silicon claims. The
portable vectors select these values; changing them requires different expected
data. Additional tests exercise EI delay, IN/OUT high byte and instruction-offset
wrap; the nine required round trips do not depend on I/O or boundary wrapping.

Other fixture policies, collected in that same file: keep EI-delay across a
synchronous native excursion; clear halted on wake; scripts explicitly order
events. The mock native payload changes AL to C7, CW to ABCD, IX to DEAD, and
temporarily inverts CY, P, AC, Z, S, BRK, IE, DIR and V relative to the saved
PSW, with MD=1 and reserved bits retained. Thus every writable PSW bit differs
from its saved value before RETI. Literal `routine` records check this live PSW
(FC46 for saved 7393, FE46 for saved 7193). RETI restores the entire saved PSW,
while general-register edits carry back.
Every service vector checks the restored emulation PSW (including MD=0) and
8080 flag byte immediately after RETI, before RETEM restores the native PSW.
This payload is arbitrary test data and implies no native instruction semantics.

RESET sets PC, SS, DS0, DS1 to 0000, PS to FFFF, MD=1, DIR=IE=BRK=0 (NEC 7-1).
All other register values are undefined by that page. The mock retains those
values solely to make replay deterministic; portable hardware checks must mask
them. The full literal final RESET state describes this fixture policy. The
pre-reset copied-out boundary is also checked. No reset-vector instruction runs.

## Language-neutral vector format, version 1

[`vectors/mode-switch.txt`](vectors/mode-switch.txt) is the canonical data.
It uses UTF-8 lines, whitespace-delimited tokens, `#` comments, hexadecimal
numbers without prefixes, and case-sensitive record names. No Rust expressions,
external libraries, inheritance chains or generated expected values are needed.

Start with `format x80-v20-contract-1`. Records before the first `case NAME` are
common to every case. A case ends with `end`. Each case starts independently.
Memory starts as **CC in every byte of a flat 1 MiB array**. Common memory patches
then case memory patches are applied in file order before BRKEM. Logical addresses
use `(segment * 16 + offset) modulo 1048576`, and multi-byte patches wrap their
offset modulo 65536. Code/data/native-stack segments share this one array, so
aliases are real, even though these round trips use disjoint ranges.

| Record | Meaning |
| --- | --- |
| `initial AW BW CW DW BP SP IX IY PS SS DS0 DS1 PC PSW` | Fourteen native 16-bit words, including full PSW. PC is the already advanced native continuation, not the address of encoded BRKEM. |
| `brkem NN` | Perform BRKEM operation with vector NN after loading memory. |
| `memory SEG OFF BYTE...` | Initial byte patch; contains vector table and 8080 code as plain data. |
| `run N` | Run N 8080 instructions with no external request; require Budget. |
| `until halt` / `until retem` | Run until the named exit, with a 64-iteration safety cap; CALLN service/RETI is automatic. |
| `until serviced` | Run until a CALLN service/RETI completes, with the same safety cap. |
| `emulation PSW` | Expected live emulation PSW and low 8080 flag byte at this script point; used immediately after RETI to witness restored flags and MD=0. |
| `signal INT NMI HOLD RESET NN RESULT` | Deliver a boundary event (line bits 0/1), NN supplies INT vector; perform one adapter run. RESULT is budget/halt/retem/serviced/wake/reset. NMI means an already latched rising edge, always vector 02. |
| `final AW ... PSW` | Complete expected final native context, same word order as initial. |
| `frame SOURCE SS SP PC PS PSW` | Expected historical frame, in write order among frames. SOURCE is brkem/calln/int/nmi. SP points to its lowest address. Words at SP, SP+2, SP+4 are PC, PS, PSW; writes occurred PSW then PS then PC. |
| `routine PSW` | Expected full live PSW after each mock payload, immediately before RETI, in service order. |
| `service AW ... PSW` | Complete expected native context on entry to each mock routine, before its arbitrary edits and RETI. |
| `changed SEG OFF BYTE...` | Expected final 8080 data/stack writes, overlaid on initial memory. Expected frame bytes are also overlaid, oldest frame first. No other memory byte may change. |
| `access PS DS0 OF PF DR DW SR SW` | Expected segment values and six hexadecimal byte-access counts: opcode fetch, operand fetch, data read/write, emulated stack read/write. Includes adapter escape operand reads. Excludes native frames and vector reads. |

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
at 3000:7FFA with PC=2345, PS=1000, PSW=F392. Native service frames are at
3000:7FF4; the complete per-case expected words remain literal in the text file.

## Scenarios and source map

All scenarios also rely on register/segment mapping (8-1), BRKEM (8-2, 12-169)
and native frame/vector layout (6-1, 4-1, 10-1). Every case except RESET ends by
RETEM (8-4/8-5, 12-177). This table adds the scenario-specific sources.

| Case | Event script after common prefix / extra NEC pages |
| --- | --- |
| brkem_retem | RETEM; 12-177 |
| calln_reti_retem | CALLN 40, mock RETI, RETEM; 6-1, 8-4, 12-176, 12-168 |
| int_enabled_running | INT high with IE=1; 6-2, 8-4, 12-168 |
| int_disabled_running | DI, masked INT, no service; 6-2 |
| nmi_running | DI, NMI, mock RETI; 6-1, 8-4, 12-168 |
| hlt_int_disabled | DI, HLT, INT wake without service; 8-2, 9-1 |
| hlt_int_disabled_retem | DI, HLT, masked INT wake directly into RETEM; 9-1, 12-177 |
| hlt_int_disabled_calln | DI, HLT, masked INT wake directly into CALLN 40, RETI, RETEM; 9-1, 12-176, 12-168 |
| hlt_int_enabled | HLT, INT service, RETI after HLT; 8-2, 9-1, 12-168 |
| hlt_nmi | DI, HLT, unconditional NMI service; 8-2, 9-1, 12-168 |
| reset_running | RESET after prefix; 7-1, 8-4 |
| int_low_running | Low INT with IE=1, no service; 6-2 |
| int_held_running | INT high simultaneous with hold, no service; 6-2 |
| nmi_over_int_and_hold | NMI wins over high INT even with hold; 6-1/6-2, 8-4, 12-168 |

Anything other than unsupported ED followed by FD or ED nn is a typed refusal;
no speculative native semantics are assigned. The current x80 public API remains
unchanged. A future native core and the embedder can port the text records and
replace mock operations while retaining these independent expectations and
explicit assumptions.
