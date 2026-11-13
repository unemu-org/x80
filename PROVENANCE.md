# Provenance

## 1. Authority and citations

Derive hardware behavior from primary sources, cited by document, edition/revision,
section or table, and printed page (also PDF page if different). Pin revision and
SHA-256 before implementing a fact; check scans against page images. Comments
also link the relevant design section. Each implemented fact needs an independent
witness test with expected values derived from that citation.

### Pinned sources

All are fetch-only and cite-only (© Intel, Zilog or Mostek, no redistribution grant).
Their entries in `oracles/MANIFEST.md` carry the URL, page count and page-to-PDF map; fetch them with
`./oracles/fetch-oracles.sh fetch` and check them with `verify`. Historical sources are
scans with a noisy OCR text layer: check every cited value against the page image.

| Path in `oracles/` | Title | Order number, date | SHA-256 | Role |
| --- | --- | --- | --- | --- |
| `intel/i8080-um.pdf` | Intel 8080 Microcomputer Systems User's Manual | 98-153B, September 1975 | `d5982f96536b1dbecbeff409d2370970638716296dce7b318624d86221597a9e` | Primary: CPU chapter (interrupt, halt, start-up, per-instruction machine-cycle/state table), instruction set and flags, 8080A data sheet |
| `intel/i8080-alpm.pdf` | Intel 8080/8085 Assembly Language Programming Manual | 9800301-04 (title page; mirror file name says 9800301D), May 1981 (printing code 581 on p. ii) | `c2f17d613159b624cf7226b0b9200192a49ff172147d88f56d50f45e94691643` | Second witness: per-opcode cycles and states, flags, interrupts; for the 8085 too |
| `intel/i8085-um.pdf` | Intel MCS-80/85 Family User's Manual | 205775-002 (cover), January 1983 (title page) | `5d08f75c75605fbda22afb5071c6a6a93f3e797aa7aa980fa658098ab0651325` | Primary for the 8085A (M3): Ch. 2 functional description (interrupts, RIM/SIM, SID/SOD, machine cycles, reset/halt/hold), Ch. 5 instruction set with 8085A and 8080A states (Table 5-1) |
| `intel/intel-embedded-controller-handbook-1988-v1.pdf` | Intel Embedded Controller Handbook, Volume I, 8-bit (1988) | contains the 8085AH data sheet 231718-001, September 1987, printed pp. 16-11 to 16-30 (PDF 958 to 977) | `796ba5e7273bd4bfbe745b771062e4be8666037af0cc1742a69d61c1476941fb` | Second source for the 8085 (M3): pins, interrupt table, machine-cycle chart, instruction summary (new instructions: RIM, SIM); no per-instruction states |

All are Intel documents: the second of each pair is independent in authorship, not a
hardware witness. The 8085AH data sheet (p. 16-11) states "100% Software Compatible with 8080A"
and lower power than the 8085A, but no instruction-level difference from the 8085A.
Treating the 8085A and 8085AH as one model is an assumption (decision-6: no variant
without a pinned source documenting a difference), not a stated equivalence.

Page maps for the 8085 sources: 205775-002 printed 2-n = PDF 23+n and 5-n = PDF 85+n
(Table 5-1 is p. 5-19, Table 5-2 is p. 5-20); the 1988 handbook printed 16-n = PDF 947+n.
205775-002 p. 5-10 is a third Intel statement of the 8080 rule that decision-8 settled:
"The CY flag is cleared and AC is set (8085). The CY flag is cleared and AC is set to the
OR'ing of bits 3 of the operands (8080)."

Where they disagree, an oracle may falsify one reading; record the case
below and in a bead.

