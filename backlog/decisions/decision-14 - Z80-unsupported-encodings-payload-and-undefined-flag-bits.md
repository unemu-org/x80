---
id: decision-14
title: Z80 unsupported encodings, payload and undefined flag bits
date: '2026-11-13'
status: accepted
---

Decider: maintainer (2026-11-13).

Rules 1, 2, 5 and 6 superseded, and rules 3–4 narrowed, by
[decision-20](<decision-20 - Z80-undocumented-behaviour-Zilog-NMOS.md>) §7: the
full normal-fetch map, full F/F' images and the undocumented flag rules.

## Context

The Z80 counterpart of decisions 3 and 11 (bead `x80-bi2.3`). M4 covers documented
Z80 behaviour only. decision-13 owns model layout and prefix dispatch;
decision-15 owns the API, fetch/refresh rules and the changes to decision-5.
This record fixes the admitted encodings, refusal boundary and diagnostic bytes,
and the treatment of unused and unknown flag bits. It proposes no implementation.

Sources are pinned in [PROVENANCE.md §1](../../PROVENANCE.md): Zilog Z80 CPU User
Manual UM008011-0816, revision 11, August 2016 (below, **UM**, PDF = printed + 14),
and Z80-CPU / Z80A-CPU Technical Manual 03-0029-01, ©1977 (**TM**, PDF = printed + 6).
TM's Tables 7.0-1–7.0-11, pp. 44–54, supply the second table transcription.
These sources share documentation lineage; their agreement is no hardware measurement.

**Stated** means printed on a cited page image; **derived** means an expansion or
count of the printed patterns with their printed field restrictions. **Library
choice** means an x80 contract, not a claim about silicon. **Unverified** means
the sources used here do not establish the claimed hardware behaviour. OCR was
used only to locate pages; the image audit below records the actual readings.

## Choice

### 1. Admit the documented set, with explicit reconciliation

**Library policy:** an encoding is admitted when the pinned sources print it as
an instruction, subject to the primary-source resolutions below. Do not infer
instructions by treating a prefix as ignorable, accepting an unconstrained bit
field, or copying an oracle's behaviour. The opcode map contains the documented
set; unsupported encodings are its complement, not a second maintained list.

**Derived:** the seven logical tables have these terminal-instruction counts.
Each immediate/address/displacement field is counted once, not once per value.
Prefix dispatch slots are counted separately and are not instructions.

| Logical table | Instructions | Dispatch slots | Unsupported terminal slots | Derivation from UM | TM cross-check |
| --- | ---: | ---: | ---: | --- | --- |
| Main | 252 | 4: CB, DD, ED, FD | 0 | Tables 6–13, pp. 42, 45, 47–52; Fig. 39 p. 54; Tables 15, 17–20, pp. 59, 61–64, reconciled below | Tables 7.0-1–7.0-11, pp. 44–54: 252 |
| CB | 248 | 0 | 8 | Fig. 39 p. 54: 7 rotate/shift operations × 8 operands = 56; Table 14 pp. 55–57 and entries pp. 243–260: 3 bit operations × 8 bits × 8 operands = 192 | Tables 7.0-7/8 pp. 50–51: 56 + 192 = 248 |
| ED | 58 | 0 | 198 | 4 I/R loads (Table 6 p. 42), 8 pair loads (entries pp. 103, 108), 8 transfer/search (Tables 9/10 pp. 48–49), NEG (Table 12 p. 51), 8 ADC/SBC (Table 13 p. 52), RLD/RRD (Fig. 39 p. 54), RETI/RETN (Table 15 p. 59), 14 register I/O + 8 block I/O (Tables 18/19 pp. 62–63; entries pp. 296, 307), 3 IM (Table 20 p. 64) | Tables pp. 44–54 give 59 if their explicit ED 70 form is included; excluding that disputed form gives the same 58 |
| DD | 39 | 1: CB | 216 | 15 indexed byte loads (Table 6 p. 42), 6 pair loads/PUSH/POP (Table 7 p. 45 and entries pp. 115–119), EX (Table 8 p. 47), 10 byte arithmetic (Table 11 p. 50), 6 pair arithmetic (Table 13 p. 52), JP (IX) (Table 15 p. 59) | Tables pp. 44–49, 52: 15 + 6 + 1 + 10 + 6 + 1 = 39 |
| FD | 39 | 1: CB | 216 | Same printed IY columns/forms on the cited pages | Same TM tables, IY forms: 39 |
| DD CB d | 31 | 0 | 225 | 7 indexed rotates/shifts (Fig. 39 p. 54, RLC entry p. 217); 8 BIT + 8 SET + 8 RES (Table 14 pp. 55–57; entries pp. 247, 255, 259) | Tables 7.0-7/8 pp. 50–51: 7 + 24 = 31 |
| FD CB d | 31 | 0 | 225 | IY forms, RLC p. 219, BIT p. 249, SET p. 257, RES p. 259; same compact tables | Same TM tables, IY forms: 31 |

