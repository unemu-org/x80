# Provenance

## 1. Authority and citations

Derive hardware behavior from primary sources, cited by document, edition/revision,
section or table, and printed page (also PDF page if different). Pin revision and
SHA-256 before implementing a fact; check scans against page images. Comments
also link the relevant design section. Each implemented fact needs an independent
witness test with expected values derived from that citation.

### Pinned sources

Both are fetch-only and cite-only (© Intel, no redistribution grant). Their rows in
`oracles/MANIFEST.md` carry the URL, page count and page-to-PDF map; fetch them with
`./oracles/fetch-oracles.sh fetch` and check them with `verify`. Both are scans with
a noisy OCR text layer: check every cited value against the page image.

| Path in `oracles/` | Title | Order number, date | SHA-256 | Role |
| --- | --- | --- | --- | --- |
| `intel/i8080-um.pdf` | Intel 8080 Microcomputer Systems User's Manual | 98-153B, September 1975 | `d5982f96536b1dbecbeff409d2370970638716296dce7b318624d86221597a9e` | Primary: CPU chapter (interrupt, halt, start-up, per-instruction machine-cycle/state table), instruction set and flags, 8080A data sheet |
| `intel/i8080-alpm.pdf` | Intel 8080/8085 Assembly Language Programming Manual | 9800301-04 (title page; mirror file name says 9800301D), May 1981 (printing code 581 on p. ii) | `c2f17d613159b624cf7226b0b9200192a49ff172147d88f56d50f45e94691643` | Second witness: per-opcode cycles and states, flags, interrupts |

Both are Intel documents: the second is independent in authorship, not a hardware
witness. Where they disagree, an oracle may falsify one reading; record the case
below and in a bead.

| Disagreement | Citations | Status |
| --- | --- | --- |
| Length of RET (C9) and JZ (CA) | 98-153B pp. 4-11, 4-12 and 9800301-04 pp. 3-32, 3-48: RET one byte, JZ three bytes; 9800301-04 p. A-5 prints "RET Adr" at C9 and "JZ" without "Adr" at CA | Resolved for the opcode map: p. A-5 misprint (operand marker one row off); the per-instruction pages of both manuals agree. Witness pins both bytes |
| CY after DAA when CY was set before DAA, or when the low adjustment carries out of bit 7 | 98-153B p. 4-8: two-step rule, "All flags are affected"; p. 4-2 general rule: CY set on a carry out of the high-order bit, "otherwise it is reset"; 9800301-04 p. 6-13: DAA's carry is needed "for adding the higher-order bytes" (MADD routine, pp. 6-12 to 6-14; subtraction routine pp. 6-14 to 6-17) | Not stated explicitly. Derived (maintainer, 2026-11-12): CY is kept from before DAA and set by a carry out of bit 7 from either step, which then triggers the high adjustment; the documented decimal routines are correct only this way. Falsifiable by the oracle run, bead x80-ecy.17 |
| EI delay edge cases: EI followed by EI, DI after EI, and RESET during the delay | 98-153B p. 4-14: EI enables "following the execution of the next instruction", DI disables "immediately following the execution of the DI instruction"; 9800301-04 pp. 3-22, 3-23 say the same; p. 5-14: RESET clears INTE. None addresses a second EI, DI cancelling a pending delay, or RESET clearing the delay | Unverified model (x80-ecy.13): a second EI re-arms the delay, DI clears it, RESET clears it. Labelled in code and tests |
| Machine cycles of HLT | 98-153B p. 2-18 state table: M1 and M2; 9800301-04 p. 3-24: one machine cycle; both give 7 states | No effect on x80, which counts states only; HLT takes 7 states, witnessed |
| Auxiliary carry after 8080 logical AND (ANA, ANI) | 98-153B p. 4-9: ANI clears CY and AC; 9800301-04 p. 1-12: AND sets AC to the logical OR of bit 3 of the operands | Unresolved, bead x80-ecy.2. Interim (decision-7, unverified): ANA and ANI set AC to bit 3 of (A OR operand) |

### Discovery inventory

The list below is **not a set of pinned sources**. Unknown metadata is marked
`source to find` rather than guessed. URLs remain unset until confirmed.

| Candidate title | Order number | Year | Public source URL | Intended role |
| --- | --- | --- | --- | --- |
| MCS-80 User's Manual; MCS-80/85 Family User's Manual | 98-153D; 205775-00x | 1977; 1983 | bitsavers mirror (not pinned) | Not pinned: later editions that repeat the 98-153B CPU chapters; a possible check for corrections |
| Intel 8085AH datasheet | source to find | source to find | source to find | 8085-specific signals and timing |
| Zilog Z80 CPU User Manual | source to find | source to find | source to find | Z80 state, instruction set, interrupts and timing |
| NEC V20/V30 user's manual (exact title to verify) | source to find | source to find | source to find | Emulation-mode entry/exit, mapping, CALLN and native interrupt handoff |

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
| 8080EXM (.COM, .MAC source) | Cringle's instruction exerciser (1994), modified for the 8080 by Bartholomew (February 2009); Mike Douglas (May 2013) added the success CRCs shown on Bartholomew's website | GPL-2.0-or-later | 25 CRC groups comparing all eight PSW bits; falsifies flag readings, including decision-7 (AND auxiliary carry) and the derived DAA carry rule |

Both are CP/M programs: they load at 0100h, print through BDOS 2 and 9 via `CALL 0005h`
and exit with `JMP 0000h`. 8080EXM sets SP from the word at 0006h; 8080PRE never sets SP
before its first `CALL`, so the host must start it with a usable SP. Neither executes an
undocumented opcode. No harness exists yet (bead x80-ecy.17). Which parts 8080EXM's
expected CRCs were measured on is not established by the pinned files; x80 records the
provenance of any CRC it relies on before treating a mismatch as a defect.

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
all bus/input polls. `tests/boundary.rs` checks the model-neutral source boundary
of decision-6 as well as the dependency boundary.
The V20 mapping in [plan §3](backlog/docs/plan.md#3-v20v30-integration-contract-sketch)
is explicitly unverified. Glossary definitions are orientation, not implementation
authority. Ordinary tests pin derived values in code and run without manuals,
corpus downloads or network access.

## 4. License surface and boundary

The library is `MIT OR Apache-2.0`. `x80-core` has zero dependencies of any kind;
`x80-core/tests/boundary.rs` checks Cargo's resolved graph for all targets/features.
`deny.toml` retains x86jit's permissive allowlist without its oracle exceptions.
Machine services and dependency-bearing oracle harnesses belong outside the core.
No repository or package is published yet; publishing is the maintainer's step.
