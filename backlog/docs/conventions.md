# Conventions

Rust edition 2021. Development toolchain: Rust 1.98.1, matching x86jit. Declared
MSRV: 1.95, provisional until tested. `x80-core` uses `no_std`, no allocation, no
dependencies and `forbid(unsafe_code)`; host integration tests may use `std`.

- Use snake_case modules, one primary concept per file; update the architecture
  map on structural changes. Cross-crate consumers use deliberate public APIs.
- Prefer typed enums for public failures; never add panic stubs that look like
  working CPU APIs. Do not add error or extension machinery speculatively.
- Names explain what; comments explain constraints and cite design sections.
  Hardware behavior cites a pinned document, revision, section/table and page;
  see [PROVENANCE.md](../../PROVENANCE.md#1-authority-and-citations).
- Centralize register pairs and flags once their semantics are verified. Keep
  expected witness values independent of implementation helpers.
- Do not invent a retry/atomicity guarantee for side-effecting bus accesses.
  Settle completion and resume behavior in the bus contract before coding it.
- Follow milestone order. File discovered work separately rather than expanding
  a fix. No speculative abstractions for later CPU models.

Use Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, `test:`,
`ci:`, `build:`), one GPG-signed squash commit per bead. Never commit or push
without an explicit maintainer request; never bypass hooks. The generic GPG UID
hook checks key/email consistency when signing is configured; it does not enable
or enforce signing itself. Setup must configure GPG signing explicitly.

Formatting and clippy with `-D warnings` must pass; see [commands](commands.md).
When facts or contracts remain uncertain, record the question rather than guessing.