| Disagreement | Citations | Status |
| --- | --- | --- |
| Length of RET (C9) and JZ (CA) | 98-153B pp. 4-11, 4-12 and 9800301-04 pp. 3-32, 3-48: RET one byte, JZ three bytes; 9800301-04 p. A-5 prints "RET Adr" at C9 and "JZ" without "Adr" at CA | Resolved for the opcode map: p. A-5 misprint (operand marker one row off); the per-instruction pages of both manuals agree. Witness pins both bytes |
| CY after DAA when CY was set before DAA, or when the low adjustment carries out of bit 7 | 98-153B p. 4-8: two-step rule, "All flags are affected"; p. 4-2 general rule: CY set on a carry out of the high-order bit, "otherwise it is reset"; 9800301-04 p. 6-13: DAA's carry is needed "for adding the higher-order bytes" (MADD routine, pp. 6-12 to 6-14; subtraction routine pp. 6-14 to 6-17) | Not stated explicitly. Derived (maintainer, 2026-11-12): CY is kept from before DAA and set by a carry out of bit 7 from either step, which then triggers the high adjustment; the documented decimal routines are correct only this way. Not falsified: 8080EXM's "daa,cma,stc,cmc" group passes (2026-11-12), subject to the CRC provenance noted in §2 |
| 8085 CY retention and propagation of the low adjustment's carry in DAA | 205775-002 §5.6.2 p. 5-9 (PDF 94): step 2 tests the high digit "now greater than 9" or CY; "All flags are affected". §5.5 p. 5-3 (PDF 88): CY reports carry, "otherwise it is reset". 9800301-04 p. 3-19 (PDF 73) gives the same two steps; pp. 6-12 to 6-14 (PDF 178-180) require decimal carry for higher bytes. Page images checked for this model | Derived (x80-4gk.12, maintainer-authorized rule): retain incoming CY and set CY on bit-7 carry from either adjustment; step 1's carry triggers step 2. Neither instruction entry explicitly settles retention. Re-derived from the decimal routine's purpose: 99+99 needs 32H,CY=AC=1 -> 98H,CY=1; 95+65 needs FAH -> 00H with carry -> 60H,CY=1. Witnesses: `i8085_daa` exhaustive two-step transcription, literal examples and all decimal sums; no 8085 hardware/oracle result claimed |
| EI delay edge cases: EI followed by EI, DI after EI, and RESET during the delay | 98-153B p. 4-14: EI enables "following the execution of the next instruction", DI disables "immediately following the execution of the DI instruction"; 9800301-04 pp. 3-22, 3-23 say the same; p. 5-14: RESET clears INTE. None addresses a second EI, DI cancelling a pending delay, or RESET clearing the delay | Unverified model (x80-ecy.13): a second EI re-arms the delay, DI clears it, RESET clears it. Labelled in code and tests |
| Machine cycles of HLT | 98-153B p. 2-18 state table: M1 and M2; 9800301-04 p. 3-24: one machine cycle; both give 7 states | No effect on x80, which counts states only; HLT takes 7 states, witnessed |
| Auxiliary carry after 8080 logical AND (ANA, ANI) | 98-153B p. 4-9: ANI clears CY and AC; 9800301-04 p. 1-12: AND sets AC to the logical OR of bit 3 of the operands | Settled by decision-8: AC = bit 3 of (A OR operand), per 9800301-04 p. 1-12. Oracle evidence, MEASURED 2026-11-12: 8080EXM's two ALU groups pass with this reading and fail with "AC cleared" (7799ea9d, b3491c2a); the pinned results page (`oracles/8080-exercisers/8080-8085-cpu-exerciser-results.pdf` pp. 3-4) reports the expected CRCs for Intel 8080A and other parts and the failing ones for AMD 9080A/8080A |
| 8085 states of CPE (EC) | 205775-002 p. 5-14 (Ccondition) and Table 5-1 p. 5-19: 9 states untaken, 18 taken; 9800301-04 App. A p. A-1, 8085 column: 9/17 | Resolved for the opcode map (x80-4gk.9): the primary's entry and its summary table agree on 9/18, as do the other Ccond entries; p. A-1's 17 is treated as a misprint. Witness `cpe_appendix_a1_timing_disagreement_is_explicit` pins both readings |
| 8085 states of PUSH rp (C5, D5, E5) | 205775-002 p. 5-15: "12 (8085)"; Table 5-1 p. 5-19: 12; 9800301-04 App. A p. A-2: 12; 9800301-04 p. 3-44: "11 (13 on 8085)" | Resolved (x80-4gk.15): 12, the primary's entry and table, also the second witness's own appendix; p. 3-44's 13 is treated as a misprint. PUSH PSW is 12 in every source |
| Encoding of LDAX B | 231718-001 Table 6, handbook p. 16-28: "00101010" (2Ah); 205775-002 p. 5-5: "00RP1010" with RP = 00 for B (p. 5-1), Table 5-2 p. 5-20: 0A LDAX B; 9800301-04 p. 3-33: "000r1010" | Resolved (x80-4gk.11): 0Ah; the data sheet's table is a misprint (2Ah is LHLD in every source, the data sheet included). Noted in code and tests |

