# Provenance

## 1. Authority and citations

Derive documented hardware behavior from primary sources, cited by document,
edition/revision, section or table, and printed page (also PDF page if different). Pin revision and
SHA-256 before implementing a fact; check scans against page images. Comments
also link the relevant design section. Each implemented fact needs an independent
witness test with expected values derived from that citation.

The planned private Intel-family tier (`x80-core/src/intel/`, decision-16) is
separate from the fact-free root, `decode` and `progress` tier, which cites decisions
only. A hardware fact may be a literal in the family tier only when both models'
pinned primaries state it: cite 98-153B and 205775-002 by revision, section/table
and page at each fact, retaining each model's second witnesses and evidential
status. A fact that differs or that only one manual states must instead come
through a model-owned `FlagRules` implementation or per-model constant; matching
code or oracle results do not establish shared authority. Independent witnesses
remain per model. `tests/boundary.rs` checks the root's manual-identifier exclusion
and each family source file's two primary identifiers, absence of Z80 identifiers
and absence of paths to concrete model modules; review enforces citations at the
individual fact.

For undocumented behavior, apply the two-key rule in
[decision-17](<backlog/decisions/decision-17 - Two-key-evidence-rule-for-undocumented-behaviour.md>):
each fact needs a written statement pinned by revision/date and SHA-256 from
class A (manufacturer or licensee/second-source document), B (die analysis or
hardware measurement) or C (third-party written analysis), plus independent
confirmation from real silicon: class B, or class A of a different origin.
Pin both sources. Independence means different origin, not different publisher;
a copy counts once. Class C alone never licenses a fact. Independent witness
tests remain required; they do not replace silicon confirmation.

Each fact carries one label: "documented by manufacturer", "documented by second
source (part)", "die analysis (chip)", "hardware-measured (part, by whom)" or
"third-party analysis", and names the silicon it holds for. Anything else stays
"unverified". When classes disagree, silicon evidence prevails over a derivative
or erroneous document; record both readings and the ruling.

Never read another emulator's source or documentation. HDL replicas and netlist
simulators are implementations; only their die-derived data (schematics,
netlists) is evidence. Per-model rulings follow source pinning in later beads;
decision-17 itself licenses no individual undocumented fact.

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
| Z80 alternate C register label | UM008011-0816 Fig. 2 p. 2 (PDF 16) prints B' in both alternate BC columns; its Registers text p. 3 (PDF 17) names the pair BC', and 03-0029-01 Fig. 2.0-2 p. 4 (PDF 10) shows B' and C' | Derived resolution (x80-bi2.7, reviewed on page images): the diagram label is a misprint; the z80 model has a distinct C' register forming BC' with B' (decision-13 rule 1) |
| Z80 ED 63 / ED 6B omitted from compact loads | UM008011-0816 Table 7 p. 45 shows only 22/2A for HL; instruction entries pp. 108/103 print ED 01dd0011 / ED 01dd1011 and explicitly include HL = 10. 03-0029-01 Table 7.0-2 p. 45 includes both patterns with dd = BC, DE, HL, SP | Derived, decision-14 rule 1 (x80-bi2.3): the explicit entries and second witness prevail over the compact omission. Both ED HL forms are documented; eight ED pair loads and 58 ED terminal encodings, not 56 |
| Z80 compact H register headings | UM008011-0816 Tables 6/11 pp. 42, 50 and Fig. 39 p. 54 label the H column F; entries pp. 71, 145, 213 and 03-0029-01 Tables 7.0-1/4/7 pp. 44, 47, 50 explicitly name H = 100 | Derived resolution, decision-14 rule 1: use H as the register operand; the compact heading errors introduce no F operand |
| Z80 compact PUSH/POP encodings | UM008011-0816 Table 7 p. 45 prints PUSH codes F6/C6/D6/E6 and DD/FD E6, and POP AF as P1. Entries pp. 115, 117–119 and 03-0029-01 Table 7.0-2 p. 45 give PUSH C5/D5/E5/F5, DD/FD E5, POP AF F1 | Derived resolution, decision-14 rule 1: use the instruction-entry patterns, independently agreeing with the second table; compact cells are misprints |
| Z80 POP IY operation line | UM008011-0816 p. 122/PDF 136 prints `IYH ← (SP – X1)` in Operation; its description increments SP before reading the high byte, and its example reads the high byte at 1001h for initial SP=1000h. 03-0029-01 Table 7.0-2 p. 45/PDF 51 explicitly prints `IYH ← (SP+1)` | Derived resolution (x80-bi2.12): the operation line is a typo; read low at SP, high at SP+1, then SP becomes SP+2. Both images re-read at 200 dpi with enlarged crops. Pinned by `pop_pairs_low_then_high_including_iy_operation_line_resolution` |
| Z80 LD (nn),HL total in Mostek | MK78515 V2.0 (October 1978) p. 2-150/PDF 164 prints 17 T-states with components (4,3,3,3,3); UM008011-0816 rev. 11 p. 107/PDF 121 prints 16 with the same components | Derived resolution (x80-bi2.24): follow the primary's 16, also the sum of Mostek's printed components; Mostek's total is a misprint. Both timing lines re-read on enlarged 200 dpi crops. Pinned by `ld_nn_hl_resolves_mostek_17_against_primary_16` in `tests/z80_timing.rs`; no hardware measurement claimed |
| Z80 compact ADD A,(HL) and DEC BC encodings | UM008011-0816 Table 11 p. 50 prints 88 for ADD A,(HL), Table 13 p. 52 prints DB for DEC BC. Entries pp. 148, 201 and 03-0029-01 Tables 7.0-4/6 pp. 47, 49 give 86 and 0B | Derived resolution, decision-14 rule 1: use 86 and 0B from the entries and second witness |
| Z80 compact rotate encodings | UM008011-0816 Fig. 39 p. 54 prints CB 06 for RLC L, CB 0E for RLC (HL), and CB 06 for RRC E. RLC entries pp. 213, 215 give CB 05 and CB 06; RRC p. 225 prints CB 00001rrr and explicitly lists E = 011, hence CB 0B. 03-0029-01 Table 7.0-7 p. 50 agrees. Indexed RLC is correctly printed as DD/FD CB d 06 in Fig. 39 and entries pp. 217, 219 | Derived resolution, decision-14 rule 1: CB 05 for RLC L, CB 06 for RLC (HL), and CB 0B for RRC E; follow the instruction-entry patterns and register fields, independently agreeing with the second table |
| Z80 compact BIT/SET/RES table corruption | UM008011-0816 Table 14 pp. 55–57 prints C8 in place of CB throughout, repeated/wrong terminal bytes (e.g. indexed BIT 3/7 as 46), and DD in some IY cells. Entries pp. 243, 245, 247, 249, 251, 253, 255, 257, 259–260 and 03-0029-01 Table 7.0-8 p. 51 give CB bit-field patterns and DD/FD CB d forms | Derived resolution, decision-14 rule 1: expand the entry patterns with their register/bit restrictions; 192 CB bit forms and 24 per indexed table. The corrupt compact cells admit no extra encodings |
| Z80 compact JP and IN A,(C) encodings | UM008011-0816 Table 15 p. 59 prints D8 for JP C,nn and EB for JP (HL); Table 18 p. 62 prints ED 7B for IN A,(C). Entries pp. 263, 275, 296 and 03-0029-01 Tables 7.0-9/11 pp. 52, 54 give DA, E9 and ED 78 | Derived resolution, decision-14 rule 1: follow the entry patterns and second witness; compact cells are misprints |
| Z80 arithmetic entry bit diagrams | UM008011-0816 ADC p. 151 (PDF 165) prints the IX prefix bits 11011110 (DE) beside hex DD; OR p. 159 (PDF 173) prints the IY terminal bits 10101110 (AE) beside hex B6. Its Table 11 p. 50 (PDF 64) and 03-0029-01 Table 7.0-4 p. 47 (PDF 53) give DD 8E d and FD B6 d | Derived resolution (x80-bi2.8, page images): use the entry hex annotations, agreeing with both compact tables; the conflicting bit diagrams are misprints. Pinned by `arithmetic_entry_bit_diagrams_use_the_agreeing_hex_and_table_readings`; no execution semantics implemented |
| Z80 rotate entry bit diagrams | UM008011-0816 RRC p. 225 (PDF 239) prints the IY terminal bits 00011110 (1E) beside hex 0E; RR p. 228 (PDF 242) prints register bits 00001rrr, which is RRC. Its Fig. 39 p. 54 (PDF 68) and 03-0029-01 Table 7.0-7 p. 50 (PDF 56) give FD CB d 0E and CB 00011rrr | Derived resolution (x80-bi2.8, page images): use RRC's hex annotation and RR's compact pattern, agreeing with the second witness. Pinned by `rotate_entry_bit_diagrams_use_the_agreeing_hex_and_table_readings`; executor witnesses must later distinguish operations sharing metadata |
| Z80 RRCA worked-example result | UM008011-0816 p. 210 (PDF 224) prints input 00010001 (11h) and result 10011000 (98h), C=1. Its RRCA description p. 209 (PDF 223) and 03-0029-01 Table 7.0-7 p. 50 (PDF 56) route each bit right once, copying bit 0 to bit 7 and C | Derived resolution (x80-bi2.17, both sources re-read on enlarged 200 dpi crops): use the primary's description and the second witness's bit routing, yielding 88h, C=1 for 11h; the example's result bit 4 is a misprint. Pinned by `rrca_preserves_szpv_despite_worked_example_bit_four` |
| Z80 ED 70 admission | 03-0029-01 Table 7.0-11 p. 54 explicitly describes IN r,(C), r = 110 as affecting only flags. UM008011-0816 IN r,(C) p. 296 (PDF 310) instead labels r = 110 "Undefined op code; set the flag" | Stated disagreement; decision-14 rules 1–2 follow the primary's explicit undefined classification and the maintainer's documented-only scope: ED 70 is Unsupported. TM's expanded ED count is 59 including that form; the admitted count is 58. ED 71 also stays out: UM p. 307 lists no r = 110 OUT operand |
| Z80 DAA upper-digit range differences | UM008011-0816 DAA p. 173/PDF 187 prints upper-digit ranges 9–0 in the first addition row and 6–7 in the last subtraction row; 03-0002-01 DAA p. 132/PDF 137 prints 0–9 and 6–F | READ, x80-bi2.15: both images re-read in enlarged 200 dpi crops. Retain the literal primary transcription: interpret the first range inclusively between its printed endpoints and keep the final range 6–7 without widening it. The wider second-witness range does not expand the primary-table hardware claims; FFh subtraction is pinned separately as an unverified library-extension example in `tests/z80_daa.rs` |
| Z80 DAA outside UM p. 173's table | UM008011-0816 DAA p. 173/PDF 187 gives thirteen rows covering 684/2048 A/N/H/C inputs, retaining the printed upper ranges 9–0 (inclusive endpoints) and 6–7; 03-0029-01 DAA entry in Table 7.0-5 p. 48/PDF 54 describes BCD correction without specifying the missing inputs | Library extension accepted by the maintainer (2026-11-13), proposed by the supervisor; the sources do not document these inputs. Table cases pinned literally. Rule: outside the table choose 06h if the input low digit exceeds 9 or H is set, and 60h if input A exceeds 99h or C is set; add/subtract according to N, C records selection of 60h, and H is the adjustment's bit-4 carry/borrow. Separate literal extension examples in `tests/z80_daa.rs` are library witnesses, not hardware claims |
| Z80 DAA H-after wording | UM008011-0816 DAA p. 174/PDF 188 refers H to p. 173/PDF 187, whose table lists H-before but no H-after column; 03-0029-01 Table 7.0-5 p. 48/PDF 54 marks H result-dependent, and section 6 p. 39/PDF 45 defines H as low-four-bit carry/borrow | Derived reading accepted by the maintainer (2026-11-13; x80-bi2.15): derive H-after from the adjustment's bit-4 carry/borrow for table cases too. The DAA entry does not state that equation explicitly; code and witnesses label the wording gap and the library rule |
| Z80 H definedness in CCF and pair arithmetic | 03-0029-01 Tables 7.0-5/6 pp. 48–49 print H unknown for CCF, ADD HL/IX/IY, ADC HL and SBC HL. UM008011-0816 entries pp. 178, 188, 190, 192, 194, 196 give explicit H rules | Stated disagreement; decision-14 rule 6 follows the primary's definedness: these forms do not invoke unknown-flag preservation and H remains in the hardware comparison mask. Arithmetic equations require their own group witnesses; this row does not settle them |
| Z80 C definedness in block I/O | 03-0029-01 Table 7.0-11 p. 54 prints C unknown for INI/INIR/IND/INDR/OUTI/OTIR/OUTD/OTDR; UM008011-0816 entries pp. 298, 301, 302, 305, 309, 312, 313, 316 explicitly say C is not affected | Stated disagreement; decision-14 rule 6 follows the primary: preserve C as a hardware claim and compare it under 43h; S/H/P/V remain unknown, rather than also masking C under 42h |
| Z80 IN r,(C) H notation | 03-0029-01 Table 7.0-11 p. 54/PDF 60 marks H affected according to the result, without an equation; UM008011-0816 IN r,(C) p. 297/PDF 311 explicitly resets H. 03-0002-01 p. 255/PDF 265 also says H reset | Source-notation discrepancy re-read at 200 dpi with crops of both conflicting flag lists (x80-bi2.21). Follow the primary's explicit H=0, independently confirmed by the programming manual; no alternative H equation is inferred from the older table's symbol. Pinned by `every_register_input_samples_old_bc_and_sets_sz_parity_clearing_hn` |
| Z80 BIT flag-list H/N label slips | UM008011-0816 BIT (HL)/(IY+d) pp. 246, 250 each print H set and then H reset. Adjacent BIT entries pp. 244, 248 and 03-0029-01 Table 7.0-8 p. 51 agree on H set, N reset | Derived resolution, decision-14 rule 6: the second H is a label slip for N; it is not another unknown flag. S and P/V remain the unknown flags for all four BIT forms |
| Z80 SBC A P/V wording | UM008011-0816 SBC A,s p. 156 (PDF 170) prints "P/V is reset if overflow; otherwise, it is reset"; the flag definitions p. 67 (PDF 81) set the Overflow flag on an overflowing subtraction; 03-0029-01 Table 7.0-4 p. 47 (PDF 53) marks SBC V (V = 1 means overflow) and §6 p. 39 sets overflow when the signed range is exceeded | Derived resolution (x80-bi2.10, confirmed by an independent review on 200-dpi images): P/V is the overflow of the subtraction; the entry's first "reset" ("reset if overflow") is a misprint for "set". Pinned in tests/z80_arithmetic_reference |
| Z80 AND/OR P/V wording | UM008011-0816 AND s p. 158 (PDF 172) prints "P/V is reset if overflow; otherwise, it is reset" and OR s p. 160 (PDF 174) "P/V is set if overflow; otherwise, it is reset"; the flag definitions p. 67 give even parity (P = 1) for logical operations; 03-0029-01 Table 7.0-4 p. 47 marks AND/OR P and §6 p. 39 defines parity for AND, OR and XOR | Derived resolution (x80-bi2.10, confirmed by an independent review): P/V is the parity of the result for AND and OR, as for XOR; the entries' overflow wording is a misprint. Pinned in tests/z80_arithmetic_reference |
| Z80 ADD IX/IY example values | UM008011-0816 ADD IX,pp p. 195 and ADD IY,rr p. 197 (PDF 209/211) give operand 333h, BC 5555h and result 8888h; their operation lines pp. 194/196 and 03-0029-01 Table 7.0-6 p. 49 give IX/IY plus the pair, and 0333h + 5555h = 5888h | Derived resolution (x80-bi2.10, confirmed by an independent review): the operation lines prevail; the examples' operand should read 3333h. Pinned in tests/z80_arithmetic_reference |
| Z80 JR NC example displacement | UM008011-0816 JR NC,e example p. 270 (PDF 284) repeats the jump with bytes 30 00 at 0480h; the same entry p. 269 (PDF 283) and 03-0029-01 Table 7.0-9 p. 52 (PDF 58) apply e to the PC already advanced by two, so 30 00 reaches 0482h and a self-repeat needs 30 FE | Derived resolution (x80-bi2.19, confirmed by an independent review on 200-dpi images): the operation text prevails; the example's displacement byte is a misprint. Pinned in tests/z80_jump.rs |
| Z80 unused F bits 5 and 3 | UM008011-0816 Tables 21/22 pp. 65–66 place the six flags at bits 7,6,4,2,1,0 and state bits 3 and 5 are not used; no zero value is stated | Library choice, decision-14 rule 5 (x80-bi2.3): F/F' images set bits 5/3 to zero and ignore those input bits; store six flags. Hardware comparisons default to D7h; separate library witnesses pin the zero image. No physical zero-bit behaviour is claimed |
| Z80 instruction flags printed unknown | UM008011-0816 BIT entries pp. 243–250: S/P/V unknown; INI/INIR/IND/INDR/OUTI/OTIR/OUTD/OTDR entries pp. 298, 301, 302, 305, 309, 312, 313, 316: S/H/P/V unknown. Historical definedness disagreements are recorded above | Library choice, decision-14 rule 6 (x80-bi2.3): leave each primary-entry unknown flag unchanged. Hardware witnesses mask BIT under 53h and block I/O under 43h; separate library witnesses check preservation from both initial values. These deterministic values are not hardware claims |
| Z80 poll polarity and deterministic call order | UM008011-0816 Pin Functions p. 6 (PDF 20), Interrupt Request/Acknowledge pp. 12–14 (PDF 26–28); 03-0029-01 §3 p. 8 (PDF 14), §4 pp. 16, 18 (PDF 22, 24) | decision-15 rule 1: stated API choice, not hardware call timing. After budget, poll stop, NMI, INT once each in that order, including while halted/disabled; record calls and levels in the trace. NMI and INT methods return true while the request is asserted (active-low pin Low), false by default, matching i8080/i8085 (maintainer, 2026-11-13) |
| Z80 NMI edge sampler and restoration | UM008011-0816 Pin Functions p. 6 (PDF 20); 03-0029-01 §3 p. 8 (PDF 14): negative-edge NMI | decision-15 rule 2: stated API sampler detects a not-asserted sample followed by an asserted one (pin High then Low); history starts/resets not asserted by library choice. Pulses wholly between polls are missed. Stop/Unsupported restore pre-sampling state as decision-12 rule 2; Halt retains samples; Budget does not sample. This does not model within-instruction edge capture |
| Z80 boundary priority and step accounting | UM008011-0816 Pin Functions p. 6, HALT Exit p. 14, EI p. 183 (PDF 20, 28, 197); 03-0029-01 §3 p. 8, §4 p. 18, §8 p. 55 (PDF 14, 24, 61) | decision-15 rule 3: stated API sequence budget, polls/stop, NMI, eligible INT, halt, fetch/execute. NMI priority and IFF-independent eligibility are stated hardware rules; host precedence and one step per acceptance are API choices |
| Z80 provided 16-bit port methods | UM008011-0816 instruction entries pp. 295–296, 298, 300, 302, 304, 306–307, 309, 311, 313, 315 (PDF printed+14); 03-0029-01 Table 7.0-11 p. 54 (PDF 60); 03-0002-01 OUTI p. 267/PDF 277, OTIR p. 269/PDF 279 | decision-15 rule 4: stated API choice adds provided methods forwarding the low byte to the required 8-bit methods. Hardware upper-byte sources are stated: A for immediate ports, B for C ports; input blocks use B before decrement, output blocks after decrement. OUTI/OTIR decremented B is explicit in both instruction descriptions |
| Z80 IN r,(C), r=110 wording | UM008011-0816 IN r,(C) p. 296/PDF 310 labels the field "Undefined op code; set the flag"; 03-0029-01 Table 7.0-11 p. 54/PDF 60 describes r=110 as affecting only flags | Decision-14 owns admission of this encoding. decision-15 rule 4 records the disagreement without adding a flags-only I/O operation: the maintainer's documented-only scope excludes ED 70. Port-address citations do not license that excluded field value |
| Z80 OpcodeFetch beyond the first byte and operand kinds | UM008011-0816 Pin Functions p. 6 (PDF 20), Registers p. 3 (PDF 17), BIT b,(IX+d) p. 247 (PDF 261); 03-0029-01 §3 p. 7 (PDF 13), §2 p. 4 (PDF 10) | decision-15 rule 5: stated API extension of decision-5 item 3 maps every memory M1 fetch to OpcodeFetch. TM explicitly states M1 for each byte of two-byte opcodes beginning CB/DD/ED/FD. Displacements and immediate operands use OperandFetch as a stated API distinction; memory data and stack retain their kinds |
| Z80 indexed-CB fourth-byte cycle | UM008011-0816 BIT b,(IX+d) p. 247 (PDF 261): DD CB d op and 20 (4,4,3,5,4); 03-0029-01 §3 p. 7 (PDF 13) covers two-byte opcodes without expressly resolving this four-byte form | decision-15 rules 5–6: unverified reading assigns OperandFetch to both d and the fourth operation byte, OpcodeFetch to DD/FD and CB, hence two R increments. The byte order is stated; the fourth byte's non-M1 classification is not explicit on the inspected pages and is not proved by its T-state count |
| Z80 R fetch count, interrupt count and LD ordering | UM008011-0816 Registers p. 3, Figs 9/10 pp. 13–14, LD A,R p. 95, LD R,A p. 97, LDIR p. 132 (PDF 17, 27–28, 109, 111, 146); 03-0029-01 §2 p. 4, §3 p. 7, §4 pp. 16, 18 (PDF 10, 13, 22, 24); 03-0002-01 LD A,R p. 49/PDF 53, LD R,A p. 51/PDF 55 | decision-15 rule 6: seven incremented bits and retained bit 7 are stated. Once per M1, including prefixes and one per successful NMI/INT acceptance, is derived from fetch/refresh descriptions. Fetch increments precede LD execution: LD A,R sees its own increments; LD R,A replaces them with all eight A bits. That ordering is derived, not explicit microcycle text. Unsupported restores R |
| Z80 refresh omitted from Bus | UM008011-0816 Instruction Fetch p. 8 (PDF 22): RD is absent during refresh; Registers p. 3 (PDF 17) describes I/R on the address bus; 03-0029-01 §2 p. 4 (PDF 10) agrees on refresh addressing | decision-15 rules 6/14: stated API choice makes no refresh bus call and exposes refresh only as R, subject to the HALT departure. Absence of RD is stated hardware behaviour; omission of refresh addresses is library scope |
| Z80 EI delay edge cases | UM008011-0816 Table 1 p. 18, DI p. 182, EI p. 183, NMI Pin Functions p. 6 (PDF 32, 196–197, 20); 03-0029-01 §8 pp. 55–56, §3 p. 8 (PDF 61–62, 14) | decision-15 rule 7: ordinary one-instruction EI inhibition, both-IFF writes and NMI eligibility are stated. Unverified edge-case model: EI EI re-arms, DI cancels the pending delay, NMI acceptance preserves it until an ordinary instruction/iteration completes. The inspected pages do not specify those latch interactions |
| Z80 HALT exit, R and resume PC | UM008011-0816 HALT Exit p. 14, HALT p. 181, RETN example p. 290 (PDF 28, 195, 304); 03-0029-01 §4 p. 18 (PDF 24) | decision-15 rule 7: stated departure retains decision-2's prompt Halt exit with no idle steps/cycles, fetches or R increments, although hardware fetches NOPs and refreshes. Entry's 4 states and NMI/eligible-INT wake are stated; keeping the post-HALT resume PC is derived from completed entry and the interrupt return-PC contract |
| Z80 CALL carry-condition flag label | UM008011-0816 CALL cc,nn p. 284 (PDF 298) labels cc=011 Carry (C) with relevant flag Z; its RET cc table p. 286 (PDF 300) gives C, and 03-0029-01 Table 7.0-10 p. 53 (PDF 59) identifies cc=011 as carry | Derived resolution (x80-bi2.20, both manuals rechecked on enlarged 200 dpi crops): use carry, agreeing with the primary's condition name and RET table and the second witness; CALL's Z is a flag-label misprint. Pinned by `call_c_uses_carry_independently_of_zero_despite_primary_table_label` with opposite C/Z values |
| Z80 RETI IFF reading and device notification | UM008011-0816 RETI p. 288 (PDF 302); 03-0002-01 RETI p. 246/PDF 255; MK78515 Table 2-1 p. 2-15/PDF 29 and RETI p. 2-200/PDF 214 | decision-15 rule 7: stated documented reading leaves IFF1/IFF2 unchanged; MK expressly says so. RETN's IFF2-to-IFF1 copy remains separate (UM p. 290/PDF 304; ALPM p. 248/PDF 257). No contrary IFF effect found in these pages. Stated API choice exposes RETI through its two opcode fetches without adding a device callback |
| Z80 NMI discarded read, push mapping and 11-state total | UM008011-0816 Nonmaskable Interrupt Response p. 13, Fig. 10 p. 14, Table 1/CPU Response p. 19, RETN p. 290, RST p. 293 (PDF 27–28, 33, 304, 307); 03-0029-01 §4 p. 18, §8 p. 56, Table 7.0-10 p. 53 (PDF 24, 62, 59); 03-0002-01 RETN p. 248/PDF 257 | decision-15 rule 8: derived one discarded OpcodeFetch read at unchanged PC from the stated read cycle; stated return PC, high-first push and 0066h target mapped to Stack writes. IFF1 clears, IFF2 stays. Total 11 is derived from restart equivalence plus RST's stated 11, not an explicitly printed NMI total on these response pages |
| Z80 common INT acknowledge address and kinds | UM008011-0816 Interrupt Request/Acknowledge pp. 12–13, Fig. 9 (PDF 26–27); 03-0029-01 §4 p. 16 (PDF 22) | decision-15 rule 9: derived mapping performs one InterruptAcknowledge read at unchanged boundary PC in all three modes. The figure places PC on the bus and the generic cycle text does not qualify its mode. IM 1 ignores the returned byte; Stack/Data mappings follow the documented save/table sequence |
| Z80 IM 0 accepted-byte restriction | UM008011-0816 CPU Response p. 19 (PDF 33); 03-0029-01 §8 p. 56 (PDF 62): hardware allows other instructions including a three-byte CALL | decision-15 rule 9: stated library scope restriction accepts RST only. Other bytes give Unsupported with acknowledge=true, no preceding bytes and full boundary restoration; only the acknowledge read survives externally. This restricts documented hardware capability; it is not a claim that other bytes are undocumented |
| Z80 odd IM 2 vector refusal | UM008011-0816 CPU Response/Mode 2 p. 20, Fig. 17 (PDF 34); 03-0029-01 §8 p. 57 (PDF 63): bit 0 must be zero; table low byte first; acknowledge/save/table costs 7/6/6 | decision-15 rule 9: stated library restriction refuses an odd acknowledge before writes/vector reads, restores boundary state and reports acknowledge=true. Hardware's odd-vector outcome is unverified, not extrapolated. For even vectors the PC-save then two Data-read order and total 19 are stated |
| Z80 IM 0 RST and IM 1 totals | UM008011-0816 CPU Response p. 19 (PDF 33): normal restart plus two clocks; RST p. 293 (PDF 307): 11; 03-0029-01 §8 p. 56 and Table 7.0-10 p. 53 (PDF 62, 59) | decision-15 rule 9: derived 13 states for each (11+2), one step per successful acceptance. Both sources state the operands of the derivation, rather than an explicit 13-state acceptance entry |
| Z80 LD A,I / LD A,R interrupt parity note | UM008011-0816 entries pp. 94–95 and Table 1 p. 18 (PDF 108–109, 32); 03-0002-01 LD A,I p. 48/PDF 52 has the interrupt note, LD A,R p. 49/PDF 53 omits it | decision-15 rule 10: IFF2-to-P/V is stated. Primary gives the interrupt-clears-parity note for both; second-witness omission gives no contrary result. Unverified interpretation uses one latch: successful NMI or INT acceptance immediately after the load clears P/V; an ordinary next instruction expires it, ineligible requests do not apply it. Budget preserves it; Stop/Unsupported restore it |
| Z80 block iteration retirement and two States slots | UM008011-0816 LDIR p. 132, LDDR p. 136, CPIR p. 139, CPDR p. 142, INIR p. 300, INDR p. 304, OTIR pp. 311–312, OTDR pp. 315–316 (PDF printed+14); 03-0029-01 Tables 7.0-3 p. 46 and 7.0-11 p. 54 (PDF 52, 60); 03-0002-01 OTIR p. 269/PDF 279 | decision-15 rule 11: PC rewind by two and per-iteration interrupt recognition/refresh are stated. Stated API choice retires one iteration per step and refetches at the next boundary. Stated 16 terminating and 21 repeating map to untaken/taken respectively; the shared machinery acquires no hardware fact |
| Z80 LDIR/LDDR P/V on a repeating iteration | UM008011-0816 LDIR p. 133/PDF 147 states "P/V is set if BC – 1 ≠ 0; otherwise, it is reset." LDDR p. 137/PDF 151 states "P/V is reset." LDI p. 130/PDF 144 and LDD p. 134/PDF 148 give the counter rule. 03-0029-01 Table 7.0-3 p. 46/PDF 52 prints P/V = 0 for both LDIR and LDDR, alongside 21 states "If BC ≠ 0" and 16 states "If BC = 0"; note 1 (P/V = 0 iff BC−1 = 0) applies to LDI/LDD. The three differing pages were re-read on 200 dpi images and enlarged crops | Supervisor ruling accepted by the maintainer (2026-11-13), replacing an earlier literal-entry reset ruling: every LDIR/LDDR iteration sets P/V = (BC after decrement != 0), exactly as LDI/LDD. Stated for LDIR by UM p. 133; derived for LDDR intermediate iterations from the LDIR/LDD entries. Interpret LDDR's own reset entry and TM's zero entries as the completed-instruction result, when BC = 0, rather than flags for each exposed iteration (decision-15 rule 11). Pinned by `ldir_sets_pv_until_final_iteration` and `lddr_sets_pv_until_final_iteration_derived`: starting BC = 2, the first step leaves BC = 1, PC rewound, 21 states and P/V = 1; the final step leaves BC = 0, PC advanced, 16 states and P/V = 0. No hardware/oracle measurement claimed |
| Z80 CPIR/CPDR terminating timing condition | UM008011-0816 CPIR p. 139/PDF 153 and CPDR p. 142/PDF 156: prose terminates on exhausted count OR match; 16-state headings say count zero AND match. 03-0029-01 Table 7.0-3 p. 46/PDF 52 says OR for the 16-state path. MK78515 V2.0 CPIR p. 2-63/PDF 77 and CPDR p. 2-59/PDF 73 independently print OR and 16 | Resolved by accepted decision-15 rule 11: use the primary's descriptions and the second witnesses' OR; the primary timing headings are treated as a conjunction misprint. Repeating path remains count nonzero AND no match, 21 states. Both images re-read at 200 dpi with enlarged crops; x80-bi2.24 additionally re-read both Mostek/UM pairs at 200 dpi with enlarged crops. Pinned by `cpir_repeats_or_stops_on_either_match_or_exhaustion` and `cpdr_repeats_or_stops_on_either_match_or_exhaustion`, each covering early match with count remaining and exhaustion without a match; independently timed by `ed_all_58_encodings_and_all_repeat_termination_paths` in `tests/z80_timing.rs`. No hardware/oracle measurement claimed |
| Z80 CPDR repeat-sentence equality | UM008011-0816 CPDR p. 142/PDF 156 says to repeat when BC is nonzero and A = (HL), contradicting its preceding match-termination sentence and its 21-state heading (A ≠ (HL)). 03-0029-01 Table 7.0-3 p. 46/PDF 52 repeats only for nonzero BC and A ≠ (HL) | Derived resolution (x80-bi2.13; both images re-read at 200 dpi with enlarged crops): the repeat sentence's equality is a misprint. Use the primary's termination sentence and 21-state heading, agreeing with the second witness and decision-15 rule 11. Pinned by `cpdr_repeats_or_stops_on_either_match_or_exhaustion`, including an early match with count remaining and a non-matching repeat |
| Z80 continuing INIR/INDR WZ | Pinned MEMPTR (©2006), INIR/INDR headings: exactly as INI/IND on each execution, hence BC_before_decrement ±1; z80test v1.2a `src/tests.asm` `.inir_nop` lines 1040–1046 / `.indr_nop` lines 1048–1054, with `idea.asm`, `testmacros.asm`, `crctab.asm` (pins below) | **Falsified, MEASURED 2026-11-13:** independent in-memory enumeration/post-BIT projection gives 0A537B63 for each group under M's rule, versus pinned F3B1BE2F. WZ=(address of this iteration's ED byte+1) modulo 65536 reproduces F3B1BE2F for both; accepted decision-20 §5 selects it as **L**, extending M's explicit continuing LDIR/LDDR/CPIR/CPDR form. M does not state that INxR equation; no independent written INxR key or whole-word licence is inferred from CRCs. Terminating INxR retains BC_before_decrement ±1. The other six continuing block rules reproduce their groups' pinned CRCs, subject to the output-repeat blind spot detailed in decision-20 §5; no CPU/oracle program or hardware run |
| Z80 reset preservation, sampler bookkeeping and construction defaults | UM008011-0816 RESET p. 6, Table 1 p. 18, HALT p. 181 (PDF 20, 32, 195); 03-0029-01 RESET p. 9, Fig. 8.0-1 p. 56 (PDF 15, 62); MK78515 Table 2-1 p. 2-15/PDF 29 | decision-15 rule 12: stated reset clears both IFFs, PC, I, R, sets IM 0 and leaves halt. Preserving other registers/flags, clearing EI/parity bookkeeping, setting NMI history to not asserted and making no calls are stated library choices, not undocumented reset facts. Zero-register Default with disabled IFFs/IM 0, no delay/halt/parity-note pending state and not-asserted NMI history is a library construction choice |
| Z80 Unsupported payload and transactional restoration | decision-5 items 6/10; decision-14 payload extension; UM008011-0816 Interrupt Enable/Disable p. 18 (PDF 32) states IFF clearing on maskable acceptance | decision-15 rule 13: stated API choice adds fact-free preceding bytes (fixed three-byte capacity plus length), keeps pc/opcode/acknowledge meanings, and broadens one-read wording to reads of the reported bytes. All CPU boundary state including R and sampler/parity/delay latches is restored with zero contributed progress. Device effects of reads cannot be undone. Interrupt refusal rollback is a departure from hardware acceptance |
| Z80 omitted arbitration, waits and pin timing | UM008011-0816 Pin Functions pp. 6–7, Instruction Fetch p. 8, Bus Request/Interrupt Request p. 12 (PDF 20–22, 26); 03-0029-01 §3 pp. 8–9, §4 p. 16 (PDF 14–15, 22) | decision-15 rule 14: stated API scope excludes BUSREQ/BUSACK, WAIT, pin timing and refresh bus addresses. Hardware's INT/BUSREQ condition and bus-request inhibition of interrupts are therefore not additional gates. Documented automatic acknowledge waits stay in totals; external waits add no modelled time |

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

