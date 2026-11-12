# Working on tasks

Open work uses beads (`bd`, prefix `x80`). Backlog.md stores docs and decisions
only. The draft plan is a design reminder, not a task board. These rules adapt
x86jit's generic shared rules.

## Roles and session start

The maintainer decides scope, priorities and outward-facing actions. The supervisor
scopes, claims, reviews, runs gates and closes beads. Workers implement a narrow
brief or perform read-only review; they never claim/close beads, commit, push,
merge or change design documents outside their brief. Delegation follows the
active session's authorization. Commits and pushes require the maintainer's request.

At session start, read `bd memories`, then `bd list --status in_progress` and
`bd show <id>` before task work. `bd <command> --help` supplies version-specific
flags. Repository rules override generic `bd prime` advice. A sandbox unable to
open the beads lock cannot reliably read memories either: the supervisor supplies
the relevant rules in its brief and handles tracking operations.

## Statuses and epics

| Status | Meaning |
| --- | --- |
| `open` | Scoped, not started; default for new beads |
| `in_progress` | Claimed before implementation |
| `closed` | Landed on `main`; close reason names the commit and applicable design section |

Blocked work is derived from dependencies; `bd ready` excludes open blockers.
Priorities run P0 (highest) through P4. Each theme gets an epic; task children use
`--parent`, `--no-inherit-labels` and their own labels. Housekeeping may stand alone.
Each milestone M0–M4 has an epic; list them with `bd list --type epic`.

```sh
bd ready --exclude-type epic
bd list --type epic
bd list --parent <epic-id>
bd show <id>
bd create --title "<title>" --description "<why and what>" --acceptance "<criteria>" \
  --type task --priority P2 --parent <epic-id> --no-inherit-labels --labels <theme>
bd update <id> --claim
bd update <id> --append-notes "MEASURED: <command> → <result>"
bd close <id> --reason "<commit> — <what landed; design section changed>"
```

Never use `--notes` on an existing bead: it overwrites the field. Label evidence
`MEASURED` (command/result), `READ` (file:line), `ASSUMED` (not checked). Discovered
work gets a separate bead with `--deps discovered-from:<id>`. A shortcut gets a
proper-solution follow-up bead. Do not close before the change reaches `main`.

Spikes use `--type spike` and a spike-key title prefix. Describe the question,
known facts, hypothesis, method, time box, done-when, failure path and document it
feeds. Close only after that document changes. A failed hypothesis produces a
plan-B bead with `discovered-from`. At the time box, record findings and ask the
maintainer whether to extend, switch plan or accept the risk.

## Definition of Done

Before landing, the supervisor verifies:

1. Requested scope and docs agree; hardware facts have precise primary citations
   and independent witnesses.
2. Pinned-toolchain `cargo fmt --all --check` and clippy with all targets/features
   and `-D warnings` pass.
3. Workspace nextest and doctests pass, including the zero-dependency boundary.
4. `cargo deny --all-features check licenses bans sources` passes.
5. The [MSRV gate](commands.md#msrv) passes.
6. A fresh session reviews the final change. Small changes need a fresh read-only
   reviewer; larger changes need correctness, provenance and test review. Verify
   findings against evidence; uncertain claims remain unconfirmed. Fixes receive
   fresh review. Multiple agents require session authorization.

Run heavy campaigns one at a time per machine. CPU witnesses should fail before
and pass after where practical; scaffolding needs relevant structural checks.
Add release/native-host checks when optimization or platform code warrants them.
Never treat an unavailable gate as green or bypass hooks to land a change.

## Git flow and publication

One bead, one branch `<type>/<bead-or-topic>`, one worktree outside the main
checkout. The supervisor squash-merges one Conventional, GPG-signed commit per
bead, describing the final change. Never commit or push without an explicit
maintainer request. Push only to configured `origin`. Repository creation,
mirroring and publishing belong to the maintainer.

The local beads database is ignored by git. A `bd dolt push`
to `refs/dolt/data` on `origin`, including associated remote metadata, is a push
requiring authorization. Setup commands can themselves commit; inspect their help
and obtain the needed authorization first.

Everything is English, publish-safe and guest-agnostic, including bead history and
memories. No proprietary guests, private hosts/paths or personal setup. Use
`bd remember` only for contributor-wide facts absent from docs. Cross-repository
ids carry a repository tag. Durable decisions go in `backlog/decisions/`;
bead-bound choices are append-notes. See [decisions](decisions.md).

## Worker briefs and session handoff

Briefs specify branch/worktree, exact finding with file:line, writable files,
tests/commands and evidence report. Restate no-commit/push/merge, publication,
primary-source provenance and the zero-dependency/core boundaries. Supply access
to pinned manuals read-only; do not assume a fresh worktree includes a source stash.
After a job, check `git diff --stat` and untracked files against the brief.
Re-check surprising claims; do not repeat trusted delegated checks without cause.
If patching loses the design, restate it or reassign the work.

At session end, append completed work, next step, blockers and commands/results:

```sh
bd update <id> --append-notes "Session pause <date>. Done: <X>. Next: <Y>. Blocker: <Z|none>."
```

The next session reads the bead before rediscovering state from the diff.
