# Commands

Run from the workspace root in `nix develop` or the direnv shell. These commands
do not install missing tools. Cargo gates use offline mode; the dependency-free core
needs no registry downloads. [Dev setup](dev-setup.md) covers provisioning.

## Build, test, lint

```sh
cargo build --workspace --offline --locked
cargo test --workspace --offline --locked
cargo nextest run --workspace --all-features --offline --locked
cargo test --workspace --doc --all-features --offline --locked
cargo clippy --workspace --all-targets --all-features --offline --locked -- -D warnings
cargo fmt --all --check
cargo deny --offline --all-features check licenses bans sources
cargo tree --offline --locked -p x80-core --all-features --target all --edges normal,build,dev
```

`cargo test` is the fallback when nextest is absent; it also runs doctests.
The boundary test invokes offline, locked `cargo tree` and rejects every dependency
kind, including optional and target-specific dependencies. The library is `no_std`
now, not a feature toggle. The separate unpublished `x80-oracle` crate supplies
the offline exerciser CLI.

Use `cargo fmt --all` to format. Add release-profile runs when optimization could
change behavior. Serialize heavy campaigns; no perf, fuzz or cross-target gate
exists yet.

## Oracle runs

Fetch the pinned artifacts first with `./oracles/fetch-oracles.sh fetch`.
The GPL-2.0-or-later exerciser programs and sources stay in the fetch-only oracle
stash; the harness does not embed or download them. See [provenance](../../PROVENANCE.md#2-oracles).
These runs are optional campaigns, outside the ordinary gates, and require no
network once the artifacts are present. Run the preliminary program first:

```sh
cargo run -p x80-oracle --release --offline --locked -- oracles/8080-exercisers/8080PRE.COM
cargo run -p x80-oracle --release --offline --locked -- oracles/8080-exercisers/8080EXM.COM --cycle-cap 100000000000
cargo run -p x80-oracle --release --offline --locked -- --model 8085 oracles/8080-exercisers/8080PRE.COM
cargo run -p x80-oracle --release --offline --locked -- --model 8085 oracles/8080-exercisers/8085EXM.COM --cycle-cap 100000000000
```

`--model 8080|8085|z80` selects the CPU (default 8080). Known programs must use their
intended model: 8080EXM requires 8080, 8085EXM requires 8085, and 8080PRE supports
either. A mismatch is an error before execution. The models share the CP/M host
services. The 8080 retains its unsupported 08h BDOS rendezvous (decision-3).
Decision-18 admits every 8085 opcode, so its host instead supplies `OUT FFh; RET`
at 0005h through instruction reads. Data reads retain the stack word at 0006h.
Only that stub's reserved-port write requests a host stop at the next boundary;
the host reads C/DE, performs BDOS and clears the request before resuming RET.
This adds exactly one retired instruction and 10 cycles per 8085 BDOS call.
An OUT reaching the overall cycle cap times out before servicing BDOS; a slice
boundary alone defers the stop to the next run without repeating the OUT.

For 8080/8085, the CLI identifies the program bytes by SHA-256, using the three pinned digests
from `oracles/SHA256SUMS.fetched` (8085EXM added at oracles commit `f379261`), then
echoes the program's console output and reports total steps, cycles, exit reason
and verdict. Only those digests can receive a PASS/FAIL verdict. Exit 0 requires
a known digest, a halt after the boot-entry stub (PC 0001h), and the preliminary
completion message or the exerciser's model-specific banner, exactly 25 complete
group results in the pinned order, and completion message, in that sequence.
Framing and descriptors are matched by length and FNV-1a fingerprint; result
fields require their exact labels. Every nonempty line must match a framing or result
line, with no unconsumed text. CRCs contain exactly eight lowercase hexadecimal
digits, with both expected and found fields required on ERROR. Partial results,
unknown lines, extra markers and trailing text fail. The grammar follows
8080EXM.MAC:114–130,436–481,898–959 and
8085EXM.MAC:115–131,437–482,899–960.
8080EXM requires all 25 to PASS with no ERROR. 8085EXM requires 24 judged groups
to PASS with no ERROR; the third group, the long ALU register group (`alu8r`,
8085EXM.MAC:136,201–207), has an unverified expected CRC. Its descriptor fingerprint
and third position must both match. Its printed result and
CRC are repeated as `reported, not judged` and excluded from the verdict, even
if the program prints ERROR for it. All 25 results must still be present with
complete CRCs, with no unknown, duplicated, reordered, missing or extra groups
(group tables: 8080EXM.MAC:132–158;
8085EXM.MAC:133–159). A whole-output ERROR check permits only the complete,
exempt third result; unknown lines also reject markers split across lines.
Rejected output includes a verdict failure reason.
Unknown programs still run, report `unknown program, no verdict`, and exit 1.
Exit 1 also covers a failing or absent verdict,
unexpected halt, unsupported instruction (with PC/opcode), timeout, invalid input
or host error. Exit 2 means a missing program file and prints the fetch command;
missing artifacts never pass.

`--cycle-cap N` sets the overall cycle limit (default 100000000000);
`--slice-cycles N` sets the nonzero per-run budget (default 1000000).
Budgets stop at instruction boundaries, so totals can overshoot by the final
instruction. The exerciser can take a long time. The digest selects the verdict
protocol regardless of filename. Optional `--oracle pre` or `--oracle exm` checks
the expected identity of a known program for the selected model; it cannot give
an unknown digest a verdict. No interrupts are requested. Only the 8085 BDOS
stub requests host stop; on the 8085, SID is low and SOD notifications are ignored.

For Z80, run the pinned preliminary test first, then the full 58-group release
campaign (the larger cap accommodates the long ALU groups):

```sh
cargo run -p x80-oracle --release --offline --locked -j 4 -- --model z80 oracles/z80-exercisers/prelim.com
cargo run -p x80-oracle --release --offline --locked -j 4 -- --model z80 oracles/z80-exercisers/zexdoc.com --cycle-cap 1000000000000
```

The Z80 path accepts only the original `prelim.com` and `zexdoc.com` SHA-256
pins in `oracles/SHA256SUMS.fetched`; unknown digests and model mismatches are
errors before execution. Filenames and `--oracle pre|exm` cannot grant a verdict.
No fetched program is needed for the harness unit or CLI tests.

The Z80 CP/M host leaves INT/NMI unasserted and supports the same BDOS functions
2 and 9, writable RAM, entry 0100h, a usable initial SP, the stack word at 0006h,
and JP 0000h exit. Its private BDOS fetch response is **ED 70**, a refused hole
under decision-14 (UM008011-0816 p. 296); 08h remains documented EX AF,AF'.
The PROVENANCE §2 audit confirms neither program executes ED holes, including
ED 70. Only an Unsupported exit at PC 0005h with preceding byte ED and refused
opcode 70, while the host awaits registers, services this rendezvous. The second
byte is synthesized only for OpcodeFetch at 0006h; Data reads of the stack word
remain intact. The retry returns the installed RET. Core semantics are unchanged.
Z80 refusals outside this rendezvous print PC, opcode and all preceding bytes;
unexpected port accesses report the full 16-bit port address (decision-15).

ZEXDOC filtering happens in loaded RAM, after digest selection, without modifying
the disk file, descriptors, expected CRCs or instruction bytes. The host first
verifies all 67 original little-endian pointers at file offset 003Ah (guest
013Ah), including the zero terminator at 00C0h; each descriptor address must be
01C2h + 0060h × its original zero-based index. A mismatch fails before any patch
or guest execution. The table is compacted to the 58 judged pointers in order,
followed by zeroes. The nine excluded groups are each printed to stderr as
`not run (decision-14)`: `alu8rx`, `incxh`, `incxl`, `incyh`, `incyl`, `ld8ixy`,
`ld8rrx`, `rotxy`, `rotz80`. They never execute; no Unsupported result is waived.

Prelim requires its exact completion output; silent early failure, a printed
failure address, extra lines or trailing text fail. ZEXDOC requires its banner,
all 58 complete descriptor/results in audited order, and completion message.
Every nonempty line is consumed; unknown, missing, duplicated, reordered or
extra lines fail. Each judged group must print the program's exact OK suffix,
which follows comparison with all four expected CRC bytes. ZEXDOC does not
print CRCs on OK. ERROR results include eight lowercase hexadecimal digits in
each expected/found field and always fail; malformed or partial results also
fail. Exit 0 additionally requires the boot-stub halt at PC 0001h. A timeout
remains FAIL even if earlier groups printed OK.

A bounded debug smoke can be run without the release campaign:

```sh
cargo run -p x80-oracle --offline --locked -j 4 -- --model z80 oracles/z80-exercisers/prelim.com
cargo run -p x80-oracle --offline --locked -j 4 -- --model z80 oracles/z80-exercisers/zexdoc.com --cycle-cap 5000000000
```

The capped command intentionally exits 1 if it reaches the cap before completion;
it is evidence for only the complete groups printed before that point.

## V20 contract fixtures

`cargo test -p x80-v20-contract --offline --locked` runs the portable vectors
and mock native operations; see [the fixture format](../../x80-v20-contract/README.md).
No native core, manual download or x86 execution is needed.

## MSRV

The declared MSRV is Rust 1.95 (`rust-version` in `Cargo.toml`). The `msrv` dev
shell pins Rust 1.95.0; this command enters it, so run it from the workspace root
in any shell:

```sh
nix develop .#msrv --no-update-lock-file -c cargo test -p x80-core --offline --locked
```

It builds `x80-core` and runs its unit, integration (including boundary) and doc tests
with Rust 1.95.0;
the boundary test's nested `cargo tree` uses the same Cargo. First use may download
the toolchain. Clippy, rustfmt, nextest and deny stay on the default shell.
Without Nix, any Rust 1.95.0 toolchain can run the same cargo command.

## Tooling

```sh
nix develop --no-update-lock-file
bash -n scripts/skills-bootstrap.sh scripts/uid-guard.sh
pre-commit validate-config .pre-commit-config.yaml
pre-commit run --all-files
pre-commit run --all-files --hook-stage pre-push
```

Nix and uncached hooks may fetch tools on first use. `pre-commit run --all-files`
covers tracked files only: before a file is first committed, pass it with
explicit `--files <paths...>` or run the checks directly.
Stages: pre-commit = hygiene, fmt and GPG UID consistency; pre-push = clippy.
Neither config nor bootstrap enables GPG signing automatically.

## CI

The [CI workflow](../../.forgejo/workflows/ci.yml) runs manually
(`workflow_dispatch`) on the project's shared Forgejo runner. Pushes do not start
CI. The `ubuntu-latest` label runs an x86-64 Linux job in a `node:20-bookworm`
container as root without systemd. Runs are serialized on the shared runner;
a newer dispatch on the same ref cancels an earlier run.

CI checks out without persisted credentials, installs pinned Nix and enters the
pinned dev shell without updating the lock file. It runs fmt, clippy with
`-D warnings`, workspace nextest with `--no-fail-fast`, doctests and cargo deny,
then tests `x80-core` in the separate MSRV shell. Cargo gates use the offline
commands above, with `--locked` where supported. Ordinary tests need no manuals
or oracle downloads. Oracle/exerciser campaigns and pre-commit stay local;
pre-commit includes the local `gpg-uid-guard` signing check.

After pushing the intended ref to origin, dispatch from an authenticated Forgejo
CLI configured for this repository:

```sh
fj actions dispatch ci.yml <ref>
```

Confirm that a new run for the intended ref appears in Forgejo Actions before
waiting for it. Read the result in the `ci / x86_64` commit status; its target
links to the run. The description names failed or skipped steps and includes
short failure excerpts when available. Every gate runs after a successful dev
shell even if an earlier gate failed. A failed, skipped or missing required step
makes the final report step fail, so missing tools cannot produce a passing run.
Gate output is teed to `ci-logs/<step>.log` in the job workspace. If posting the
commit status fails, consult the Actions job log and result for the gate verdict.
