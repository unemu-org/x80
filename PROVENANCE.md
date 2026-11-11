# Provenance

## 1. Authority and citations

Derive hardware behavior from primary sources, cited by document, edition/revision,
section or table, and printed page (also PDF page if different). Pin revision and
SHA-256 before implementing a fact; check scans against page images. Comments
also link the relevant design section. Each implemented fact needs an independent
witness test with expected values derived from that citation.

The list below is a discovery inventory, **not a set of pinned sources**. No
8080-family document is pinned or verified yet (bead x80-2g7.1). Unknown metadata is
marked `source to find` rather than guessed. URLs remain unset until confirmed.

| Candidate title | Order number | Year | Public source URL | Intended role |
| --- | --- | --- | --- | --- |
| Intel 8080 Microcomputer Systems User's Manual | source to find | 1975 (brief; edition to verify) | source to find | 8080 state, instructions, flags, machine cycles and state tables |
| Intel 8080/8085 Assembly Language Programming Manual | source to find | source to find | source to find | Instruction semantics and programming model |
| Intel MCS-80/85 Family User's Manual | source to find | source to find | source to find | Family architecture, interrupts and timing |
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

8080 diagnostics/8080EXM-family programs, Z80 ZEX-family programs and available
hardware-captured single-step corpora are candidates only. Their origin,
licenses, exact versions and checksums must be checked before any vendoring or
execution harness is added. Hardware captures describe the measured silicon, not
every family member. No oracle artifacts are currently included.

## 3. Witnesses and current scope

Only a module skeleton exists; no hardware semantics or timing constants are
implemented. The V20 mapping in [plan §3](backlog/docs/plan.md#3-v20v30-integration-contract-sketch)
is explicitly unverified. Glossary definitions are orientation, not implementation
authority. Ordinary tests will pin derived values in code and run without manuals,
corpus downloads or network access.

## 4. License surface and boundary

The library is `MIT OR Apache-2.0`. `x80-core` has zero dependencies of any kind;
`x80-core/tests/boundary.rs` checks Cargo's resolved graph for all targets/features.
`deny.toml` retains x86jit's permissive allowlist without its oracle exceptions.
Machine services and dependency-bearing oracle harnesses belong outside the core.
No repository or package is published yet; publishing is the maintainer's step.