### NEC V20/V30 8080 emulation mode (M2)

Pinned by the x86jit project in the shared stash (oracles 1431bbc, bead xj-vao (x86jit)) and
used by x80 for M2: `nec/v20-v30-um.pdf`, NEC uPD70108/uPD70116 (V20/V30) User's Manual,
Stock No. 500350, October 1986, 228 pp, sha256
`8252fe5ddc0623380b7f28819009e6bb7d2b8e345ef347d9e6f2d299ce97865f`, © NEC, cite-only.
Page map: printed 8-n = PDF 31+n; 3-3 = PDF 16; 6-1 = PDF 26; 9-1 = PDF 38;
12-151 = PDF 196; 12-168 = PDF 213; 12-169 = PDF 214; 12-176 = PDF 221; 12-177 = PDF 222.
A scan with an OCR text layer: check every value against the page image.

Facts read from the page images (M2 analysis, 2026-11-12):

- Register mapping (p. 8-1, Figs 8-1, 8-2): the 8080 A is the low byte of AW; HL, BC and
  DE are BW, CW and DW; the 8080 SP is BP; PC is PC. The 8080 flags are the low eight bits
  of the PSW, S Z 0 AC 0 P 1 CY. AH, the native SP, IX, IY and the segment registers PS,
  SS, DS0, DS1 are not addressable from emulation mode. Which byte of BW is H is drawn in
  Fig. 8-1 only.
- Segments (p. 8-1): "the segment base of the program is determined by the PS register";
  "the segment base of the memory operands (including the stack) is determined by the DS0
  register". In x80 terms: OpcodeFetch and OperandFetch use PS, Data and Stack use DS0.
- Instruction set (p. 8-1): 8080 mode executes "the instruction set for the uPD8080AF and
  the RETEM and CALLN instructions"; native mode executes all instructions except RETEM
  and CALLN. BUSLOCK and POLL are unavailable; I/O and memory bus operation is as in native
  mode.
- Encodings: CALLN = ED ED imm8 (p. 12-176); RETEM = ED FD (p. 12-177); BRKEM = 0F FF imm8
  (p. 12-169). Clocks given by NEC (V20 / V30 odd / V30 even): CALLN 58/58/38, RETEM
  39/39/27, BRKEM 50/50/38, RETI 39/39/27 (p. 12-168); NMI and INT in Table 6-1 (p. 6-1).
- Interrupts (p. 8-4, Fig. 8-6; p. 6-1): NMI and INT in emulation mode are serviced as in
  native mode; the routine runs native and RETI returns to emulation. The frame (PSW with
  MD = 0, PS, PC) goes on the native stack.
- HLT (pp. 8-2, 9-1): standby; INT with interrupts disabled leaves standby and resumes 8080
  mode after the HLT without service; NMI is unconditional; RESET is a native reset and
  ends emulation.
- BRKEM (p. 12-169, p. 8-2) and RETEM (p. 12-177, Fig. 8-7): BRKEM pushes PSW, PS, PC and
  enters emulation with PS:PC from the vector; RETEM pops PC, PS, PSW, so the 8080 flags do
  not carry back to native mode; registers do. CALLN (p. 12-176) calls native code, which
  returns with RETI. BRKEM inside a native routine reached from emulation is not allowed
  (p. 8-5).
- Interrupt-routine entry (p. 6-1): the general sequence pushes PSW, PS, PC, then
  "IE <- 0, BRK <- 0, MD <- 1", and lists BRKEM and CALLN among the vectored sources. For
  CALLN this agrees with p. 12-176 (MD <- 1), so CALLN clears IE and BRK. The 8080 interrupt
  enable is PSW.IE: Fig. 8-2 shows IE as PSW bit 9, and p. 9-1 uses IE for the emulation-mode
  INT and HLT rules.

Not stated in the manual, so not modelled as facts:

- No 8080 instruction clocks for the V20 and no 8080 flag rules (AC after AND, DAA,
  parity); under the V20 x80 reports Intel 98-153B T-states, which are not V20 clocks.
- ASSUMED: the uPD8080AF behaves as the Intel 8080A, including decision-8 and the derived
  DAA carry rule. No uPD8080AF source is pinned.
