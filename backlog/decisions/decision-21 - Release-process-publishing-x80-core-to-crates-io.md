---
id: decision-21
title: Release process publishing x80-core to crates.io
date: '2026-11-16'
status: accepted
---

Decider: maintainer (2026-11-16). Bead `x80-0q8`, epic `x80-tqc`. This record
is documentation only; the beads of the epic implement it.

> [decision-22](<decision-22 - GitHub-is-the-primary-forge.md>) made GitHub the
> primary forge on the same day. Where this record assumes a Forgejo
> development origin or a Forgejo mirror, decision-22 applies.

## Context

- The repository is public at <https://github.com/unemu-org/x80>. Forgejo stays
  the development origin; its CI (`.forgejo/workflows/ci.yml`) runs the gates on
  every push to `main`.
- No crate is published yet. The workspace version is 0.1.0, the licence
  `MIT OR Apache-2.0` and the MSRV Rust 1.95. `x80-oracle` and
  `x80-v20-contract` already set `publish = false`.
- The ordinary gates do not run the CPU test programs. These run locally as
  optional campaigns from the hash-pinned oracle stash
  ([commands](../docs/commands.md#oracle-runs)).
- On 2026-11-16 the crates.io API returned 404 for `x80-core`, `x80`,
  `x80-oracle` and `x80-v20-contract` (MEASURED).

## Decision

1. **Scope.** Only `x80-core` is published, under that name. `x80-oracle` and
   `x80-v20-contract` stay unpublished.
2. **Trigger.** A pushed tag `vX.Y.Z` on GitHub starts
   `.github/workflows/release.yml`. The workflow fails unless `X.Y.Z` equals
   the `x80-core` version and the tagged commit is reachable from GitHub
   `main`. GitHub `main` must equal Forgejo `main` before tagging.
3. **Checks before publication.** The release workflow runs on GitHub:
   - every Forgejo gate: formatting, Clippy, nextest, doctests, cargo-deny
     and the MSRV test;
   - the CPU test programs in the release profile: 8080PRE, 8080EXM, 8085EXM,
     Z80 prelim, ZEXDOC, ZEXALL and z80test.

   The CPU test job downloads only those programs, from the upstream URLs in
   `oracles/fetch-oracles.sh`, and verifies each one against
   `oracles/SHA256SUMS.fetched` before use. It does not commit, cache or
   redistribute them. A failed check stops the release before publication.
4. **Approval.** The publish job runs in the GitHub environment `release`.
   The environment requires the maintainer's approval, so publication stays a
   maintainer step.
5. **Authentication.** The publish job uses crates.io Trusted Publishing
   (OpenID Connect, `rust-lang/crates-io-auth-action`). No long-lived crates.io
   token is stored. crates.io accepts a trusted publisher only for an existing
   crate, and "initial publish requires an API token" (READ: crates.io
   Trusted Publishing page, Prerequisites; page source
   `svelte/src/routes/docs/trusted-publishing/+page.svelte` in
   `rust-lang/crates.io` at commit `b92b83f1`, read 2026-11-16). The maintainer
   therefore publishes 0.1.0 by hand once, with a short-lived API token.
6. **Repeated runs.** If the tagged version already exists on crates.io, the
   workflow skips `cargo publish` and continues. A rerun after a later failure
   is therefore safe, and the tag `v0.1.0` after the manual first publication
   only creates the GitHub Release.
7. **GitHub Release.** The workflow creates it last, with the `CHANGELOG.md`
   section of the version as its notes. Until the checks pass, only the tag is
   public.
8. **Versions.** Releases follow SemVer as Cargo applies it: before 1.0 a
   minor version may break the API, a patch version may not. `CHANGELOG.md`
   records every released version.
9. **Workflow hygiene.** Actions are pinned to full commit SHAs. Each job gets
   the least permissions it needs: `contents: read` by default, `id-token: write`
   only for publication, `contents: write` only for creating the GitHub Release.

## Alternatives

- **A GitHub Release created by hand as the trigger** (`release: published`).
  Rejected on 2026-11-16: the release page is public before the checks run, and
  a failed check leaves a release without a crate.
- **A manual `workflow_dispatch` that creates the tag.** Rejected: CI would need
  permission to push tags, and a tag push is as simple.
- **A crates.io API token in a repository secret.** Rejected: a long-lived
  credential that Trusted Publishing makes unnecessary.
- **Gates only, without the CPU test programs.** Rejected: the programs are the
  widest end-to-end check of the release profile, and publication cannot be
  undone (crates.io allows only a yank).
- **Publication without approval.** Rejected for the same reason.
- **Publishing as `x80`, or publishing `x80-oracle` as well.** Rejected: the
  published name stays the workspace package name, and `x80-oracle` is a test
  host for fetched GPL programs, not a library.

## Consequences

- New files: a GitHub CI workflow callable from the release workflow, a job for
  the CPU test programs, `.github/workflows/release.yml`, `CHANGELOG.md` and a
  release guide under `backlog/docs/`. The Forgejo workflow does not change.
- The `x80-core` manifest gains a `repository` field, its package ships the
  licence files, and its README renders on crates.io.
- One-time maintainer steps: publish 0.1.0 by hand, configure the trusted
  publisher on crates.io (repository `unemu-org/x80`, workflow `release.yml`,
  environment `release`) and create the `release` environment on GitHub with a
  required reviewer.
- Still open: how GitHub `main` follows Forgejo `main` (a Forgejo push mirror
  or a push in the landing step). Until the maintainer decides, `main` is
  pushed to GitHub by hand.

## Revisit when

- crates.io or GitHub change Trusted Publishing or environment approvals;
- the CPU test job approaches GitHub's job time limit;
- another crate needs publication;
- the API is stabilised for 1.0.
