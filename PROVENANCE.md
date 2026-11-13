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
| Z80 ED 63 / ED 6B omitted from compact loads | UM008011-0816 Table 7 p. 45 shows only 22/2A for HL; instruction entries pp. 108/103 print ED 01dd0011 / ED 01dd1011 and explicitly include HL = 10. 03-0029-01 Table 7.0-2 p. 45 includes both patterns with dd = BC, DE, HL, SP | Derived, decision-14 rule 1 (x80-bi2.3): the explicit entries and second witness prevail over the compact omission. Both ED HL forms are documented; eight ED pair loads and 58 ED terminal encodings, not 56 |
| Z80 compact H register headings | UM008011-0816 Tables 6/11 pp. 42, 50 and Fig. 39 p. 54 label the H column F; entries pp. 71, 145, 213 and 03-0029-01 Tables 7.0-1/4/7 pp. 44, 47, 50 explicitly name H = 100 | Derived resolution, decision-14 rule 1: use H as the register operand; the compact heading errors introduce no F operand |
| Z80 compact PUSH/POP encodings | UM008011-0816 Table 7 p. 45 prints PUSH codes F6/C6/D6/E6 and DD/FD E6, and POP AF as P1. Entries pp. 115, 117–119 and 03-0029-01 Table 7.0-2 p. 45 give PUSH C5/D5/E5/F5, DD/FD E5, POP AF F1 | Derived resolution, decision-14 rule 1: use the instruction-entry patterns, independently agreeing with the second table; compact cells are misprints |
| Z80 compact ADD A,(HL) and DEC BC encodings | UM008011-0816 Table 11 p. 50 prints 88 for ADD A,(HL), Table 13 p. 52 prints DB for DEC BC. Entries pp. 148, 201 and 03-0029-01 Tables 7.0-4/6 pp. 47, 49 give 86 and 0B | Derived resolution, decision-14 rule 1: use 86 and 0B from the entries and second witness |
| Z80 compact rotate encodings | UM008011-0816 Fig. 39 p. 54 prints CB 06 for RLC L, CB 0E for RLC (HL), and CB 06 for RRC E. RLC entries pp. 213, 215 give CB 05 and CB 06; RRC p. 225 prints CB 00001rrr and explicitly lists E = 011, hence CB 0B. 03-0029-01 Table 7.0-7 p. 50 agrees. Indexed RLC is correctly printed as DD/FD CB d 06 in Fig. 39 and entries pp. 217, 219 | Derived resolution, decision-14 rule 1: CB 05 for RLC L, CB 06 for RLC (HL), and CB 0B for RRC E; follow the instruction-entry patterns and register fields, independently agreeing with the second table |
| Z80 compact BIT/SET/RES table corruption | UM008011-0816 Table 14 pp. 55–57 prints C8 in place of CB throughout, repeated/wrong terminal bytes (e.g. indexed BIT 3/7 as 46), and DD in some IY cells. Entries pp. 243, 245, 247, 249, 251, 253, 255, 257, 259–260 and 03-0029-01 Table 7.0-8 p. 51 give CB bit-field patterns and DD/FD CB d forms | Derived resolution, decision-14 rule 1: expand the entry patterns with their register/bit restrictions; 192 CB bit forms and 24 per indexed table. The corrupt compact cells admit no extra encodings |
| Z80 compact JP and IN A,(C) encodings | UM008011-0816 Table 15 p. 59 prints D8 for JP C,nn and EB for JP (HL); Table 18 p. 62 prints ED 7B for IN A,(C). Entries pp. 263, 275, 296 and 03-0029-01 Tables 7.0-9/11 pp. 52, 54 give DA, E9 and ED 78 | Derived resolution, decision-14 rule 1: follow the entry patterns and second witness; compact cells are misprints |
| Z80 ED 70 admission | 03-0029-01 Table 7.0-11 p. 54 explicitly describes IN r,(C), r = 110 as affecting only flags. UM008011-0816 IN r,(C) p. 296 (PDF 310) instead labels r = 110 "Undefined op code; set the flag" | Stated disagreement; decision-14 rules 1–2 follow the primary's explicit undefined classification and the maintainer's documented-only scope: ED 70 is Unsupported. TM's expanded ED count is 59 including that form; the admitted count is 58. ED 71 also stays out: UM p. 307 lists no r = 110 OUT operand |
| Z80 H definedness in CCF and pair arithmetic | 03-0029-01 Tables 7.0-5/6 pp. 48–49 print H unknown for CCF, ADD HL/IX/IY, ADC HL and SBC HL. UM008011-0816 entries pp. 178, 188, 190, 192, 194, 196 give explicit H rules | Stated disagreement; decision-14 rule 6 follows the primary's definedness: these forms do not invoke unknown-flag preservation and H remains in the hardware comparison mask. Arithmetic equations require their own group witnesses; this row does not settle them |
| Z80 C definedness in block I/O | 03-0029-01 Table 7.0-11 p. 54 prints C unknown for INI/INIR/IND/INDR/OUTI/OTIR/OUTD/OTDR; UM008011-0816 entries pp. 298, 301, 302, 305, 309, 312, 313, 316 explicitly say C is not affected | Stated disagreement; decision-14 rule 6 follows the primary: preserve C as a hardware claim and compare it under 43h; S/H/P/V remain unknown, rather than also masking C under 42h |
| Z80 BIT flag-list H/N label slips | UM008011-0816 BIT (HL)/(IY+d) pp. 246, 250 each print H set and then H reset. Adjacent BIT entries pp. 244, 248 and 03-0029-01 Table 7.0-8 p. 51 agree on H set, N reset | Derived resolution, decision-14 rule 6: the second H is a label slip for N; it is not another unknown flag. S and P/V remain the unknown flags for all four BIT forms |
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
| Z80 RETI IFF reading and device notification | UM008011-0816 RETI p. 288 (PDF 302); 03-0002-01 RETI p. 246/PDF 255; MK78515 Table 2-1 p. 2-15/PDF 29 and RETI p. 2-200/PDF 214 | decision-15 rule 7: stated documented reading leaves IFF1/IFF2 unchanged; MK expressly says so. RETN's IFF2-to-IFF1 copy remains separate (UM p. 290/PDF 304; ALPM p. 248/PDF 257). No contrary IFF effect found in these pages. Stated API choice exposes RETI through its two opcode fetches without adding a device callback |
| Z80 NMI discarded read, push mapping and 11-state total | UM008011-0816 Nonmaskable Interrupt Response p. 13, Fig. 10 p. 14, Table 1/CPU Response p. 19, RETN p. 290, RST p. 293 (PDF 27–28, 33, 304, 307); 03-0029-01 §4 p. 18, §8 p. 56, Table 7.0-10 p. 53 (PDF 24, 62, 59); 03-0002-01 RETN p. 248/PDF 257 | decision-15 rule 8: derived one discarded OpcodeFetch read at unchanged PC from the stated read cycle; stated return PC, high-first push and 0066h target mapped to Stack writes. IFF1 clears, IFF2 stays. Total 11 is derived from restart equivalence plus RST's stated 11, not an explicitly printed NMI total on these response pages |
| Z80 common INT acknowledge address and kinds | UM008011-0816 Interrupt Request/Acknowledge pp. 12–13, Fig. 9 (PDF 26–27); 03-0029-01 §4 p. 16 (PDF 22) | decision-15 rule 9: derived mapping performs one InterruptAcknowledge read at unchanged boundary PC in all three modes. The figure places PC on the bus and the generic cycle text does not qualify its mode. IM 1 ignores the returned byte; Stack/Data mappings follow the documented save/table sequence |
| Z80 IM 0 accepted-byte restriction | UM008011-0816 CPU Response p. 19 (PDF 33); 03-0029-01 §8 p. 56 (PDF 62): hardware allows other instructions including a three-byte CALL | decision-15 rule 9: stated library scope restriction accepts RST only. Other bytes give Unsupported with acknowledge=true, no preceding bytes and full boundary restoration; only the acknowledge read survives externally. This restricts documented hardware capability; it is not a claim that other bytes are undocumented |
| Z80 odd IM 2 vector refusal | UM008011-0816 CPU Response/Mode 2 p. 20, Fig. 17 (PDF 34); 03-0029-01 §8 p. 57 (PDF 63): bit 0 must be zero; table low byte first; acknowledge/save/table costs 7/6/6 | decision-15 rule 9: stated library restriction refuses an odd acknowledge before writes/vector reads, restores boundary state and reports acknowledge=true. Hardware's odd-vector outcome is unverified, not extrapolated. For even vectors the PC-save then two Data-read order and total 19 are stated |
| Z80 IM 0 RST and IM 1 totals | UM008011-0816 CPU Response p. 19 (PDF 33): normal restart plus two clocks; RST p. 293 (PDF 307): 11; 03-0029-01 §8 p. 56 and Table 7.0-10 p. 53 (PDF 62, 59) | decision-15 rule 9: derived 13 states for each (11+2), one step per successful acceptance. Both sources state the operands of the derivation, rather than an explicit 13-state acceptance entry |
| Z80 LD A,I / LD A,R interrupt parity note | UM008011-0816 entries pp. 94–95 and Table 1 p. 18 (PDF 108–109, 32); 03-0002-01 LD A,I p. 48/PDF 52 has the interrupt note, LD A,R p. 49/PDF 53 omits it | decision-15 rule 10: IFF2-to-P/V is stated. Primary gives the interrupt-clears-parity note for both; second-witness omission gives no contrary result. Unverified interpretation uses one latch: successful NMI or INT acceptance immediately after the load clears P/V; an ordinary next instruction expires it, ineligible requests do not apply it. Budget preserves it; Stop/Unsupported restore it |
| Z80 block iteration retirement and two States slots | UM008011-0816 LDIR p. 132, LDDR p. 136, CPIR p. 139, CPDR p. 142, INIR p. 300, INDR p. 304, OTIR pp. 311–312, OTDR pp. 315–316 (PDF printed+14); 03-0029-01 Tables 7.0-3 p. 46 and 7.0-11 p. 54 (PDF 52, 60); 03-0002-01 OTIR p. 269/PDF 279 | decision-15 rule 11: PC rewind by two and per-iteration interrupt recognition/refresh are stated. Stated API choice retires one iteration per step and refetches at the next boundary. Stated 16 terminating and 21 repeating map to untaken/taken respectively; the shared machinery acquires no hardware fact |
| Z80 CPIR/CPDR terminating timing condition | UM008011-0816 CPIR p. 139/PDF 153 and CPDR p. 142/PDF 156: prose terminates on exhausted count OR match; 16-state headings say count zero AND match. 03-0029-01 Table 7.0-3 p. 46/PDF 52 says OR for the 16-state path | Proposed resolution, decision-15 rule 11: use the primary's descriptions and the second witness's OR; the primary timing headings are treated as a conjunction misprint. Repeating path remains count nonzero AND no match, 21 states. Require witnesses for both ways to terminate separately; no hardware/oracle measurement claimed |
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

