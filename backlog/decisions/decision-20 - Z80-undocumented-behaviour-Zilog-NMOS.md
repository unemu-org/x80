---
id: decision-20
title: Z80 undocumented behaviour (Zilog NMOS)
date: '2026-11-13'
status: accepted
---

Decider: maintainer (2026-11-13). Bead `x80-mn3.5`, documentation only; this
record changes no executable behaviour by itself. The maintainer's binding
directions of 2026-11-13 are: undocumented Z80 behaviour is always on, without
an embedder profile; target Zilog NMOS Z80; the 8080 retains Unsupported.
The maintainer accepted the choices below on 2026-11-13. They supersede decisions 13–15 only in
the respects listed in §7; they do not silently amend those accepted records.

## Context

decision-14 admits 698 terminal encodings, refuses the remaining prefixed forms,
zeros F bits 5/3 and preserves primary-manual unknown flags. That deliberately
documented-only contract cannot describe the selected NMOS target. decision-17
requires a pinned written statement and independent identified-silicon evidence
for each undocumented hardware fact. decision-18's manufacturer amendment
permits labelled second-source licensing, subject to all five conditions;
none of the new sources here is a qualifying manufacturer data sheet. Mostek's
shared Zilog documentation lineage does not become an independent die key.

**Admission to the library is distinct from verification as hardware.** The
decision supplies deterministic behaviour for every normally fetched encoding.
Where two keys do not establish a subfact, an explicitly labelled library rule
fills the gap. Such a rule is not an undocumented hardware fact licensed by a
CRC. Refusing all such encodings would defeat the requested always-on scope;
claiming every rule is silicon-confirmed would defeat decision-17.

## Evidence vocabulary and pins

