# Commands

Run from the workspace root in `nix develop` or the direnv shell. These commands
do not install missing tools. Cargo gates use offline mode; the empty core needs
no registry downloads. [Dev setup](dev-setup.md) covers provisioning.

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
now, not a feature toggle. No binary or emulator CLI exists.

Use `cargo fmt --all` to format. Add release-profile runs when optimization could
change behavior. Serialize heavy campaigns; no perf, fuzz or cross-target gate
exists at this skeleton stage.

## MSRV

The declared MSRV is Rust 1.95 (`rust-version` in `Cargo.toml`). The `msrv` dev
shell pins Rust 1.95.0; this command enters it, so run it from the workspace root
in any shell:

```sh
nix develop .#msrv --no-update-lock-file -c cargo test -p x80-core --offline --locked
```

It builds `x80-core` and runs its unit, boundary and doc tests with Rust 1.95.0;
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
