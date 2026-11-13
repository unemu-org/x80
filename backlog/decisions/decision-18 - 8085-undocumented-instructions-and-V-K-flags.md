---
id: decision-18
title: 8085 undocumented instructions and V/K flags
date: '2026-11-13'
status: accepted
---

Decider: maintainer (2026-11-13). Bead `x80-mn3.3`. Revision 1 applies the maintainer's binding
amendment of 2026-11-13; this record remains a documentation proposal, with no
executable behaviour changes. On acceptance it supersedes decision-11 **in
part** (§6). Admitted 8085 behaviour is always on under decision-17; there is
no optional undocumented-instruction profile.

## Context

Intel 205775-002 (January 1983), Table 5-2 p. 5-20/PDF 105 leaves ten holes.
Its PUSH PSW image (§5.6.5 p. 5-15/PDF 100) marks bits 5/3/1 undefined;
POP PSW (p. 5-16/PDF 101) specifies only the five documented destinations.
decision-11 therefore refuses the holes and chooses filler bits 0/0/1.
Tundra manufactures a pin- and function-compatible second-source part and
documents the extended set. The maintainer permits that documentation to
license the Intel 8085 **library model**, with explicit limits below.

**ADMITTED** here describes the scope selected by this record for
implementation after acceptance. It does not mean measured on Intel silicon.
All ten encodings, operations, lengths and path totals in §2 are admitted;
remaining unverified hardware subfacts do not reinstate the blanket refusal.

## Amendment to decision-17

Maintainer direction, 2026-11-13: a data sheet issued by the **manufacturer** of
a **pin- and function-compatible second-source part** may license operations,
lengths, flag statements and timings on the corresponding original-part model.
This applies to future qualifying second-source manufacturer data sheets too,
not only Tundra. The following conditions are mandatory:

1. Label each licensed fact **documented by second source (part), unverified
   for [original part]**; name the actual documented silicon. For this record
   the exact label is **documented by second source (CA80C85B), unverified for
   Intel 8085**. Model admission is not a claim of original-part verification.
2. Independent die analysis must confirm the **encoding of every admitted
   instruction**. Decode confirmation need not independently establish all
   of its documented operations or timings. Keep that distinction explicit.
3. Silicon evidence wins a conflict. Preserve both readings and record the
   selected reading and its scope, including a corrected second-source rule.
4. Rule explicitly on internal inconsistency or incompleteness, citing the
   readings, the reason for choosing one, and a resolution label. A copied
   article may clarify the manufacturer's damaged text but remains the same
   origin, never an additional key.
5. A fact with neither qualifying second-source documentation nor the ordinary
   two keys stays **unverified as hardware**. The library may adopt an explicitly
   labelled API mapping or deterministic library rule, as decision-12 did;
   that choice does not manufacture evidence. In particular, totals and byte
   placements do not establish temporal bus order.

Pinning, origin accounting, independent witness tests, oracle limits and the
other decision-17 requirements remain. This is a scoped licence, not permission
to count a copied document twice or to claim that a decode row proves a timing.
decision-17 receives a pointer to this amendment; AGENTS.md receives one clause.

## 1. Sources, origins and scope