The total is **698 terminal encodings**, derived as 252 + 248 + 58 + 39 + 39 +
31 + 31. The main-table subtotal can independently be reproduced by instruction
group: 77 byte loads + 15 pair loads/PUSH/POP + 4 exchanges + 88 byte arithmetic
+ 8 general/control + 12 pair arithmetic + 4 accumulator rotates + 16 jumps
+ 26 calls/returns/restarts + 2 immediate I/O = 252 (UM tables cited above; TM
Tables 7.0-1–7.0-11 pp. 44–54).

**Stated and derived:** `ED 63 nn nn` and `ED 6B nn nn` are documented, despite
their omission from UM's compact Table 7 p. 45. `LD (nn),dd` p. 108 prints
`ED 01dd0011`, and `LD dd,(nn)` p. 103 prints `ED 01dd1011`; both explicitly
include `HL = 10`. TM Table 7.0-2 p. 45 independently prints both patterns with
dd = BC, DE, HL, SP. Thus these two forms belong in the eight ED pair loads and
the ED count is 58, not 56. They are not excluded as undocumented aliases merely
because unprefixed `22`/`2A` also load HL.

**Derived, reproducible terminal sets:** CB admits `00–2F, 38–FF`. For DD and
FD the 39 ordinary terminal bytes are:

```text
09 19 21 22 23 29 2A 2B 34 35 36 39
46 4E 56 5E 66 6E 70 71 72 73 74 75 77 7E
86 8E 96 9E A6 AE B6 BE E1 E3 E5 E9 F9
```

`CB` additionally dispatches to the indexed-bit table. Each indexed-bit table
admits only `06 0E 16 1E 26 2E 3E` and `46 + 8b`, `86 + 8b`, `C6 + 8b` for
b = 0…7. These are the seven memory rotate/shift forms and the three memory bit
families, not register-copy forms (UM Fig. 39 p. 54, entries pp. 217, 219,
247, 249, 255, 257, 259; TM Tables 7.0-7/8 pp. 50–51).

The ED terminal set, expanded from the cited group patterns, is:

```text
40 41 42 43 44 45 46 47 48 49 4A 4B 4D 4F
50 51 52 53 56 57 58 59 5A 5B 5E 5F
60 61 62 63 67 68 69 6A 6B 6F
72 73 78 79 7A 7B
A0 A1 A2 A3 A8 A9 AA AB B0 B1 B2 B3 B8 B9 BA BB
```

**Source resolutions (derived readings):** instruction-entry patterns and field
definitions prevail over conflicting compact cells when TM's table agrees.
PROVENANCE §1 records each of these disagreements:

- UM Tables 6/11 pp. 42, 50 and Fig. 39 p. 54 label the H register column
  F. Use H: the register-field definitions in entries pp. 71, 145, 213 and
  TM Tables 7.0-1/4/7 pp. 44, 47, 50 explicitly name H = 100. The erroneous
  heading does not introduce an F operand.
- UM Table 7 p. 45 prints PUSH codes ending in 6 and POP AF as `P1`. Use PUSH
  `C5/D5/E5/F5`, DD/FD `E5`, and POP AF `F1`, from entries pp. 115, 117–119
  and TM Table 7.0-2 p. 45.
- UM Table 11 p. 50 prints `88` for ADD A,(HL); entry p. 148 and TM
  Table 7.0-4 p. 47 give `86`. Table 13 p. 52 prints `DB` for DEC BC;
  entry p. 201 and TM Table 7.0-6 p. 49 give `0B`.
- UM Fig. 39 p. 54 prints `CB 06` for RLC L, `CB 0E` for RLC (HL),
  and `CB 06` for RRC E. Entries pp. 213, 215 give `CB 05` and `CB 06`;
  p. 225 gives `CB 00001rrr` with E = 011, hence RRC E is `CB 0B`.
  TM Table 7.0-7 p. 50 agrees. Indexed RLC is correctly printed as
  `DD/FD CB d 06` in Fig. 39 and entries pp. 217, 219.