- To confirm jointly with x86jit: IE after BRKEM (p. 6-1's general sequence would clear it
  but sets MD <- 1, while p. 12-169 sets MD <- 0 and lists only MD, so the general sequence
  does not describe BRKEM exactly); whether the native EI delay (p. 6-4; p. 12-172) applies
  to the 8080 EI in emulation mode; the PC that CALLN pushes; the high address byte during
  8080 IN/OUT.
- Misprints noticed: p. 12-176 "SP <- PS - 6"; p. 12-177 refers to BRKEM where RETEM is meant.

### Z80 sources (M4: documented behaviour only)

Accepted by the maintainer, 2026-11-13: UM008011-0816 is the primary; the four
historical manuals below are second witnesses. All five are pinned fetch-only in
the shared stash; the URLs and page counts are in `oracles/MANIFEST.md`.

| Path in `oracles/` | Title | Document number, revision/printing, date; PDF pages read | SHA-256 | Role |
| --- | --- | --- | --- | --- |
| `zilog/z80-um.pdf` | Z80 CPU User Manual | UM008011-0816, revision 11, August 2016; title/copyright PDF 1–2, revision history iii/PDF 3; 332 PDF pages | `e3c83da5a5d8e372364c20fa53665e6fbb165ec6ac38c8c1eebc359603447b5e` | Primary: architecture, pin functions, timing, interrupt response, op-code tables, instruction entries with flags and states |
| `zilog/z80-tm-1977.pdf` | Z80-CPU / Z80A-CPU Technical Manual | 03-0029-01, ©1977; no separate printing date found; cover/colophon PDF 1–2; 83 PDF pages | `9c7304b4842dac6d34c9cd1552f006c2f1b6fa9373f7e6d1b614956d91efc485` | Second witness: §3 pins, §4 CPU timing, §6 flags, §7 op codes and execution times, §8 interrupt response, §12 instruction summary |
| `zilog/z80-alpm-1978.pdf` | Z80-Assembly Language Programming Manual | 03-0002-01 (back cover, PDF 315); January 1978 (contents, PDF 3); colophon PDF 2 marks “3.0 D.S.” and “REL. 2.1”, ©1977; cover PDF 1; 315 PDF pages | `d9d18bd2bc074d61faf6b46f985122e940925b04d37fa6480372b5c3794faa6c` | Second witness: §VI instruction encodings, descriptions, flags, machine cycles and T-states |
| `zilog/z80-ps-1978.pdf` | Z80-CPU / Z80A-CPU Product Specification | 03-0027-02 (PDF 10), March 1978 (PDF 1), ©1977 (PDF 10); 10 PDF pages | `98ef3a633edf3b1c9350c2b123e12608133b83c0ffb40233e0a2771f5293aee9` | Second witness: instruction summary pp. 4–5, timing diagrams and characteristic tables |
| `mostek/mk78515-z80-pm.pdf` | Z80 Programming Manual | MK78515, V2.0 (title PDF 3; cover PDF 1), printed October 1978, ©1978 (back cover PDF 312); 312 PDF pages | `1f5217c39e03aa2a55a087305649e2079f8b3d94a512a2e4d415d15b7dba43fd` | Second witness: §2 instruction entries with flags and states, Table 2-1 interrupt enable/disable, Appendix A opcode listing |

READ, 2026-11-13: titles, colophons and the page-map checks below were read on
page images rendered with `pdftoppm -r 150 -f N -l N -png`; OCR only located pages.
Counts include covers and blank pages. Zilog and Mostek retain copyright; the
scans and the vendor PDF grant no redistribution right. Cite them, never vendor them.

MEASURED by the supervisor, 2026-11-13: the live vendor URL
`https://www.zilog.com/docs/z80/um0080.pdf` answers 301 to a vendor landing page,
not a PDF. The Wayback CDX index records at least two different PDF digests at
that URL over 2020–2023. The pin is the vendor's own file archived at that vendor
URL in capture `20231217134046`:
`https://web.archive.org/web/20231217134046id_/https://www.zilog.com/docs/z80/um0080.pdf`.
Local metadata confirms title “Z80 CPU User Manual”, created 2016-08-31 with
Acrobat Distiller 9.0.0; local size is 1,587,333 bytes. The title page confirms
UM008011-0816 and ©2016 Zilog. The hash selects the edition, not the live filename.
The four historical scans use the same bitsavers mirror as the Intel and NEC pins;
this local-only pass did not compare them with bitsavers.org itself.

