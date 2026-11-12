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

`--model 8080|8085` selects the CPU (default 8080). Known programs must use their
intended model: 8080EXM requires 8080, 8085EXM requires 8085, and 8080PRE supports
either. A mismatch is an error before execution. Both models share the CP/M host,
including the undocumented 08h BDOS rendezvous (decisions 3 and 11).

The CLI identifies the program bytes by SHA-256, using the three pinned digests
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
an unknown digest a verdict. No interrupts or host stop are requested; on the
8085, SID is low and SOD notifications are ignored.

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

## CI decision

The reference Forgejo workflow assumes a particular container runner, installs
Nix with runner-specific flags, fetches x86-only corpora, cross-checks FreeBSD and
posts custom commit statuses. It is not generic enough to copy safely; no
`.forgejo/workflows/ci.yml` is created, as the brief permits.

Once hosting and runner provisioning are chosen, CI should check out without
persisted credentials, enter the pinned shell, then run fmt, clippy, workspace
nextest, doctests, deny and the MSRV test using the commands above. Ordinary
tests need no manuals or oracle downloads. Keep failed gates visible; missing
tools/data must not become successful checks. No remote or publication is
configured here.