- UM Table 14 pp. 55–57 is corrupted throughout: notably `C8` prefixes in
  place of CB, repeated terminal bytes, BIT 3/7 indexed terminals printed
  as 46, and DD in some IY prefix cells. Use the BIT/SET/RES entry patterns
  pp. 243, 245, 247, 249, 251, 253, 255, 257, 259–260 and TM Table 7.0-8
  p. 51; no extra encodings are admitted from those erroneous cells.
- UM Table 15 p. 59 prints `D8` for JP C,nn and `EB` for JP (HL).
  Entries pp. 263, 275 and TM Table 7.0-9 p. 52 give `DA` and `E9`.
  UM Table 18 p. 62 prints ED 7B for IN A,(C); entry p. 296 and TM
  Table 7.0-11 p. 54 give ED 78.
- TM Table 7.0-11 p. 54 explicitly describes r = 110 as flag-only IN.
  UM p. 296 instead labels that field **"Undefined op code; set the flag"**.
  Follow the primary's explicit undefined classification and the maintainer's
  documented-only scope: ED 70 is Unsupported. This is an intentional source
  precedence decision, not an OCR omission.

### 2. Reject every remaining encoding at the first failing decode

**Library policy**, extending decision-3: Unsupported contributes no steps and
no cycles. Restore the whole pre-attempt CPU state, including registers, flags,
PC, SP, **R**, interrupt/refresh bookkeeping and all execution/sampler latches
from before that boundary's samples. There are no memory writes, data/stack
reads or port accesses before refusal. Previously retired progress remains in
the report. Instruction-stream reads already delivered to the bus cannot be
undone; repeating the call with the same responses produces the same refusal.
This is not a claim about hardware execution of an undocumented encoding.

The hole classes, **derived from rule 1's documented set**, are:

- DD/FD followed by a byte with no documented index form: including index-half
  forms such as DD 44 and an otherwise ordinary opcode with an ignored-prefix
  interpretation, plus DD DD, DD FD, DD ED and their FD counterparts. Refuse
  the second byte; do not scan ahead to find a later instruction.
- ED holes: all 198 complements, including ED 70, ED 71 and alternate NEG,
  RETN/RETI or IM bytes absent from the printed forms. ED 70 is explicitly
  undefined (UM p. 296); OUT (C),r p. 307 lists only B,C,D,E,H,L,A, omitting
  r = 110, so ED 71 has no documented form. TM p. 54's ED 70 disagreement
  does not admit either encoding under this policy.
- CB 30–37: the missing eighth rotate/shift operation (UM Fig. 39 p. 54;
  TM Table 7.0-7 p. 50); SLL is outside M4.
- DD CB d op or FD CB d op without one of the 31 documented memory forms:
  all register-result/copy forms, alternate BIT encodings and the missing
  shift, including DD CB d 47 and DD CB d 36 (UM entries and TM tables in rule 1).

**DD CB needs an explicit distinction:** DD CB is the start of a documented
four-byte form, not itself an unsupported two-byte instruction (UM p. 247;
TM p. 51). It dispatches, reads d and then op, and refuses only if op has no
documented form. Treating DD CB itself as a hole would also reject documented
BIT/SET/RES and rotate/shift instructions. There is no truncated-stream exit on
the synchronous infallible Bus: each requested byte is supplied by the embedder.

### 3. Extend Unsupported with fact-free preceding bytes

**Accepted API choice**, additive to decision-5 item 6 and independent of CPU
semantics. Retain `#[non_exhaustive]` and its existing fields; add:

```rust
pub preceding: [u8; 3],
pub preceding_len: u8,
```

`preceding_len` is 0…3. The used prefix of `preceding` contains, in read order,
every instruction-stream byte read before the failed byte, including a
displacement when present. Unused elements are zero. `opcode` is the byte that
failed to decode; `pc` is the address of the instruction's **first** byte.
The names and representation encode no Z80 prefix taxonomy at the crate root.
For 8080/8085 refusals and the single-byte rejected interrupt acknowledges of
decision-15, the preceding array is zero and the length is zero;
`acknowledge` retains its existing meaning. IM 0/vector restrictions remain
decision-15's subject.

**Derived capacity:** three preceding bytes suffice because the longest
refusal path admitted by this dispatch policy is DD/FD CB d op (UM indexed
entries cited above). A repeated DD/FD is rejected immediately rather than
growing an unbounded prefix buffer. Bytes after the failure are neither read
nor reported, even if memory contains a later valid instruction.

### 4. Expose precisely the reads needed to decide

