# x80

A guest-agnostic Intel 8080-family interpreter, planned as a pure-Rust library:
8080 first, then 8085 and Z80. Embedders provide memory, port I/O and interrupts;
machines, devices and file formats live outside the core.

**Skeleton only:** `x80-core` has empty documented modules and zero dependencies.
No instructions or execution API are implemented. M0 (skeleton and tooling) is in
progress. Read the [draft plan](backlog/docs/plan.md),
[architecture](backlog/docs/architecture.md) and [provenance rules](PROVENANCE.md).

```sh
cargo build --workspace --offline --locked
cargo nextest run --workspace --all-features --offline --locked
cargo clippy --workspace --all-targets --all-features --offline --locked -- -D warnings
cargo fmt --all --check
```

[Setup](backlog/docs/dev-setup.md), [commands](backlog/docs/commands.md) and
[agent instructions](AGENTS.md). Open work uses beads (`bd`, prefix `x80`);
Backlog.md is for docs and decisions.

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