### 8085 undocumented-instruction sources (M6; x80-mn3.2)

READ locally, 2026-11-13: the two-page article and the Tundra title/colophon and
CA80C85B section on `pdftoppm` page images; all four HTML bodies, using Python
`gzip` to decompress the two `.html.gz` files for reading. All six are fetch-only
and cite-only, with no redistribution grant. The paths below are in `oracles/`;
its `MANIFEST.md` records the exact URLs, byte sizes and capture timestamps, and
`SHA256SUMS.fetched` pins the original bytes, including gzip encoding.

| Path | Title / author or organisation / date | SHA-256 | Class, label and source statement / locator |
| --- | --- | --- | --- |
| `intel/i8085-undocumented/dehnhardt-sorensen-1979.pdf` | Unspecified 8085 op codes enhance programming — Wolfgang Dehnhardt and Villy M. Sorensen; Electronics, Engineer's notebook, 18 January 1979 | `3e518e15baecefe525d676962631568442ecc56aef0030ab84205af96634cda4` | **C; third-party analysis.** pp. 144–145/PDF 1–2 report ten opcodes and two flags, X5 and V; PDF 2's “New 8085 instructions” table supplies operations, encodings, flags, cycles and states. The authors say they found them while testing an assembler-disassembler module; no tested manufacturer, part marking or specimen is identified |
| `intel/i8085-undocumented/tundra-8000-series-1996.pdf` | 8000 Series Components Manual — Tundra Semiconductor Corporation; 9000000.MD302.01, ©1996; layout headers 13 June 1997 | `ffd52ddb16cf474e515608c81bff9f8a3f48e145694ef21c81175fb215cd7a2e` | **A; documented by second source (CA80C85B).** CA80C85B High Performance 8-bit CMOS Microprocessor, pp. 2-1–2-16/PDF 25–40; p. 2-4 states ten additional instructions and two additional flags. Figure 2-9 “Extended Condition Codes” and Table 2-10 “CA80C85B Extended Instruction Set” document Tundra's CMOS part; this is not evidence for a named Intel specimen |
| `intel/i8085-undocumented/shirriff-8085-undocumented-flags.html.gz` | Silicon reverse engineering: The 8085's undocumented flags — Ken Shirriff; February 2013 (URL month); Wayback capture 20260905021235 | `62ff642a672cf3b5b860b1fdad877ba68a32f759759f19d9c7f97ad65eb9a09e` | **B; die analysis (8085; exact part unverified).** “The circuit for the K and V flags” and “The silicon” give the reverse-engineered circuit; notes [1], [3], [4] identify the earlier article, Tundra's copied description and instruction-specific flag discussion |
| `intel/i8085-undocumented/shirriff-8085-flag-circuits.html.gz` | Reverse-engineering the flag circuits in the 8085 processor — Ken Shirriff; July 2013 (URL month); Wayback capture 20261027125023 | `3cf42d5c7d235592556def16c500f4383d5a0935a70f6e7f161cb27cb7095fe9` | **B; die analysis (8085; exact part unverified).** Opening flag list and flag-storage schematic describe PSW bus paths, bit 3 and DSUB multi-byte zero; “The carry logic” describes carry selection and the LDHI/LDSI temporary carry latch |
| `intel/i8085-undocumented/shirriff-8085-alu.html` | Reverse-engineering the 8085's ALU and its hidden registers — Ken Shirriff; July 2013 (URL month); Wayback capture 20261014020057 | `1f7efa97c63fd8285b660b7515eee0c537802876eb5fc298cba6227a3d1ef7ee` | **B; die analysis (8085; exact part unverified).** “The real architecture of the 8085 ALU”, ACT/TMP register discussion, “Building the 8-bit ALU from single-bit slices”, and “The control lines” table describe die-derived data paths and instruction-controlled signals |
| `intel/i8085-undocumented/zima-8085-decode-rom.html` | Intel 8085 Instruction Decode ROM — Pavel Zima (pastraiser.com attribution; no byline or publication date in capture); Wayback capture 20260606111153 | `d7afbcdfa0db020772393c171e6c697593aefd5b6da0b16161d32858cadb2eb5` | **B; die analysis (Intel 8085 as headed; exact part unverified).** Introductory ROM description and 48-line table transcribe physical decode lines, including DSUB, ARHL, RDEL, LDHI/LDSI, SHLX/LHLX, RSTV and JNK/JK. The heading names Intel 8085; the URL and linked layer drawing use AMD9085, without resolving the exact part |

Page maps (one-based PDF), checked on rendered images:

| Source | Printed → PDF | Images / locators checked |
| --- | --- | --- |
| Article (2 PDF pages) | 144–145 → 1–2 | Both images: byline/title/text and “New 8085 instructions” table. The scan does not display the issue date or printed page numbers; the publication locator comes from Shirriff's note [1] and the supplied publication record. Note [1] misspells Dehnhardt as “Sehnhardt”; the scan's byline governs the author name |
| Tundra (186 PDF pages), title/colophon | i–ii → 1–2 | Title, document 9000000.MD302.01, ©1996 and prohibition on copying; layout stamp Friday, June 13, 1997, 12:49 PM |
| Tundra CA80C85B, printed 2-n | **PDF = 24+n**, for 2-1–2-16 | Every page 2-1 → 25 through 2-16 → 40; extended-set statement 2-4 → 28; Figure 2-9 2-15 → 39; Table 2-10 2-16 → 40 |

Reading limits: article PDF 2 prints RSTV hex C8 beside binary 11001011 and
“reset” in the JX5 prose despite its positive test. Tundra Table 2-10 has a
trailing comma in the DSUB flag list, subtraction remnants in the jump rows,
a data-byte box for LHLX, and the SHLX description cut off at the original page
end. These observations preserve the source readings; they do not correct or
admit an encoding, timing or flag rule.

Independence under decision-17: the article and Tundra count as **one origin**.
Tundra derives its extended-set description from the article; Shirriff's
undocumented-flags note [3] explicitly describes that copying and its introduced
errors. The die analysis is a **separate origin** from that written lineage.
Shirriff's credits/conclusions identify Visual6502 photographs and Zima's mask
layers/transistor data. Shirriff's three posts and Zima's decode table share that
die-data lineage; they do not supply four independent silicon confirmations.
Only the written analyses and transcribed die data were read, not simulator or
emulator implementations.

Silicon scope remains limited to what the pinned pages identify: Tundra names
CA80C85B; the article names only 8085; Shirriff names 8085, and Zima's title and
heading say Intel 8085 despite the AMD9085 paths. No package marking or revision
in these captured pages resolves that discrepancy. Do not infer an Intel 8085A,
8085AH or AMD 9085 variant equivalence from those names or paths.

The undocumented-flags post's opening discussion and circuit section state that
the article's K/X5 formula, repeated as UI by Tundra, is mistaken, and derive
K = V XOR result sign, with a separate INX/DCX path. This is a recorded source
disagreement. Instruction/flag rulings, per-fact confirmation and any changes to
decision-11 belong to **x80-mn3.3**; these pins license no behaviour by themselves.

### 8085 undocumented rulings (decision-18; x80-mn3.3)

| Record / key | Scope and locator |
| --- | --- |
| Status | [decision-18](<backlog/decisions/decision-18 - 8085-undocumented-instructions-and-V-K-flags.md>) is **accepted** by the maintainer (2026-11-13); implemented by **x80-mn3.6** (V/K) and **x80-mn3.7** (the ten instructions); no new hardware measurement yet. It applies the maintainer's 2026-11-13 amendment and supersedes decision-11 in part |
| Second-source manufacturer licence | decision-18 “Amendment to decision-17”: a manufacturer data sheet for a pin- and function-compatible second-source part may license operations, lengths, flag statements and timings on the original-part model, with the part-qualified Intel-unverified label, independent die confirmation of every encoding, explicit defect rulings, silicon precedence and labelled unverified/library gaps. This general licence applies to future qualifying manufacturer sheets too |
| W / T | W: pinned article pp. 144–145/PDF 1–2 (C); T: pinned Tundra 9000000.MD302.01, CA80C85B p. 2-1/PDF 25 compatibility statement, p. 2-4/PDF 28 extended set, Fig. 2-9 p. 2-15/PDF 39, Table 2-10 p. 2-16/PDF 40 (A). **One written origin**; Tundra is the manufacturer of CA80C85B; its p. 2-1 states pin and functional compatibility with 8085/8085A |
| KV / FC / ALU / ROM | Four pinned HTML sources above, B and **one die-data origin**: KV circuit and notes [3]–[5]; FC opening/storage/carry subsections; ALU architecture, operation and control-line tables; ROM introduction and 48 rows counted from DI/EI as row 1. Independent of W/T, not independent of one another |
| Silicon scope and labels | **S** below expands to **documented by second source (CA80C85B), unverified for Intel 8085** and applies to each licensed operation, length, mask and timing. **D** = **die analysis (8085; exact part unverified)**. **L** = **adopted library rule; exact hardware effect unverified**, including selected single-origin die readings. ROM's Intel heading/AMD9085 paths do not identify a package/revision. No hardware-measured or Intel manufacturer-documented label is claimed |
| Revision-1 re-reading | All six original hashes rechecked; W PDF 1–2, T PDF 1–2, 25, 28, 39–40 and Intel PDF 100–101,105 re-read as rendered images; KV/FC decompressed with Python gzip, relevant sections of all four HTML bodies re-read. Original-draft audit additionally records Intel PDF 91–97; that earlier arithmetic/logical image audit is retained, not claimed as freshly read. No network or linked implementations/timing ROM read |