**API trace choice**, applying decision-15 rule 5: opcode-fetch (M1) reads are
`OpcodeFetch`, prefixes included; displacement d is `OperandFetch`. For an
indexed-bit sequence, the terminal op is also `OperandFetch`, following
decision-15's explicitly **unverified** non-M1 reading of the fourth-byte
cycle (UM BIT b,(IX+d) p. 247; TM §3 p. 7 does not settle the four-byte case).
An operation byte can therefore be reported as `opcode` even when its access
kind is OperandFetch: that field identifies the failed decode, not the bus
kind. decision-15 owns this reading and its PROVENANCE row, and the hardware
fetch/refresh accounting. Refusal restores R regardless of any tentative
accounting. All addresses below wrap in u16 arithmetic.

| Refusal class | Bus reads before exit, in order | Count |
| --- | --- | ---: |
| Unprefixed unsupported byte, if a documented group is temporarily unimplemented | opcode at pc: OpcodeFetch | 1 |
| DD/FD byte without an index form, including another DD/FD or ED | prefix at pc, failing byte at pc+1: both OpcodeFetch | 2 |
| ED hole, including 70/71 | ED at pc, failing byte at pc+1: both OpcodeFetch | 2 |
| CB hole 30–37 | CB at pc, failing byte at pc+1: both OpcodeFetch | 2 |
| DD/FD CB d hole | DD/FD at pc and CB at pc+1: OpcodeFetch; d at pc+2 and failing op at pc+3: OperandFetch | 4 |

The first row is only the staging convention of decision-5 item 7, if used
during M4; the completed main table has no unsupported terminal byte. Nothing
in this record authorizes a permanent documented-but-unimplemented path.

**Worked examples** (hexadecimal bytes, `pc = P`, `acknowledge = false`):

| Bytes in memory | preceding / length | opcode | Reads and stopping point |
| --- | --- | --- | --- |
| DD 44 | [DD, 00, 00] / 1 | 44 | 2 OpcodeFetch reads; no register transfer |
| ED 70 | [ED, 00, 00] / 1 | 70 | 2 OpcodeFetch reads; no port input |
| DD CB d 47 | [DD, CB, d] / 3 | 47 | 2 OpcodeFetch, then 2 OperandFetch reads; no read at IX+d |
| DD DD 21 nn nn | [DD, 00, 00] / 1 | DD | 2 OpcodeFetch reads; 21 and nn are never read |

In each example PC is restored to P, R and all latches are restored, and no
step/cycle is retired. The same rules apply to FD. The only bus effects are
the listed reads, whose device side effects cannot be restored by the CPU.
decision-15 records the wording change to decision-5 item 6 from **"only the
read of opcode has reached the bus"** (and its opcode-or-acknowledge wording)
to **"only the reads of the reported bytes have reached the bus"**. The
existing 8080/8085 single-byte trace is the zero-preceding-bytes case.

### 5. Store six flags and fix unused F bits at zero

**Stated:** UM Table 21 p. 65 places S,Z,H,P/V,N,C at bits 7,6,4,2,1,0;
Table 22 and text p. 66 say bits 3 and 5 are **"not used"**. **Library choice:**
Flags stores these six flags only; the byte images of F and F' always have
bits 5 and 3 zero, including after loading an AF image. Loading an image ignores
those two bits. This does not assert that physical hardware always reads zero.
PROVENANCE §1 records this choice explicitly.

**Derived witness mask:** hardware-claim comparisons default to D7h
(11010111b). A separate library-contract witness checks the zero-bit image
without masking. Do not use an oracle's full-byte result to introduce values
for bits 5/3 or hidden state such as WZ/MEMPTR.

### 6. Preserve flags the primary calls unknown

**Library choice:** if an instruction's primary entry calls a flag unknown,
leave that flag unchanged. This deterministic value is not claimed as hardware
behaviour. Continue to implement the entry's known flags and its explicit
unaffected flags. PROVENANCE §1 records the rule separately from the unused-bit
image choice.

**Stated inventory / derived comparison masks:** the UM instruction entries
print unknown flags for the following forms. Masks remove the unknown flags
from D7h; library witnesses separately check preservation from both initial
values. Removing a flag from a hardware mask never excuses failing to preserve
it under the library contract.

| Instruction entry | Unknown flags | UM printed pages (PDF pages) | Hardware comparison mask |
| --- | --- | --- | --- |
| BIT b,r | S, P/V | 243–244 (257–258) | 53h |
| BIT b,(HL) | S, P/V | 245–246 (259–260) | 53h |
| BIT b,(IX+d) | S, P/V | 247–248 (261–262) | 53h |
| BIT b,(IY+d) | S, P/V | 249–250 (263–264) | 53h |
| INI | S, H, P/V | 298 (312) | 43h |
| INIR | S, H, P/V | 301 (315) | 43h |
| IND | S, H, P/V | 302 (316) | 43h |
| INDR | S, H, P/V | 305 (319) | 43h |
| OUTI | S, H, P/V | 309 (323) | 43h |
| OTIR | S, H, P/V | 312 (326) | 43h |
| OUTD | S, H, P/V | 313 (327) | 43h |
| OTDR | S, H, P/V | 316 (330) | 43h |