The user manual's revision history (iii/PDF 3) says revision 11 (August 2016)
changes formatting on pp. 39–41. Revision 10 (August 2016) adds the instruction
notation summary and corrects typos/errors; 09 (May 2016) and 08 (January 2016)
also correct typos/errors. Earlier entries identify corrections to POP qq
(2015, p. 119), the status-flags table bit 4 and EXX bit 0 (2014, pp. 65, 126),
RLCA's hex code and illustration (2005, pp. 55, 205), and IM 0/1/2 bit patterns
(2004, pp. 184–186). This history warrants the exact revision pin; it does not
establish that all remaining text is correct.

Page maps (printed → one-based PDF), checked on rendered images:

| Document / parts to cite | Rule | Images checked (printed → PDF) |
| --- | --- | --- |
| UM008011-0816: architecture, pin functions, timing, interrupt response; op-code tables; instruction entries | PDF = n+14 | 1 → 15; 5 → 19; 7 → 21; 11 → 25; 42 → 56; 64 → 78; 72 → 86; 305 → 319; 316 → 330 |
| 03-0029-01: §3 pin description, §4 timing, §7 instruction tables, §8 interrupt response, §12 summary | PDF = n+6 | 7 → 13; 11 → 17; 43 → 49; 53 → 59; 55 → 61; 57 → 63; 73 → 79; 75 → 81 |
| 03-0027-02: instruction summary and characteristic tables | PDF = n (pp. 1–9; last sheet PDF 10 has no printed number) | 4 → 4; 6 → 6 |
| MK78515: §2, including instruction entries 2-21–2-258 and Table 2-1 at 2-15 | printed 2-n = PDF 14+n | 2-15 → 29; 2-21 → 35; 2-258 → 272 |
| MK78515: Appendix A opcode listing (listing's own page numbers, 1–15) | PDF = 274+n | listing 1 → 275; listing 15 → 289 |
| MK78515: unnumbered Appendix D reference tables | Cite table number and PDF page | D-1/D-2 → 309; D-3/D-4/D-5 → 310 |

03-0002-01 §VI and its instruction entries need a piecewise map because the scan
includes unnumbered separator pages. Each range below was checked on two images:

| Printed pages n | PDF rule | Images checked (printed → PDF) |
| --- | --- | --- |
| 24–25 (instruction-set introduction and contents) | n+3 | 24 → 27; 25 → 28 |
| 26–98 | n+4 | 27 → 31; 98 → 102 |
| 99–145 | n+5 | 99 → 104; 145 → 150 |
| 146–162 | n+6 | 146 → 152; 162 → 168 |
| 163–201 | n+7 | 163 → 170; 201 → 208 |
| 202–218 | n+8 | 202 → 210; 218 → 226 |
| 219–251 | n+9 | 219 → 228; 251 → 260 (also 250 → 259; OCR misreads its footer as 260) |
| 252–274 | n+10 | 252 → 262; 274 → 284 |

Lineage caveat: all four second witnesses share Zilog's documentation lineage,
including Mostek's restatement. Agreement can catch retyping errors; it cannot
independently establish the design or rule out a shared design error. None is a
hardware measurement. Pinning these sources encodes no instruction behaviour.
Record future disagreements and their primary citations in this file's §1 before
resolving them; an oracle can falsify a reading but never license it.

### Discovery inventory

The list below is **not a set of pinned sources**. Unknown metadata is marked
`source to find` rather than guessed. URLs remain unset until confirmed.

| Candidate title | Order number | Year | Public source URL | Intended role |
| --- | --- | --- | --- | --- |
| MCS-80 User's Manual | 98-153D | 1977 | bitsavers mirror (not pinned) | Not pinned: a later edition that repeats the 98-153B CPU chapters; a possible check for corrections |
| MCS-85 User's Manual | 9800366E | September 1978 | not on bitsavers (not pinned) | Not pinned: its 8085A chapters carry the same text as 205775-002, so not an independent witness |

Sources live in the organisation's shared oracle stash, `unemu-org/oracles`, included here
as x80's own `oracles/` submodule with its own pin (maintainer decision, 2026-11-11; x86jit
pins the same repository independently). 8080-family sources are added to that repository
(its `MANIFEST.md` records name, edition/order number, sha256, licence and the facts each
source witnesses; `./oracles/fetch-oracles.sh verify | fetch | verify --upstream`), then the
submodule pin here is bumped. Fetch-only sources (no redistribution licence) are recorded by
hash, not committed.
Record source license/redistribution terms separately from the library's license;
a public scan alone grants no redistribution permission.