All ten rows below are **ADMITTED and implemented** in the model. The independent
key confirms each encoding, not the complete operation or timing. S licenses
the documented facts despite absent Intel timing measurements; two keys are
not required anew for each timing under the maintainer's amendment.

| Admitted instruction / length / operation | Independent encoding key and additional die scope | Machine cycles / T-states | Label |
| --- | --- | --- | --- |
| DSUB 08; 1 byte; HL←HL−BC | ROM row 22 `0000 1000`; ALU non-A subtraction/arith_to_flags; FC combined Z, not complete register/flag sequencing | 3 / 10 | **S** for operation, length, totals; **D** for encoding |
| ARHL 10; 1 byte; arithmetic HL shift right, H7 retained, H0→L7, L0→CY | ROM row 25 `0001 0000`; ALU right-shift and ARHL selector; FC ACT0→CY. High-input prose reconciled below; no die confirmation of sign fill | 2 / 7 | **S**; **D** encoding |
| RDEL 18; 1 byte; DE left through old CY, old D7→CY | ROM row 23 `0001 1000`; ALU non-A addition/carry controls, not full byte routing | 3 / 10 | **S**; **D** encoding |
| LDHI 28; 2 bytes; DE←HL+byte 2, HL preserved | ROM row 24 `001x 1000`; FC constant+pair addition/temporary carry/CY preservation, not extension or exact register routing | 3 / 10 | **S**; **D** encoding |
| LDSI 38; 2 bytes; DE←SP+byte 2, SP preserved | Same ROM row 24 and FC/ALU scope | 3 / 10 | **S**; **D** encoding |
| RSTV CB; 1 byte; V=1 pushes return PC high at old SP−1, low at old SP−2, SP←SP−2, PC←0040h; V=0 sequential | ROM row 8 `1100 1011`, not vector, predicate, stack trace or latches | V=0 untaken 1 / 6; V=1 taken 3 / 12 | **S** operation/length/both totals; **D** encoding |
| SHLX D9; 1 byte; [DE]←L, [DE+1]←H, DE preserved | ROM row 11 `1101 1001`, no complete bus trace | 3 / 10 | **S**, W completion of damaged T; **D** encoding |
| JNK DD; 3 bytes; K=0 jumps to byte 3:byte 2, else PC after instruction | ROM row 32 `11x1 1101`, pair not polarity/operand trace | K=1 untaken 2 / 7; K=0 taken 3 / 10 | **S** operation/length/both totals, polarity reconciliation; **D** encoding |
| LHLX ED; 1 byte; L←[DE], H←[DE+1], DE preserved | ROM row 12 `1110 1101`, no complete operand/transfer trace | 3 / 10 | **S**, W/T text reconciliation; **D** encoding |
| JK FD; 3 bytes; K=1 jumps to byte 3:byte 2, else PC after instruction | Same ROM row 32 and limits | K=0 untaken 2 / 7; K=1 taken 3 / 10 | **S** operation/length/both totals, polarity reconciliation; **D** encoding |

| Fact / disagreement | Source readings and exact locators | Accepted ruling / label |
| --- | --- | --- |
| Conditional path totals | W p. 145/T Table 2-10 give RSTV 1 or 3 / 6 or 12, jumps 2 or 3 / 7 or 10; conditional operation names continuation versus transfer | **S, second-source path interpretation**: smaller total untaken, larger taken. Both paths admitted. ROM introduction explicitly identifies a separate timing ROM; decode alone does not confirm totals |
| JNUI DSUB remnants; JUI trailing subtraction | T Table 2-10 says Jump on NOT UI, then saved-HL/HL−BC text; JUI explicitly says UI set but also appends HL−BC. W p. 145 JNX5 gives if(not X5), target byte3:byte2 | Take the jump headings/diagrams and W's complete predicate, discard subtraction remnants: JNK if K=0, JK if K=1. **S, second-source text reconciliation** |
| SHLX cut-off description | T Table 2-10 ends “The contents of register H are”; W p. 145 gives [DE]←L, [DE+1]←H and succeeding-location prose | Complete T with W, same origin. **S, second-source completion**; not a second written key |
| RSTV C8/CB | W p. 145 prints C8 beside binary 11001011; T Table 2-10 prints 11001011; ROM row 8 is 1100 1011 | CB, reject C8 as hex slip. **D, die-analysis encoding resolution** |
| LHLX extra data byte and parentheses | T Table 2-10 has a data box, register-indirect label and next-location prose but a malformed H equation; W p. 145 has one opcode byte and H←memory[DE+1] | One byte, no immediate operand; H receives memory at DE+1, not memory[DE]+1. **S, second-source text reconciliation** |
| JK/JX5 polarity | W p. 145 if(X5) conflicts with “reset” prose; T JUI explicitly “is set” | K=1 wins, complementing NOT UI; retain losing reset reading. **S, second-source polarity resolution** |
| DSUB truncated mask | T Table 2-10 ends Z,S,P,CY,comma; W p. 145 says all and enumerates Z,S,P,CY,AC,X5,V | All seven updated; trailing comma is incomplete, not preservation. **S, second-source flag-list completion**. P/AC/S/CY byte-step choices are **L** below; exact latch selection/sequencing stay hardware-unverified |
| ARHL written operation and ALU shift input | W p. 145/T Table 2-10 specify H7=H7, H0→L7, L0→CY; FC “Generating the shift_right input signal” says CY feeds ARHL's high-order ALU input. FC “The carry logic” describes a temporary carry latch for LDHI/LDSI; ARHL's carry-preparation micro-sequence is unpublished | **S, second-source operation; compatible die input path**: admit the written operation under the licence. Preparing carry from old H7 before the high-byte shift and old H0 before the low-byte shift reconciles the readings. This is a possible sequence, not a claim that ARHL uses the temporary latch; no demonstrated conflict and no die confirmation of sign fill. Intel-unverified |
| LDHI/LDSI extension and widths | W/T say immediate byte, W p. 144 illustrates an 8-bit index; FC describes inter-byte carry, not signed extension or boundaries | **L**: zero-extend byte 2 and wrap pair/address/PC/SP arithmetic at 16 bits. Exact 80h–FFh extension/edge behaviour stays hardware-unverified |
| Non-destination register preservation | W/T equations name destination registers/memory, not an exhaustive measured preservation inventory | Preserve unassigned registers as the **S written-assignment interpretation**; no independent exhaustive silicon claim |
| LHLX/SHLX physical order and access kinds | W p. 145/T row (SHLX completed by W) place low at DE, high at DE+1; ROM rows 12/11 show decode only | **L, adopted API mapping**: OpcodeFetch then low-before-high Data reads/writes, no dummy calls; DE equal to SP does not make them Stack. Hardware order/status/dummy activity unverified |
| RSTV saved PC/order/access set | W/T place high at SP−1, low at SP−2, vector 0040h and sequential untaken path; ROM row 8 gives no bus trace | **L, adopted restart API mapping**: OpcodeFetch, taken high(p+1) then low(p+1) Stack writes, no dummy calls; untaken only fetch. Saved next PC and precise temporal/access set remain hardware-unverified |
| JK/JNK operand fetch sequence | W/T show low byte2/high byte3, but do not establish untaken fetch consumption | **L**: taken low/high OperandFetch; untaken reads byte2 only, skips byte3 and advances PC by 3, matching the existing 8085 Jcc API by choice, not by an alleged timing proof |
| Other admitted-opcode fetch kinds | W/T lengths supply stream positions, not API kinds | **L**: OpcodeFetch for each opcode; OperandFetch for LDHI/LDSI byte2. Machine cycles need not entail bus callbacks |
| Hidden flag image | W p. 145 diagram/T Fig. 2-9; independently KV opening/circuit/bus discussion and FC opening/storage | **D/S**: seven independent flags, V1,K5,zero3; bit3 has a zero driver, not a writable eighth latch. Per-model 8085 type |
| V equation | W/T signed overflow for 8/16-bit arithmetic; KV circuit carry into top bit XOR carry out | **D/S**: internal carries, not exposed CY borrow; whole-word arithmetic uses high-byte operation with low-byte propagated carry/borrow |
| K formula conflict | W p. 145/T Fig. 2-9 O1·O2+O1·R+O2·R, W complemented O2 for subtract/compare; T replacement not visibly distinguished. KV circuit/note [4] says V XOR result sign and shows XOR failure | **Silicon wins, die-analysis conflict resolution of licensed S rule**: K=V XOR result sign when updated; independent INX/DCX path remains. The losing formula does not block admission of the correction under the explicit maintainer ruling |
| K discriminating example | W formula versus KV note [4], derived XRA 80h,80h → 00h,V=0 | Die K=0, printed K=1. Witness specification only, not a hardware measurement or another independent origin |
| ADD/ADI/ADC/ACI V/K | W arithmetic definition/T Fig. 2-9; KV circuit; ALU store_v_flag/arith_to_flags | **D/S**: signed byte overflow including carry input, K=V XOR computed-result sign |
| SUB/SUI/SBB/SBI/CMP/CPI V/K | W complemented-O2 rule/T Fig. 2-9; KV circuit/note [5]; ALU enables | **D/S**: signed byte subtraction overflow including borrow where applicable; K=V XOR temporary-result sign. CMP leaves A |
| INR V/K | W/T arithmetic rule, KV note [4]/ALU enables | **D/S**, derived +1: V iff 7F→80; K=V XOR result sign, CY preserved. FF→00 is not V |
| DCR V/K | W/T arithmetic rule, KV note [4]/ALU enables | **D/S**, derived −1: V iff 80→7F; K=V XOR result sign, CY preserved. 00→FF is not V |
| INX/DCX K; V gap | W p. 144/T Fig. 2-9 overflow/underflow; KV incrementer circuit; ALU carry_to_k_flag | K **D/S**, set only on FFFF→0000 INX / 0000→FFFF DCX, clear otherwise. V preserved as **L**, neither gives an exact effect; omitted enables alone are not an independent key |
| ANA/ANI/ORA/ORI/XRA/XRI | KV note [4] V=0,K=result sign; ALU constant carries and enables. W/T general description is not an independent exact logical-update key | Adopt V=0,K=result sign as **L following explicit die reading**, exact hardware effect unverified under two keys. XOR takes silicon-corrected K, not the printed formula |
| RRC/RAR | KV note [4] V=0, ALU V enable; no independent exact rotate key or explicit K effect | V=0 **L following explicit die reading**; K preserved **L**. Do not recompute K from preserved architectural S |
| RLC/RAL | KV note [4] A+A (+CY for RAL), ALU addition/V controls; no exact independent rotate key or explicit K effect | V from signed addition overflow **L following explicit die reading**; K preserved **L** |
| DAA V/K and top-digit discrepancy | KV note [4] restricts V to top digit 7→8 and gives K=sign XOR V; ALU “The accumulator and ACT registers” loads one combined 00h/06h/60h/66h ACT correction through two nibble enables; KV V circuit gives carry-in XOR carry-out | **L**: with old A=`a`, choose `lo=06h` iff low digit>9 or old AC; let widened `t=a+lo`, choose `hi=60h` iff `(t & F0h)>90h`, old CY or `t>FFh`; otherwise each correction part is zero. Let `c=lo+hi`, `r=(a+c) & FFh`; V iff `(~(a ^ c) & (a ^ r) & 80h)!=0`, K=V XOR result sign. CY selects `c`, not an extra addend. Reviewer counterexample: A=20h,CY=1,AC=0 → +60h → 80h,V=1,K=0, top digit 2→8; retain KV's 7→8 prose as a disagreeing/approximate reading. A=7Ah,CY=0,AC=0 → +06h → 80h,V=1,K=0 even without a high correction. Examples are library witnesses, not measurements; exact hidden behaviour lacks an independent key, existing five-flag derivations stand |
| DAD V update discrepancy; K gap | T Fig. 2-9/W define 16-bit arithmetic V; KV note [4] explicitly DAD signed overflow via high-byte addition; ALU addition controls include DAD but V/arithmetic enable lists omit it | **S / die-reading reconciliation**: take explicit V update over omission, signed 16-bit overflow. K preserved **L**, no exact K statement. Do not infer a measured preservation result from an omitted list item |
| DSUB V/K and all-seven mask | W all-flags completion of T; T Fig. 2-9 V; KV corrected K; ALU arith_to_flags includes DSUB, store_v_flag omits it | **S, second-source mask reconciliation**: V signed 16-bit subtraction overflow, K=V XOR sign16(result), all seven updated. Explicit mask wins omission; separate **L** byte-step/S/P/AC/CY choices below do not claim exact hardware sequencing |
| DSUB byte steps and S/P/AC/CY | ALU “Building the 8-bit ALU from single-bit slices” establishes an 8-bit ALU; “The accumulator and ACT registers” supplies DSUB's non-A path and distinguishes the 16-bit INX/DCX incrementer. These do not publish DSUB's full byte/latch sequence | **L** sequence: low L−C with no incoming borrow, then high H−B−b, `b=1` iff old L<old C. **L S**=high result bit7; **L P**=even parity of high result; **L AC**=high-step bit-3 carry/no nibble borrow, iff `(old H & 0Fh)>=(old B & 0Fh)+b`; **L CY**=final borrow, iff `old H<old B+b` (equivalently unsigned old HL<old BC). Use widened comparisons; incoming CY ignored. Exact hardware P/AC and latch selection/sequencing remain unverified |
| DSUB combined zero | W p. 145 full-word subtraction/all flags; FC storage explicitly multi-byte AND for DSUB | **D/S**: Z iff both result bytes zero; does not independently settle P/AC |
| ARHL V-mask discrepancy | W/T CY only; ALU explicitly includes ARHL store_v_flag and shift-right with constant carries | **Die-analysis override**: V=0 adopted as **L die-derived**, hardware exact effect unverified; CY=old L0, K/S/Z/AC/P preservation **S**. Retain losing CY-only reading |
| RDEL V-enable discrepancy | W/T CY,V; T Fig. 2-9 V for 16-bit arithmetic; ALU addition/carry paths but omitted store_v_flag entry | **S, second-source mask reconciliation**: update V from signed DE+DE+old CY overflow; CY=old D7, K/S/Z/AC/P preserved. An omission is not an explicit silicon no-update result |
| LDHI/LDSI mask | W/T no flags; FC carry-input explicitly temporary carry to preserve CY | All seven preserved **S**; CY additionally **D** |
| RSTV/SHLX/JNK/LHLX/JK mask | W/T no flags, W supplies complete SHLX row | All seven preserved **S**, not an independent full latch trace |
| Other documented instruction V/K effects | Neither W/T nor explicit die prose gives an exhaustive preservation inventory; ALU internal-use lists are not update specifications for every flag | Preserve V/K as **L** where unstated, including transfers, CMA/CMC/STC, branch/call/return, RIM/SIM, DI/EI, NOP/HLT, PUSH and ordinary interrupt entry; exceptions are the chosen rows above |
| PUSH PSW image | W/T image; KV bus drivers/FC storage independently show flag-to-bus output | **D/S**: K5,V1,zero3, with existing documented A/flag stack placement. Seven-flag model image replaces decision-11 filler; image evidence does not establish hidden-flag preservation |
| PUSH PSW hidden-flag preservation | Neither W/T image nor the cited die output paths state an exact V/K preservation effect; decision-18 §4 documented-instruction table selects the unstated-effect policy | Preserve V/K as **L**; keep this policy separate from the **D/S** output-image evidence |
| POP PSW / set_psw | KV circuit explicitly V/K databus loads; FC bus_to_flags and ALU POP PSW entry same die origin; W/T positions do not state exact load destinations | **L following explicit die reading**, hardware exact load unverified: adopt input1→V, input5→K independently, discard bit3. No implementation block pending another key; never reconstruct K from S/V |
| Default/reset | Existing decision-12 rule 9 preserves flags on reset; no hidden-bit power-up guarantee supplied | **L construction/reset policy**: Default clears V/K, reset preserves them; bit3 packs zero. No new bus calls or elapsed reset time |
| RSTV versus interrupt acceptance | W/T conditional software restart 0040h; neither settles interrupt latches | Admitted **S** operation uses explicit 0040h, not opcode & 38h. **L** preserve INTE/source latches, retire as ordinary instruction for EI delay, no acknowledge access. decision-12 INTR remains documented RST n/CALL only, no CB extension |
| decision-11 partial supersession / decision-16 ownership | Decisions 17/18 source policy; i8085 flags.rs/alu.rs/state.rs and reset/PSW conversion; accepted decision-19 | All ten operations/lengths/totals implemented; primary-only refusal and filler1/5 replaced by seven flags and chosen PSW rules. Preserve bit3=0, unsupported transaction for other unadmitted behaviour, documented operand-marker reconciliation, all 8080 rules and model-specific types. Decision-19 amends decision-16 with four flag-only hooks; further methods still require an amendment |
| 8085EXM masked CRC invariance | Pinned 8085EXM.MAC, May 2020, SHA-256 caf4bf988ee3127947e5d22877cd0a120b5d2acabbb6d0eefc39bc82d296172e; lines 38–39, 186–378, 390–392, 846–855; lines 810–825 DAD before PUSH | **Library verification scope**: all 25 masks D5h exclude bits5/3/1, snapshot masked before CRC. Changes confined to these bits cannot change judged CRCs; no V/K/ten-opcode witness, new oracle run or guarantee for other flags/state. Keep §2's 24 judged/one reported qualification |

### Z80 undocumented-behaviour sources (M6; x80-mn3.5, source pins only)