TM Table 7.0-8 p. 51 agrees on BIT's unknown S/P/V. Three disagreements
affect how this inventory is read (PROVENANCE §1):

- TM Tables 7.0-5/6 pp. 48–49 print H unknown for CCF and ADD/ADC/SBC pair
  arithmetic. UM gives explicit H rules for CCF p. 178 and ADD HL, ADC HL,
  SBC HL, ADD IX, ADD IY pp. 188, 190, 192, 194, 196. Follow the primary:
  these do not enter the unknown inventory, and H is not masked merely
  because TM marks it unknown. This record settles definedness, not the
  arithmetic equations that the group records/witnesses must verify.
- TM Table 7.0-11 p. 54 also prints C unknown for the eight block-I/O forms;
  the UM pages in the inventory explicitly say C is not affected. Follow
  the primary and compare C under 43h, rather than TM's narrower 42h.
- UM pp. 246 and 250 each print both H set and H reset in the BIT flag list.
  Read the second H as N: adjacent BIT entries pp. 244, 248 and TM p. 51
  agree on H = 1, N = 0. It is a label error, not an extra unknown flag.

## Alternatives

- Commonly observed aliases, ignored/repeated prefixes, index halves, SLL or
  indexed register-copy operations: outside the maintainer's M4 scope; no
  undocumented-behaviour profile is introduced.
- Compact-table cells alone as the documented set: rejected because they both
  omit explicitly documented ED HL loads and contradict the instruction entries.
- A prefix enum in the root Unsupported payload: rejected because it would put
  model facts in the model-neutral core (decisions 6 and 13).
- Read beyond the failing byte for a fuller diagnostic: rejected because it
  adds observable bus effects unrelated to deciding the refusal.
- Keep eight flag bits or import oracle-derived unknown flags: rejected because
  the first adds out-of-scope state and the second cannot license behaviour.
- Clear unknown flags instead of preserving them: equally possible as a library
  rule, but preservation gives a deterministic result without a new flag value.

## Consequences

- The decoder and independent whole-set witness must agree on all seven counts,
  field restrictions and hole classes. Literal witnesses must distinguish ED
  63/6B from ED 70/71, DD 44 from documented indexed loads, and DD DD from DD CB.
- Refusal witnesses check the complete pre-boundary state (including R and
  sampled-input latches), payload, read addresses/kinds, absence of data/port
  accesses and zero new progress. These are future implementation obligations,
  not tests claimed to pass in this documentation bead.
- Hardware witnesses use the primary's per-instruction masks; library witnesses
  independently pin zero unused bits and preservation of unknown flags.
- ZEXDOC groups that execute excluded encodings remain not run, as recorded in
  PROVENANCE §2. An exerciser result cannot expand the documented set or settle
  an unknown flag.
- The undocumented-behaviour row in deferred.md points here. PROVENANCE §1 gains
  the source-disagreement rows and the two library-choice rows. decision-15 owns
  the other labelled API readings and the pointers from decision-5.

## Revisit when

A pinned primary source documents an excluded encoding or defines a currently
unknown flag, the maintainer expands documented-only scope, or a longer admitted
encoding needs more preceding-byte capacity. A newly discovered manual
disagreement is recorded in PROVENANCE §1 before changing this set or its masks.

## Image audit

READ, 2026-11-13, with `pdftoppm -r 150 -f N -l N -png`; no network or oracle
execution. Every page image actually read for this record is listed below;
additional rendered pages not read are not evidence. PDF numbers follow the
maps stated in Context.

- UM printed pp. 42, 45, 47–66, 71; 103, 108, 115, 117–119, 145, 148, 178;
  188–197, 201, 213, 217, 219, 224–225; 243–260, 263, 275;
  296, 298, 301–302, 305, 307, 309, 312–313, 316 (PDF = printed + 14).
- TM printed p. 7 and pp. 43–54 (PDF 13, 49–60).

No unresolved maintainer choice remains; the maintainer accepted this record on
2026-11-13. Hardware behaviour for excluded encodings and the actual values of
unknown flags remain **unverified and out of scope**, not inferred from refusal
or preservation. The indexed-CB fourth-byte cycle remains **unverified** as
labelled in decision-15; rule 4 fixes the access tag under that reading.