## 2. Oracles

Exercisers, diagnostic ROMs, other emulators and hardware captures can falsify a
claim; they never authorize a behavioral fact. Never obtain semantics from another
emulator's source. When an oracle disagrees, record initial state, bytes, measured
result, artifact pin and the primary citation resolving the discrepancy. Do not
silently mask a divergence.

### Pinned oracles

Chosen by the maintainer (2026-11-12, bead x80-ecy.3), pinned FETCH-ONLY in the shared
`oracles/` submodule (`8080-exercisers/`, URL and sha256 in its `MANIFEST.md` and
`SHA256SUMS.fetched`), run locally and never committed:

| Program | Origin | Licence | Role |
| --- | --- | --- | --- |
| 8080PRE (.COM, .MAC source) | Frank D. Cringle's preliminary test (1994), modified for the 8080 by Ian Bartholomew (undated) | GPL-2.0-or-later | Sanity pass before the exerciser |
| 8080EXM (.COM, .MAC source) | Cringle's instruction exerciser (1994), modified for the 8080 by Bartholomew (February 2009); Mike Douglas (May 2013) added the success CRCs shown on Bartholomew's website | GPL-2.0-or-later | 25 CRC groups comparing all eight PSW bits; falsifies flag readings; it settled the AND auxiliary carry (decision-8) and does not falsify the derived DAA carry rule |

Both are CP/M programs: they load at 0100h, print through BDOS 2 and 9 via `CALL 0005h`
and exit with `JMP 0000h`. 8080EXM sets SP from the word at 0006h; 8080PRE never sets SP
before its first `CALL`, so the host must start it with a usable SP. Neither executes an
undocumented opcode. The `x80-oracle` crate runs them (commands.md, "Oracle runs").
Results, 2026-11-12, MEASURED: 8080PRE passes; 8080EXM passes all 25 groups. The parts
8080EXM's expected CRCs were reported for are recorded on the author's results page, pinned
as `8080-exercisers/8080-8085-cpu-exerciser-results.pdf` (pp. 3-4: contributor measurements per
part, not Intel documentation); x80 cites it before treating a mismatch as a defect.

For the 8085 (M3, bead x80-4gk.6): `8085EXM` (.COM and .MAC source), the same exerciser
modified for the 8085 by Bartholomew (October 2009) with flag mask D5h, GPL-2.0-or-later,
pinned fetch-only under `8080-exercisers/` (oracles f379261). Its 24 expected CRCs other
than the long ALU register group equal the D8085AH-1 column of the pinned results page
(p. 3; checked value by value). The long ALU register group's CRC (0CFD75B5) was added in
May 2020 per the header from an unnamed 8085, so it is unverified; that group can be
reported, not judged. Unpinned, READ by research only: another exerciser build run on a
P8085AH-2 (2023 forum report) gives different CRCs for the ALU, DAA and INR/DCR groups;
whether that reflects a different build, mask or part is unresolved. 8080PRE also serves as
the 8085's preliminary test (the results page).