READ locally, 2026-11-13, without network access: Young's PDF title/copyright,
history, relevant opcode/flag prose and tables on rendered page images;
the supplied MEMPTR text; Banks' measurement Markdown and result-page link
inventory. The z80test archive was extracted temporarily to inspect its README,
licence, changes and test/host assembly sources (not an emulator). Full URLs,
byte sizes, all 22 wiki page hashes and specimen inventory are in
[oracles/MANIFEST.md](oracles/MANIFEST.md#z80-undocumented-behaviour-sources-x80-m6--fetch-only);
original bytes are pinned in `oracles/SHA256SUMS.fetched`. All remain fetch-only.
The supplied wiki commit association is the supervisor's retrieval record;
there is no local wiki git metadata or capture-image corpus to authenticate it
independently in this offline audit. Revision plus page hashes identify exactly
what was read. These pins make no behaviour ruling or change to decision-14.

| Path in `oracles/` | Title / author / revision or date | SHA-256 | Class, label, silicon and source locators |
| --- | --- | --- | --- |
| `zilog/z80-undocumented/young-undocumented-z80-documented-v0.91.pdf` | The Undocumented Z80 Documented — Sean Young, maintained by Jan (named in §§1.1/1.4; surname not in the document); v0.91, 18 September 2005; Wayback 20260108105255 | `6413048f39c2e735373b1fb23102599133bc64ccd0ed63b16fbc1173643d7a9d` | **C; third-party analysis.** GFDL 1.1-or-later, no invariant sections/cover texts (p. i/PDF 2); 52 PDF pages. §§3.1–3.7 pp. 11–14: index halves, SLL, ED holes/aliases, ignored/repeated prefixes, DD/FD CB copies, ED 70/71. §§2.3, 4.1–4.7 pp. 8, 15–18 and §§8.1–8.11 pp. 26–35: per-instruction bits 5/3 and an explicitly incomplete hidden-register account. §1.1 describes collection/checking; §3.4 quotes a Spectrum bus experiment, without identifying a specimen. No per-fact NMOS/CMOS scope established |
| `zilog/z80-undocumented/memptr-boo-boo-2006.txt` | MEMPTR, esoteric register of the ZiLOG Z80 CPU — Boo-boo, draft English translation Vladimir Kladov; ©2006 zx.pk.ru; supplied 2014 third-party gist copy (retrieval record: gist revision cdd2889d; raw identifier 0157588ecd91bb82eec06d85d5be0ca2a3472c30) | `78d19f0809c31dfb516150ad5e7ba747063f40bbb528439f1316539c7f0c612a` | **C; third-party analysis.** No licence grant. Opening experiment description, instruction-by-instruction MEMPTR list and footer credits; CPI/CPD/BIT probing recovers 14 low bits. Real-chip testers named Wlodek, CHRV, icebear, molodcov_alex, goodboy; ordinary Z80 parts unidentified, KP1858BM1/T34BM1 exceptions explicitly named. A mirrored account of experiments is not an identified-Zilog B capture |
| `zilog/z80-undocumented/z80decoder-wiki/Undocumented-Flags.md` | Z80Decoder wiki, Undocumented Flags — David Banks; git 49e542dd40c36ef7367cb5806292764416dc987a, includes literal update 9/7/2026; read 2026-11-13 | `a5b100b3046e88485a939b5f1a0a76536c4ec59c050ca800db4b76fa1652e9ef` | **B; hardware-measured (Zilog NMOS Z80, by David Banks)** for the author's reported verification; exact date code not given here. “Interrupted block instructions” reports LDxR/CPxR/INxR/OTxR repeat-step flags; “SCF/CCF” covers predecessor/Q, POP AF, NEC NMOS/ST CMOS differences and DD/FD prefixes. No licence grant found. Distinguish these reports from linked/simulator material and later screen results |
| `zilog/z80-undocumented/z80decoder-wiki/Unstable-CCF-SCF-Behaviour.md` and `Results-*.md` | Z80Decoder wiki, Unstable CCF/SCF Behaviour and result matrices — David Banks; same pinned git revision, no publication date in these pages; read 2026-11-13 | `485c12182ee05bffc734fd30545e763f81b6f5723eaf94aaa7da9e3c10395f01` (write-up); each result page pinned separately in MANIFEST/SHA256SUMS.fetched | **B; hardware-measured (listed Z80 parts, by David Banks)**, reported Acorn Z80 Second Processor screen experiments, 2.0/3.5/6.0 MHz; “Introduction”, “Conclusions”, “Test Program” and per-part pages. Zilog NMOS 8039/8644/8811, ambiguous 8817; Zilog CMOS 9044/0047/1930; SGS NMOS, NEC NMOS/CMOS and ST CMOS (date codes unstated). No licence grant in Markdown; images are linked, absent and unread |

Young page map: title PDF 1, copyright p. i/PDF 2, contents pp. ii–iii/PDF 3–4;
Arabic printed pages 5–52 equal their PDF page numbers. §4.1 p. 15 admits
untested hidden-register cases; §8.5 p. 30 states A-derived SCF/CCF flags without
the later Q qualification. §4.5 p. 17 discusses repeat/interruptibility without
Banks' later interrupted-repeat flag findings. No systematic NMOS/CMOS or CMOS
ED 71 output comparison is present. These are reading limits, not corrections.

The MEMPTR copy's original-host URL is cpcwiki; the supplied retrieval record
says that host refused automated access. Its actual pin is the third-party gist,
not an authenticated author-host capture. The full URL and distinction between
the supplied gist revision and raw identifier are recorded in MANIFEST. The
text credits its own experiment team while referring to Young and another
project; those linked implementations/documentation were not consulted.

Banks' eleven per-part result entries are enumerated in MANIFEST, with
aggregate pages by frequency and all results. Preserve two unresolved readings:
`Results-Zilog-NMOS-Date-Code-8817.md` and aggregate headings say NMOS, while
its own heading and image roots say CMOS; the unstable conclusions say 8634
where result filenames/headings/paths say 8644. The Markdown does not establish
package markings for SGS/NEC/ST. Capture images/chip photos are not included;
no image-level result is claimed verified. The later SCF/CCF specimen list must
not be retroactively applied to every earlier interrupted-block statement.
Capture setup notes are context; the exported NMI/simulator discussion is
excluded from evidence. Fetch uses a **git-pinned Markdown export**, not raw-page
URLs; offline verification hashes the exported pages.

Independence must be established **per fact**. Young compiles other origins
(§1.1 and bibliography [1], [2], [5]), including Cringle's exercisers. MEMPTR
cites Young but reports additional experiments by its credited team. Banks
starts with Young and credits Patrik Rak's 2012 SCF/CCF discovery; z80test's
README cites Banks. Those retellings count once. Banks' own Acorn measurements
and Rak's Spectrum CRCs have distinct reported acquisition origins where the
same fact is actually discriminated; shared prose/discovery is not another key.
ZEXALL/ZEXDOC share the Cringle lineage and unidentified hardware: falsifiers
only. A Young statement plus ZEXALL CRC is not automatically independent.
No source here is class A or qualifies for decision-18's second-source
manufacturer amendment. Class C alone never licenses; oracle CRCs never supply
the written fact. Individual NMOS/CMOS findings remain limited to named parts.

### Z80 undocumented rulings (decision-20; x80-mn3.5)

READ / hash-checked locally, 2026-11-13; no network or oracle execution.
[decision-20](<backlog/decisions/decision-20 - Z80-undocumented-behaviour-Zilog-NMOS.md>)
was **accepted by the maintainer on 2026-11-13**. Until each implementation bead
lands, the code keeps the M4 behaviour recorded above. These rows record every
selected rule and evidence gap;
the historical M4 and source-pin rows above remain historical evidence.

Keys U/T/Y/M/B/S/R, their full hashes, dates, origin accounting and exact
locators are defined in decision-20's evidence table and the source tables
above. R means the **in-memory** contents of the pinned v1.2a ZIP, not a
downloaded implementation. Only exerciser assembly was inspected. **A** =
documented by manufacturer (historical Z80/Z80A); **H** = hardware-measured
(Zilog Z80 in a 48K Spectrum, by Patrik Rak), with process/package/date code
unverified; selection for the NMOS model is a library scope choice. **L** =
adopted library rule, hardware unverified under two keys. H uses the cited
independent written statement plus R's identified acquisition only for
observables its vectors discriminate. No H label confirms elapsed states,
output-bus data or all hidden-register bits merely because a group exists.
Independent literal implementation witnesses are still required. B/S are one
Banks origin; Rak's distinct Spectrum acquisition may be a second key, its
retelling of Banks may not. No source here qualifies for decision-18's
second-source manufacturer licence.

Young pp. 8, 11–24, 26–35 were read as rendered page images, and
pp. 19–24 in full text. Young pp. 15/17/30 were reread at **200 dpi**;
TM Table 7.0-11 p. 54/PDF60 and UM p. 296/PDF310 were reread at **200 dpi**.
Disputed UM pp. 20, 94–95, 243–250, 288, 298, 301–302, 305, 309, 312–313, 316,
TM p. 57 and
Mostek MK78515 pp. 2-15/2-200 (PDF29/214) were reread in full text, as were
M, the relevant Banks pages and the R test/README/change passages. Thus rows
below distinguish actual disagreements from omissions/measurement limits.
Linked chip photographs/screens are absent and were not claimed read.

| Ruling / decision-20 locator | Written key and independent silicon key, or competing readings | Selection, label and exact limit |
| --- | --- | --- |
| ED70 / §1 | T Table 7.0-11 p. 54 explicitly gives r=110 flag-only input, 2 bytes/3 cycles/12 states. U p. 296 calls it undefined while saying set the flag; U p. 297 supplies input flag effects. Y §3.4 pp. 12–13 / §8.11 p. 35 agrees on operation. R `.in_c` lines 1000–1006 captures CPU flags/registers at input BFh | **A** for historical manufacturer's flag-only instruction/length/total; admit despite U's undefined classification. **H** for discriminated input flags/register preservation, not a new timing measurement. C preserved, H=N=0; Y/X from input under §4 |
| ED71 emitted byte / §1 | Y §3.4 p. 13 / §8.11 p. 35 states OUT (C),0, 2 bytes/12 states. R `.out_c_0` lines 1070–1075 snapshots CPU state, not port data. U p. 307 excludes r=110 | **H** for tested CPU preservation; output 00h at full BC, 2 bytes/12 states are **L** without an independent bus/timing key. Alleged CMOS FFh output lacks a pinned qualifying statement/confirmation here; record as unverified future-variant research, not an observed disagreement |
| NEG duplicates / §1 | Y §3.4 pp. 12–13; R `.neg_` lines 389–394 toggles terminal mask 38h | **H** for all ED44/4C/54/5C/64/6C/74/7C results/flags. **L** 2-byte/8-state alias timing extension from Y p. 30/U pp. 176–177; no timed R observation |
| RETN/RETI duplicate return operation / §1 | Y §3.4 pp. 12–13; R `.reti_retn` lines 1202–1207 toggles 38h, covering ED45/4D/55/5D/65/6D/75/7D | **H** for tested PC/SP/register/flag operation; **L** 2-byte/14-state alias timing (U pp. 288/290; Y p. 34). Preserve canonical ED4D fetch visibility, no device callback |
| RETI IFF conflict / §1 | Y §3.4 p. 13 / §5.3 pp. 20–21 / §8.10 p. 34 states IFF1←IFF2, including an unidentified-chip NMI experiment. U p. 288 describes no enabling of entry-disabled interrupts; Mostek MK78515 p. 2-200 explicitly says no IFF1/IFF2 effect, Table 2-1 p. 2-15 unchanged. R neither snapshots IFFs nor injects NMI | **L** choose Y's IFF2-copy for all ED returns, explicitly replacing decision-15 rule 7; hardware conflict **unverified**, not resolved by R's return CRC or by counting Y's own retelling twice. A later unequal-IFF witness tests the library rule, not silicon |
| IM duplicates / §1 | Y §3.4 pp. 12–13 maps 46/4E/66/6E→0, 56/76→1, 5E/7E→2. Quoted bus experiment has unidentified Spectrum silicon. R `.im_n` lines 1459–1465 generates all eight but does not observe mode | **L** assignments and 2 bytes/8 states (canonical U pp. 184–186); instruction presence is not mode confirmation |
| ED NOP complements / §1 | Y §3.4 pp. 12–13 says other ED entries, except block instructions, act as two NOPs; R contains no exhaustive named-silicon hole capture | **L** two bytes/8 states, two M1/R increments, no other state/port/data/stack effects; one retirement. Derived 78 operation slots/178 NOP slots; terminal prefixes are consumed bytes, not redispatch. No remaining normal ED refusal |
| Index-byte INC/DEC / §2 | Y §§3.2–3.3 pp. 11–12 / §8.4 p. 30; R `.inc_x`/`.dec_x` lines 708–721 | **H** operations/flags for DD/FD24/25/2C/2D; **L** new total 8 states/2 bytes without a timing capture |
| Index-byte immediate loads / §2 | Y §8.1 p. 26; R `.ld_x_n` lines 1306–1311 | **H** DD/FD26/2E selected byte; **L** 3-byte/11-state timing |
| Register LD index halves and ignored LD prefixes / §2 | Y §3.2 / §8.1 p. 26; R `.ld_x_x` lines 1278–1283 selects DD/FD and main40–7F, HALT skipped by `idea.asm` | **H** both H/L substitutions and B/C/D/E/A-only ignoring over tested forms; **L** 2-byte/8-state timing. No IX/IY half mixing |
| Index-byte ALU / §2 | Y §3.2 / §8.4 p. 30; R `.alo_a_x`/`.alo_a_y` lines 487–499 varies operation and high/low selection | **H** all eight ALU operations on selected index half; **L** 2-byte/8-state timing |
| Indexed-memory H/L exceptions / §2 | Y §§3.2–3.3 pp. 11–12; U pp. 78–85; R `.ld_r_xyd`/`.ld_xyd_r` lines 1285–1297 | **A/H** retain ordinary H/L with (IX/IY+d), including store sources. Main76 remains displacement-free HALT, a documented operation with **L** ignored-prefix extension; do not read every H/L use as an index byte |
| Ignored general/ED/EX prefixes / §2 | Y §3.2 p. 11 excepts EX DE,HL/EXX/ED and ignores DD/FD for other no-index operations; R's independent match is the register-LD subset above | **L** general ignoring and +1 byte/+4 states per prefix; documented main/ED semantics retained. No broad H inference from `.ld_x_x` |
| Repeated prefixes / §2 | Y §3.7 p. 14 says last DD/FD wins, ED cancels selection, CB selects indexed-CB. No independent R chain acquisition | **L** finite arbitrary chains, no count cap; one terminal instruction/retirement. Extra prefix costs accumulate outside u8 table length/cost; decision-13 rule 5 narrowed accordingly |
| Fetch, R and boundary mapping / §2 | Y §6.1 p. 23 states prefix/R counts; T §3 p. 7/U p. 6 establishes ordinary M1. Y §5.5 pp. 21–22 reports INT tests, explicitly has not tested NMI. No independent chain/timing capture | **L** instruction-wide no-poll/no-retirement/no-EI-expiry rule, one R increment per DD/FD and ordinary M1, indexed-CB d/op OperandFetch with two R increments. Q-only prefix effect is not a boundary. Repeating block refetch begins at ED, not earlier ignored prefixes |
| Long-prefix accounting / §2 | Software requirement, not a chip finding | **L** model-local selector/start PC and saturating u64 accumulated total, wrapping PC, no prefix allocation/cap. Terminal table costs remain u8; widen fact-free progress accounting as required. Infinite prefix stream need not terminate step; payload records final decode context ([last DD/FD, CB, d] or [CB]), omits earlier selectors and retains start PC; single-prefix payloads unchanged. Implemented in `cpu.rs`, witnessed in `z80_undocumented_index.rs` |
| Normal-fetch refusal closure / §§2/7 | Complete maps derived from Y plus labelled fallback rules above | **Library scope** remove fetched-byte refusals after §§1–3. Keep root Unsupported payload/rollback and decision-15 IM0 non-RST / odd-IM2 restrictions. 8080/V20 ED escape unchanged |
| SLL / §3 | Y §3.1 p. 11 / §8.7 p. 32; R `.slia_a`/`.slia_r`, `.sro_xyd*` | **H** result=(byte<<1) OR1, old bit7→C, S/Z/PV/Y/X from result, H=N=0. **L** extension totals 8/15 states for register/(HL), 23 indexed; no timing key |
| Indexed-CB memory/register copies / §3 | Y §§3.5–3.6 pp. 13–14 / §§8.7–8.8 pp. 32–33; R `.sro_xyd_r`, `.set_n_xyd_`, `.res_n_xyd_` | **H** computed rotate/shift/RES/SET result to memory plus selected ordinary B/C/D/E/H/L/A, no copy for field110; **L** physical/API ordering and write-ignoring-device behaviour (R tests RAM). Do not reread memory into destination |
| Indexed-CB BIT aliases / §3 | Y §3.5 p. 14; R `.bit_n_xyd_` lines 833–838 | **H** all low-bit encodings same test, unchanged registers/memory; **L** no Bus write mapping, four-byte/20-state new-alias totals. Modifying aliases four bytes/23 states; not timed by R |
| Full F/F' storage / §4 | Y §2.3 p. 8 / §§8.2–8.3 pp. 28–29; R `.selftest`, `.poppush_af`, `.ex_af_af` | **H** tested complete images/transfers; **library choice** independently store Y=bit5/X=bit3, preserve arbitrary input/POP/swap/output, Default zero/reset preserve. Supersedes decision-14 rule 5 zero image without claiming electrical-reset values |
| Byte ALU/INC/DEC/NEG Y/X / §4 | Y §2.3 p. 8 / §§8.4–8.5 p. 30; R corresponding immediate/register/memory/index ALU and INC/DEC/NEG descriptors | **H** bits5/3 from 8-bit result; independent literal witnesses remain required |
| CP Y/X / §4 | Y §8.4 p. 30 footnote1; R `.cp_n` and `.alo_a_*` | **H** bits from source operand, not A/subtraction result; excludes block compare |
| DAA Y/X / §4 | Y §4.7 p. 18 / §8.5 p. 30; R `.daa` | **H** bits from adjusted A. No new ruling on correction/H/carry outside the accepted manual-table domain; preserve the existing labelled DAA extension |
| CPL / accumulator-rotate Y/X / §4 | Y §§8.5/8.7 pp. 30/32; R `.cpl`, `.rlca`/`.rrca`/`.rla`/`.rra` | **H** bits from new A; other documented flag effects remain |
| CB rotate/shift and RLD/RRD Y/X / §4 | Y §8.7 p. 32; R rotate/shift groups, `.rld`/`.rrd` | **H** shifted-byte result for CB/indexed-CB; new A for nibble rotates, not memory result |
| Pair-arithmetic Y/X / §4 | Y §4.6 p. 17 / §8.6 p. 31; R `.add_hl_rr`/`.add_ix_rr`/`.add_iy_rr`/`.adc_hl_rr`/`.sbc_hl_rr` | **H** high byte of 16-bit result, not WZ/old H |
| LD A,I/R and IN Y/X / §4 | Y §§8.1/8.11 pp. 27/35; R `.ld_a_i`/`.ld_a_r`, `.in_r_c`/`.in_c` | **H** loaded-A bits; input-byte bits at observed BFh. No input sweep or injected-interrupt confirmation; general input rule follows Y |
| BIT register Y/X disagreement / §4 | Y §4.1 p. 15 explicitly copies masked test result, with BIT7,A resetting Y/X; R `.bit_n_a`/`.bit_n_r` distinguishes operand and masked-result candidates but supplies no written operand equation | **L** choose original register bits5/3 for every n. The selected rule lacks an independent pinned written statement; no two-key hardware resolution claimed. Witness BIT0,B with B=28h distinguishes it. Source reread at 200 dpi; do not silently correct Y |
| BIT indexed/HL Y/X / §§4–5 | Y §4.1 p. 15 / M opening/indexed entries; R `.bit_n_xyd*`, `.bit_n_hl` and `idea.asm`124–127 | **H** only projected address/hidden-register observations discriminated by those vectors; choose effective-address high bits for indexed, retained WZ high bits for HL. Exact WZ word/update table **L**, not inferred from projected CRC |
| BIT S/PV unknowns / §4 | U pp. 243–250 unknown S/PV versus Y §4.1 p. 15 explicit S=(n=7 and set), PV=Z; R `.bit_n_*` | **H** select measured rule, replace decision-14 rule 6 preservation for S/PV. Keep H=1,N=0,C preserved. This is separate from Y's incorrect register Y/X claim |
| Block transfer/compare Y/X / §§4/6 | Y §4.2 p. 16; R `.ldi`…`.cpdr` lines 912–980 | **H** nonrepeat/terminating transfer t=A+data, compare t=A−data−halfborrow, Y=t.bit1/X=t.bit3. Extra repeat-state changes are separate, not a conflicting final-state measurement |
| Block I/O full flags conflict / §4 | U pp. 298,301–302,305,309,312–313,316 says N=1, C unaffected, S/H/PV unknown; T p. 54 marks C unknown. Y §4.3 pp. 16–17 / §7.1 p. 24 / §8.11 p. 35 states data-derived N, k-derived H/C/PV, decremented-B S/Z/Y/X. R corresponding block-I/O groups independently discriminate CPU flags (input BFh only) | **H** where discriminated: S/Z/Y/X from decremented B, N=data7, H=C overflow of k, PV=parity((k&7) XOR B), input k=data+wrapped(C±1), output k=data+L_after. Silicon acquisition prevails over U's N/C claims there; unevaluated cases use written equations as labelled library extension. Replace unknown preservation and the old 43h judging contract |
| Block-I/O port-B ordering conflict / §4 | Y §4.4 p. 17 says input decremented/output before decrement. U INI p. 298 explicitly inputs before decrement, OUTI p. 309 explicitly decrements before output; accepted decision-15 rule 4 has corresponding entry witnesses | Retain **A** documented ordering and existing API: input old B, output decremented B. R's fixed Spectrum input and unobserved output bus do not settle all high-address cases. Both readings reread in full/Young at 200 dpi |
| Y/X preservation classes / §4 | Y instruction tables pp. 26–35 and R matching ordinary transfer/exchange/flow/SET/RES/control/I/O groups | **H** for discriminated listed groups; **L** universal preservation extension to untested HALT/RST/interrupts, new NOP holes/prefixes. POP AF and EX AF,AF' transfer images rather than recomputing; do not infer flag changes from ALU internals |
| WZ model/default/reset / §5 | M opening/instruction list; U RESET p. 6/T p. 9 omit WZ; R has no reset measurement | **L** Z80-owned 16-bit latch, supplied/snapshotted state, Default=0/reset preserves, full rollback, no new bus call; replaces decision-15 hidden-register omission. Not a power-up assertion |
| WZ A-memory loads/stores / §5 | M `LD A,(addr)/(rp)` and `LD (addr)/(rp),A`; R `.ld_a_mem`/`.ld_a_rr`/`.ld_mem_a`/`.ld_rr_a` stays on one page, varies store A only bit0 | **L** reads address+1; stores high=A, low=address+1 without carry. BM1 clone high=0 exception not used. CRC does not confirm exact +1, arbitrary high byte or carry suppression |
| WZ pair memory / stack exchange / §5 | M `LD (addr),rp`/`LD rp,(addr)`/`EX (SP),rp`; R corresponding fixed-page/limited-word groups | **L** nn+1 for pair loads/stores; new stack-loaded pair for EX. Include HL/IX/IY and ED63/6B; full word and all carries unverified |
| WZ arithmetic old-H discrepancy / §5 | Y §4.1 p. 15 says old H; M `ADD/ADC/SBC rp1,rp2` says old destination+1. They differ when low-byte carry changes an exposed bit. R pair-arithmetic vectors lines 768–801 include low FFh and varied high bits | **L** exact word=old destination+1 following M; R can discriminate some exposed carries, not whole-word storage. Y's incomplete old-H approximation is not used at carry edges; full equation remains two-key-unverified |
| WZ RLD/RRD and control flow / §5 | M RLD/RRD, JR/DJNZ/RET/RETI/RST and JP/CALL headings; R fixed-page branch/return/nibble groups, no RST capture | **L** HL+1 for nibble rotates; taken relative/return/restart destination; immediate JP/CALL target even untaken; JP(register)/untaken relative/return preserve. Exact full-word/path extensions and RST unverified |
| WZ I/O / §5 | M names IN A,(n)/(C), OUT (n),A/(C),A; R `.in_*`/`.out_*` has n=FEh/no output-bus capture | **L** immediate IN full old-A:n+1, immediate OUT high=A/low=n+1 no carry; BC+1 for register I/O. All-register/ED70/ED71 completion is a library extension beyond M's named A forms, not a quotation |
| WZ memory block operations / §5 | M LDIR/LDDR, CPI/CPD, CPIR/CPDR headings; R completed/self-overwrite groups | **L** continuing LDxR/CPxR p+1 (p=ED address); terminating LDxR preserves; CPI/CPD and terminating CPxR old WZ±1. No arbitrary full-word/interrupt confirmation |
| WZ block I/O / §5 | M INI/IND/INIR/INDR says old BC±1 each execution; OUTI/OUTD/OTIR/OTDR says decremented BC±1. M explicitly gives instruction-address+1 only for continuing LDxR/CPxR. R `.inir_nop`/`.indr_nop` corrected v1.2a MEMPTR CRCs are F3B1BE2F | **L**, with explicit falsification/replacement: MEASURED in-memory vector/post-BIT calculation gives 0A537B63 for both under M's continuing-INxR rule, F3B1BE2F for both with WZ=(address_of_ED+1) modulo 65536. Choose the latter on B_after!=0; retain old BC±1 for INI/IND/terminating INxR, decremented BC±1 for output. Missing independent written INxR equation and whole-word evidence remain explicit; a CRC cannot license the word. Consistency calculations also reproduce LDxR EC8A174B, self-overwriting LDxR 88679670, CPIR C83E7419, CPDR B9D65AD9, OTIR 9FF6297F, OTDR D7CF070F; output groups hide continuing WZ behind the terminating assignment. Full audit in decision-20 §5; no oracle program/hardware run |
| WZ indexed operands, interrupts and unlisted preservation / §5 | M indexed/interrupt headings and opening preservation sentence; R indexed page-local vectors/no injected interrupts | **L** effective address for every indexed-memory form; actual handler destination on successful interrupt; unlisted operations preserve, refusals restore. No complete word/reset/interrupt second key |
| z80memptr discrimination / §5 | R `idea.asm`124–127 loads HL,data then BIT0,(HL); `z80memptr.asm`/README expressly disclaims stress coverage | **Library judging limit** only WZ13/11 plus C reaches post-BIT flags CRC; other word bits invisible, setup state not arbitrary. Fixed-page addresses cannot distinguish many +1/old/new candidates. Require per-vector differential audit; no PASS currently claimed |
| SCF/CCF predecessor rule / §6 | Y §8.5 p. 30 unconditional A versus B “SCF/CCF” / S stable pattern A; R `z80ccf`, post-CCF macros and `.scf`/`.ccf`/paired/POP groups independently acquire predecessor observations | Select deterministic pattern A: flag-modifying predecessor→A5/3, else old F5/3 OR A5/3. **hardware-measured (Zilog NMOS Z80 date code 8039, by David Banks)**, corroborated only in R's separate specimen scope. Not a universal NMOS equation; preserve each origin/part label |
| Q specimen disagreement / §6 | Full S Conclusions reports 8039 stable A, 8811 stable unconditional-A B, unstable conclusion 8634 versus result page/path8644. `Results-Zilog-NMOS-Date-Code-8817.md` own heading/image roots say CMOS. All these Markdown pages reread in full | **Library selection** pattern A, no randomness/profile; 8811 and unstable cases explicitly outside equivalence claim. Preserve 8634/8644 identity ambiguity; exclude 8817 from named-NMOS evidence. Per-page SHA-256 values in decision-20/source manifest; no absent images claimed audited |
| Q classification/prefix/reset / §6 | B explicitly excludes POP AF; update literally9/7/2026 says DD/FD before SCF/CCF loses predecessor state; R post-CCF groups cover many ordinary instructions, not that new prefix case or reset/interrupts | **L** boolean classification, not F-byte inequality; DD/FD clears Q without retiring; terminal instruction class sets it. Universal table, EX AF classification, Q=0 Default/reset/interrupt entry and full snapshot/rollback are library bookkeeping; prefix update lacks independent matching key |
| Repeat-boundary flags / §6 | B “Interrupted block instructions” exact PC/high-bit and I/O PV/H equations; R `.ldir_nop`/`.lddr_nop` and `.inir_nop`/`.indr_nop` exposes repeat state without actual interrupt | **H** only discriminated LDxR/INxR observations; **L** arbitrary p routing, CPxR/OTxR interrupt paths/full formula domain. Apply extra-five-state changes on every continuing iteration, before boundary; no override on termination. Y's omission is an incomplete earlier account, not a contradictory intermediate capture; no SCF date-code retroassignment |
| LD A,I/R P/V / §6 | U pp. 94–95 explicitly notes interrupt-during-execution P/V=0; Y ordinary IFF2 statement; R has no injected interrupt vector | **A** manufacturer note, **L** decision-15 rule10 boundary pending-note window for NMI/INT/expiry/rollback; NMOS-exclusive scope, CMOS contrast and precise timing remain unverified |
| EI/NMI page / §6 | Banks NMI notes20October2018, wiki pin49e542d…, SHA-256 `0effe42fea3f73f690985d648b673d50c98552adf6ea1f807d1fa15c256b50ba`; opening says simulator IFF2=0 is erroneous, desired IFF1=0/IFF2=1; no independent named-chip acquisition | **Unverified; no new anomaly.** Exclude linked simulator/transistor discussion as confirmation. Retain **L** decision-15 EI-delay mapping, EI sets both, NMI clears IFF1/preserves IFF2. Page title cannot license an IFF2-clearing quirk |
| Odd IM2 claim / §7 | Y §5.2 p. 20 reports an unidentified-chip odd-vector experiment against the manual's even-vector requirement (U p. 20/T p. 57, already recorded M4); R no vector acceptance capture | No independent identified-silicon key: retain **L library refusal**, not a hardware odd-vector claim. IM0 stays RST-only; normal fetched completion does not extend acknowledge scope |
| NMOS target and variants / §7 | Maintainer2026-11-13; S's differing NMOS/CMOS/maker reports do not establish family equivalence | **Library scope** replace decision-13 rule6 assumption with Zilog NMOS and explicit Q selection. Future decision-6 variants need source/witness/scope decision; alleged ED71/PV CMOS differences remain unverified. 8080 stays Unsupported; zero-dependency model boundary unchanged |
| ZEXDOC nine new runnable groups / §7 | Existing §2 binary/source audit, groups7/29–32/49/51/60–61 | **Library campaign plan** `alu8rx`, `incxh`, `incxl`, `incyh`, `incyl`, `ld8ixy`, `ld8rrx`, `rotxy`, `rotz80`; preserve actual DD encoding in misleading incyh/incyl names. Historical58 judged/9 not run is not a new67-group PASS. Replace ED70 refusal rendezvous before admission, measure host overhead |
| ZEXALL Y/X judging / §7 | Pinned source495f686b…; lines34–39 unidentified real-Z80 CRCs; descriptor block all67 masksFFh, `flgmsk` line1102 uses selected descriptor mask | **Falsifier only** including bits5/3; no named independent hardware key. Require its own COM/source/digest/encoding audit and safe host rendezvous; keep GPL fetch-only, no PASS here |
| R six-variant host/campaign / §7 | Pinned ZIP README, loader.bas/Makefile/main.asm/print.asm/idea.asm and `src/z80*.asm` | **Library plan** Spectrum-style oracle host at8000h/data8800h, writable RAM/stack, ROM-service stubs/system variables, FEh precheckBFh, caller EI/RET; run full/doc/flags/docflags/ccf/memptr separately. Documented flags masks do not exclude undocumented encodings. No silent I/O skips, no code/test run here, core remains guest-agnostic |

Decision-20 §2 implementation (`x80-mn3.14`): register-only index-half
operations retain the H labels above; new costs/general ignoring/chaining and
access/boundary/diagnostic conventions remain L, and ordinary H/L beside
indexed memory remain A/H. `z80_undocumented_index.rs` pins all 256 main
positions under both selectors, literal half results/flags, memory exceptions,
ED cancellation, last selector, HALT, indexed-CB, no mid-chain polls, long
PC/R wrapping, block refetch and refusal payload/rollback. This introduces no
new hardware claim. Bits 5/3 and later decision-20 sections remain deferred.

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
claim. An oracle alone never licenses a behavioral fact: a CRC confirms a stated
fact, it cannot state one. Under [decision-17](<backlog/decisions/decision-17 - Two-key-evidence-rule-for-undocumented-behaviour.md>),
a program whose expected values come from identified hardware (for example,
z80test) may supply the independent silicon confirmation only for facts its
groups discriminate, after pinning the artifact and hardware provenance.
Unidentified-hardware CRCs (ZEXALL, ZEXDOC) remain falsifiers only. This eligibility
does not pin or approve a candidate for use. Never read another emulator's source
or documentation. When an oracle disagrees, record initial state, bytes, measured
result, artifact pin, both source readings and the ruling under §1. Do not
silently mask a divergence; documented behavior keeps its primary-source rule.

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

The 8085 host supplies `OUT FFh; RET` at BDOS entry 0005h through
OpcodeFetch/OperandFetch responses, preserving Data reads of the stack word
at 0006h. Decision-18 admits all 256 opcodes, so 08h (DSUB) can no longer be
an Unsupported rendezvous. Only the stub's reserved-port write requests a
boundary stop; the runner reads C/DE, services BDOS 2/9 and resumes at RET.
This is a private harness ABI, not a hardware claim, and costs one additional
step / 10 cycles per BDOS call. The 8080 08h rendezvous, digest identities
and output grammar are unchanged.
Decision-20 §1/§7 replaces the Z80 ED70 rendezvous with the output/RET
boundary-stop ABI described below. Synthetic CLI fixtures
witness repeated calls, split budgets, preserved registers/flags/stack word,
port/BDOS errors and the overall cap at the OUT boundary.

Results, 2026-11-13, MEASURED: the release `--model 8085` commands in
[commands.md, Oracle runs](backlog/docs/commands.md#oracle-runs) pass: 8080PRE
exits 0 at 1,061 steps / 7,750 cycles; 8085EXM exits 0 at 2,919,050,719 steps /
23,955,340,446 cycles, with all 24 judged groups PASS. Compared with the prior
08h rendezvous (1,060 / 7,740 and 2,919,050,442 / 23,955,337,676), the
respective increases are exactly 1 step / 10 cycles (one BDOS call) and
277 steps / 2,770 cycles (277 BDOS calls). The long ALU register group
(`alu8r`, 8085EXM.MAC:136,201–207) prints PASS, CRC 0cfd75b5; its result is
reported, not judged, because the expected CRC remains unverified. Both programs
halt at the boot-entry stub (PC 0001h). These runs retain the D5h mask and
unchanged digest-keyed output grammar; they do not witness the ten extensions.
The accompanying release 8080 reruns also exit 0, PASS, with unchanged output
and totals: 8080PRE 1,060 steps / 7,804 cycles; 8080EXM 2,919,050,421 steps /
23,803,378,398 cycles (all 25 groups PASS).

Closure rerun, 2026-11-13, **MEASURED on `766906e`**: all four release runs
reproduce these verdicts and step/cycle totals exactly, including the reported,
not judged long-ALU CRC `0cfd75b5`. Commands follow commands.md with local
fetch-only program paths, offline/locked Cargo, target directory
`/private/tmp/x80-x80-mn3.8-target`, empty `RUSTC_WRAPPER`, and
`RUSTFLAGS="-C link-arg=-isysroot -C link-arg=$SDKROOT"`. No new silicon
confirmation follows from these reruns.

Rejected or not chosen: the Microcosm 8080/8085 CPU diagnostic (CPUDIAG/TST8080, 1980;
licence unclear, so fetch-only would be the most it could be), the SuperSoft CPU test
(commercial, binary only). No hardware-captured single-step corpus for the 8080 was
identified (searched 2026-11-12: the SingleStepTests and ProcessorTests collections);
emulator-generated corpora are not hardware witnesses. Hardware
captures describe the measured silicon, not every family member.

### Z80 exercisers: fetch-only pins and documented-encoding audit

Maintainer decision, 2026-11-13 (x80-bi2.5): M4 covers documented Z80 behavior
only. That historical refusal policy is superseded by decision-20 §§1–2:
all ED terminals, index halves and ignored/repeated DD/FD now execute with the
A/H/L limits in §1. SLL and undocumented indexed-CB terminals still refuse
pending §3; bits 5/3 and WZ/MEMPTR await later sections. The two programs
below are falsifying oracles, never sources of semantics. GPL files, including
sources and COPYING, are fetched locally and never committed. Pins and fetch
entries occupy separate Z80 exerciser blocks in the shared stash.

Origin for both: Frank D. Cringle, © 1994, original yaze 1.14 distribution,
[begoon/yaze at 993d8647eabf95c61e49b134198a240ab577358b](https://github.com/begoon/yaze/tree/993d8647eabf95c61e49b134198a240ab577358b).
The supplied lineage identifies an unchanged import (README; import commit
4424f29, 2012-10-30). Raw files are pinned at that full commit, not a branch.
Only the test sources and COPYING were inspected; emulator implementation is
not evidence.

| Program | Origin paths at the above pin | SHA-256 (.COM; source) | Licence | CRC provenance and role |
| --- | --- | --- | --- | --- |
| Preliminary Z80 test | `test/prelim.com`, `test/prelim.z80` | `3b3578f19030a4df7e25ce852f763af26053b12582a576c4dffb014aa7c590d1`; `afefdd15c42ea8d0c123ad099108962ca5ad26b3ffd043d2ba578c2b01cab1fd` | GPL-2.0-or-later (source header) | No CRC: explicit comparisons and completion text; **judged**, documented encodings only |
| ZEXDOC | `test/zexdoc.com`, `test/zexdoc.z80` | `34923a7ed82285d3038b2d54bd64899e12173eebb61f9d07b4fc72e78af2ae8f`; `4f7b83d5ed1b4ce34f1b6f19b996e40588e3698db9aa2e31bece377db5b91d99` | GPL-2.0-or-later (source header) | Source lines 34–39 say expected CRCs were measured on a real Z80; no part, revision, per-group contributor or capture is identified. 67 groups: **58 judged**, **9 not run** |

Local paths are under `oracles/z80-exercisers/`; byte sizes, raw URLs and the
COPYING pin are in the stash's MANIFEST and SHA256SUMS.fetched. COPYING is the
GPL v2 text; the program headers permit version 2 or later. The claimed hardware
origin of the CRCs is not independently authenticated. Source lines 49–51
attribute the documented flag selection to the Mostek Z80 Technical Manual;
that attribution does not make Mostek or these CRCs an authority for M4.
“Judged” below means eligible for a later run, not a measured PASS.

#### Binary/source correspondence and audit method

READ/MEASURED, 2026-11-13: a local parser expanded `tstr` and `tmsg`
(zexdoc.z80:170–193), resolving `msbt=0103h`, little-endian words, byte
truncation of -1 and zero instruction padding. Each of the 67 complete
96-byte descriptors matched a unique contiguous region of zexdoc.com:
one mask byte, three 20-byte vectors, four CRC bytes, and a 30-byte dot-padded
message plus '$'. All 67 little-endian descriptor pointers then matched the
binary in source table order (lines 100–168), including its zero terminator.
Thus the requested table order, messages, expected CRC bytes and all vector
data correspond to this source. This is a structural build check, not a
whole-file reassembly or an execution result.

The table starts at file offset 003Ah / guest address 013Ah; its terminator
is at 00C0h / 01C0h. The first descriptor is at 00C2h / 01C2h and descriptors
are 0060h bytes apart. Within a descriptor, mask is +00h, base is +01h,
increment is +15h, shift is +29h, CRC is +3Dh and message is +41h. CRCs below
are the four stored bytes in display order (big-endian per `phex8`), not
little-endian integers.

For classification, the four instruction bytes of each base, increment and
shift vector were expanded: every subset of increment bits is XORed with
the base, with each eligible shift bit inverted in turn; the initial base
is included. State-vector variations cannot change the opcode bytes.
The remaining bytes of each four-byte instruction slot execute as NOP
padding where unused. `tlp` (lines 767–775) skips both 76 and DD/FD 76,
so `ld8rr` and `ld8rrx` do not execute those HALTs. All other generated
encodings count, even when the group label omits them. This enumerates
instruction space, not the much larger full state campaign.

Authority: Zilog **UM008011-0816**, “Instruction Op Codes”, Tables 6–15 and
20 / Figure 39, checked against rendered page images. Printed page n is
PDF page n+14 (332 PDF pages). In the table, Tn means Table n of that
revision; r means B,C,D,E,H,L,A; ALU means ADD,ADC,SUB,SBC,AND,XOR,OR,CP.
Hexadecimal braces denote alternatives; DD/FD denotes either prefix.
Indexed displacements are fixed at 01h except in `ld8rrx`'s indexed memory
forms, where the displacement is 00h. Variations of the index-register state
do not vary the displacement byte. Absolute nn is 0103h in the memory groups;
immediate n/nn vary only where their vectors permit.

The summary tables contain visible printing errors, including the H column
labelled F, C8 in place of CB, and DB for DEC BC. Detailed opcode bit fields
disambiguate these: LD r,r' p. 71, DEC ss p. 201, RLC r p. 213, and BIT/SET/RES
pp. 243, 245, 247, 249, 251, 253, 255, 257, 259. In particular Table 14's
incorrect indexed BIT entries do not introduce a hole: the detailed pages
give DD/FD CB d (46h+8×bit). No emulator implementation was used to fill gaps.

| # / source label | Table slot: file offset / guest address → descriptor guest address | Executed instruction(s) / opcode set | Flag mask | Expected CRC | Verdict / primary citation or offending encodings |
| --- | --- | --- | --- | --- | --- |
| 1. `adc16` | `003A/013A → 01C2` | ADC/SBC HL,{BC,DE,HL,SP}; ED {42,4A,52,5A,62,6A,72,7A} | `C7` | `F8B4EAA9` | **judged** — T13 p. 52 |
| 2. `add16` | `003C/013C → 0222` | ADD HL,{BC,DE,HL,SP}; {09,19,29,39} | `C7` | `89FDB635` | **judged** — T13 p. 52 |
| 3. `add16x` | `003E/013E → 0282` | ADD IX,{BC,DE,IX,SP}; DD {09,19,29,39} | `C7` | `C133790B` | **judged** — T13 p. 52 |
| 4. `add16y` | `0040/0140 → 02E2` | ADD IY,{BC,DE,IY,SP}; FD {09,19,29,39} | `C7` | `E8817B9E` | **judged** — T13 p. 52 |
| 5. `alu8i` | `0042/0142 → 0342` | ALU A,n; {C6,CE,D6,DE,E6,EE,F6,FE} n | `D7` | `48799360` | **judged** — T11 p. 50 |
| 6. `alu8r` | `0044/0144 → 03A2` | ALU A,r/(HL); 80–BF | `D7` | `FE43B016` | **judged** — T11 p. 50 |
| 7. `alu8rx` | `0046/0146 → 0402` | ALU A,{IXH,IXL,IYH,IYL} | `D7` | `A4026D5A` | **not run** — index halves: DD/FD {84,85,8C,8D,94,95,9C,9D,A4,A5,AC,AD,B4,B5,BC,BD} (absent from T11 p. 50) |
| 8. `alu8x` | `0048/0148 → 0462` | ALU A,(IX/IY+01); DD/FD {86,8E,96,9E,A6,AE,B6,BE} 01 | `D7` | `E849676E` | **judged** — T11 p. 50 |
| 9. `bitx` | `004A/014A → 04C2` | BIT 0–7,(IX/IY+01); DD/FD CB 01 {46,4E,56,5E,66,6E,76,7E} | `53` | `A8EE0867` | **judged** — T14 pp. 55–56; pp. 247, 249 |
| 10. `bitz80` | `004C/014C → 0522` | BIT 0–7,r/(HL); CB 40–7F | `53` | `7B55E6C8` | **judged** — T14 pp. 55–56; pp. 243, 245 |
| 11. `cpd1` | `004E/014E → 0582` | CPD/CPDR; ED {A9,B9} | `D7` | `A87E6CFA` | **judged** — T10 p. 49 |
| 12. `cpi1` | `0050/0150 → 05E2` | CPI/CPIR; ED {A1,B1} | `D7` | `06DEB356` | **judged** — T10 p. 49 |
| 13. `daa` | `0052/0152 → 0642` | DAA/CPL/SCF/CCF; {27,2F,37,3F} | `D7` | `9B4BA675` | **judged** — T12 p. 51 |
| 14. `inca` | `0054/0154 → 06A2` | INC/DEC A; 3C,3D | `D7` | `D18815A4` | **judged** — T11 p. 50 |
| 15. `incb` | `0056/0156 → 0702` | INC/DEC B; 04,05 | `D7` | `5F682264` | **judged** — T11 p. 50 |
| 16. `incbc` | `0058/0158 → 0762` | INC/DEC BC; 03,0B | `D7` | `D2AE3BEC` | **judged** — T13 p. 52; DEC ss p. 201 |
| 17. `incc` | `005A/015A → 07C2` | INC/DEC C; 0C,0D | `D7` | `C284554C` | **judged** — T11 p. 50 |
| 18. `incd` | `005C/015C → 0822` | INC/DEC D; 14,15 | `D7` | `4523DE10` | **judged** — T11 p. 50 |
| 19. `incde` | `005E/015E → 0882` | INC/DEC DE; 13,1B | `D7` | `AEC6D42C` | **judged** — T13 p. 52 |
| 20. `ince` | `0060/0160 → 08E2` | INC/DEC E; 1C,1D | `D7` | `E175AFCC` | **judged** — T11 p. 50 |
| 21. `inch` | `0062/0162 → 0942` | INC/DEC H; 24,25 | `D7` | `1CED847D` | **judged** — T11 p. 50 |
| 22. `inchl` | `0064/0164 → 09A2` | INC/DEC HL; 23,2B | `D7` | `FC0D6D4A` | **judged** — T13 p. 52 |
| 23. `incix` | `0066/0166 → 0A02` | INC/DEC IX; DD {23,2B} | `D7` | `A54DBE31` | **judged** — T13 p. 52 |
| 24. `inciy` | `0068/0168 → 0A62` | INC/DEC IY; FD {23,2B} | `D7` | `505D51A3` | **judged** — T13 p. 52 |
| 25. `incl` | `006A/016A → 0AC2` | INC/DEC L; 2C,2D | `D7` | `56CD06F3` | **judged** — T11 p. 50 |
| 26. `incm` | `006C/016C → 0B22` | INC/DEC (HL); 34,35 | `D7` | `B83ADCEF` | **judged** — T11 p. 50 |
| 27. `incsp` | `006E/016E → 0B82` | INC/DEC SP; 33,3B | `D7` | `5DACD527` | **judged** — T13 p. 52 |
| 28. `incx` | `0070/0170 → 0BE2` | INC/DEC (IX/IY+01); DD/FD {34,35} 01 | `D7` | `20581470` | **judged** — T11 p. 50 |
| 29. `incxh` | `0072/0172 → 0C42` | INC/DEC IXH | `D7` | `6F463662` | **not run** — index halves: DD {24,25} (absent from T11 p. 50) |
| 30. `incxl` | `0074/0174 → 0CA2` | INC/DEC IXL | `D7` | `027BEF2C` | **not run** — index halves: DD {2C,2D} (absent from T11 p. 50) |
| 31. `incyh` | `0076/0176 → 0D02` | INC/DEC IXH; descriptor says IYH | `D7` | `2D966CF3` | **not run** — index halves: DD {24,25}; descriptor says IYH (absent from T11 p. 50) |
| 32. `incyl` | `0078/0178 → 0D62` | INC/DEC IXL; descriptor says IYL | `D7` | `FBCBBA95` | **not run** — index halves: DD {2C,2D}; descriptor says IYL (absent from T11 p. 50) |
| 33. `ld161` | `007A/017A → 0DC2` | LD {BC,DE},(nn); ED {4B,5B} nn | `D7` | `4D45A9AC` | **judged** — T7 p. 45 |
| 34. `ld162` | `007C/017C → 0E22` | LD HL,(nn); 2A nn | `D7` | `5F972487` | **judged** — T7 p. 45 |
| 35. `ld163` | `007E/017E → 0E82` | LD SP,(nn); ED 7B nn | `D7` | `7ACEA11B` | **judged** — T7 p. 45 |
| 36. `ld164` | `0080/0180 → 0EE2` | LD {IX,IY},(nn); DD/FD 2A nn | `D7` | `858BF16D` | **judged** — T7 p. 45 |
| 37. `ld165` | `0082/0182 → 0F42` | LD (nn),{BC,DE}; ED {43,53} nn | `D7` | `641E8715` | **judged** — T7 p. 45 |
| 38. `ld166` | `0084/0184 → 0FA2` | LD (nn),HL; 22 nn | `D7` | `A3608B47` | **judged** — T7 p. 45 |
| 39. `ld167` | `0086/0186 → 1002` | LD (nn),SP; ED 73 nn | `D7` | `16585FD7` | **judged** — T7 p. 45 |
| 40. `ld168` | `0088/0188 → 1062` | LD (nn),{IX,IY}; DD/FD 22 nn | `D7` | `BA102A6B` | **judged** — T7 p. 45 |
| 41. `ld16im` | `008A/018A → 10C2` | LD {BC,DE,HL,SP},nn; {01,11,21,31} nn | `D7` | `DE391969` | **judged** — T7 p. 45 |
| 42. `ld16ix` | `008C/018C → 1122` | LD {IX,IY},nn; DD/FD 21 nn | `D7` | `227DD525` | **judged** — T7 p. 45 |
| 43. `ld8bd` | `008E/018E → 1182` | LD A,(BC)/(DE); {0A,1A} | `D7` | `B0818935` | **judged** — T6 p. 42 |
| 44. `ld8im` | `0090/0190 → 11E2` | LD r/(HL),00; {06,0E,16,1E,26,2E,36,3E} 00 | `D7` | `F1DAB556` | **judged** — T6 p. 42 |
| 45. `ld8imx` | `0092/0192 → 1242` | LD (IX/IY+01),n; DD/FD 36 01 n | `D7` | `26DB477E` | **judged** — T6 p. 42 |
| 46. `ld8ix1` | `0094/0194 → 12A2` | LD {B,C,D,E},(IX/IY+01); DD/FD {46,4E,56,5E} 01 | `D7` | `CC1106A8` | **judged** — T6 p. 42 |
| 47. `ld8ix2` | `0096/0196 → 1302` | LD {H,L},(IX/IY+01); DD/FD {66,6E} 01 | `D7` | `FA2A4D03` | **judged** — T6 p. 42 |
| 48. `ld8ix3` | `0098/0198 → 1362` | LD A,(IX/IY+01); DD/FD 7E 01 | `D7` | `A5E9AC64` | **judged** — T6 p. 42 |
| 49. `ld8ixy` | `009A/019A → 13C2` | LD {IXH,IXL,IYH,IYL},00 | `D7` | `24E8828B` | **not run** — index halves: DD/FD {26,2E} 00 (absent from T6 p. 42) |
| 50. `ld8rr` | `009C/019C → 1422` | LD r/(HL),r/(HL); 40–7F except 76 (skipped) | `D7` | `744B0118` | **judged** — T6 p. 42 |
| 51. `ld8rrx` | `009E/019E → 1482` | DD/FD 40–7F except 76 (skipped); register moves and indexed loads/stores with d=00 | `D7` | `478BA36B` | **not run** — DD/FD on register-only LD: 49 encodings per prefix, including 24 index-half and 25 ignored-prefix forms; exact set below (absent from T6 p. 42) |
| 52. `lda` | `00A0/01A0 → 14E2` | LD (nn),A / LD A,(nn); {32,3A} nn | `D7` | `C9262DE5` | **judged** — T6 p. 42 |
| 53. `ldd1` | `00A2/01A2 → 1542` | LDD/LDDR, BC=1; ED {A8,B8} | `D7` | `94F42769` | **judged** — T9 p. 48 |
| 54. `ldd2` | `00A4/01A4 → 15A2` | LDD/LDDR, BC=2; ED {A8,B8} | `D7` | `5A907ED4` | **judged** — T9 p. 48 |
| 55. `ldi1` | `00A6/01A6 → 1602` | LDI/LDIR, BC=1; ED {A0,B0} | `D7` | `9ABDF6B5` | **judged** — T9 p. 48 |
| 56. `ldi2` | `00A8/01A8 → 1662` | LDI/LDIR, BC=2; ED {A0,B0} | `D7` | `EB59891B` | **judged** — T9 p. 48 |
| 57. `neg` | `00AA/01AA → 16C2` | NEG; ED 44 only | `D7` | `6A3C3BBD` | **judged** — T12 p. 51 |
| 58. `rld` | `00AC/01AC → 1722` | RRD/RLD; ED {67,6F} | `D7` | `955BA326` | **judged** — Fig. 39 p. 54 |
| 59. `rot8080` | `00AE/01AE → 1782` | RLCA/RRCA/RLA/RRA; {07,0F,17,1F} | `D7` | `251330AE` | **judged** — Fig. 39 p. 54 |
| 60. `rotxy` | `00B0/01B0 → 17E2` | RLC/RRC/RL/RR/SLA/SRA/SLL/SRL (IX/IY+01); DD/FD CB 01 {06,0E,16,1E,26,2E,36,3E} | `D7` | `713ACD81` | **not run** — SLL: DD CB 01 36 and FD CB 01 36 (absent from Fig. 39 p. 54) |
| 61. `rotz80` | `00B2/01B2 → 1842` | RLC/RRC/RL/RR/SLA/SRA/SLL/SRL r/(HL); CB 00–3F | `D7` | `EB604D58` | **not run** — SLL: CB 30–37 (absent from Fig. 39 p. 54) |
| 62. `srz80` | `00B4/01B4 → 18A2` | RES/SET 0–7,r/(HL); CB 80–FF | `D7` | `8B57F008` | **judged** — T14 pp. 56–57; pp. 251, 253, 259 |
| 63. `srzx` | `00B6/01B6 → 1902` | RES/SET 0–7,(IX/IY+01); DD/FD CB 01 {86,8E,96,9E,A6,AE,B6,BE,C6,CE,D6,DE,E6,EE,F6,FE} | `D7` | `CC63F98A` | **judged** — T14 pp. 56–57; pp. 255, 257, 259 |
| 64. `st8ix1` | `00B8/01B8 → 1962` | LD (IX/IY+01),{B,C,D,E}; DD/FD {70,71,72,73} 01 | `D7` | `04626ABF` | **judged** — T6 p. 42 |
| 65. `st8ix2` | `00BA/01BA → 19C2` | LD (IX/IY+01),{H,L}; DD/FD {74,75} 01 | `D7` | `6A1A8831` | **judged** — T6 p. 42 |
| 66. `st8ix3` | `00BC/01BC → 1A22` | LD (IX/IY+01),A; DD/FD 77 01 | `D7` | `CCBE5A96` | **judged** — T6 p. 42 |
| 67. `stabd` | `00BE/01BE → 1A82` | LD (BC)/(DE),A and LD A,(BC)/(DE); {02,0A,12,1A} | `D7` | `7A4C114F` | **judged** — T6 p. 42 |

The nine **not run** labels are `alu8rx`, `incxh`, `incxl`, `incyh`,
`incyl`, `ld8ixy`, `ld8rrx`, `rotxy`, `rotz80`. Additional audit details:

- `ld8rrx` has 49 offending register-only combinations for each prefix:
  `DD/FD (40h + 8×dst + src)`, with dst and src each in {0,1,2,3,4,5,7}.
  Of these, 24 per prefix involve field 4 or 5 (index halves), and the other
  25 merely prefix non-indexed register moves. Examples are DD 44 (LD B,IXH),
  FD 65 (LD IYH,IYL), DD 40 (ignored prefix on LD B,B), and FD 7F (ignored
  prefix on LD A,A). Only its 14 indexed memory forms per prefix are documented.
  Its possible 76 forms are skipped by the guest, not accepted as documented.
- `incyh` and `incyl` actually contain DD 24/25 and DD 2C/2D, matching the
  binary; the IYH/IYL messages do not match those IX-half encodings. Do not fix
  these fetched bytes or infer a different test from the message.
- `stabd` also executes 0A/1A loads, because increment mask 18h varies both
  bits 3 and 4 of base 02h. `ld8rr` includes (HL) moves despite its message.
  `ld8im` and `ld8ixy` use immediate 00h; their shift vectors vary A,
  not the immediate. These label/coverage differences are in the matched bytes.
- All indexed CB groups retain the low three opcode bits at 110:
  `bitx` and `rotxy` use mask 38h, `srzx` uses 78h (also varying RES/SET).
  **No DD/FD CB d register-copy form is generated**. `srzx` is judged.
  The exclusions in the two rotate groups are SLL, not register-copy forms.
- No group generates an ED hole or a duplicate NEG encoding: NEG is ED 44
  only, and the other ED variations stay within T7/T9/T10/T13. ED 70 is
  explicitly labelled “Undefined op code” at p. 296 (PDF 310); it is absent.
  No additional undocumented encoding was found in the guest's setup, CRC,
  output or dispatch routines; NOP, DI/EI are T20 p. 64, other support
  instructions use T6–T15 as cited above.
- Mask D7h compares S,Z,H,P/V,N,C; C7h omits H; BIT mask 53h compares Z,H,N,C,
  excluding S/P/V as well as bits 5 and 3. No group compares bits 5 or 3
  after its result mask. Masking flags does not legalize undocumented opcodes.

The later oracle host must match the **original .COM digest above**, compact
the in-memory pointer table to retain these 58 entries in order and write a
zero terminator. It must report all nine excluded labels as **not run**, without
executing their cases, rewriting their CRCs or suppressing Unsupported.
The file on disk stays unchanged. No filtering or Z80 harness is implemented
by this provenance change.

#### Preliminary test and host services

READ, prelim.z80:37–341: **judged**, no undocumented encoding found in the
program or its expanded macros. It uses LD (8/16-bit, indexed and stack),
EX AF,AF', EXX, CP, AND, RRCA, INC A/HL/IX/IY, JP/CALL/RET (including conditions),
JR Z/NZ/C/NC and DJNZ. The source's `reladr` macro expands only documented
IX/IY memory addressing, never index-half registers; it includes signed
displacements -128, -1, 0, +1, +127. Primary opcode tables: T6 p. 42,
T7 p. 45, T8 p. 47, T11 p. 50, T13 p. 52, Figure 39 p. 54,
T15 p. 59. Its emergency HALT is documented (T20 p. 64); the adjacent
FF byte, normally unreachable data, would also be a documented RST 38h
(T17 p. 61), not a prefixed hole. There is no CRC mask or expected CRC.
Errors terminate silently early, or print the failing return address later;
only the completion message signifies success.

| Host requirement | Preliminary test | ZEXDOC |
| --- | --- | --- |
| Load / entry | 0100h, writable RAM | 0100h, writable RAM including self-modifying instruction and mask bytes |
| BDOS | CALL 0005h: C=2, character E; C=9, '$'-terminated text at DE (`allok`, `conout`) | Same functions through register-preserving `bdos` wrapper (lines 1194–1203); no disk or input functions |
| SP | Host must supply a usable initial SP for CALL at 0111h, before the first LD SP. Then SP=04ACh (`regs1`), 04D4h (`regs2+20`), finally 0600h (`stack`); confirmed immediates in .COM | Entry loads SP from little-endian word 0006h (lines 80–81); host supplies safe stack RAM. Each test saves host SP, POPs state using msbt+2, loads trial SP from spbt, captures post-test SP and restores host SP (lines 1075–1095) |
| Exit | JP 0000h on success and failure; HALT at 010Fh is an emergency, not success | JP 0000h after completion; group failures print ERROR and continue |
| Completion | “Preliminary tests complete”; found at file 034Ah / guest 044Ah with '$' terminator | “Tests complete”; requires all 58 retained groups, results in pinned order and explicit not-run reporting for the other nine |
| Interrupts / ports | No port device or injected interrupt needed | No ports; guest DI/EI around each case; no injected interrupt needed |

The existing `x80-oracle` host recognizes only the 8080/8085 digests and models.
Its synthetic 08h Unsupported rendezvous at BDOS entry cannot be carried into
a Z80 host: 08h is documented EX AF,AF' (T8 p. 47). Selecting an appropriate
Z80 host rendezvous and implementing digest-keyed table filtering belong to
the later harness bead. Neither of these Z80 programs was executed in this audit.

Results structure (x80-bi2.26): the digest-selected Z80 harness now runs prelim
as judged and compacts ZEXDOC's verified in-memory table to the 58 judged groups,
reporting the nine excluded labels as `not run (decision-14)`. Its private BDOS
rendezvous now follows decision-20 §1/§7: instruction reads at 0005h supply
`OUT FFh; RET`; only this stub's full-width output arms the next-boundary
stop. The runner reads C/DE, services BDOS and resumes RET. This private
harness ABI adds one retired instruction / 11 states per call. ED70 executes
as input outside the stub. The audit above excludes ED holes from both
guests. The host preserves 08h as EX AF,AF' and the Data stack word at 0006h.
The exact output grammar and release commands are in
[commands.md, Oracle runs](backlog/docs/commands.md#oracle-runs).

Debug smoke, 2026-11-13, MEASURED: prelim exits 0, PASS, at 898 steps /
8,703 cycles, boot-stub halt PC 0001h. ZEXDOC with cycle cap 5,000,000,000
prints OK for `adc16`, `add16`, `add16x`, then times out during `add16y` at
618,447,422 steps / 5,000,000,002 cycles; exit 1, FAIL (incomplete fourth group
and no completion). All nine exclusions are reported. This is partial debug
execution evidence only.

Release campaign, 2026-11-13, MEASURED (supervisor, release build, commands in
[commands.md](backlog/docs/commands.md#oracle-runs)): prelim prints "Preliminary
tests complete", exit 0, PASS, 898 steps / 8,703 cycles. ZEXDOC with cycle cap
10^12 prints OK for all 58 judged groups in table order, then "Tests complete";
exit 0, PASS, 4,432,084,992 steps / 35,946,550,360 cycles, about 97 s wall
time. The nine excluded groups are reported "not run (decision-14)". The pass
found no falsifying case; it licenses no behaviour, which stays pinned to the
cited pages and witnesses.

#### Additional pinned Z80 oracles (M6; x80-mn3.5)

READ/MEASURED locally, 2026-11-13: source/header/host and SHA-256 audit only;
neither program was run. These pins supersede the earlier candidate-only
inventory without expanding the M4 host or its documented-encoding audit.

| Program / revision | Files and SHA-256 | Licence | Decision-17 class, label and oracle role |
| --- | --- | --- | --- |
| ZEXALL — Frank D. Cringle, ©1994, original yaze 1.14; same `993d8647eabf95c61e49b134198a240ab577358b` distribution pin as ZEXDOC | `z80-exercisers/zexall.com`: `6e2da55147a04f28d303d5da6a1e6b771557ac244653590a0f24a2d39c8537e8`; `z80-exercisers/zexall.z80`: `495f686b3f4418d50a110ba6ae4c6d86d715506e1eb6117040bd6f1eb4e011c5` | GPL-2.0-or-later (source header), same pinned COPYING as ZEXDOC; fetch-only | **B, reported hardware measurement; hardware-measured (Z80; exact part and measurer unverified)**. Source lines 34–39 say empirical real-Z80 CRCs but identify no maker, revision or capture. **Falsifier only**, never an independent identified-silicon confirmation. Includes flag bits 5/3 and undocumented encodings; shares origin with ZEXDOC |
| Zilog Z80 CPU test suite (z80test) — Patrik Rak, v1.2a, 2 December 2023 (`changes.txt`) | `z80-exercisers/z80test-1.2a.zip`: `7df0443d703e6b3114ea04b4cdef3e13b91421c62e37185c1036d06864cacbaf` | MIT, `license.txt`, ©2012–2023 Patrik Rak; kept fetch-only | **B; hardware-measured (Zilog Z80 in a 48K Spectrum, by Patrik Rak)**. README: “values obtained from a real 48K Spectrum with Zilog Z80 CPU”. No package/date code or NMOS/CMOS revision specified. Identified-hardware CRCs **may confirm stated facts only where the groups discriminate them**, with an independent written origin; never license a fact alone |

ZEXALL host, read at `org`, `start`, `done`, `bdos`: writable load/entry 0100h,
SP from word 0006h, CALL 0005h BDOS functions 2 (E) and 9 ('$'-terminated DE),
JP 0000h exit. DI/EI surrounds cases; no injected interrupt/port device needed.
A source/binary correspondence and digest/encoding audit is still required for
this artifact before a harness run; the existing ZEXDOC audit does not establish
ZEXALL correspondence. No expectation was derived from distribution emulator code.

z80test archive URL, complete host requirements and coverage are also recorded
in [MANIFEST's exerciser section](oracles/MANIFEST.md#z80-cpu-exercisers-x80-oracles-falsify-never-define--fetch-only).
Locators in the extracted `z80test-1.2a/` tree:

- `readme.txt` and `src/z80*.asm`: full/register and flags-only, documented-mask,
  post-CCF, MEMPTR/BIT and screen variants. `src/tests.asm` `testtable` and
  descriptors include index halves, SLL (SLIA), DD/FD CB copies/BIT aliases,
  per-instruction bits 5/3, ED aliases, IN (C), OUT (C),0, and self-overwriting
  LDxR/INxR cases exposing repeat-step state. This is not an exhaustive audit
  of holes, ignored prefixes or injected-interrupt paths. OUT CRCs observe
  CPU/memory state, not output-bus bytes; the ED 71 mnemonic is no confirmation
  of its emitted value or of any NMOS/CMOS difference.
- `readme.txt`: CCF CRCs assume the genuine Zilog variant, warns of other-maker
  differences and board-dependent randomness. `z80ccfscr` visualises it. MEMPTR
  vectors were **not designed to stress MEMPTR**, so many cases may escape.
  `changes.txt` v1.2a corrects interrupted INIR/INDR MEMPTR CRCs. NEC/ST-named
  descriptors do not establish identified-hardware CRC provenance for those parts.
- `src/loader.bas`, `Makefile`, `main.asm`, `print.asm`: Spectrum TAP/BASIC host,
  CLEAR 32767 and USR 32768; code load/entry **8000h**. Writable RAM and caller
  stack; CHAN-OPEN **1601h** (A=2), RST **10h** printing and Spectrum system
  variables (IY **5C3Ah**). Printing temporarily enables interrupts; tests use DI.
  Ordinary exit restores HL'/IY, EI, RET to BASIC/caller and prints a final pass
  or failure count. `z80ccfscr.asm` writes Spectrum screen/attributes and loops
  without normal exit; no CP/M CALL 5 service applies.
- `main.asm` `.incheck`, `idea.asm`, `tests.asm`: Spectrum I/O, OUT (FEh),07h
  and expected IN (FEh)=BFh, otherwise affected input tests report a precheck
  failure and are avoided. `idea.asm` warns that moving the **8800h** data area
  changes expected CRCs. Preserve linked addresses and self-modifying code.

Pinning/eligibility is not a PASS or a blanket witness. Banks/Rak shared
SCF/CCF discovery/prose and Young/Cringle references require per-fact origin
accounting as in §1. Cringle's exact silicon/capture remains unverified;
z80test names its machine and maker but not a specimen/revision. No hardware
behaviour ruling or harness change is made by this audit.

#### z80test host: TAP pins and correspondence (x80-mn3.12)

MEASURED locally, 2026-11-13, from members decoded in memory after verifying the
ZIP pin above. Every used member also passes its central-directory CRC-32;
all four blocks of every TAP pass their XOR checksum. Members remain fetch-only.
These are artifact/layout measurements and harness conventions, **no new hardware
claim**. The runtime reader repeats the audit before executing requested images.

| TAP member under `z80test-1.2a/` | SHA-256 (MEASURED) | CODE payload bytes / table address (MEASURED) |
| --- | --- | --- |
| `z80full.tap` | `cdfebeff306e5c0756ecb3295a1aebb8e71c336b2de35ecc3c0397146d73813c` | 14298 / 887Ah |
| `z80doc.tap` | `4b06ce9fd517fd5f5a86d4ff2ef05a3f0ab7ed20eb075b87b0623d42fbd3e4bd` | 14298 / 887Ah |
| `z80flags.tap` | `4dbe34a28ed564ac782b538736e1a1c57c562876165a3294bd93af25ea3df653` | 14298 / 887Ah |
| `z80docflags.tap` | `cf64a8d66f248bc054c5255f3a51ba50a511b166b29054b7192ca91e306a528d` | 14298 / 887Ah |
| `z80ccf.tap` | `a35b3b269b58371015cf033f28a08ad1ac92cce6dd27b428caf67b7f9ce8f5b7` | 14783 / 887Fh |
| `z80memptr.tap` | `444582ddfa4d05711b6235e743ddf68295231e97ba92cc838c5d09a106c9a10f` | 14298 / 887Ah |

| Audit / harness boundary | READ locators in pinned v1.2a / MEASURED correspondence |
| --- | --- |
| Build/loader/entry | `src/Makefile:14–19`, `loader.bas:1–2`, `main.asm:7–14`: BASIC autostart 10, CLEAR 32767/USR 32768, CODE header/payload length agreement and load/entry 8000h; entry DI/PUSH IY/EXX/PUSH HL bytes agree. Five images end at B7DAh exclusive; CCF ends at B9BFh. No assembler rebuild was performed |
| Workspace and tables | `idea.asm:7–9,264–289`, `main.asm:22–24`; data=8800h, data.jump=8810h (INC BC or post-CCF, then JP), workspace page retained; table follows at 887Ah or 887Fh. `tests.asm:32–232,297–1465`, `testmacros.asm:35–101`: all 160 pointers in order, zero terminator, contiguous descriptors, flag masks, names, variant-selected big-endian CRCs and precheck bytes agree; final descriptor ends exactly at CODE payload end. No correspondence mismatch found; instruction/vector bytes were not independently reassembled |
| Host services/system variables | `main.asm:14,68–72`, `print.asm:8–9,67–80`: A=2 CHAN-OPEN 1601h and A-output RST 10h receive harness-owned fetch-only RETs with boundary stops, +1 step/+10 cycles each. Service fetches must start an instruction after an audited call with the expected return word at SP: main CALL printinit at 8005h / return 8008h, then LD A,2 / JP 1601h; printchr RST 10h / return to its following DI. The six TAPs' printinit addresses are 815Bh/815Ah/815Ch/8160h/815Ah/815Dh in variant-table order; their RST sites are printinit+51h. Call-site bytes are checked before execution. Synthetic witnesses cover valid tail-call/output costs, stray JP 0010h, prefixed terminal fetch at 0010h, unaudited CALL 1601h/RST 10h, forged return context and incorrect stack words/pointers; invalid entries supply no RET and capture no output. Writable 64 KiB RAM, caller SP=7FF0h/sentinel 7FFEh, initial IY=5C3Ah; EI/RET stops before sentinel fetch (+0 overhead). Main/print read no system-variable memory; ERR-NR at 5C3Ah is initialised FFh as a harness convention; stubs require no other variables |
| Archive input bound | MEASURED pinned ZIP size 61595 bytes; the host reads at most 61596 bytes and rejects excess before SHA-256 or parsing. An endless-reader witness verifies the exact read bound; a sparse oversized-file CLI witness verifies rejection before pinning |
| Interrupts and I/O | `main.asm:9,102–117`, `print.asm:72–75`, `idea.asm:46–47,87–104`, `tests.asm:229–230,984–1103`: no interrupt wait, generated HALT skipped, RST/HALT omitted from table; no 50 Hz injection required, INT/NMI remain low. FEh outputs accepted/counted; inputs return BFh after suite OUT FEh=07h, independent of upper address bits; unexpected ports/setup are host failures. Input groups are never patched out |
| Results and limits | `main.asm:26–72,74–168`: numbered lines, CRC pairs, skipped failcheck alternatives, `.incheck` failures and final counts parsed with complete-group/order/count checks. Four NEC/ST failcheck groups are separately diagnostic, not judged. `z80doc*` uses documented flag masks, not documented encodings; CCF and MEMPTR observe their limited projections (decision-20 §§5–7). Refusal/budget/partial output cannot pass; no oracle result licenses CPU behaviour (decision-17) |

MEASURED 2026-11-13, release `z80test all`, 10000000000-step / 600-second budgets
per variant, against the unchanged Z80 model at `f7964bb628e2e65cd5f154841ac6f6f0fa139971`.
**Non-final baseline, not conformance.** The former `ED 7C` refusal is gone.
All six now stopped on `DD BD` in group 026 `ALO A,[HX,LX]`, after 26/160
complete result lines (22 judged, four diagnostic).
The refused next instruction is one-based step `retired steps + 1`. No final
count/caller exit or input group was reached: `.incheck` failures=0 means no
coverage here, not a precheck/input PASS. No program member or CPU rule changed.

| Variant | Judged OK / FAILED | Diagnostic executed / skipped | Retired steps | Cycles including host RETs | Refusal PC / next step | Host RET overhead steps / cycles (MEASURED) |
| --- | --- | --- | --- | --- | --- | --- |
| z80full | 0 / 22 | 4 / 0 (all FAILED) | 82980170 | 498660881 | 832Bh / 82980171 | 1443 / 14430 |
| z80doc | 22 / 0 | 0 / 4 | 67170652 | 403887624 | 832Fh / 67170653 | 500 / 5000 |
| z80flags | 0 / 22 | 4 / 0 (all FAILED) | 35820790 | 244178264 | 832Bh / 35820791 | 1444 / 14440 |
| z80docflags | 22 / 0 | 0 / 4 | 28889812 | 196932282 | 832Fh / 28889813 | 506 / 5060 |
| z80ccf | 0 / 22 | 4 / 0 (all FAILED) | 38116887 | 259157671 | 8335h / 38116888 | 1442 / 14420 |
| z80memptr | 0 / 22 | 4 / 0 (all FAILED) | 36117410 | 247441047 | 832Bh / 36117411 | 1445 / 14450 |

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
- Historical M3 library PSW image: decision-11 chose bits 5/3/1 as 0/0/1.
  Decision-18 supersedes that image with stored K/V and zero bit 3; the
  undocumented closure list below records the current rules and witnesses.
  D5h still masks hidden bits out of documented-flag oracle comparisons.
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

### Residual facts at the 8085 undocumented closure (2026-11-13)

READ implementation at `766906e`: decision-18 §§2–5 and decision-19 are
implemented. This index restates their chosen facts and the §1 rulings, without
upgrading their evidence. **S** = documented by second source (CA80C85B),
unverified for Intel 8085; **D** = die analysis (8085; exact part unverified);
**L** = adopted library rule, exact hardware effect unverified. D/S retains
both scopes. Decode confirmation does not establish operations, timings or
physical bus traces. W/T count as one origin; KV/FC/ALU/ROM as another, not
four independent keys. The full revision/date/hash pins and exact locators
remain in §1 and decision-18 §1.

One row per implemented fact or instruction contract follows. Witness names
are exact test functions: **U** = `x80-core/tests/i8085_undocumented.rs`,
**V** = `x80-core/tests/i8085_vk_flags.rs`, **T** =
`x80-core/tests/i8085_timing.rs`, **C** = `x80-core/tests/i8085_coverage.rs`,
**H** = `x80-oracle/tests/cli.rs`; other paths are given in full. Machine-cycle
counts remain source context in §1; x80 reports only T-state totals.

| Implemented fact / retained limit | Label | Independent witness (file key :: test name) |
| --- | --- | --- |
| All 256 fetched bytes execute: the documented set plus ten always-on extensions; decision-11's ten-hole refusal is superseded | L admission under licensed S/D facts | C :: `every_admitted_opcode_completes_a_step`; `literal_sets_partition_all_bytes_once` |
| DSUB 08, one byte, HL←HL−BC, 10 states | S operation/length/total; D encoding ROM row 22 | U :: `dsub_word_result_and_all_seven_literal_flags`; T :: `op_08` |
| ARHL 10, one byte, signed HL shift right with H7 retained, H0→L7, old L0→CY, 7 states; carry-input prose is compatible, not die confirmation of sign fill | S operation/length/total; D encoding row 25 | U :: `arhl_sign_fill_low_bit_carry_cleared_v_and_preserved_other_flags`; T :: `op_10` |
| RDEL 18, one byte, DE left through old CY, old D7→CY, 10 states | S operation/length/total; D encoding row 23 | U :: `rdel_carry_in_out_and_word_overflow_preserve_k_and_documented_latches`; T :: `op_18` |
| LDHI 28, two bytes, DE←HL+byte 2 with HL preserved, 10 states | S operation/length/total; D encoding row 24 | U :: `ldhi_ldsi_unsigned_byte_two_and_wrapped_sum_preserve_all_flags_and_base`; T :: `op_28` |
| LDSI 38, two bytes, DE←SP+byte 2 with SP preserved, 10 states | S operation/length/total; D encoding row 24 | U :: `ldhi_ldsi_unsigned_byte_two_and_wrapped_sum_preserve_all_flags_and_base`; T :: `op_38` |
| RSTV CB, one byte, V=1 saves return address high at SP−1/low at SP−2 and enters 0040h with SP−2, else sequential; CB resolves W's C8 typo | S operation/length; D encoding row 8 and typo resolution | U :: `rstv_both_paths_next_pc_stack_wrap_vector_and_ordinary_latch_retirement`; T :: `op_cb` |
| SHLX D9, one byte, [DE]←L and [DE+1]←H with DE preserved, 10 states; W completes T's cut-off high-byte sentence | S operation/length/total and completion; D encoding row 11 | U :: `shlx_lhlx_low_before_high_data_even_at_sp_and_wrapped_address`; T :: `op_d9` |
| JNK DD, three bytes, jump to byte 3:byte 2 iff K=0; reject T's stray DSUB text | S operation/length and text reconciliation; D encoding row 32 | U :: `jnk_jk_both_polarities_only_low_operand_when_untaken_with_pc_wrap`; T :: `op_dd` |
| LHLX ED, one byte, L←[DE], H←[DE+1] with DE preserved, 10 states; reject T's extra data box and malformed parentheses | S operation/length/total and text reconciliation; D encoding row 12 | U :: `shlx_lhlx_low_before_high_data_even_at_sp_and_wrapped_address`; T :: `op_ed` |
| JK FD, three bytes, jump to byte 3:byte 2 iff K=1; T's set/W's equation prevail over W's reset prose and stray subtraction | S operation/length and polarity reconciliation; D encoding row 32 | U :: `jnk_jk_both_polarities_only_low_operand_when_untaken_with_pc_wrap`; T :: `op_fd` |
| RSTV smaller total is V=0 continuation (6 states), larger is V=1 restart (12); no independent Intel timing measurement | S conditional-path interpretation | U :: `rstv_both_paths_next_pc_stack_wrap_vector_and_ordinary_latch_retirement`; T :: `op_cb` |
| JK/JNK smaller total is untaken (7 states), larger is taken (10); no independent Intel timing measurement | S conditional-path interpretation | U :: `jnk_jk_both_polarities_only_low_operand_when_untaken_with_pc_wrap`; T :: `op_dd`; `op_fd` |
| Whole-set timing inventory includes 256 admitted bytes and 27 double-path conditionals | L inventory of admitted totals | T :: `table_covers_256_admitted_bytes_and_27_double_path_conditionals` |
| Non-destination registers remain unchanged; assignments are the written basis, exhaustive hardware preservation is unverified | S written-assignment interpretation; L exact preservation contract | U :: all seven tests compare the complete expected CPU, including `dsub_word_result_and_all_seven_literal_flags` |
| LDHI/LDSI zero-extend immediate 80h–FFh; signed-extension hardware edges remain unverified | L operand-width rule | U :: `ldhi_ldsi_unsigned_byte_two_and_wrapped_sum_preserve_all_flags_and_base` |
| Pair results, PC/SP and addresses wrap at 16 bits, including FFFFh/0000h edges | L width rule | U :: `dsub_word_result_and_all_seven_literal_flags`; `ldhi_ldsi_unsigned_byte_two_and_wrapped_sum_preserve_all_flags_and_base`; `shlx_lhlx_low_before_high_data_even_at_sp_and_wrapped_address`; `rstv_both_paths_next_pc_stack_wrap_vector_and_ordinary_latch_retirement`; `jnk_jk_both_polarities_only_low_operand_when_untaken_with_pc_wrap` |
| Each admitted opcode is OpcodeFetch; LDHI/LDSI byte 2 is OperandFetch; no bus callback is implied merely by a machine cycle | L API mapping | U :: `dsub_word_result_and_all_seven_literal_flags`; `arhl_sign_fill_low_bit_carry_cleared_v_and_preserved_other_flags`; `rdel_carry_in_out_and_word_overflow_preserve_k_and_documented_latches`; `ldhi_ldsi_unsigned_byte_two_and_wrapped_sum_preserve_all_flags_and_base` |
| LHLX reads low then high as Data, SHLX writes low then high as Data, including DE=SP; no dummy accesses | L API order/kinds/access set, physical trace unverified | U :: `shlx_lhlx_low_before_high_data_even_at_sp_and_wrapped_address` |
| RSTV taken saves next PC high-before-low as Stack; untaken only fetches; neither has acknowledge/dummy calls | L restart API mapping, saved-PC/physical trace unverified | U :: `rstv_both_paths_next_pc_stack_wrap_vector_and_ordinary_latch_retirement` |
| JK/JNK taken fetch low/high operands; untaken fetch byte 2 only, skip byte 3 and advance PC by 3 | L API mapping, totals do not prove physical operand consumption | U :: `jnk_jk_both_polarities_only_low_operand_when_untaken_with_pc_wrap` |
| Seven independently stored flags pack as S Z K AC 0 P V CY: K5, V1 and zero3; no eighth latch | D/S image | V :: `psw_images_push_pop_and_independent_loaded_flags`; `x80-core/tests/i8085_registers.rs` :: `decision_18_image_stores_v_k_and_discards_bit_3` |
| Updated arithmetic K uses V XOR computed-result sign; silicon corrects the printed W/T universal formula, not an invariant over POP or INX/DCX | D conflict resolution of S rule | V :: `arithmetic_and_compare_all_sources_and_immediates`; `logical_clears_v_and_uses_result_sign_including_xor_conflict`; `psw_images_push_pop_and_independent_loaded_flags`; `pair_increment_decrement_wraps_and_clears_k_otherwise` |
| ADD/ADI/ADC/ACI V is signed byte overflow including carry input, K=V XOR result sign | D/S | V :: `arithmetic_and_compare_all_sources_and_immediates` |
| SUB/SUI/SBB/SBI V is signed byte subtraction overflow including borrow; internal overflow is distinct from exposed CY borrow, K=V XOR result sign | D/S | V :: `arithmetic_and_compare_all_sources_and_immediates` |
| CMP/CPI use the temporary subtraction result for V/K while preserving A | D/S | V :: `arithmetic_and_compare_all_sources_and_immediates` |
| INR V iff old byte=7Fh, otherwise clear, K=V XOR result sign; CY preserved, FF→00 is not overflow | D/S derived +1 specialization | V :: `inr_dcr_all_destinations` |
| DCR V iff old byte=80h, otherwise clear, K=V XOR result sign; CY preserved, 00→FF is not overflow | D/S derived −1 specialization | V :: `inr_dcr_all_destinations` |
| INX BC/DE/HL/SP sets K only for old pair=FFFFh, clears it otherwise; independent incrementer path | D/S | V :: `pair_increment_decrement_wraps_and_clears_k_otherwise` |
| DCX BC/DE/HL/SP sets K only for old pair=0000h, clears it otherwise | D/S | V :: `pair_increment_decrement_wraps_and_clears_k_otherwise` |
| INX/DCX preserve V, no exact independently confirmed effect | L | V :: `pair_increment_decrement_wraps_and_clears_k_otherwise` |
| ANA/ANI/ORA/ORI/XRA/XRI clear V and set K=result sign; XRA 80h,80h gives K=0 rather than printed-formula K=1 | L following explicit die reading; counterexample is no silicon measurement | V :: `logical_clears_v_and_uses_result_sign_including_xor_conflict` |
| RRC/RAR clear V | L following explicit die reading | V :: `rotates_update_v_but_preserve_independent_k_and_szap` |
| RLC/RAL set V from signed left-add overflow (old CY enters RAL), equivalently old/new A sign differs | L following explicit die reading | V :: `rotates_update_v_but_preserve_independent_k_and_szap` |
| All four rotations preserve independent K, rather than recomputing from stored S | L unstated-effect policy | V :: `rotates_update_v_but_preserve_independent_k_and_szap` |
| DAA uses combined correction c=lo+hi: lo=06h iff low digit>9 or AC, hi=60h iff widened a+lo has high digit>9, old CY, or byte carry; otherwise each part is zero | L combined-ACT rule retaining existing five-flag derivation | V :: `daa_overflow_uses_combined_correction`; `x80-core/tests/i8085_daa.rs` :: `exhaustive_daa_all_1024_accumulator_carry_auxiliary_inputs` |
| DAA V is signed overflow of old A+c and K=V XOR result sign; CY selects c, not an extra addend; 20h+60h and 7Ah+06h refute the restricted top-digit reading | L exact hidden flags lack an independent key | V :: `daa_overflow_uses_combined_correction` |
| DAD V is signed word overflow including low-byte carry; explicit W/T/KV update prevails over ALU enable-list omission | S / die-reading reconciliation | V :: `dad_signed_word_overflow_includes_low_byte_carry_and_preserves_k` |
| DAD preserves K | L unstated-effect policy | V :: `dad_signed_word_overflow_includes_low_byte_carry_and_preserves_k` |
| DSUB updates all seven flags; W completes T's truncated mask, explicit update wins ALU enable-list omission | S mask reconciliation | U :: `dsub_word_result_and_all_seven_literal_flags` |
| DSUB V is signed HL−BC word overflow including low-byte borrow, K=V XOR word-result sign | S, silicon-corrected K | U :: `dsub_word_result_and_all_seven_literal_flags` |
| DSUB Z iff the whole word is zero, combined across both byte steps | D/S | U :: `dsub_word_result_and_all_seven_literal_flags` |
| DSUB subtracts L−C with no incoming borrow, then H−B−low borrow, ignoring incoming CY; exact latch sequence unverified | L byte-step rule | U :: `dsub_word_result_and_all_seven_literal_flags` |
| DSUB S is high-result bit 7 | L | U :: `dsub_word_result_and_all_seven_literal_flags` |
| DSUB P is even parity of the high result only | L exact hardware selection unverified | U :: `dsub_word_result_and_all_seven_literal_flags` |
| DSUB AC iff (old H & 0Fh)>=(old B & 0Fh)+low borrow, widened; high-step bit-3 carry/no nibble borrow | L exact hardware selection unverified | U :: `dsub_word_result_and_all_seven_literal_flags` |
| DSUB CY iff old H<old B+low borrow, widened, equivalently unsigned old HL<old BC | L final-borrow rule | U :: `dsub_word_result_and_all_seven_literal_flags` |
| ARHL clears V despite W/T's CY-only mask; die-derived override is not fully confirmed hardware behavior | L | U :: `arhl_sign_fill_low_bit_carry_cleared_v_and_preserved_other_flags` |
| ARHL preserves K/S/Z/AC/P | S written mask | U :: `arhl_sign_fill_low_bit_carry_cleared_v_and_preserved_other_flags` |
| RDEL V is signed DE+DE+old CY word overflow; explicit written CY/V mask wins omitted enable-list entry | S mask reconciliation | U :: `rdel_carry_in_out_and_word_overflow_preserve_k_and_documented_latches` |
| RDEL preserves K/S/Z/AC/P | S written mask | U :: `rdel_carry_in_out_and_word_overflow_preserve_k_and_documented_latches` |
| LDHI/LDSI preserve all seven flags, with additional die evidence for temporary-carry CY preservation | S mask; D additionally for CY | U :: `ldhi_ldsi_unsigned_byte_two_and_wrapped_sum_preserve_all_flags_and_base` |
| RSTV/SHLX/JNK/LHLX/JK preserve all seven flags | S written masks, SHLX completed from same-origin W | U :: `rstv_both_paths_next_pc_stack_wrap_vector_and_ordinary_latch_retirement`; `shlx_lhlx_low_before_high_data_even_at_sp_and_wrapped_address`; `jnk_jk_both_polarities_only_low_operand_when_untaken_with_pc_wrap` |
| PUSH emits K5,V1,zero3 with the documented A/flag stack placement | D/S output image | V :: `psw_images_push_pop_and_independent_loaded_flags` |
| PUSH preserves V/K; image evidence alone does not establish preservation | L | V :: `psw_images_push_pop_and_independent_loaded_flags`; `remaining_documented_instructions_preserve_v_k` |
| POP/set_psw load seven independent flags, input1→V/input5→K, discard bit3 without reconstructing K | L following explicit single-origin die reading, exact POP hardware unverified | V :: `psw_images_push_pop_and_independent_loaded_flags`; `x80-core/tests/i8085_registers.rs` :: `pop_psw_style_setter_keeps_seven_independent_flags_and_a` |
| PSW API packs A high and the seven-flag image low | L API representation | V :: `psw_images_push_pop_and_independent_loaded_flags` |
| Default clears V/K, not a silicon power-up claim | L construction rule | V :: `default_clears_and_reset_preserves_hidden_flags` |
| Reset preserves V/K like the other flags, with no added bus calls or elapsed time | L reset API rule | V :: `default_clears_and_reset_preserves_hidden_flags`; `x80-core/tests/i8085_model.rs` :: `reset_preserves_registers_flags_and_sp_by_library_choice` |
| Remaining documented transfers, CMA/CMC/STC, branches/calls/returns, RIM/SIM, DI/EI, NOP/HLT preserve V/K | L unstated-effect policy | V :: `remaining_documented_instructions_preserve_v_k` |
| Ordinary external interrupt entry preserves V/K for every accepted source and INTR RST/CALL | L unstated-effect policy | V :: `ordinary_interrupt_entry_preserves_v_k` |
| Normally fetched RSTV preserves INTE/source latches and retires as an ordinary instruction for EI delay | L latch/retirement rule | U :: `rstv_both_paths_next_pc_stack_wrap_vector_and_ordinary_latch_retirement` |
| INTR remains documented RST n/CALL only; CB/RSTV and other acknowledge bytes are refused transactionally | L retained decision-12 scope, no new external instruction admission | `x80-core/tests/i8085_interrupts_intr.rs` :: `every_other_acknowledge_including_ei_di_restores_state_and_repeats` |
| decision-19 pair_step hook supplies pair transition/direction to model-owned INX/DCX flag rules | L sharing contract, effects retain labels above | V :: `pair_increment_decrement_wraps_and_clears_k_otherwise`; `x80-core/tests/i8080_arithmetic.rs` :: `inx_dcx_all_pairs_wrap_and_preserve_all_flags` |
| decision-19 rotate hook supplies operation and old/new A/CY to model-owned rotate flag rules | L sharing contract, effects retain labels above | V :: `rotates_update_v_but_preserve_independent_k_and_szap`; `x80-core/tests/i8080_logical.rs` :: `rotates_exhaustive_values_carry_and_preserved_flags` |
| decision-19 dad hook supplies operands/result to model-owned DAD flag rules | L sharing contract, effects retain labels above | V :: `dad_signed_word_overflow_includes_low_byte_carry_and_preserves_k`; `x80-core/tests/i8080_arithmetic.rs` :: `dad_all_pairs_changes_only_hl_and_carry` |
| decision-19 daa hook supplies old A/combined correction/result to model-owned DAA flag rules | L sharing contract, effects retain labels above | V :: `daa_overflow_uses_combined_correction`; `x80-core/tests/i8080_daa.rs` :: `exhaustive_daa_matches_manual_two_step_rule_for_1024_inputs` |
| All four 8080 hooks are no-ops beyond the existing executor effects; types and hardware facts remain per model, no latch/timing/bus hooks admitted | L ownership/cap rule of decisions 16/19 | The four 8080 witnesses in the preceding rows pin behavior; `x80-core/tests/boundary.rs` :: `intel_family_sources_cite_both_models_and_use_no_model_paths` pins the structural boundary; hook cap READ in `intel/mod.rs` |
| 8085 host replaces the refused-08h BDOS rendezvous with fetch-only OUT FFh; RET at 0005h; Data stack word at 0006h remains intact | L private harness ABI | H :: `intel_models_keep_console_output_with_model_specific_rendezvous_costs`; `i8085_bdos_preserves_stack_word_registers_and_flags_across_slices` |
| Stub OUT requests a boundary stop, BDOS reads C/DE and clears the request before RET; one extra step / 10 cycles per call, stable across slices/repeated calls | L host stop/retirement rule | H :: `i8085_bdos_preserves_stack_word_registers_and_flags_across_slices`; §2 measured totals |
| Overall cap at OUT stops before BDOS service; invalid ports/BDOS remain failures | L harness error/cap rule | H :: `i8085_bdos_errors_ports_and_cap_at_out_boundary_remain_failures` |

Intel specimen/revision equivalence remains unverified throughout this list.
Exact bus order/status/dummy activity, offset edge cases, POP destinations,
DSUB high-byte/latch choices and single-origin hidden-flag effects remain
unverified where labelled L; admission does not remove those limits.
8085EXM's D5h mask excludes bits 5/3/1 and cannot witness V/K, arbitrary POP
hidden bits or the ten opcodes. That mask claim is READ from the pinned
8085EXM.MAC locations in §1/decision-18 §5, not a new hardware witness;
the unmasked tests above pin the chosen library behavior. The long-ALU CRC
remains reported, not judged. The earlier M3 residuals still apply except
for the explicitly superseded PSW image.

### Residual unverified or derived facts at the M4 closure (2026-11-13)

These qualifications concern the documented Z80 model; none establishes silicon
revision equivalence. Paths below are under `x80-core/src/z80/` and witnesses
under `x80-core/tests/`. UM = UM008011-0816 rev. 11 (August 2016), TM =
03-0029-01 (1977), ALPM = 03-0002-01 (January 1978), MK = MK78515 V2.0
(October 1978); pins, page maps and the shared documentation-lineage caveat
remain in §1. This inventory collects existing decisions, reconciliations and
code/witness labels; it introduces no new hardware readings. Oracle results
remain in §2. Each bullet distinguishes a derivation, unverified reading or
library contract from a stated hardware fact.

- Library layout choices: concrete Z80 state/API names, eleven group modules, five physical maps sharing DD/FD context, prefix dispatch and model-owned execution rather than a family CPU/trait; decision-13 rules 1–4/7 (UM pp. 2–3, 18, 42, 70–316; TM pp. 44–54), `mod.rs`, `cpu.rs`, `opcodes.rs` and `boundary.rs`.
- Derived layout/timing bounds: section end pages follow the next heading; terminal no-wait entries fit two cost slots and `u8`, with maximum 23 states; decision-20 §2 narrows that bound to terminal fields, with wrapping PC and saturating u64 extra prefix costs, not a claim about every silicon encoding; decision-13 rules 2/5 (UM pp. 128–129, 221–222; TM Tables 7.0-1–11 pp. 44–54), `opcodes/tests.rs`, `z80_timing.rs`.
- Unverified assumption / library scope: one model for NMOS and CMOS, without established instruction-level equivalence or a Z80/Z80A variant; decision-13 rule 6 (UM pp. 1, 15–16; TM p. 1), §1 and `mod.rs`.
- Library admission/refusal policy and derived counts: the reconciled documented set has 698 logical terminal encodings (252/248/58/39/39/31/31); decision-20 §§1–2 supersede the old ED and index-prefix refusals, yielding 1322 executing logical terminals including ignored MAIN fallbacks; only SLL and undocumented indexed-CB terminals still refuse pending §3; decision-14 rules 1–2 (UM summary/entry pages and TM Tables 7.0-1–11 pp. 44–54 cited there), `opcodes/tests.rs`, `z80_coverage.rs`.
- Derived alternate-register resolution: the repeated B' label is read as C', from the BC' text and second diagram; §1 (UM Fig. 2 p. 2, p. 3; TM Fig. 2.0-2 p. 4), `state.rs`, `z80_registers.rs`.
- Derived ED load admission: ED 63/6B follow explicit pair fields despite compact-table omissions; §1 / decision-14 rule 1 (UM pp. 45, 103, 108; TM p. 45), `opcodes/tests.rs`, `z80_load16.rs`.
- Derived H-register heading resolution: compact F headings mean H = 100, not an F operand; §1 / decision-14 rule 1 (UM pp. 42, 50, 54, 71, 145, 213; TM pp. 44, 47, 50), `opcodes/tests.rs`.
- Derived PUSH/POP encoding resolution: use C5/D5/E5/F5, DD/FD E5 and POP AF F1 over corrupt compact cells; §1 / decision-14 rule 1 (UM pp. 45, 115, 117–119; TM p. 45), `opcodes/tests.rs`, `z80_load16.rs`.
- Derived POP IY operation-line resolution: read high at SP+1, following the description/example and second witness; §1 (UM p. 122; TM p. 45), `z80_load16.rs::pop_pairs_low_then_high_including_iy_operation_line_resolution`.
- Derived LD (nn),HL timing resolution: primary 16 and the sum of Mostek's components prevail over Mostek's printed 17; §1 (UM p. 107; MK p. 2-150), `z80_timing.rs::ld_nn_hl_resolves_mostek_17_against_primary_16`.
- Derived ADD A,(HL)/DEC BC encoding resolution: 86/0B prevail over compact 88/DB; §1 / decision-14 rule 1 (UM pp. 50, 52, 148, 201; TM pp. 47, 49), `opcodes/tests.rs`.
- Derived compact rotate resolution: RLC L = CB 05, RLC (HL) = CB 06, RRC E = CB 0B, from entry fields and the second table; §1 / decision-14 rule 1 (UM pp. 54, 213, 215, 225; TM p. 50), `opcodes/tests.rs`.
- Derived BIT/SET/RES compact-table resolution: entry bit fields and restrictions prevail over corrupt CB/terminal/DD/FD cells, admitting no extra forms; §1 / decision-14 rule 1 (UM pp. 55–57, 243–260; TM p. 51), `opcodes/tests.rs`.
- Derived compact JP/IN encoding resolution: JP C,nn = DA, JP (HL) = E9, IN A,(C) = ED 78; §1 / decision-14 rule 1 (UM pp. 59, 62, 263, 275, 296; TM pp. 52, 54), `opcodes/tests.rs`.
- Derived arithmetic bit-diagram resolution: DD 8E d and FD B6 d follow hex annotations and both tables; §1 (UM pp. 50, 151, 159; TM p. 47), `opcodes/tests.rs::arithmetic_entry_bit_diagrams_use_the_agreeing_hex_and_table_readings`.
- Derived rotate bit-diagram resolution: FD CB d 0E for RRC and CB 00011rrr for RR follow agreeing hex/compact/second-table readings; §1 (UM pp. 54, 225, 228; TM p. 50), `opcodes/tests.rs::rotate_entry_bit_diagrams_use_the_agreeing_hex_and_table_readings`.
- Derived RRCA example resolution: 11h rotates to 88h, C=1, rather than the example's 98h; §1 (UM pp. 209–210; TM p. 50), `z80_rotate_shift.rs::rrca_preserves_szpv_despite_worked_example_bit_four`.
- Decision-20 §1 ED admission replaces the old refusals: **A/H** ED70 flags-only input (TM Table 7.0-11 p. 54; UM pp. 296–297; Rak `.in_c`), **H** NEG/return operations and ED71 CPU preservation, **L** ED71 emitted zero, alias costs/IM assignments and 178 NOP holes (Young v0.91 §3.4 pp. 12–13 / §§8.5/8.10/8.11 pp. 30/34/35). `opcodes/tests.rs`, `z80_coverage.rs`, `z80_undocumented_ed.rs`; bits 5/3 await §4.
- Library F/F' image: six flags only, bits 5/3 read zero and are ignored on input, including AF/AF' loads; this is not a physical zero-bit claim; decision-14 rule 5 (UM pp. 65–66), `flags.rs`, `state.rs`, `z80_registers.rs`, `z80_load16.rs`.
- Library unknown-flag values / derived masks: preserve primary-entry unknown flags; hardware comparisons use D7h normally, 53h for BIT and 43h for block I/O, with separate unmasked preservation witnesses; decision-14 rules 5–6 (UM pp. 243–250, 298, 301–302, 305, 309, 312–313, 316), `bit_set_reset_test.rs`, `input_output.rs` and their witnesses.
- Library source precedence for definedness: follow primary H rules for CCF/pair arithmetic and unaffected C for block I/O over TM unknown marks; IN r,(C) clears H, also explicit in ALPM; §1 / decision-14 rule 6 (UM pp. 178, 188, 190, 192, 194, 196, 297–298, 301–302, 305, 309, 312–313, 316; TM pp. 48–49, 54; ALPM p. 255), `z80_general_control.rs`, `z80_arithmetic16.rs`, `z80_input_output.rs`.
- Derived BIT H/N label resolution: the second H in two flag lists means N reset; §1 / decision-14 rule 6 (UM pp. 244, 246, 248, 250; TM p. 51), `z80_bit_set_reset_test.rs`.
- Derived SBC overflow wording resolution: P/V sets on subtraction overflow; AND/OR use result parity, following the general definitions and second table over entry slips; §1 (UM pp. 67, 156, 158, 160; TM §6 p. 39, Table 7.0-4 p. 47), `alu.rs`, `z80_arithmetic_reference/`.
- Derived ADD IX/IY example resolution: operation lines prevail over the examples' 333h operand (0333h + 5555h = 5888h); §1 (UM pp. 194–197; TM p. 49), `z80_arithmetic_reference/`.
- Derived JR NC example resolution: 30 00 at 0480h reaches 0482h; self-repeat needs 30 FE, following the twice-advanced PC rule; §1 (UM pp. 269–270; TM p. 52), `z80_jump.rs`.
- Derived CALL carry-condition label resolution: cc=011 tests carry despite the primary's Z label; §1 (UM pp. 284, 286; TM p. 53), `z80_call_return.rs::call_c_uses_carry_independently_of_zero_despite_primary_table_label`.
- Derived ALU equations: bit-4 XOR carry/borrow and sign-change overflow equations implement the printed flag rules; `alu.rs` (UM pp. 67–68, 145–146, 152, 154, 156; TM §6 p. 39, p. 47), exhaustive independent `z80_arithmetic_reference/`.
- Derived block-compare H equation: low-nibble comparison represents borrow from bit 4; `exchange_block_search.rs` (UM pp. 138, 140–143; TM p. 46), `z80_exchange_block_search.rs`.
- Derived DAA range reading: retain primary 9–0 as inclusive endpoints and final 6–7 without widening to ALPM's 6–F; §1 (UM p. 173; ALPM p. 132), `z80_daa.rs` pins all 684 table inputs out of 2048.
- Maintainer-accepted library DAA extension (2026-11-13; not documented by the sources): outside those 684 inputs choose 06h for low digit >9 or H, 60h for A>99h or C, add/subtract by N, and set C by the 60h choice; §1 (UM p. 173; TM p. 48), `general_arithmetic_cpu_control.rs`, `z80_daa.rs::library_extension_outside_manual_table`.
- Derived DAA H-after / library rule, maintainer-accepted (2026-11-13): use adjustment bit-4 carry/borrow for table cases too; the primary table supplies H-before but no H-after equation; §1 (UM pp. 173–174; TM pp. 39, 48), `general_arithmetic_cpu_control.rs`, `z80_daa.rs`.
- Unverified EX (SP) access order / library choice: read SP, read SP+1, write SP+1, write SP; cited entries establish byte mapping, not temporal order; `exchange_block_search.rs`, `z80_exchange_block_search.rs` (UM pp. 127–129; TM p. 46).
- Derived LDDR intermediate P/V, maintainer-accepted (2026-11-13): set while decremented BC is nonzero, as for LDIR/LDD; interpret LDDR/TM reset entries as completed-instruction results; §1 ruling (UM pp. 130, 133–134, 137; TM p. 46), `z80_exchange_block_search.rs::lddr_sets_pv_until_final_iteration_derived`.
- Derived CPIR/CPDR reconciliation: termination is exhaustion OR match, and CPDR repeats for nonzero BC AND no match, resolving heading conjunction/equality misprints; §1 / decision-15 rule 11 (UM pp. 139, 142; TM p. 46; MK pp. 2-63, 2-59), `z80_exchange_block_search.rs`, `z80_timing.rs`.
- Derived width-edge results: 16-bit PC/SP/pair/address wraps, signed indexed/relative address wraps and eight-bit B underflow follow widths and the entries' operations rather than explicit wrap measurements; UM pp. 2–3 and per-entry citations in group modules, wrap witnesses in `z80_load8.rs`, `z80_load16.rs`, `z80_arithmetic8.rs`, `z80_arithmetic16.rs`, `z80_jump.rs`, `z80_call_return.rs`, `z80_input_output.rs`, `z80_exchange_block_search.rs` and interrupt tests.
- Library bus/API representation: provided `input16`/`output16` forward low bytes by default; Z80 uses full-width addresses, Intel models keep eight-bit calls; decision-15 rule 4 (UM pp. 295–296, 298, 300, 302, 304, 306–307, 309, 311, 313, 315; TM p. 54; ALPM pp. 267, 269), `input_output.rs`, `root_contract.rs`, `z80_input_output.rs`.
- Library access-kind mapping: memory M1s (prefixes included) use OpcodeFetch, immediate/displacement reads OperandFetch, other memory Data, SP accesses Stack and INT reads InterruptAcknowledge; decision-15 rule 5 (UM pp. 3, 6, 247; TM pp. 4, 7), `cpu.rs`, group/interrupt trace witnesses.
- Unverified indexed-CB fourth-byte cycle: DD/FD and CB are OpcodeFetch, d and terminal op OperandFetch, so R increments twice; byte order is stated but non-M1 classification is not established by the timing tuple; decision-15 rules 5–6 (UM p. 247; TM §3 p. 7), `cpu.rs`, `z80_run.rs`, indexed bit/rotate witnesses.
- Derived R accounting and load ordering: once per M1 including prefixes and successful NMI/INT fetches; LD A,R sees its own increments, LD R,A replaces them with all A bits; decision-15 rule 6 (UM pp. 3, 13–14, 95, 97, 132; TM pp. 4, 7, 16, 18; ALPM pp. 49, 51), `cpu.rs`, `z80_load8.rs`, interrupt tests.
- Library poll polarity/order and boundary accounting: asserted-true NMI/INT (inactive by default); budget, stop/NMI/INT polls, stop, NMI, eligible INT, halt, execution; complete costs/overshoot, saturating counters and one step per acceptance; decisions 2/5 and decision-15 rules 1/3 (UM pp. 6, 12–14, 183; TM pp. 8, 16, 18, 55), `interrupt.rs`, `cpu.rs`, `z80_run.rs`, `z80_determinism.rs`.
- Library NMI sampler/restoration: consecutive committed levels detect the edge, initial/reset history is inactive, sub-poll pulses can be missed; Budget does not sample, Stop/Unsupported restore history, Halt retains it; decision-15 rule 2 (UM p. 6; TM p. 8), `cpu.rs`, `z80_interrupts_nmi.rs`.
- Unverified EI-delay edge cases: EI EI re-arms, DI cancels, NMI preserves the delay until an ordinary instruction/iteration completes; Stop/Budget/Unsupported do not consume it; decision-15 rule 7 (UM pp. 6, 18, 182–183; TM pp. 8, 55–56), `cpu.rs`, `z80_machine_control.rs`, interrupt tests.
- Unverified parity-note window: successful NMI or INT immediately after LD A,I/R clears P/V; an ordinary next instruction expires the note, an ineligible request does not apply it, Budget preserves it and Stop/Unsupported restore it; decision-15 rule 10 (UM pp. 18, 94–95; ALPM pp. 48–49), `cpu.rs`, `z80_load8.rs`, interrupt tests.
- Derived post-HALT resume PC / stated library departure: retain PC after entry and return prompt Halt with no idle steps, cycles, fetches or R advance, unlike hardware's NOP-fetch/refresh loop; decision-15 rule 7 (UM pp. 14, 181, 290; TM p. 18), `z80_machine_control.rs`, interrupt halt witnesses.
- Library RETI notification scope: observe ED 4D fetches without a daisy-chain callback; unchanged IFFs are the stated documented reading, not a RETN alias; decision-15 rule 7 / §1 (UM p. 288; ALPM p. 246; MK Table 2-1 p. 2-15, p. 2-200), `call_return.rs`, `z80_call_return.rs`.
- Derived NMI mapping/total: one discarded OpcodeFetch at unchanged boundary PC and 11 states from restart equivalence and RST's total, not an explicit NMI total on the response pages; decision-15 rule 8 (UM pp. 13–14, 19, 290, 293; TM pp. 18, 53, 56; ALPM p. 248), `nmi.rs`, `z80_interrupts_nmi.rs`, `z80_timing.rs`.
- Derived common INT acknowledge mapping: one InterruptAcknowledge read at unchanged boundary PC in all modes; decision-15 rule 9 (UM pp. 12–13, Fig. 9; TM p. 16), `acknowledge.rs`, `z80_interrupts_int.rs`.
- Derived IM 0 RST / IM 1 totals: 13 = RST's 11 + two acknowledge clocks; decision-15 rule 9 (UM pp. 19, 293; TM pp. 53, 56), `acknowledge.rs`, `z80_interrupts_int.rs`, `z80_timing.rs`; IM 2's 19 is stated, not derived.
- Library IM 0 restriction: accept RST only; other documented injected instructions, including CALL, return Unsupported with full restoration; decision-15 rule 9 (UM p. 19; TM p. 56), `acknowledge.rs`, `z80_interrupts_int.rs`.
- Library odd IM 2 refusal / unverified hardware outcome: reject bit 0 set before writes/table reads, rather than extrapolate hardware; decision-15 rule 9 (UM p. 20, Fig. 17; TM p. 57), `acknowledge.rs`, `z80_interrupts_int.rs`.
- Library block retirement: one iteration per step; map stated 16 terminating / 21 repeating to Untaken/Taken, retaining refetch and boundary polls on repeat; decisions 13 rule 5 / 15 rule 11 (UM pp. 132, 136, 139, 142, 300, 304, 311–312, 315–316; TM pp. 46, 54; ALPM p. 269), block and determinism witnesses.
- Library reset preservation/bookkeeping: preserve SP, both general/AF sets, IX/IY and flags; clear EI/parity bookkeeping and initialise NMI history inactive, with no bus/polls or elapsed-cycle report; decision-15 rule 12 (UM pp. 6, 18, 181; TM pp. 9, 56; MK p. 2-15), `cpu.rs`, `z80_model.rs`.
- Library construction/packing: zero registers/flags, disabled IFFs, IM 0 and clear delay/halt/parity/NMI history are Default values, not power-up claims; pair and AF/AF' packing are model-owned API representations; decisions 13 rule 1 / 15 rule 12 (UM pp. 2–3; TM pp. 3–5), `state.rs`, `flags.rs`, `z80_model.rs`, `z80_registers.rs`.
- Library Unsupported transaction/payload / derived capacity: restore all CPU boundary state including R/samplers/delay/parity, contribute zero progress, preserve irreversible effects of consumed reads; decision-20 §2 defines the fixed payload as the final decode context ([last DD/FD, CB, d] for indexed-CB, [CB] for SLL), omitting earlier selectors while retaining instruction-start PC; single-prefix payloads are unchanged and all consumed reads remain observable; decision-14 rules 2–4 / decision-15 rule 13 (UM p. 247; TM p. 51), `cpu.rs`, `z80_coverage.rs`, `root_contract.rs`; interrupt rollback deliberately departs from acceptance IFF clearing (UM p. 18).
- Library timing scope: no BUSREQ/BUSACK or external WAIT gating/time, pin timing, refresh-address bus calls or sub-instruction polls; automatic acknowledge waits stay in totals, R is the refresh exposure subject to HALT's departure; decision-15 rules 6/14 (UM pp. 6–8, 12; TM pp. 4, 8–9, 16), `cpu.rs`, boundary/interrupt witnesses.
- Unverified oracle CRC provenance / library judging scope: ZEXDOC source lines 34–39 attribute expected CRCs to a real Z80 without part/revision or per-group capture; §2 pins the GPL-2.0-or-later artifacts, source/binary audit, 58 judged groups and nine not run under decision-14. `x80-oracle` keeps digest checks, filtering and the decision-20 §7 output/RET host rendezvous outside core semantics; the recorded PASS licenses no hardware fact.

## 4. License surface and boundary

The library is `MIT OR Apache-2.0`. `x80-core` has zero dependencies of any kind;
`x80-core/tests/boundary.rs` checks Cargo's resolved graph for all targets/features.
`deny.toml` retains x86jit's permissive allowlist without its oracle exceptions.
Machine services and dependency-bearing oracle harnesses belong outside the core.
No repository or package is published yet; publishing is the maintainer's step.