All pins are in [PROVENANCE §1](../../PROVENANCE.md#1-authority-and-citations),
with the oracle acquisition limits in §2. SHA-256 values were checked locally
on 2026-11-13. Young's pp. 8, 11–24, 26–35 were read as page images;
pp. 19–24 were also read in full extracted text. Disputed Young pp. 15/17/30
were reread at 200 dpi; TM p. 54 and UM p. 296 at 200 dpi. Other disputed UM
entries and all relevant measurement/test passages were reread in full text.
The z80test ZIP was read in memory, including its test assembly; no emulator
implementation, linked implementation, network resource or simulator was used.

| Key | Pin / locator convention | Class and origin |
| --- | --- | --- |
| U | Zilog UM008011-0816, revision 11, August 2016; `e3c83da5a5d8e372364c20fa53665e6fbb165ec6ac38c8c1eebc359603447b5e`; printed page +14 = PDF page | A, manufacturer documentation |
| T | Zilog 03-0029-01, ©1977; `9c7304b4842dac6d34c9cd1552f006c2f1b6fa9373f7e6d1b614956d91efc485`; printed +6 = PDF | A, same Zilog documentation lineage |
| Y | Young, *The Undocumented Z80 Documented*, v0.91, 18 September 2005; `6413048f39c2e735373b1fb23102599133bc64ccd0ed63b16fbc1173643d7a9d`; Arabic page = PDF page | C, compilation; references are not new independent acquisitions |
| M | Boo-boo/Kladov, *MEMPTR*, ©2006, supplied gist copy; `78d19f0809c31dfb516150ad5e7ba747063f40bbb528439f1316539c7f0c612a`; instruction headings | C, written experiment account; credits real-chip testers but does not identify their Zilog specimens |
| B | Banks, `Undocumented-Flags.md`, wiki git `49e542dd40c36ef7367cb5806292764416dc987a`, read 2026-11-13, update literally 9/7/2026; `a5b100b3046e88485a939b5f1a0a76536c4ec59c050ca800db4b76fa1652e9ef`; headings | B for explicitly reported measurements; same origin for its own restatements |
| S | Banks, `Unstable-CCF-SCF-Behaviour.md`, same wiki revision/read date; `485c12182ee05bffc734fd30545e763f81b6f5723eaf94aaa7da9e3c10395f01`; Introduction, Conclusions, Test Program, result pages | B, Acorn Z80 Second Processor acquisitions at 2.0/3.5/6.0 MHz; named parts below |
| N | Banks, `NMI-during-EI-Anomaly.md`, notes dated 20 October 2018, same wiki pin/read date; `0effe42fea3f73f690985d648b673d50c98552adf6ea1f807d1fa15c256b50ba` | Simulator-discrepancy discussion, excluded as silicon confirmation |
| R | Rak, z80test v1.2a, 2 December 2023; ZIP `7df0443d703e6b3114ea04b4cdef3e13b91421c62e37185c1036d06864cacbaf`; `src/tests.asm` dot-labels, unless another member is named | B, expected values acquired by Patrik Rak from a real 48K Spectrum with Zilog Z80 CPU, per `readme.txt` |

**A** below means **documented by manufacturer**, with the cited historical
Z80/Z80A scope. **H** means **hardware-measured (Zilog Z80 in a 48K Spectrum,
by Patrik Rak)**, using the cited independent written key plus R's acquisition.
R does not give a date code, package or explicit process revision. Selecting
those observations for the NMOS library target is a **library scope choice**,
not proof of all Zilog NMOS revisions. An H row confirms only what its vectors
discriminate, not bus order, elapsed states, internal latch bits or injected
interrupts absent from the test. **L** means **adopted library rule; hardware
unverified under two keys**, with the missing key/reason given locally.
**B8039** means **hardware-measured (Zilog NMOS Z80, date code 8039, by David
Banks)**; it is not a label for the 8811 or unidentified R specimen.

R's source attribution supplies the second key; no oracle program was executed
for this decision. Source inspection is READ evidence; §5's in-memory vector
and CRC calculations are MEASURED projection evidence, not a new hardware
capture or a whole-suite PASS. Y/M/B/S have overlapping citations; count copied
discoveries once.
Banks' Acorn measurements and Rak's reported Spectrum acquisition are distinct
origins where they actually test the same fact. Cringle's unidentified ZEXALL/
ZEXDOC hardware is never an identified-silicon key. Each implementation still
needs independent literal witness values and per-fact source comments.

## Choice, in implementation order

### 1. ED input/output, holes and duplicates

Hexadecimal byte lists below are terminal bytes after ED, not single-byte
instructions. Documented ED 63/6B remain four-byte pair loads, not holes.

| Fact / selected behaviour | Written key; independent silicon key; label and limits |
| --- | --- |
| ED 70, `IN (C)` / `IN F,(C)`: input once from full BC, discard the byte, leave general registers unchanged; S/Z/PV from input, H=N=0, C preserved; Y/X from input (§4). Two bytes, 12 states | **A** for flag-only operation/length/total: T Table 7.0-11 p. 54 explicitly says r=110 affects flags only and gives 2/3/12. U pp. 296–297 supplies the usual input flag rules but calls r=110 undefined. Y §3.4 pp. 12–13 and §8.11 p. 35 agree on flag-only input. R `.in_c` lines 1000–1006 independently captures flags/registers at IN(FEh)=BFh: **H** for that observed behaviour, not every input or timing |
| ED 71, `OUT (C),0`: output once at full BC, retain all flags/registers, two bytes, 12 states | Y §3.4 p. 13 / §8.11 p. 35 states zero output and 2/12. R `.out_c_0` lines 1070–1075 captures CPU state but **does not capture the emitted byte**. Preserve-state aspect **H**; output 00h, length and timing **L** following Y, because no independent output-bus or timing capture is pinned. The commonly proposed CMOS FFh difference is **unverified in this corpus**; do not assert it as an established variant or use it in the NMOS model |
| NEG: 44/4C/54/5C/64/6C/74/7C, all the documented NEG operation | Y §3.4 pp. 12–13; R `.neg_` lines 389–394 varies terminal mask 38h, hence all eight: **H** for encoding/results/flags. Two bytes / 8 states: **L** alias-cost extension of U NEG pp. 176–177 and Y §8.5 p. 30; R does not time it |
| RETN family: 45/55/5D/65/6D/75/7D, plus canonical RETI 4D; all pop PC, SP+=2 | Y §3.4 pp. 12–13; R `.reti_retn` lines 1202–1207 varies mask 38h: **H** for return/data/flags. All 2 bytes / 14 states: **L** alias-cost extension of U pp. 288/290 and Y §8.10 p. 34 |
| IFF1←IFF2 for all those returns, including ED 4D | Y §3.4 p. 13 and §5.3 pp. 20–21 states this, reporting an NMI experiment with an unidentified chip. R runs with DI and snapshots no IFFs, so it cannot confirm this effect. **L**, following Y's explicit assignment; hardware unverified. This deliberately replaces decision-15 rule 7's canonical RETI preservation reading. Only ED 4D is the canonical peripheral RETI signature; retain fetch visibility and no daisy-chain callback. No claim about external decoder aliases is made |
| IM 0: 46/4E/66/6E; IM 1: 56/76; IM 2: 5E/7E | Y §3.4 pp. 12–13. Its Gerton Lunter bus experiment is a retold, unidentified Spectrum acquisition, not a second named-chip key. R `.im_n` lines 1459–1465 emits all eight, but does not accept interrupts or snapshot IM: **L** for alternate-mode assignments and 2 bytes / 8 states (canonical costs U pp. 184–186). A passing CRC would not confirm mode selection |
| Every other ED terminal byte is a two-byte, 8-state NOP: flags/registers/WZ unchanged; R increments twice; one retirement | Y §3.4 pp. 12–13 says unlisted entries, excluding block instructions, behave as two NOPs. No exhaustive identified-silicon ED-hole capture exists in R: **L**, including the one-retirement interpretation and §2's access mapping. The second byte is consumed even if it is CB/DD/ED/FD; it does not start a new prefix |

**Derived map:** ED 40–7F contains the printed operations except 77/7F; add
the 16 block bytes A0–A3, A8–AB, B0–B3, B8–BB. There are 78 operation slots
and **178 NOP slots** (00–3F; 77/7F; 80–9F; A4–A7; AC–AF; B4–B7; BC–FF).
All 256 ED terminal bytes execute. This count is a map derivation from Y,
not 256 independent silicon measurements. NOP means no port/data/stack call;
the two OpcodeFetch reads still happen. ED-hole WZ/Q preservation/reset uses
§§5–6's library rules. Unsupported does not remain for these slots.

### 2. Index halves and ignored/repeated DD/FD

Y §§3.2–3.3 pp. 11–12 is the written rule: DD selects IX, FD selects IY.
For **register-only** main-table H/L uses, substitute the selected index high/
low byte. This is not a blanket textual HL replacement. Preserve documented
pair/index forms, indexed-memory displacement forms and their costs.

| Main-table class after one DD/FD | Selected rule / exact coverage | Two keys / label |
| --- | --- | --- |
| INC/DEC H/L: 24/25/2C/2D | Operate on selected high/low index byte; 2 bytes, 8 states | Y §3.2, §8.4 p. 30; R `.inc_x`/`.dec_x` lines 708–721 selects DD/FD and high/low: operation/flags **H**, new timing **L** |
| LD H/L,n: 26/2E | Selected byte receives n; 3 bytes, 11 states | Y §8.1 p. 26; R `.ld_x_n` lines 1306–1311: operation **H**, timing **L** |
| LD r,r': 40–7F with neither operand code 6 | Replace every H/L operand, including H←L, L←H and self-loads. Other B/C/D/E/A operands retain their meaning. 2 bytes, 8 states | Y §3.2 and §8.1 p. 26; R `.ld_x_x` lines 1278–1283 covers both selectors/all main LD bytes: index substitution and ignored-prefix register moves **H**; timing **L** |
| ALU A,H/L: 84/85,8C/8D,94/95,9C/9D,A4/A5,AC/AD,B4/B5,BC/BD | ADD/ADC/SUB/SBC/AND/XOR/OR/CP uses selected high/low byte. 2 bytes, 8 states | Y §3.2 / §8.4 p. 30; R `.alo_a_x`/`.alo_a_y` lines 487–499: operation/flags **H**, timing **L** |
| A main (HL) operand | Existing (IX/IY+d) form. LD H/L,(index+d) and LD (index+d),H/L use **ordinary H/L**, not index halves | Y §3.2 pp. 11–12; U LD entries pp. 78–85; R `.ld_r_xyd`/`.ld_xyd_r` lines 1285–1297: **A/H** for the exceptions. DD/FD 76 remains HALT, without a displacement |
| EX DE,HL (EB), EXX (D9), all ED operations | DD/FD does not substitute IX/IY into these operations | Y §3.2 p. 11; R tests unprefixed EX and ED forms, not all these prefixed cases: general ignored-selector rule **L** |
| Other main opcodes having no index substitution | Execute the main instruction, with its original immediates/addresses; add one prefix byte and 4 states per ignored DD/FD | Y §§3.2/3.7 pp. 11–14, §6.1 p. 23; only register-LD subset above has an independent R key. General ignoring, extra 4-state cost and arbitrary repetition **L**; not established by a result CRC |

No mixing IXH with IYH in one instruction. For a finite DD/FD chain, the
**last** selector wins (Y §3.7 p. 14); ED cancels the selection, CB uses the
indexed-CB table when a selector is active, and the selection ends after the
terminal instruction. Thus `DD FD 21 lo hi` loads IY; `FD DD ED 44` performs
NEG with two ignored index prefixes; `DD EB` exchanges DE/HL. These arbitrary
chains have only Y's written key: **L**, not H inferred from a single prefix.

**Access/retirement mapping, L except for the documented M1 foundation:**
each DD/FD is OpcodeFetch and adds one low-seven-bit R increment (bit 7
retained); ordinary terminal main/CB/ED bytes follow decision-15 rules 5–6.
Indexed CB has prefix and CB as OpcodeFetch, d and terminal op as OperandFetch,
so two R increments plus one per extra leading index prefix. Y §6.1 p. 23
states these R counts, but does not independently confirm the physical cycle
kind. U p. 6/T §3 p. 7 establish ordinary opcode-fetch classification.

Do not poll stop/NMI/INT, retire a step, expire the parity note or consume EI
delay between a prefix and its opcode, or between repeated prefixes. Complete
the whole instruction once, then expose the next boundary. Y §5.5 pp. 21–22
reports INT tests but explicitly says NMI was not tested; neither R nor a
named independent capture supplies the missing key. The full boundary rule is
therefore **L**, preserving decision-15's instruction-boundary API. The Q-only
prefix effect in §6 does not create a boundary. Repeating block operations
still rewind to the ED byte: extra leading DD/FD bytes are not refetched on
later iterations; this extension is **L**.

**Required layout amendment, library choice:** no fixed prefix-count limit
and no prefix-buffer allocation. Retain the start PC and current selector;
accumulate extra length/cost outside the terminal table's u8 fields, with
wrapping PC and saturating u64 total accounting. Add 4 per redundant prefix;
existing table totals already include one effective DD/FD. Whole-instruction
totals can exceed 255, so decision-13 rule 5's u8 conclusion must be narrowed
to terminal-table costs. A fact-free progress cost-width adjustment may be
needed; no CPU rule goes into shared decode/progress. An endless prefix stream
has no retirement/boundary and need not return from step; this contract does
not promise mid-instruction cancellation. Literal witnesses must cover >255
states and PC/R wrapping, without treating a prefix as a completed step.

After §§1–3 there is **no normally fetched Z80 encoding refusal**. IM 0
non-RST acknowledgements and odd IM 2 vectors remain Unsupported under
decision-15 rule 9; they are interrupt API restrictions, not opcode holes.
The fixed preceding-byte payload and rollback remain for those refusals;
no unbounded prefix chain needs to fit that payload. The 8080 is unchanged.

### 3. SLL and indexed-CB register copies

| Fact / selected rule | Written key; independent silicon key; label |
| --- | --- |
| CB 30–37 SLL/SLIA: result=((old<<1)\|1)&FFh, C=old bit7; S/Z/PV from result, H=N=0; Y/X from result | Y §3.1 p. 11 / §8.7 p. 32; R `.slia_a`/`.slia_r`, `testtable` lines 81–100: **H** for operation/flags, including (HL). Length 2, 8 states for register / 15 for (HL): **L** extension of the neighbouring shift costs |
| DD/FD CB d op: rotate/shift (SLL included), RES or SET writes result to addressed memory and, if op low bits !=110, also B/C/D/E/H/L/A selected by those bits | Y §§3.5–3.6 pp. 13–14 / §§8.7–8.8 pp. 32–33; R `.sro_xyd_r` lines 671–676, `.set_n_xyd_`, `.res_n_xyd_` lines 903–908: **H** for memory/register results. H/L destinations are ordinary H/L, not index halves |
| BIT indexed-CB: all eight low-bit encodings test the same memory bit; no memory write or register copy | Y §3.5 p. 14; R `.bit_n_xyd_` lines 833–838: **H** for flags and unchanged registers/memory, not a pin-level no-write measurement. No Bus write is the **L API mapping** |
| Indexed-CB length/cost/trace | 4 bytes, 23 states for modifying forms, 20 for BIT; same total irrespective of copy destination. Y §§8.7–8.8 pp. 32–33 and U indexed entries pp. 217–241/247–250 support the documented forms. New copy/SLL totals **L**, no independent timing key. Use §2's two-M1 mapping; one Data read, then one Data write for modifying forms, plus direct register assignment of that computed result: **L temporal/API rule** |

Copies use the computed result even if a device ignores the memory write;
do not reread memory into the destination register (Y §3.5 pp. 13–14).
R uses writable RAM, not a ROM/write-ignoring device: that particular
observation has no independent second key, so this is **L** following Y.
All 256 CB and all 256 terminal slots of each indexed-CB table execute.
Do not make rotate flag semantics apply to SET/RES: they preserve F.

### 4. Full F/F' images and per-class YF/XF

Store all eight bits independently in Z80-owned flags, including YF=bit5,
XF=bit3. Arbitrary POP AF/AF register input preserves them; PUSH AF and
EX AF,AF' carry them unchanged. Y §2.3 p. 8 and §§8.2–8.3 pp. 28–29 is the
written description; R `.selftest` lines 297–302, `.poppush_af` lines
1218–1223 and `.ex_af_af` supply the observed flag-image/exchange key: **H**
for the tested transfers. Public representation and independent stored bits
are a **library choice**. Default F/F'=0; reset preserves both, including Y/X,
by decision-15's library reset policy, not a hardware reset claim.

Let `yx(v)=v & 28h`. The following table exhausts instruction classes; a
register/memory/immediate or index-half operand follows its own class. Effects
on documented flags remain as accepted unless explicitly changed below.

| Class | Y/X chosen | Written key + independent R locator / status |
| --- | --- | --- |
| ADD/ADC/SUB/SBC/AND/XOR/OR 8-bit; INC/DEC byte; NEG | yx(8-bit result) | Y §2.3 p. 8 / §8.4–8.5 p. 30; R immediate `.add_a_n`…`.or_n`, `.alo_a_*`, `.inc_*`/`.dec_*`, `.neg`/`.neg_`: **H** |
| CP (all ordinary forms) | yx(source operand), not subtraction result or A | Y §8.4 p. 30 footnote 1; R `.cp_n`, `.alo_a_*`: **H**. CPI/CPD are the separate block rule |
| DAA | yx(adjusted A) | Y §4.7 p. 18 / §8.5 p. 30; R `.daa`: **H** for Y/X relation. This does not silently replace the accepted out-of-table DAA correction/H rules; this decision rules only on the Y/X relation |
| CPL; RLCA/RRCA/RLA/RRA | yx(new A) | Y §§8.5/8.7 pp. 30/32; R `.cpl`, `.rlca`/`.rrca`/`.rla`/`.rra`: **H** |
| CB and indexed-CB rotates/shifts, including SLL | yx(shifted byte) | Y §8.7 p. 32; R `.rlc_*`…`.srl_*`, `.sro_xyd*`: **H** |
| RLD/RRD | yx(new A), not the memory result | Y §8.7 p. 32; R `.rld`/`.rrd`: **H** |
| ADD HL/IX/IY,rr; ADC/SBC HL,rr | yx(high byte of 16-bit result) | Y §4.6 p. 17 / §8.6 p. 31; R `.add_hl_rr`, `.add_ix_rr`, `.add_iy_rr`, `.adc_hl_rr`, `.sbc_hl_rr` lines 768–801: **H** |
| LD A,I / LD A,R | yx(loaded A) | Y §8.1 p. 27; R `.ld_a_i`/`.ld_a_r` lines 1436–1448: **H**, not confirmation of the interrupt P/V window |
| IN r,(C), including ED 70 | yx(input byte) | Y §8.11 p. 35; R `.in_r_c`/`.in_c` at input BFh: **H** for that discriminating input; no arbitrary port-data sweep. Full-input generalisation follows Y |
| BIT n,r | yx(original register byte), regardless of bit n | **L**: Y §4.1 p. 15 instead says yx(r & (1<<n)); its BIT 7,A example resets both. R `.bit_n_a`/`.bit_n_r` lines 805–824 distinguishes these candidates, but no independent pinned written statement of the selected operand rule was found. Do not call a CRC the missing written key. Example literal witness: BIT 0,B with B=28h gives Y=X=1 under the library rule |
| BIT n,(IX/IY+d), all aliases | yx(high byte of effective address); WZ=effective address | Y §4.1 p. 15 / M indexed-address entry, R `.bit_n_xyd*`: agrees at the tested addresses (**H** for observed address-vs-data selection); exact 16-bit WZ assignment **L** (§5). R varies d=0/1 near the fixed data page; it does not establish all address bits or carries |
| BIT n,(HL) | yx(high byte of retained WZ); do not set WZ=HL or copy memory's Y/X | Y §4.1 p. 15 / M opening statement; R `.bit_n_hl` and memptr variant `idea.asm` lines 124–127 observe only WZ bits13/11, with setup dependencies. **H** for those discriminated flag observations; full latch/update policy **L** (§5) |
| LDI/LDD, terminating LDIR/LDDR | t=(A+transferred byte)&FFh; Y=t.bit1, X=t.bit3 | Y §4.2 p. 16; R `.ldi`/`.ldd`/`.ldir`/`.lddr` lines 912–938: **H**; repeat-step override §6 |
| CPI/CPD, terminating CPIR/CPDR | t=(A-byte-Hborrow)&FFh; Y=t.bit1, X=t.bit3; use the half borrow of A-byte | Y §4.2 p. 16; R `.cpi`…`.cpdr` lines 954–980: **H**; repeat-step override §6 |
| INI/IND/OUTI/OUTD, terminating repeat I/O | yx(B after decrement); other flags by rule below | Y §§4.3/8.11 pp. 16–17/35; R `.ini`/`.ind`/`.outi`/`.outd` and terminating `.inir`/`.indr`/`.otir`/`.otdr`: **H** for discriminated CPU flags, subject to IN data restriction; repeat-step override §6 |
| SCF/CCF | Q-dependent §6, not unconditional A | B “SCF/CCF”, S Conclusions; R post-CCF variant; specimen-qualified there |
| POP AF; EX AF,AF' | Load/swap complete F, no recomputation from A | Flag-image keys above; **H** for tested transfers; Q classification is separate |
| All remaining classes | Preserve Y/X: other loads, LD I/R,A, pair INC/DEC, other exchanges, PUSH/POP except AF, jumps/calls/returns/RST, RES/SET, IN A,(n), all non-block OUT, NOP/ED holes, HALT, DI/EI/IM and interrupt entry | Y §§8.1–8.11 tables pp. 26–35 show preservation for listed instructions; R corresponding groups discriminate many ordinary cases. **H** only for listed tested groups; untested HALT/RST/interrupt entry, new holes/prefix cases and universal preservation are **L**, without inferring an update from internal ALU activity |

For BIT in every form also choose S=(n==7 and tested bit set), Z=PV=not
tested bit, H=1, N=0, C preserved. Y §4.1 p. 15 plus R `.bit_n_*` is the
**H** rule for S/PV, replacing decision-14's unknown S/PV preservation;
it does not cure Y's separate incorrect Y/X statement for registers.

**Block I/O, including formerly unknown documented bits:** let b be decremented
B, data the actual byte transferred, direction δ=+1 for increment or −1 for
decrement, and use 8-bit wrapping for `(C+δ)`. For input set
`k=data+((C+δ)&FFh)`; for output set `k=data+L_after`. Set S/Z/Y/X from b,
N=data.bit7, H=C=(k>FFh), PV=even_parity((k&7) XOR b). This is Y §4.3
pp. 16–17's written key, independently discriminated by R's block-I/O flags
groups: **H** over their observable vectors, with no arbitrary input sweep.
It explicitly replaces U's N=1/C-unaffected entries (pp. 298, 301–302, 305,
309, 312–313, 316) as well as unknown preservation. Silicon acquisition wins
where it discriminates; unexercised cases retain the written equation as a
labelled library extension. The repeat extra-five-state rule is separate (§6).