These identifiers refer to the full SHA-256 pins, document metadata and dated
captures in [PROVENANCE §1](../../PROVENANCE.md#8085-undocumented-instruction-sources-m6-x80-mn32).
All six original files were hash-checked locally for this revision. No network,
simulator/emulator implementation or linked timing ROM was read. A missing
key means missing **in this pinned corpus**, not absent everywhere.

| Key | Pinned source and exact locators used | Class / origin |
| --- | --- | --- |
| W | Dehnhardt and Sorensen, Electronics, 18 January 1979, pp. 144–145/PDF 1–2; p. 144 prose/examples; p. 145 “New 8085 instructions”, condition-code diagram and each instruction entry | C; article lineage |
| T | Tundra 8000 Series Components Manual, 9000000.MD302.01, ©1996, layout 13 June 1997; CA80C85B pp. 2-1, 2-4/PDF 25, 28; Fig. 2-9 p. 2-15/PDF 39; Table 2-10 p. 2-16/PDF 40 | A; **same origin as W**, not a second independent confirmation |
| KV | Shirriff, “Silicon reverse engineering: The 8085's undocumented flags”, February 2013; capture 20260905021235; opening flag list, “The circuit for the K and V flags”, “The silicon”, “Driving the data bus with a superbuffer”, notes [1], [3]–[5], Credits | B; die-data lineage |
| FC | Shirriff, “Reverse-engineering the flag circuits in the 8085 processor”, July 2013; capture 20261027125023; opening flag list/storage schematic, “The carry logic”, its three subsections and rotate footnotes | B; same die-data lineage |
| ALU | Shirriff, “Reverse-engineering the 8085's ALU and its hidden registers”, July 2013; capture 20261014020057; “The real architecture of the 8085 ALU”, “The accumulator and ACT registers”, “Building the 8-bit ALU from single-bit slices”, “The control lines” table | B; same die-data lineage |
| ROM | Zima, “Intel 8085 Instruction Decode ROM”, undated; capture 20260606111153; introduction and 48 decode rows, counted from DI/EI as row 1 | B; same die-data lineage |

W/T count once (KV note [3] identifies copying, including introduced errors).
KV/FC/ALU/ROM also count once: Visual6502 photographs and Zima's mask/transistor
data underlie the posts. A written statement in KV plus FC's explanation of
the same circuit is **not** two independent keys. W/T and the die lineage are
independent origins where they actually address the same fact.

Silicon labels are deliberately narrow: **die analysis (8085; exact part
unverified)** means the chip identified as 8085 by these analyses. ROM's heading
says Intel 8085, but its AMD9085 paths do not resolve manufacturer, package or
revision. T names **CA80C85B**, a CMOS second-source part. The manufacturer
states pin and functional compatibility with 8085/8085A (T p. 2-1/PDF 25);
that qualifies the document for the amendment above,
not as a measurement on an Intel specimen. Application of die findings to a
particular Intel revision remains unverified. Do not relabel any finding here
“hardware-measured” or “documented by manufacturer”.

## 2. Admitted encodings, operations, lengths and totals

**Every operation, operand, length and timing in the following table is
ADMITTED with the label “documented by second source (CA80C85B), unverified
for Intel 8085”.** T Table 2-10 p. 2-16/PDF 40 is the manufacturer licence;
W p. 145/PDF 2 clarifies its damaged entries under the rulings immediately
below. Neither is a silicon timing measurement. ROM rows are counted from
DI/EI as row 1. Each encoding has W/T plus its independent ROM key, labelled
**die analysis (8085; exact part unverified)**.

`b2` is the immediate byte; `a16 = byte 3:byte 2`. Totals are machine cycles
and T-states, with conditional paths named explicitly. Flags are in §4.

| Instruction / opcode | Bytes | Admitted operation | Machine cycles / T-states | Independent encoding key and exact limit |
| --- | ---: | --- | --- | --- |
| DSUB / 08 | 1 | HL ← HL − BC | 3 / 10 | ROM row 22 `0000 1000`; ALU non-A subtraction and `arith_to_flags`, FC combined zero. Confirms decode/category and combined Z, not the entire transfer/flag sequence |
| ARHL / 10 | 1 | Arithmetic HL shift right one: retain H7, old H0 → L7, old L0 → CY | 2 / 7 | ROM row 25 `0001 0000`; ALU non-A right shift and ARHL-specific `shift_right_in_select`, FC low ACT bit → CY. High-input prose reconciled below; no die confirmation of sign fill |
| RDEL / 18 | 1 | DE ← (DE << 1) with old CY entering E0; old D7 → CY | 3 / 10 | ROM row 23 `0001 1000`; ALU non-A addition and carry controls. Does not trace complete DE byte routing |
| LDHI / 28 | 2 | DE ← HL + b2; HL preserved | 3 / 10 | ROM row 24 `001x 1000`; FC constant+pair addition, temporary inter-byte carry and preservation of CY; not exact register selection or offset extension |
| LDSI / 38 | 2 | DE ← SP + b2; SP preserved | 3 / 10 | Same ROM row 24 and FC/ALU addition evidence; not exact register selection or offset extension |
| RSTV / CB | 1 | If V=1, save return PC high at old SP−1, low at old SP−2, SP ← SP−2, PC ← 0040h; otherwise continue sequentially | V=0 untaken: 1 / 6; V=1 taken: 3 / 12 | ROM row 8 `1100 1011`; confirms CB, not polarity/vector/stack trace or interrupt latches |
| SHLX / D9 | 1 | [DE] ← L; [DE+1] ← H; DE preserved | 3 / 10 | ROM row 11 `1101 1001`; distinct SHLX decode, not DE routing or temporal write order |
| JNK / DD | 3 | If K=0, PC ← a16; otherwise continue after the three-byte instruction | K=1 untaken: 2 / 7; K=0 taken: 3 / 10 | ROM row 32 `11x1 1101`; JNK/JK pair, not polarity or operand consumption |
| LHLX / ED | 1 | L ← [DE]; H ← [DE+1]; DE preserved | 3 / 10 | ROM row 12 `1110 1101`; distinct LHLX decode, not operand/transfer sequence |
| JK / FD | 3 | If K=1, PC ← a16; otherwise continue after the three-byte instruction | K=0 untaken: 2 / 7; K=1 taken: 3 / 10 | Same ROM row 32; pair, not polarity or operand consumption |

Smaller conditional totals are assigned to sequential continuation and larger
totals to the branch/restart operation: **second-source path interpretation**
of W's conditional entries and T's paired totals, with the same Intel-unverified
label. ROM's introduction expressly sends the decode outputs to a **separate**
timing ROM. No independently confirmed Intel timing is claimed or required by
this amendment to admit these documented totals.

### Internal-inconsistency rulings

| Disagreement / exact readings | Selected reading and reason | Resolution label |
| --- | --- | --- |
| T JNUI row contains “Jump on NOT UI”, then DSUB's saved-HL prose/equation, without a complete conditional sentence | JNK is jump if K=0. Take T's heading/opcode/address boxes and W p. 145 `if (not X5)`/reset prose; discard the stray subtraction. T JUI's trailing DSUB equation is likewise discarded | **Second-source text reconciliation**, Intel-unverified |
| T SHLX ends “The contents of register H are” at the page boundary | Complete it with W p. 145's equation and sentence: H goes to the succeeding location, [DE+1]. Both give D9, register-indirect, 3/10 and no flags | **Second-source completion from same-origin W**, Intel-unverified; not an independent key |
| W RSTV prints hex C8 beside binary 11001011; T prints binary 11001011 | CB wins: both binary diagrams and independent ROM row 8 agree. C8 is the written hexadecimal slip | **Die-analysis encoding resolution** |
| T LHLX has a `data` box but calls it register indirect and describes only DE-addressed memory; its high-byte equation misplaces parentheses | One instruction byte; H ← memory[DE+1]. Take W's one-byte diagram and complete equation, T's addressing/prose, and the distinct ED decode. Reject the extra instruction-stream byte and “memory[DE]+1” reading | **Second-source text reconciliation**, Intel-unverified |
| W JX5 equation `if (X5)` conflicts with its “reset” prose | JK tests K=1. T JUI explicitly says “is set”, agreeing with W's equation and the complementary NOT-UI entry. Retain both W readings in the audit | **Second-source polarity resolution**, Intel-unverified |
| T DSUB flags end `Z,S,P,CY,` versus W's explicit Z,S,P,CY,AC,X5,V / “all” | Update all seven; the trailing comma is incomplete, not a preservation statement. W completes it, T Fig. 2-9 defines the two extra flags, FC confirms combined Z. Detailed subfacts are bounded in §4 | **Second-source flag-list completion**, Intel-unverified |
| FC “Generating the shift_right input signal” says CY feeds the high-order ALU shift input for ARHL; W p. 145/T Table 2-10 specify H7=H7, H0→L7, L0→CY | These statements are reconcilable if carry is prepared from old H7 before shifting the high byte, then from old H0 before shifting the low byte. FC “The carry logic” also describes a temporary carry latch for LDHI/LDSI, but neither post publishes ARHL's carry-preparation micro-sequence or says that ARHL uses that latch. No demonstrated conflict exists. Admit the written operation under the licence; the reconciliation is a possible sequence, not die confirmation of sign fill | **S, second-source operation; compatible die input path**, Intel-unverified |

LDHI/LDSI say immediate **byte**, not signed displacement. Adopt zero extension
of b2 and 16-bit wrapping for pair results, addresses and PC/SP arithmetic as
**library width/operand rules (hardware edge cases unverified)**. W p. 144's
8-bit-index example supports the unsigned reading but neither W/T nor the die
posts explicitly discriminates 80h–FFh sign extension. Likewise, unchanged
non-destination registers follow the written assignment interpretation; no
independent exhaustive register-preservation measurement is claimed.

## 3. Bus order and access kinds

The following sequences are **adopted library API mappings (hardware temporal
order and exact access set unverified)** under decision-5 item 3/decision-12.
W/T establish the byte placements in §2, not the order of physical transactions,
dummy activity or pin-status classification. `p`, `s`, `d` are old PC, SP, DE;
all address arithmetic wraps by the library width rule.

| Instruction / path | Adopted API sequence | Evidence and limit |
| --- | --- | --- |
| LHLX | `OpcodeFetch(p)`; `Data` read d into L, then d+1 into H; PC=p+1 | W p. 145/T row placement, ROM row 12 decode only. Low-before-high, no extra/dummy reads: library mapping |
| SHLX | `OpcodeFetch(p)`; `Data` write L at d, then H at d+1; PC=p+1 | W's completion of T, ROM row 11 decode only. Low-before-high, no extra/dummy writes: library mapping |
| RSTV taken | `OpcodeFetch(p)`; `Stack` write high(p+1) at s−1, then low(p+1) at s−2; SP=s−2; PC=0040h | W/T byte placement and vector. Saved **next** PC, high-before-low and no dummy calls: restart API mapping, not a die-traced sequence |
| RSTV untaken | `OpcodeFetch(p)`; PC=p+1; no stack or dummy calls | W/T sequential path; exact access set is library mapping |
| JK/JNK taken | `OpcodeFetch(p)`; `OperandFetch(p+1)` low, then `OperandFetch(p+2)` high; PC=a16 | W/T byte positions; fetch sequence/kinds are library mapping |
| JK/JNK untaken | `OpcodeFetch(p)`; `OperandFetch(p+1)` only; skip byte 3, PC=p+3 | **Library mapping** chosen to match the existing 8085 conditional-jump API. Neither the 2/7 total nor ROM row 32 proves that hardware skips this read |

For LDHI/LDSI use `OpcodeFetch(p)` then `OperandFetch(p+1)`; for the other
one-byte instructions use one `OpcodeFetch`. These are API mappings, not claims
that every machine cycle entails a bus call. LHLX/SHLX stay `Data` even if DE
numerically equals SP. Normally fetched RSTV has no `InterruptAcknowledge`.
The trace policy makes the admitted instructions implementable without claiming
that this corpus settles their physical bus order.

## 4. V/K and the PSW

### Storage and K conflict

Admit seven independent booleans in the **per-model** 8085 flag type; image
`S Z K AC 0 P V CY`: V at bit 1, K/X5/UI at bit 5, no bit-3 storage, output
bit 3 always zero. W p. 145/T Fig. 2-9 are the written key; KV opening list,
flag latch/bus discussion and superbuffer section, and FC's opening/storage
sections independently confirm it: **die analysis (8085; exact part unverified)**.
The second-source statements also carry **documented by second source
(CA80C85B), unverified for Intel 8085**. No 8080 filler bit or shared flag type.

W p. 145 and T Fig. 2-9 print `K = O1·O2 + O1·R + O2·R`; W complements O2 for
subtraction/comparison, while T's replacement line does not visibly distinguish
the replacement. KV's circuit instead gives **K = V XOR result_sign**, and
KV note [4] identifies the both-negative XOR failure. **Silicon wins:** admit
the die equation as the **die-analysis conflict resolution** of the licensed
second-source K rule; do not block it merely because the losing formula cannot
be an agreeing key. This is the maintainer's explicit ruling. It does not
turn two members of the die lineage into independent sources.

ADD/SUB can be reconciled algebraically with the printed formula, but a derived
counterexample is XRA with A=80h and operand=80h: result=00h, V=0, die K=0,
printed K=1. This is a discriminating witness specification, not a new silicon
measurement. K is recomputed by this equation **only where updated**, not as
an invariant over all stored flags. INX/DCX use the separate incrementer path;
POP can load S, V, K independently.

V is the XOR of internal carries into/out of the top ALU bit (KV circuit),
not the exposed subtraction borrow CY. W/T define signed overflow for both
8- and 16-bit arithmetic. Whole-word overflow below uses the high-byte operation
including the propagated low-byte carry/borrow, not two unrelated byte flags.

### Documented instructions: chosen V/K effects

**D** means **die analysis (8085; exact part unverified)**, with the W/T
written rule and independent die key stated below. **S** means **documented
by second source (CA80C85B), unverified for Intel 8085**, including the licensed
rule corrected by silicon. **L** means **adopted library rule; exact hardware
effect unverified**. L may follow an explicit single-origin die reading;
that does not assert the missing independent key exists. All rows are chosen
model behaviour; no row silently leaves a flag indeterminate. `Sr` is the
computed result's sign, including a temporary CMP result, rather than necessarily
the architectural S latch.

| Instruction group (all register/memory/immediate forms) | V chosen | K chosen | Keys, label and reason |
| --- | --- | --- | --- |
| ADD/ADI/ADC/ACI | Signed 8-bit overflow, including carry input | V XOR Sr | **D/S**: W arithmetic definition, T Fig. 2-9; KV circuit; ALU `store_v_flag`/`arith_to_flags`. Arithmetic specialisation, K conflict resolved as above |
| SUB/SUI/SBB/SBI, CMP/CPI | Signed 8-bit subtraction overflow, including borrow where applicable | V XOR Sr | **D/S**: W complemented-O2 rule/T Fig. 2-9; KV circuit/note [5], ALU enable lists. CMP keeps A; K for ordinary CMP denotes signed A<operand |
| INR r/M | 1 iff old byte=7Fh, else 0 | V XOR Sr | **D/S**, derived +1 specialisation: W/T arithmetic rules; KV note [4], ALU enables. FF→00 does not set V; CY remains preserved |
| DCR r/M | 1 iff old byte=80h, else 0 | V XOR Sr | **D/S**, derived −1 specialisation: same sources; KV note [4] explicitly distinguishes 00→FF from signed overflow; CY preserved |
| INX BC/DE/HL/SP | Preserved | 1 iff old pair=FFFFh, else 0 | K **D/S**: W p. 144/T Fig. 2-9; KV incrementer circuit, ALU `carry_to_k_flag`. V **L**: neither states an exact V effect; omission from the V-enable list is not a second key |
| DCX BC/DE/HL/SP | Preserved | 1 iff old pair=0000h, else 0 | K **D/S**, same incrementer evidence; V **L**, same preservation rule |
| ANA/ANI, ORA/ORI, XRA/XRI | 0 | Sr | **L, following explicit die reading** KV note [4] and ALU constant-carry/enables. T's general arithmetic flag description supports the flag functions, not an independent exact logical-update statement; hardware effect remains unverified under two keys. For XOR the selected K equation overrides the printed universal formula |
| RRC/RAR | 0 | Preserved | V **L, following explicit die reading** KV note [4]/ALU V enable; no independent exact rotate key. K **L**: neither gives an explicit rotate K effect; preserve, do not recompute from the unchanged S latch |
| RLC/RAL | Signed overflow of A+A (+ old CY for RAL) | Preserved | V **L, following explicit die reading** KV note [4]/ALU addition/V controls. K **L**: no explicit K update statement, preserve |
| DAA | Signed 8-bit overflow of old A + the combined correction (00h/06h/60h/66h), defined below | V XOR Sr | **L**, following ALU's combined ACT correction and V circuit. KV note [4]'s “top digit goes from 7 to 8” is a disagreeing/approximate reading, not the chosen predicate: 20h + 60h = 80h also overflows. Exact hidden-flag behaviour lacks an independent key; existing five-flag DAA derivations stand |
| DAD BC/DE/HL/SP | Signed 16-bit overflow from the high-byte addition | Preserved | V **S / die-reading reconciliation**: T Fig. 2-9's 16-bit arithmetic V, W arithmetic definition and KV note [4]'s explicit DAD statement take precedence over ALU's omission of DAD from `store_v_flag`. An omitted list item is not an explicit measured preservation result. K **L**: no exact DAD K statement, preserve |
| POP PSW | Load input bit 1 | Load input bit 5 | **L, following explicit die reading**, detailed below; independent arbitrary inputs, not a recomputed equation |
| PUSH PSW and all other documented instructions | Preserved | Preserved | **L** wherever neither source states an effect: includes data transfer, CMA/CMC/STC, branches, calls/returns, RIM/SIM, DI/EI, NOP/HLT and ordinary interrupt entry. Internal ALU/databus use alone does not imply an update |

For DAA, let `a` be old A. Select `lo=06h` iff `(a & 0Fh)>9` or
old AC, otherwise `lo=00h`. In widened arithmetic let `t=a+lo`; select
`hi=60h` iff `(t & F0h)>90h`, old CY, or `t>FFh`, otherwise `hi=00h`.
This retains the existing five-flag rule's low-adjustment carry propagation.
Set `correction=lo+hi` and `r=(a+correction) & FFh`. ALU “The accumulator
and ACT registers” describes one ACT correction selected from 00h/06h/60h/66h
with the two nibble enables; adopt overflow of that combined ALU addition.
The **L** rule is `V = ((~(a ^ correction) & (a ^ r) & 80h) != 0)` and
`K = V XOR ((r & 80h) != 0)`. Incoming CY selects the correction; it is
not an extra addend. With old CY=1, AC=0, `20h+60h=80h` gives V=1,K=0,
although the top digit changes 2→8, contradicting KV note [4]'s literal
7→8 restriction. With old CY=0, AC=0, `7Ah+06h=80h` also gives V=1,K=0;
its overflow is retained even though no high-digit correction is selected.
These are library-rule examples, not silicon measurements.

These L rows adopt stated die readings and fill preservation gaps without
claiming new hardware authority. They are not a general waiver of decision-17.
Later Intel evidence may revise them; admission of the second-source instruction
set and timings does not depend on falsely upgrading them to two-key facts.

### Undocumented instructions: flags and explicit conflicts

| Instruction | Selected V/K and documented-flag mask | Evidence / resolution label |
| --- | --- | --- |
| DSUB | Update all seven (**S** mask): V signed 16-bit HL−BC overflow (**S**); K=V XOR sign16(result) (**S**, silicon-corrected); Z iff the whole 16-bit result is zero (**D/S**); S/P/AC from the high-byte step and CY from the final borrow (**L**, equations below) | W all-flags completion of T's truncated mask; T Fig. 2-9 16-bit V, silicon-corrected K. FC confirms combined Z. ALU `arith_to_flags` includes DSUB but `store_v_flag` omits it: take explicit all-flags prose over an omission, **second-source mask reconciliation**. ALU's eight slices and non-A subtraction path support bytewise arithmetic but do not publish DSUB's full micro-sequence or exact P/AC latch selection; the deterministic choices below are library rules |
| ARHL | CY ← old L0; V ← 0; K and S/Z/AC/P preserved | W/T say CY only, but ALU explicitly includes ARHL in `store_v_flag`; its shift-right operation has constant internal carries, yielding V=0. **Die-analysis override**, V value **L (die-derived, exact hardware unverified)**; do not silently retain the losing CY-only V mask. K/other preservation **S** from the written mask |
| RDEL | CY ← old D7; V signed overflow of DE+DE+old CY; K and S/Z/AC/P preserved | **S**, W/T CY,V plus T Fig. 2-9's 16-bit V; derived rotation-as-add reading supported by ALU addition/carry controls. ALU omits RDEL from `store_v_flag`: select explicit W/T update rather than infer preservation from omission, **second-source mask reconciliation** |
| LDHI/LDSI | All seven preserved | **S** W/T “none”; CY additionally **D**, FC explicitly uses a temporary carry latch to preserve it |
| RSTV, SHLX, JNK, LHLX, JK | All seven preserved | **S**, W/T “none” (including W completion of SHLX); no independent complete flag-latch trace claimed |

DSUB uses the following explicit **L byte-step rule**: subtract old C from
old L with no incoming borrow, then subtract old B and the low-step borrow
from old H. ALU “Building the 8-bit ALU from single-bit slices” establishes
an 8-bit ALU; “The accumulator and ACT registers” provides DSUB's non-A
path and distinguishes the separate 16-bit INX/DCX incrementer. Neither
section publishes DSUB's full low/high sequencing. The chosen sequence is
implementable without claiming that missing hardware evidence.

Let `b=1` iff old L < old C, otherwise 0; let `l=(old L-old C) mod 256`
and `h=(old H-old B-b) mod 256`. All comparisons below use widened operands:

- **L, S:** set iff bit 7 of `h` is 1.
- **L, P:** set iff `h` has even parity; do not combine both bytes' parity.
- **L, AC:** use the high-byte subtraction's bit-3 carry, the existing 8085
  no-low-nibble-borrow convention: set iff `(old H & 0Fh) >= (old B & 0Fh)+b`.
- **L, CY:** set iff `old H < old B+b`, the final borrow, equivalently
  unsigned old HL < old BC; incoming CY is ignored.
- **D/S, Z:** set iff `h=0` and `l=0`; FC's storage discussion explicitly
  confirms DSUB's multi-byte AND of zero results.
- **S, V/K:** use the signed 16-bit subtraction overflow and silicon-corrected
  K equation selected above, with `h`'s bit 7 as the result sign.

The exact DSUB P/AC selection and byte-latch sequencing remain **unverified
hardware subfacts**. These labelled choices do not withdraw its operation,
length, all-seven mask or 3/10 admission.

### PUSH PSW / POP PSW, reset and Default

PUSH emits K5,V1,zero3 with the existing documented A/flag-byte placement;
W/T diagrams plus KV/FC bus drivers give **D**, with the second-source label
retained for model scope. PUSH preserves V/K as **L**, the unstated-effect
policy in the documented-instruction table; the image evidence does not
establish hidden-flag preservation.

POP loads seven independent flags, including V from input bit 1 and K from
input bit 5, and ignores bit 3. `Registers::set_psw` uses the same conversion;
`psw` packs A high, the seven-flag byte low. KV's circuit section explicitly
states both hidden databus loads; FC storage and ALU `bus_to_flags` confirm the
path but are the **same origin**. W/T diagrams independently give positions,
not exact POP destinations. Therefore choose this as **L, following explicit
die reading; exact POP hardware behaviour unverified under two keys**. It is
adopted, not blocked pending another source. Never reconstruct K from S/V on
POP; contradictory loaded combinations must round-trip, apart from bit 3.

`Default` clears V and K, like the existing flags; reset preserves V and K,
like the other register flags under decision-12 rule 9. Both are **library
construction/reset rules**, not power-up or reset-silicon claims. Bit 3 always
packs zero. No additional reset bus accesses or elapsed time are introduced.

## 5. Integration and oracle consequences

**READ current code:** `i8085/flags.rs` stores five booleans, outputs fixed bit 1
and ignores input bits 1/3/5. `state.rs::{psw,set_psw}` delegates that conversion;
`stack_io_control.rs` uses it for PUSH/POP. `alu.rs::{result_flags,add,subtract}`
construct only five flags; arithmetic/logical executors replace or selectively
update them. The admitted seven-flag design requires operation-specific
update/preserve rules from §4, including INR/DCR versus INX/DCX, and independent
witnesses. Reusing the five-flag result constructor cannot determine V/K.

**decision-16:** preserve distinct model flag types and keep 8085-only facts in
`i8085/flags.rs`/`alu.rs` or model-owned execution arms. Shared PUSH/POP must use
the model conversions. Its closed `FlagRules` list does not automatically gain
V/K methods. Operations that preserve some hidden flags may need model-owned
arms; a new family hook needs the decision-16 amendment it already requires.
No new 8085 fact becomes a shared 8080 literal merely because both use an ALU.

**decision-12:** a normally fetched RSTV is a conditional **software** restart
to **0040h**, not RST 0 (0000h), RST 7 (0038h), or an external interrupt vector.
That operation is **admitted**, labelled
**documented by second source (CA80C85B), unverified for Intel 8085**. Do not
pass CB through `opcode & 38h`, which cannot produce 0040h. The existing
restart helper's explicit target is the appropriate API seam; its saved PC is
next-instruction PC by the §3 API mapping, not INTR's boundary PC.
Neither written entry explicitly settles interrupt latches. Preserve INTE and
all interrupt-source latches by **library rule (hardware unverified)**; retire
RSTV as an ordinary instruction for the existing EI-delay bookkeeping. Do not
disable INTE as on external acceptance. This record does not add RSTV to INTR: decision-12 rule 5 still accepts documented
RST n and CALL only, with other acknowledge bytes returning Unsupported.
External priorities, masks, sampling, vectors and acknowledge kinds stand.

**8085EXM:** locally re-read the pinned `8085EXM.MAC` (May 2020 header;
SHA-256 `caf4bf988ee3127947e5d22877cd0a120b5d2acabbb6d0eefc39bc82d296172e`).
Lines 38–39 explain D5h; all 25 descriptor masks at lines 186–378 are D5h;
lines 390–392 patch `flgmsk`, and lines 846–855 mask the flag snapshot before
CRC accumulation. D5h=11010101b excludes 2Ah, exactly bits 5/3/1. Therefore
changing **only** those bits cannot change its judged CRCs; it neither licenses
nor tests V/K, POP's hidden destinations, or the ten opcodes. The harness even
executes DAD while saving SP before PUSH PSW (lines 810–825), so its raw saved
hidden flags would not be an instruction-local witness without changes.
This is a static non-regression argument for the hidden-bit image, not a new
oracle run or a guarantee that a future executor preserves the other flags.
Keep the 24 judged groups and reported-only long ALU group qualification in
PROVENANCE §2; no expectations, mask or oracle result changes here.

## 6. Supersession, limits and follow-through

On acceptance:

- Supersede decision-11's primary-only admission restriction and blanket refusal
  of these ten encodings with their **ADMITTED** operations, lengths, timings
  and selected flag rules here, under decision-17 as amended above.
- Replace decision-11's fixed bits 1/5 with independently stored V/K and the
  chosen PUSH/POP/conversion rules. Bit 3 remains output zero, now supported
  by written and die evidence; there is no eighth flag. Replace its rejection
  of more than five stored flags. Default/reset follow §4's library policy.
- Preserve decision-11's documented operand-marker reconciliation, typed
  Unsupported transaction for remaining unadmitted behaviour, all 8080 rules
  and decision-12's external interrupt contract. Keep model-owned flag types
  and decision-16's closed sharing boundary.
- Keep D5h oracle comparisons for documented flags. Add unmasked independent
  witnesses for hidden flags, arbitrary POP values, XOR's K conflict, signed
  overflow boundaries, 16-bit INX/DCX wraps and all conditional paths; keep
  source labels on witnesses. Oracle success does not supply a missing key.

This record selects admitted behaviour; it changes no code and claims no new
hardware measurement or executed test. Remaining gaps concern Intel specimen/
revision confirmation, detailed hidden-flag subfacts and physical bus traces,
not a requirement for a second timing key before using the licensed totals.

## Alternatives

- Treat decode labels as proof of complete operations or timings. Rejected:
  decode establishes only the encoding; the second-source licence supplies
  the documented operations and totals.
- Retain the old blanket timing refusal despite manufacturer documentation.
  Rejected by the maintainer's 2026-11-13 amendment; retain the Intel-unverified
  label while admitting those timings to the model.
- Count T independently from W, or the four die pages as four confirmations.
  Rejected: copied/shared origins count once.
- Use the printed K formula universally, or passing masked CRCs as authority.
  Rejected: silicon wins the K conflict; an oracle cannot state missing facts.

## Revisit when

Pinned independent Intel evidence identifies the die/revision, changes the
second-source readings or totals, settles physical bus/access traces, resolves
DSUB P/AC and the hidden-enable discrepancies, or supplies exact POP and
single-origin flag confirmation. Replace labelled library rules explicitly;
do not silently expand silicon scope.

## Reading audit

Revision 1: all six pinned hashes rechecked. Re-read rendered W PDF 1–2 and
T PDF 1–2 (title/colophon), 25 (2-1 compatibility), 28 (2-4 extended set),
39 (2-15, Fig. 2-9) and 40 (2-16, Table 2-10), and Intel 205775-002 PDF
100–101 (PSW), 105 (opcode table). T's printed 2-n map is **24+n**. Re-read the decompressed
KV/FC HTML flag, circuit, carry, storage and note sections, ALU architecture/
control tables, and ROM introduction/48 rows. Python `gzip` was used for both
compressed captures. No linked timing ROM, network or other emulator was read.

The original draft's reading audit additionally records Intel 205775-002 page
images 91–97; this revision retains that earlier audit without claiming fresh
reading of those arithmetic/logical page images.
Re-read decisions 5, 11, 12, 16, 17, the current i8085 flags/ALU/state and reset
paths, and the pinned 8085EXM mask/snapshot path. Sources and rendered reading
aids remain outside the repository; no fetched source bytes are redistributed.
