# Development setup

## Toolchain

The flake selects Rust **1.98.1**, matching x86jit, with rust-src, rust-analyzer,
clippy and rustfmt. `flake.lock` preserves its generic input pins. The shell adds
nextest, cargo-deny, pre-commit and Backlog.md; no cross targets, emulator libraries
or bindgen tools. No Nix package output is defined yet.

```sh
nix develop --no-update-lock-file
```

Initial provisioning may require network access.
Declared MSRV 1.95 is inherited provisionally and needs a dedicated check before
release. Host Rust 1.98.1 can run the dependency-free gates without Nix; see
[commands](commands.md).

## Local activation

Run these by hand once per clone:

```sh
direnv allow
scripts/skills-bootstrap.sh
pre-commit install --install-hooks
```

Review the existing local hook before installing both hook stages. Uncached hook
environments may download tools. Configure GPG signing and matching email/key UID
yourself. The generic UID hook checks consistency only when signing is already
enabled; it does not enforce or configure signing.

Canonical vendored skills belong under `.agents/skills/`; the bootstrap script
links them into ignored `.claude/skills/`. Optional `repoctx` is tool-managed and not installed by this project.

## Beads

Install bd separately on the host; it is not in this shell. x86jit's reference
workflow uses bd ≥ 1.3.1; confirm a compatible installed version and its help.
Never open a newer schema with an older tool.

The project is initialized with prefix `x80`, M0–M4 epics and the shared
memories. In a fresh clone, use the documented `bd bootstrap` recovery flow;
`bd init` would mint a new project identity. First remote setup can create
commits and needs explicit maintainer authorization.

The database is ignored; later synchronization targets only the configured
`origin`, with push authorization. See [working on tasks](working-on-tasks.md).