### Z80 exercisers: fetch-only pins and documented-encoding audit

Maintainer decision, 2026-11-13 (x80-bi2.5): M4 covers documented Z80 behavior
only. Bits 5 and 3, IXH/IXL/IYH/IYL, SLL, WZ/MEMPTR and all prefixed holes are
out of scope; every undocumented encoding returns Unsupported. The two programs
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

#### Recorded Z80 candidates

| Candidate | Disposition | Licence / host evidence |
| --- | --- | --- |
| ZEXALL | Rejected for M4: compares flag bits 5 and 3, outside the documented behavior set; no fetch pin | Same Cringle exerciser family; specific candidate bytes and licence header **to confirm** if reconsidered. CP/M-style host expected from family lineage; exact build's host needs **to confirm** |
| Patrik Rak's z80test | Evaluated candidate only; not selected, fetched or run | No local pinned source or licence inspected: exact licence and redistribution terms **to confirm**. Prior local knowledge suggests ZX Spectrum-oriented builds, requiring a target-specific loading/ROM/I/O environment rather than assuming the CALL 5 ABI; exact build, load address, entry/exit, stack, ROM calls, ports, interrupt/timing needs and any alternative host **to confirm** |

Unresolved provenance: the exact Z80 part/revision and per-group capture behind
Cringle's expected CRCs are unstated. Candidate licence/host details above remain
unverified; no network lookup was made. The Zilog manual pin is maintained
separately; this block records the revision and page images used for classification.

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