Do **not** copy Y §4.4 p. 17's reversed port-B ordering: retain decision-15
rule 4, input uses B before decrement, output uses B after. U pp. 298/309 and
T p. 54 supply the documented rule; R's fixed Spectrum input and uncaptured
output bus cannot independently settle all high-address ordering.

### 5. Model MEMPTR/WZ explicitly

Add a Z80-owned 16-bit WZ latch. F bits13/11 exposure through BIT (HL) requires
state across instructions; recomputing it from current HL is insufficient.
This replaces decision-15's omission of a hidden register. **Library choice:**
carry WZ in supplied/snapshotted model state, deterministic continuation,
Default, reset and rollback; no new bus access or shared-family register.
Default WZ=0000h; reset **preserves** WZ, because U RESET p. 6/T p. 9 do not
specify it and neither M nor R measures reset. This is an explicit library
value/preservation rule, not a power-up or electrical-reset assertion.

The following is the complete **L exact-latch rule table**, following M's
instruction headings except the explicit replacements and completions below.
M is the written key; its credited experiment team is not a second identified-Zilog origin. R's
`z80memptr` may confirm only resulting flag observations, never a whole word.
All arithmetic wraps in 16 bits unless low-byte-only assignment is specified.
Unlisted instructions preserve WZ, including BIT (HL), LDI/LDD, POP, EX DE,HL,
EXX, LD SP,HL/IX/IY, pair INC/DEC, NOP and ignored prefixes.