Results, 2026-11-13, MEASURED: the release `--model 8085` commands in
[commands.md, Oracle runs](backlog/docs/commands.md#oracle-runs) pass: 8080PRE
exits 0 at 1,060 steps / 7,740 cycles; 8085EXM exits 0 at 2,919,050,442 steps /
23,955,337,676 cycles, with all 24 judged groups PASS. The long ALU register group
(`alu8r`, 8085EXM.MAC:136,201–207) prints PASS, CRC 0cfd75b5; its result is
reported, not judged, because the expected CRC remains unverified. Both programs
halt at the boot-entry stub (PC 0001h); no core change was needed.

Rejected or not chosen: the Microcosm 8080/8085 CPU diagnostic (CPUDIAG/TST8080, 1980;
licence unclear, so fetch-only would be the most it could be), the SuperSoft CPU test
(commercial, binary only). No hardware-captured single-step corpus for the 8080 was
identified (searched 2026-11-12: the SingleStepTests and ProcessorTests collections);
emulator-generated corpora are not hardware witnesses. Hardware
captures describe the measured silicon, not every family member.

## 3. Witnesses and current scope

The core types, the run loop and the 8080 opcode map exist; every documented
instruction executes, and eligible interrupts are accepted with an acknowledged
RST (see backlog/docs/status.md). The opcode map (which bytes are documented, their
lengths and groups) is cited from 98-153B pp. 4-1 and 4-4 to 4-14 and witnessed
against the summaries on p. 4-15 and 9800301-04 p. A-5. Each implemented
instruction cites its behaviour, state totals and bus order in its group module
(98-153B Ch. 4 and pp. 2-16 to 2-20) and has witnesses with literal or
independently derived expectations. The register pairs and the flag-byte layout
of the processor status word are cited from 98-153B pp. 2-2, 2-3, 4-2, 4-3 and
4-13 and 9800301-04 p. 3-45. Interrupt acceptance cites pp. 2-11, 2-13,
2-18 to 2-20 and 5-14. Interim and derived readings are listed in §1.
Whole-set execution is witnessed by `tests/i8080_coverage.rs`, with literal
sets independent of the opcode map. `tests/i8080_determinism.rs` compares
mixed-program replays and budget splits, including a scripted interrupt and
all bus/input polls. `tests/boundary.rs` checks the dependency boundary and that
the model-neutral root files cite no manual; `compile_fail` doctests in `lib.rs`
keep the model's types off the crate root (decision-6).

### Residual unverified or derived facts at the M1 closure (2026-11-12)

None is presented as a stated fact; where each qualification is recorded is noted.

- Derived, not stated: CY after DAA (kept, and set by a carry out of either step);
  not falsified by 8080EXM. §1, `arithmetic.rs` and `tests/i8080_daa.rs`.
- Unverified model: the EI-delay edge cases (a second EI, DI during the delay, RESET
  during the delay). §1, `cpu.rs`, `stack_io_control.rs` and their tests.
- Derived from the 16-bit register width, not stated on a page: wrap of PC, SP and
  16-bit addresses at FFFFh/0000h. Code comments and wrap witnesses.
- Derived API representation: the 16-bit PSW value with A in the high byte, from the
  PUSH PSW stack order (98-153B p. 4-13; 9800301-04 p. 3-45). `state.rs` and
  `tests/i8080_registers.rs`.
- Library choices, not hardware claims: the `Default` register and latch values;
  98-153B p. 2-13 says the registers are indeterminate at power-up. `state.rs`,
  `cpu.rs`.
- Stated departure: an `Unsupported` interrupt acknowledge keeps the
  interrupt-enable latch, although p. 5-14 resets INTE at the acknowledge.
  decision-5 item 6 and `cpu.rs`.
- API-level, not a status-word distinction: the `OperandFetch` access kind.
  decision-5 item 3 and `bus.rs`.
- Not documented, so not modelled: the upper address byte during I/O; the port is
  the 8-bit device number. decision-5 item 3.
- The manuals differ on HLT's machine-cycle count (two versus one); no effect on
  states. §1 and `tests/i8080_timing.rs`.
The V20 mapping in [plan §3](backlog/docs/plan.md#3-v20v30-integration-contract) is
cited from the NEC manual (§1); its open items are listed there. Glossary definitions are orientation, not implementation
authority. Ordinary tests pin derived values in code and run without manuals,
corpus downloads or network access.

### Residual unverified or derived facts at the M3 closure (2026-11-13)

These qualifications concern the 8085 only; none is a stated hardware guarantee.
Module paths below are under `x80-core/src/i8085/`, witnesses under
`x80-core/tests/`. Source disagreements remain in the §1 table, including the
8085 DAA carry derivation; oracle qualifications and results remain in §2.

- Unverified reading: no interrupt, TRAP included, is recognized at the boundary
  immediately after EI or DI; consecutive EI/DI instructions re-arm that
  inhibition. decision-12 rule 6, `cpu.rs`, `stack_io_control.rs`,
  `i8085_machine_control.rs` and `i8085_interrupts_vectored.rs`.
- Unverified: RIM uses the stored interrupt-enable bit during the EI delay
  (unless TRAP's saved status applies). `rim_sim.rs` and
  `i8085_rim_sim.rs::rim_ie_bit_during_pending_ei_delay_is_unverified`.
- Unverified: reset clears the EI delay. decision-12 rule 9, `cpu.rs`,
  `i8085_model.rs` and `i8085_machine_control.rs`.
- Unverified bus detail: XTHL reads SP then SP+1 and writes SP+1 then SP;
  the cited 8085 sources give RRWW, not the address order within each pair.
  `stack_io_control.rs` and `i8085_stack_io_control.rs` label and pin the choice.
- Derived acceptance totals: internally vectored interrupts and INTR RST take
  12 states (6+3+3); INTR CALL takes 18 (6+3+3+3+3). decision-12 rules 4/5,
  `vectored.rs`, `acknowledge.rs`, `i8085_interrupts_vectored.rs`,
  `i8085_interrupts_intr.rs` and the independent `i8085_timing.rs`.
- Derived DAA carry rule: see the 8085 DAA disagreement row in §1, with its
  derivation and primary citations. `arithmetic.rs` and `i8085_daa.rs`;
  the qualified 8085EXM results are in §2.
- Derived subtraction AC: no low-nibble borrow, from the two's-complement
  bit-3 carry reading. `alu.rs` (205775-002 §§5.5/5.6.2 pp. 5-3/5-6;
  9800301-04 Ch. 1 pp. 1-9 to 1-12), its exhaustive tests and
  `i8085_arithmetic_reference/mod.rs`.
- Stated timing simplification: SIM updates SOD and notifies every enabled
  write during SIM itself, including an unchanged level. decision-12 rule 8,
  `interrupt.rs`, `rim_sim.rs` and `i8085_rim_sim.rs` record the departure from
  205775-002 §2.3.8 p. 2-19's transfer during the following instruction.
- API choice: consecutive boundary levels sample TRAP/RST 7.5 edges; shorter
  pulses can be missed. Stop and Unsupported preserve pre-sampling latches,
  so a pulse seen only there must still be held to be sampled next run; Halt
  retains samples. decision-12 rule 2 and its Stop/Unsupported clarification,
  `cpu.rs`, `interrupt.rs` and `i8085_run.rs`.
- Library choices: reset preserves working registers, flags and SP; sampler
  history and TRAP saved-status bookkeeping are reset. Reset has no bus
  parameter and sends no SOD notification. decision-12 rule 9, `cpu.rs`,
  `state.rs` and `i8085_model.rs`; the indeterminate hardware reset results
  are cited there from 205775-002 p. 2-18 and 231718-001 handbook p. 16-13.
- Library choices, not power-up claims: default registers and flags are zero;
  default latches set the masks, arm TRAP and clear the other fields.
  `state.rs`, `flags.rs` and `i8085_model.rs`.
- Library PSW image: undefined bits 5/3/1 are stored as 0/0/1; hardware
  expectations use mask D5h. decision-11, `flags.rs`, `i8085_registers.rs`
  and `i8085_stack_io_control.rs`.
- Derived API packing: PSW has A high and the flag byte low, from PUSH PSW's
  stack order. `state.rs` (205775-002 §5.6.5 p. 5-15; 9800301-04 p. 3-45)
  and `i8085_registers.rs`.
- Assumption: one model covers the 8085A and 8085AH, not a documented
  instruction-level equivalence. decision-10, §1, `mod.rs` and `i8085_timing.rs`.
- Unverified oracle expectation: 8085EXM's long-ALU register CRC is reported,
  not judged, even though it prints PASS. See §2 for the artifact, provenance
  limitation and measured result; no hardware equivalence follows from it.
- Derived, not explicitly stated: 16-bit PC, SP, register-pair and address wrap
  at FFFFh/0000h. Group comments and wrap witnesses in `i8085_data_transfer.rs`,
  `i8085_arithmetic.rs`, `i8085_branch.rs`, `i8085_stack_io_control.rs`,
  `i8085_machine_control.rs`, `i8085_rim_sim.rs` and
  `i8085_interrupts_vectored.rs`.
- API scope and stated departure: only RST/CALL INTR acknowledges execute;
  other bytes restore the boundary state, including interrupt enable despite
  hardware clearing it. decision-12 rule 5, decision-5 item 6, `acknowledge.rs`
  and `i8085_interrupts_intr.rs`.
- API representation: `OperandFetch` distinguishes operand reads; ports carry
  only an eight-bit device number. decision-5 item 3, decision-12's retained
  model-neutral contract and `stack_io_control.rs`; no upper I/O address byte
  is represented by this bus API.

## 4. License surface and boundary

The library is `MIT OR Apache-2.0`. `x80-core` has zero dependencies of any kind;
`x80-core/tests/boundary.rs` checks Cargo's resolved graph for all targets/features.
`deny.toml` retains x86jit's permissive allowlist without its oracle exceptions.
Machine services and dependency-bearing oracle harnesses belong outside the core.
No repository or package is published yet; publishing is the maintainer's step.
