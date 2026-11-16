---
id: decision-22
title: GitHub is the primary forge
date: '2026-11-16'
status: accepted
---

Decider: maintainer (2026-11-16). Bead `x80-41o`. Supersedes the parts of
[decision-21](<decision-21 - Release-process-publishing-x80-core-to-crates-io.md>)
that assume a Forgejo development origin; its release process stands
otherwise.

## Context

decision-21 kept a private Forgejo instance as the development origin and
GitHub as a public copy. On 2026-11-16 the copy ran as a Forgejo push mirror
(branch filter `main`, all tags). It worked, but two forges meant two CI
systems, a deploy key and a token scope problem: GitHub rejected a deploy-key
push that changed `.github/workflows/` until the key was added again with a
token holding the `workflow` scope. The beads database lived only on Forgejo, although its
content is written to be publishable anyway (AGENTS.md hard rules).

## Decision

1. **GitHub is the primary forge** for code, CI, releases and planning:
   <https://github.com/unemu-org/x80> is `origin`. Landings push `main` there,
   and `git push origin vX.Y.Z` starts the release workflow.
2. **CI runs only on GitHub Actions** (`.github/workflows/ci.yml`,
   `cpu-tests.yml`, `release.yml`). The Forgejo workflow and its status script
   are removed.
3. **The beads database is public in the same repository.** `bd dolt push`
   writes `refs/dolt/data` and the branch `__dolt_remote_info__` on `origin`;
   a fresh clone recovers it with `bd bootstrap`. Bead text stays English and
   publish-safe, as before. The maintainer accepted publishing the existing
   database history as it is.
4. **Pull requests are disabled** on GitHub while the project matures. Issues
   are welcome, and conversations about them stay open there.
5. **Forgejo keeps a backup.** The maintainer's Forgejo instance pulls a mirror
   of the GitHub repository, beads data included. The former Forgejo
   repository is archived.

## Alternatives

- **Forgejo primary with a push mirror to GitHub** (decision-21, run on
  2026-11-16). Rejected: two CI systems and a mirror credential, for a copy
  that is public anyway.
- **GitHub primary with beads on a private Forgejo remote.** Rejected: beads
  are not secret, and one remote keeps `bd bootstrap` simple.
- **Beads in a separate GitHub repository.** Rejected: an extra sync
  configuration for every clone, to hide one branch.
- **Pull requests closed by a bot.** Unnecessary: GitHub can disable pull
  requests for a repository.

## Consequences

- `backlog/docs/commands.md` describes only the GitHub workflows. CI is
  dispatched with `gh workflow run` and read in GitHub's checks.
- The README says that development happens on GitHub, pull requests are
  disabled and issues are welcome.
- decision-21 item 2 no longer needs GitHub `main` to equal Forgejo `main`;
  its open point about keeping the two in sync is settled by this record.
- Everything pushed to `origin`, including bead history, is public at once.

## Revisit when

- the project accepts contributions;
- GitHub changes its handling of pull requests, Actions or Trusted Publishing;
- the beads tool changes how it stores its database in git.