| Instruction / path | Chosen WZ after operation | Independent confirmation and remaining gap |
| --- | --- | --- |
| LD A,(nn); LD A,(BC/DE) | address+1 | R `.ld_a_mem`/`.ld_a_rr` tests addresses on the fixed data page; high-bit flags cannot distinguish address from address+1 here. Exact +1/carry unverified |
| LD (nn),A; LD (BC/DE),A | high=A_before; low=low(address+1), no carry into high | M distinguishes BM1 clones; R `.ld_mem_a`/`.ld_rr_a` varies A by only bit0 and addresses within one page, so it does not validate arbitrary high-byte routing or carry suppression |
| LD rr,(nn); LD (nn),rr, including HL/IX/IY and ED 63/6B | nn+1 | R `.ld_*_mem`/`.ld_mem_*` at fixed addresses, not all carries or word bits |
| EX (SP),HL/IX/IY | new pair value read from stack | R `.ex_sp_hl`/`.ex_sp_xy` observes only bits13/11 of limited word patterns; not the full assignment |
| ADD HL/IX/IY,rr; ADC/SBC HL,rr | destination_before+1 | M explicit equation; Y §4.1 p. 15's old H is an incomplete approximation that misses low-byte carry. R `.add_*_rr`/`.adc_hl_rr`/`.sbc_hl_rr` varies high bits and low carry edges; can discriminate those exposed carries, not all 16 bits |
| RLD/RRD | HL+1 | M; R `.rld`/`.rrd` fixed page, not +1 versus unchanged address |
| Taken JR/DJNZ, RET/RET cc, RETN/RETI, RST | destination PC | M; R return/jump groups have fixed data/jump page targets; no RST group or full-word observation. Untaken relative branch/return preserves WZ by unlisted-path library rule |
| JP nn/JP cc,nn and CALL nn/CALL cc,nn, even untaken | immediate destination nn | M; R `.jp_*`/`.call_*` uses fixed targets. JP (HL/IX/IY) preserves WZ, per M exclusion |
| IN A,(n) | ((A_before<<8)\|n)+1 | M; R `.in_a_n` at n=FEh does not test low-byte overflow |
| IN r,(C), including ED 70; OUT (C),r, including ED 71 | BC_before+1 | M explicitly names only IN A,(C)/OUT (C),A. Extension to all register codes and ED70/71 is **L completion**, not a statement M makes. R `.in_r_c`/`.out_c_r`/special groups gives only flag projection, no emitted byte or full word |
| OUT (n),A | high=A; low=(n+1)&FFh, no carry into high | M; R `.out_n_a` fixed n=FEh, not FFh carry suppression |
| LDIR/LDDR, continuing iteration | address_of_ED+1 | M; R `.ldir_nop`/`.lddr_nop` exposes repeat state. Fixed PC does not establish arbitrary address or low word |
| LDIR/LDDR, terminating iteration | preserve WZ | M says unchanged when incoming BC=1; R ordinary repeat groups expose final projections only |
| CPI/CPD; terminating CPIR/CPDR | old WZ+1 / old WZ−1 | M probing account; R `.cpi`/`.cpd`/`.cpir`/`.cpdr` is not a 14-bit recovery experiment |
| Continuing CPIR/CPDR | address_of_ED+1, overriding the CPI/CPD increment/decrement | M; reproduced R `.cpir`/`.cpdr` projections below are consistent, but do not establish a general interrupted compare state or full word |
| INI/IND; terminating INIR/INDR (B_after=0) | BC_before_decrement +1 for incrementing / −1 for decrementing | M; R `.ini`/`.ind` and reproduced `.inir`/`.indr` final projections are consistent, not full-word confirmation |
| Continuing INIR/INDR (B_after!=0) | address_of_ED+1, overriding the INI/IND BC-derived assignment | **L library replacement:** the rule is WZ = (address of this iteration's ED byte + 1) modulo 65536. M instead says exactly as INI/IND on each execution; that continuing rule is **falsified** by R `.inir_nop`/`.indr_nop` (0A537B63 versus pinned F3B1BE2F). The selected PC-derived projection reproduces F3B1BE2F for both. This extends M's explicit continuing LDxR/CPxR form as a library choice; no pinned independent written INxR equation or whole-word confirmation is claimed. A CRC cannot license the whole WZ word |
| OUTI/OUTD, OTIR/OTDR each iteration | BC_after_decrement ±1 | M; reproduced R `.otir`/`.otdr` final projections are consistent, but termination overwrites WZ and those groups cannot discriminate a continuing PC-derived override |
| Every (IX/IY+d) operation, BIT included | effective address | M; R indexed groups are clustered around the fixed data page. No claim to confirm unseen bits/carry edges |
| Successful NMI, IM0 RST, IM1 or IM2 entry | actual handler destination PC | M says interrupt call behaves as CALL; extension to each accepted mode is **L**. R has no injected interrupt vectors; refused acknowledges leave WZ untouched by rollback |

**What z80memptr can discriminate:** `idea.asm` lines 124–127 executes
`LD HL,data; BIT 0,(HL)` after each tested sequence. HL loading preserves WZ
under the selected rule; flags-only CRC then observes WZ bits13/11 and C,
not bits15/14/12/10–0. The preceding setup also contributes retained WZ;
this is not independent arbitrary-WZ initialisation. Most memory/branch
vectors keep addresses on one page, so many +1/−1/old/new candidates share the
same projected bits. Arithmetic vectors with low FFh and high bits varied
can expose a carry into an observed bit. Require a per-vector differential
audit before upgrading any L equation to hardware-confirmed; a whole-suite
PASS alone cannot do it. R itself warns in `readme.txt`/`z80memptr.asm` that
the vectors were not designed to stress MEMPTR. The full table remains L.

**Continuing block projection audit — MEASURED, 2026-11-13:** independently
reproduced in memory from the pinned ZIP, without executing a CPU or oracle
host. `testmacros.asm` lines 45–73 gives the 20-byte vector layout;
`idea.asm` lines 20–44 initialises the masked counter to its mask, and lines
190–233 enumerates it down to zero for each shifter (initially zero, then
each masked bit in byte/low-bit order). The calculation XORs those with the
base vector, accounts for self-overwriting ED terminals, and projects the
post-sequence `BIT 0,(HL)` flags per lines 124–127: H=1, S=N=0,
Z=PV=NOT bit0 of the original `data.F`, C retained from the sequence, Y/X from WZ
bits13/11. The immediately preceding `LD SP,(data.sp)` seeds WZ=880Fh under
the table's pair-load rule. The published `z80memptr.tap` setup bytes locate
the ED instruction at 832Bh, so the selected continuing WZ is 832Ch and its
Y/X projection is 00h. `crctab.asm`'s four byte planes were reconstructed and
checked against polynomial EDB88320h; CRC starts FFFFFFFFh with no final
complement, matching `idea.asm` lines 55–58/150–183 and `main.asm` lines 130–143.
Vector/expected-CRC bytes also matched the published TAP; subset enumeration
matched a literal counter/shifter transcription, and a second CRC implementation
agreed.

| R group (`src/tests.asm`) | Vectors | Reproduced selected-rule CRC = pinned CRC | Differential / limit |
| --- | --- | --- | --- |
| `.inir_nop` (1040–1046), `.indr_nop` (1048–1054) | 208 each | F3B1BE2F each | M's continuing BC_before_decrement ±1 instead gives 0A537B63 each: falsified, not merely a possible discrepancy. BFh input overwrites the terminal byte with BFh, an ED NOP, exposing the first iteration's WZ |
| `.ldir_nop` (940–945), `.lddr_nop` (947–952) | 960 each | 88679670 each | Continuing address_of_ED+1 is consistent; preserving incoming WZ instead gives EEA47B6F |
| `.ldir` (926–931), `.lddr` (933–938) | 1408 each | EC8A174B each | Continuing address_of_ED+1, then preserve on termination; preserving WZ throughout instead gives 825BAE42 |
| `.cpir` (968–973), `.cpdr` (975–980) | 2560 each | C83E7419 / B9D65AD9 | Continuing address_of_ED+1, then old WZ±1 on termination; omitting the continuing PC assignment instead gives B1546AA8 for both |
| `.otir` (1091–1096), `.otdr` (1098–1103) | 240 each | 9FF6297F / D7CF070F | Selected per-iteration BC_after_decrement ±1 is consistent. A continuing PC override gives the same CRCs because the last iteration overwrites WZ: these groups do not discriminate the continuing rule |
| `.inir` (1024–1030), `.indr` (1032–1038) | 11 each | 371C7DBA each | Terminating BC_before_decrement ±1 is consistent; the final assignment hides earlier WZ values |

Thus the selected LDIR/LDDR/CPIR/CPDR/OTIR/OTDR rules are consistent with
their pinned groups, with the output-repeat blind spot stated above. These
calculations settle the INxR fallback's falsification and replacement projection;
they do not license an exact 16-bit latch equation, arbitrary-PC routing,
unexercised interrupt paths or a universal NMOS hardware claim. M's INxR prose
is retained as the losing reading; `changes.txt` alone states no replacement
equation. The independent written INxR key and full-word evidence remain missing.

### 6. Q, repeat-boundary flags and interrupt quirks

#### SCF/CCF: select a deterministic, specimen-qualified Q rule

Y §8.5 p. 30 says unconditional A-derived Y/X. B “SCF/CCF” reports predecessor
dependence on Zilog NMOS, credits Rak's 2012 discovery, and excludes POP AF
from flag-modifying predecessors. S independently reports Acorn screen tests:
8039 uses pattern A (Q); 8811 uses pattern B (unconditional A); the unstable
case's conclusion says 8634 while its result page and image paths say 8644.
Result-page hashes at the common wiki pin are:

| Result page / specimen | SHA-256 | Reading limit |
| --- | --- | --- |
| Zilog NMOS 8039 | `44f80895f0dec8bf54a2a6c66f49aece86cc3f7c378c58128f1cd0256e8fdd4e` | S's written conclusion reports stable pattern A |
| Zilog NMOS 8811 | `7e0f7a32e96fde80cac6559c5dcfe847a5e6b2329ba409ddac5134812f9287e9` | S reports stable pattern B, including its Spectrum +2B origin |
| Zilog NMOS 8644 | `5fc5f12e166b622909329c3b2feef7016ba034a347defbf153879ec4ab6c926c` | Result page says 8644; unstable conclusion says 8634; do not silently merge identifications |
| File named Zilog NMOS 8817 | `f09dd650ed77dde7f98745760060e88d9d7d991e18ecc226e30ff2ec0dc8b4ae` | Own heading/image roots say CMOS; exclude it from named-NMOS confirmation |

The linked chip/screen images are absent. These are pinned written measurement
reports, not independently reread screen captures. B/S are one Banks origin;
R's README retells Banks/Rak and is not another written discovery. R's distinct
Spectrum acquisition and `z80ccf`/`testmacros.asm` post-CCF construction supply
independent observed predecessor discrimination, including `.scf`, `.ccf`,
`.scfccf`, `.ccfscf` and POP AF-related groups. S's pattern A statement plus R
is the two-key support for the chosen stable rule, **B8039/H**, with different
specimen scopes retained. It cannot prove a universal Zilog NMOS Q equation.

**Library selection:** choose pattern A, without randomness or a profile:
if the preceding instruction was classified as flag-modifying, Y/X=yx(A);
otherwise Y/X=yx(A) OR old Y/X. Keep other SCF/CCF effects (including H=old C
for CCF). The target is Zilog NMOS with this explicit deterministic selection;
do not claim equivalence with 8811 or board/frequency-sensitive cases. Later
silicon-qualified variants need decision-6 and maintainer scope approval.

Add a model-owned boolean Q recording that predecessor classification, not
whether F's numerical value changed. Classify byte ALU/INC/DEC/DAA/CPL/NEG,
rotates/shifts/RLD/RRD, BIT, pair ADD/ADC/SBC, LD A,I/R, IN r,(C), block
operations, SCF/CCF as modifying. Classify other loads, POP AF, exchanges
(including EX AF,AF'), PUSH, RES/SET, pair INC/DEC, flow control, OUT, IN A,(n),
NOP/ED holes, HALT, DI/EI/IM as not modifying. B's explicit POP exception
and R post-CCF groups support tested distinctions; universal classification,
interrupt-entry Q=0, Default/reset Q=0 and snapshot rules are **L bookkeeping**.

B's update 9/7/2026 says DD/FD before SCF/CCF loses predecessor information:
choose Q=0 on each DD/FD fetch, then let the terminal instruction set Q.
This has one pinned written report and no independent matching R vector:
**L**, following that update. It does not make prefixes retire or poll.
Stop/Unsupported restore Q; Budget preserves it; prompt Halt does not invent
idle predecessor instructions. WZ and Q belong to complete boundary snapshots.

#### Flags on every continuing block iteration

B “Interrupted block instructions” attributes extra flag changes to the five
repeat states, not to interrupt acceptance. Therefore expose them at every
continuing iteration's boundary, including Budget/Stop and a self-overwrite
that prevents the next repeat. Let p be the ED address and b the decremented B.
First compute §4's ordinary block flags, then:

- LDxR/CPxR: replace Y=p.bit13, X=p.bit11; retain other ordinary flags.
- INxR/OTxR: also replace Y/X from p; if ordinary C=1 and data.bit7=1,
  PV ^= even_parity((b−1)&7) XOR 1 and H=(b&0Fh)==0; if C=1 and data.bit7=0,
  PV ^= even_parity((b+1)&7) XOR 1 and H=(b&0Fh)==0Fh; if C=0,
  PV ^= even_parity(b&7) XOR 1, with H retained. S/Z/N/C retain base values.

The written key is B's exact equations. R `.ldir_nop`/`.lddr_nop` and
`.inir_nop`/`.indr_nop` independently expose continuing-step flags without
injecting interrupts: **H for those discriminated LDxR/INxR observations**.
Fixed p does not prove arbitrary PC routing. CPxR/OTxR's arbitrary interrupted
paths, full PC dependence and the complete branch domain of the I/O formula
lack independent matching acquisition here: **L**, following B. Banks' later
SCF/CCF date codes cannot be assigned retroactively to these block experiments.
Never apply these overrides to a terminating 16-state iteration. Source
omission of the extra-five-state effects in Y §§4.2–4.5 pp. 16–17 is an
incomplete earlier account; do not call Y's final-iteration rules a conflicting
measurement of intermediate state.

#### LD A,I/R P/V and NMI during EI

U pp. 94–95 explicitly states P/V=IFF2 and zero parity if an interrupt occurs
during execution. Y §5.1 p. 19 / §8.1 p. 27 gives the ordinary IFF2 rule,
not an independently measured NMOS interrupt window. R `.ld_a_i`/`.ld_a_r`
runs without injected interrupts. **A** for the manufacturer's note;
**unverified** for NMOS-specific scope, CMOS contrast, precise window and
NMI applicability. Retain decision-15 rule 10's pending-note boundary mapping
for both successful NMI/INT, its expiry, preservation and rollback, explicitly
**L**, without relabelling it confirmed by z80test.

N's opening describes IFF2=0 as the simulator's erroneous result and says the
desired result is IFF1=0/IFF2=1. Its ensuing simulator/transistor explanation
is excluded from evidence; linked simulators/netlists were not consulted.
There is no second independent named-chip acquisition for an EI/NMI quirk in
this corpus. **Unverified; no new anomaly is admitted.** Keep the library
rule: EI sets both IFFs, successful NMI clears IFF1 and preserves IFF2, leaves
the pending EI delay for the next ordinary retirement (decision-15 rules 7–8).
Do not implement an IFF2-clearing anomaly from that page title.

### 7. Supersession, scope and oracle consequences

These replacements apply from acceptance (2026-11-13); they are not edits to the
old records, which carry pointers here:

| Record / rule | Replacement or retained limit |
| --- | --- |
| decision-14 rule 1 documented-only admission / 698 count | §§1–3 admit the full normal-fetch map, distinguishing H/A facts from L fallbacks; the historical count remains the documented subset |
| decision-14 rule 2 refusal list | Remove ED-hole, index-half/ignored/repeated-prefix, SLL and indexed-CB refusals. No finite terminating normal instruction is refused |
| decision-14 rules 3–4 | Retain root payload and transactional rollback for remaining interrupt refusals; obsolete fetched-refusal examples and three-byte maximum rationale no longer constrain prefix parsing |
| decision-14 rule 5 six flags / zero image | §4's full F/F' storage, load/packing/preservation rules |
| decision-14 rule 6 unknown preservation / masks | Replace BIT S/PV and block-I/O unknown/contradicted effects by §§4/6. Keep unrelated documented rulings and unverified DAA/API rules. Hardware comparisons for admitted flags use all eight bits; masked campaigns remain distinct, labelled diagnostic checks |
| decision-13 rules 3–4 | Retain five private arrays and group ownership; allow model-owned chained/ignored prefixes and complete terminal maps |
| decision-13 rule 5 | Two path slots suffice for terminal entries; u8 no longer bounds whole chained-instruction length/cost (§2) |
| decision-13 rule 6 | Replace one-NMOS/CMOS assumption with **Zilog NMOS target**, Q selection explicitly qualified in §6. CMOS, other makers and NMOS subvariants require later decision-6 variants; no undocumented profile |
| decision-15 state, rules 5–6/12–13 | Add WZ/Q and complete snapshot/default/reset/rollback responsibilities; apply §2's prefix/R/access rules and widened total accounting. Physical indexed-CB cycle kind still unverified |
| decision-15 rule 7 RETI / rule 11 blocks | §1's labelled IFF2-copy rule for RETI and §6's continuing-step flags supplement the existing return/block API. Canonical peripheral signature remains ED 4D |
| decision-15 rules 9–10/14 | IM0 RST-only / odd IM2 refusals, LD A,I/R boundary-note mapping and excluded pins/timing remain. Y's odd-IM2 claim (§5.2 p. 20) has no independent key here; it does not remove the library refusal |
| decision-3 / decision-9 / Intel models | Unchanged: 8080 Unsupported and V20 ED escape remain; no cross-model flag/latch executor sharing |

CMOS differences are a **future variant investigation**, not scope silently
shared by this model: alleged ED71 FFh output, LD A,I/R P/V-window differences,
and maker/revision/board-sensitive Q behaviour. S reports some Zilog CMOS
parts with pattern A; this is not evidence for universal CMOS equivalence.
The second-source manufacturer licence cannot be invoked for Y/M/B/R.

**ZEXDOC:** the nine previously not-run groups become runnable after §§2–3:
`alu8rx`, `incxh`, `incxl`, `incyh`, `incyl`, `ld8ixy`, `ld8rrx`, `rotxy`,
`rotz80`. Preserve PROVENANCE §2's source/binary audit and the `incyh`/`incyl`
descriptor-name error (actual DD, not inferred FD). Existing 58 judged results
are historical; neither that record nor this decision is a new 67-group PASS.
The ED70 Unsupported host rendezvous must be replaced before ED70 admission:
use an explicitly harness-owned output/RET boundary-stop convention, with
measured extra steps/cycles, rather than stealing another now-executable opcode.

**ZEXALL:** its 67 descriptor masks are FFh (`zexall.z80` descriptor block
lines 195–731; flags masked at `flgmsk` line 1102 using the selected mask).
It becomes a falsifier for Y/X and newly admitted encodings, without a new
source of hardware rules. Audit its own COM/source correspondence/digests,
state coverage and host rendezvous before execution; ZEXDOC's audit does not
prove ZEXALL's binary. Keep GPL-2.0-or-later artifacts fetch-only.

**z80test harness bead:** implement a Spectrum-style test host in the oracle
crate, outside zero-dependency/guest-agnostic core. Run pinned `z80full`,
`z80doc`, `z80flags`, `z80docflags`, `z80ccf`, `z80memptr` separately. Retain
the 8000h load/entry, 8800h data/workspace, writable/self-modifying RAM and
caller stack; provide CHAN-OPEN 1601h, RST 10h output, IY=5C3Ah/system variables,
EI/RET return-to-caller behaviour and final-count capture (`loader.bas`,
`Makefile`, `main.asm`, `print.asm`, `idea.asm`). Model the suite's Spectrum
I/O precheck, OUT FEh=07h / expected IN FEh=BFh; do not silently skip input
groups. `.incheck` failures and skipped/diagnostic groups must be reported.
Keep original linked addresses and TAP/build correspondence. `z80ccfscr` is
optional visual evidence work: it writes screen/attributes and loops forever,
so needs separate host/termination treatment and supplies no new capture here.

`z80doc*` masks describe flag comparison, not a restriction to documented
encodings. `z80ccf` observes predecessor classification through a post-CCF;
`z80memptr` observes a two-bit projection through post-BIT, with the limits
in §5. Separate literal bus/timing/interrupt/reset witnesses remain mandatory.
Run no emulator implementation as a reference. Oracle failures falsify a
rule/host/correspondence hypothesis; investigate them against the written keys
before changing the model. Report all masks, variants, omissions and specimen
limits; do not promote unidentified CRCs or one projected PASS to authority.

## Implementation beads

These briefs were the supervisor handoff. No code change, commit or oracle
execution belongs to this record. After acceptance the supervisor created
beads under epic `x80-mn3`, in section order: §1 `x80-mn3.9`, §2 `x80-mn3.14`,
§3 `x80-mn3.10`, §4 `x80-mn3.15`, §5 `x80-mn3.11`, §6 `x80-mn3.16`; row 7
splits into the z80test host `x80-mn3.12`, ZEXDOC/ZEXALL `x80-mn3.13` and
closure `x80-mn3.17`. Serialise heavy campaigns.

| Order / section | Bead brief | Required evidence / completion boundary |
| --- | --- | --- |
| 1 / §1 | ED70/71, ED aliases and NOP holes; replace oracle rendezvous | Exhaustive terminal map; independent literal port/register/flag/PC/R/cost/return witnesses; retain H/A/L distinctions, explicitly test ED71 output as a library expectation and RETI with unequal IFFs |
| 2 / §2 | Index halves and ignored/repeated prefixes | Complete main-table substitution/exclusion map; last selector, ED cancellation, HALT and indexed-memory H/L exceptions; no mid-prefix polls/retirement; >255-state and wrapping traces; widened accounting without shared hardware rules |
| 3 / §3 | SLL and all indexed-CB copies/BIT aliases | Exhaustive terminal coverage, independent shifts/flags, ordinary H/L destinations, memory plus register, write-ignoring device witness; correct length/cost/access mapping |
| 4 / §4 | Full flags and all instruction-class rules | Unmasked F/F' input/output/exchange witnesses, CP operand versus result, BIT rules, terminating blocks and full block-I/O formulas; retain explicit unverified BIT-register written-key gap |
| 5 / §5 | WZ latch, update rules and projection audit | Table-complete literal state witnesses, reset/default/rollback/continuation, carry/no-carry edges; audit which R vectors distinguish competing WZ rules; do not claim exact-word confirmation |
| 6 / §6 | Q and continuing-block flags; interrupt-quirk witnesses | Pattern A selection and named-specimen limits; prefix-Q library rule; terminating/repeating and self-overwrite/interrupt/step-budget cases; no new EI/NMI anomaly; retain parity-window uncertainty |
| 7 / §7 | Normal-fetch closure and exerciser hosts/campaigns | All nine additional ZEXDOC groups; independent ZEXALL artifact audit; Spectrum-style host and six R variants; report judged/skipped/masked groups, elapsed commands/results and remaining evidence gaps |

Stages that temporarily depend on later flag/WZ/Q work must not report final
oracle conformance early. Hardware witness literals and oracle expectations
must remain independent of implementation helpers. Any newly discovered
source disagreement gets a PROVENANCE §1 ruling before changing behaviour.

## Alternatives

- Preserve documented-only refusals or add an undocumented profile: conflicts
  with the maintainer's always-on direction.
- Call every Y table entry independently verified by ZEXALL: violates the
  named-silicon, independence and per-fact requirements of decision-17.
- Claim one uniform NMOS/CMOS or Zilog NMOS Q implementation: contradicted
  by the pinned specimen reports; the deterministic selection must be explicit.
- Infer all WZ bits, ED71 bus data, timing or interrupt behaviour from CRCs:
  those campaigns do not observe all those quantities.

## Consequences and revisit triggers

The Z80 becomes complete at its normal-fetch boundary with full flags and
model-owned WZ/Q, while retaining explicit hardware evidence gaps and the
existing interrupt/pin API limits. This is an accepted software contract,
not a claim that the corpus establishes every transistor-level effect.

Revisit when independent written/silicon evidence settles BIT-register Y/X,
ED71 NMOS/CMOS output, prefix timing/boundaries, RETI IFFs, full WZ/reset or
repeat-I/O WZ, the LD A,I/R window, or Q specimen/board dependence; when the
six-variant harness discriminates a conflicting reading; or when a maintainer
requests a named variant under decision-6. Preserve both readings and silicon
precedence. The maintainer accepted this record on 2026-11-13; implementation is later work.
