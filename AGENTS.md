# AGENTS.md — x80

Canonical repository instructions for coding agents. `CLAUDE.md` imports this file.

## What this is

`x80` is a guest-agnostic Intel 8080-family interpreter library in pure Rust:
8080 first, then 8085 and Z80. Memory, port devices, scheduling, interrupt sources,
file formats and machines belong to embedders. No JIT, x86 execution or machine
emulation. `x80-core` has zero dependencies; its boundary test enforces the rule.
Today milestones M0-M5 are done: the `i8080`, `i8085` and `z80` models execute
through `step` and `run`, and the V20/V30 8080 mode runs as the `i8080` model under
an integration profile (decision-9); see [status](backlog/docs/status.md).

Open work uses beads (`bd`, prefix `x80`); design, guides and decisions live in
`backlog/`. Backlog.md is for docs and decisions only. The beads database is
initialized; in a fresh clone use `bd bootstrap`, never `bd init`.

## Where things live — load on demand

| Need | Path or command | When to load |
| --- | --- | --- |
| Draft goals, design and milestones | [plan](backlog/docs/plan.md) | Design or scope questions; proposals are not accepted APIs |
| Ready work / active claims | `bd ready --exclude-type epic`, `bd list --status in_progress`, `bd show <id>` | Session start |
| Shared rules | `bd memories`, `bd recall <key>` | Session start; docs remain the fallback |
| Behavior, citations and licenses | [PROVENANCE.md](PROVENANCE.md) | Before encoding a hardware fact or resolving an oracle disagreement |
| Layout and module map | [architecture](backlog/docs/architecture.md) | Structural changes or unfamiliar code |
| Rust and commit conventions | [conventions](backlog/docs/conventions.md) | Before changing code |
| Lifecycle, review, gates and handoff | [working on tasks](backlog/docs/working-on-tasks.md) | Before starting tracked work |
| Build and verification | [commands](backlog/docs/commands.md) | Running checks |
| Releases | [releasing](backlog/docs/releasing.md) | Preparing or cutting an `x80-core` release |
| Shell, direnv, hooks and bd setup | [dev setup](backlog/docs/dev-setup.md) | Onboarding or tooling work |
| Status / deliberate deferrals | [status](backlog/docs/status.md), [deferred](backlog/docs/deferred.md) | Checking support or considering extra scope |
| Decision capture | [decisions guide](backlog/docs/decisions.md), [records](backlog/decisions/) | Durable contract choices |
| Terminology | [glossary](backlog/docs/glossary.md) | Unfamiliar family terms |
| Backlog doc/decision workflow | [backlog skill](.agents/skills/backlog/SKILL.md) | Using the Backlog.md CLI |
| Structural code navigation | `.claude/skills/repoctx/SKILL.md`, if generated | Definitions, callers and impact; use text search when unavailable |

Load the relevant file or cited section, not the whole documentation tree.
Vendored skills live under `.agents/skills/`; `scripts/skills-bootstrap.sh` links
them into ignored `.claude/skills/`. `repoctx` is tool-managed and not vendored.

## Working on tasks

The supervisor scopes, claims, reviews and closes beads; workers implement or
review a narrow brief and report evidence. Claim before implementation; close only
after landing on `main`, citing the commit. One bead, one branch, one separate
worktree; serialize heavy tests. Never overwrite existing notes with `--notes`:
use `--append-notes`, labeling evidence `MEASURED`, `READ` or `ASSUMED`.

Start with shared rules and in-progress beads. End mid-task with completed work,
next step, blockers and commands/results on the bead. A sandboxed worker unable
to read bd takes the relevant rules from its supervisor's brief. Detailed workflow
and adapted shared rules live in [working-on-tasks.md](backlog/docs/working-on-tasks.md).
Use `bd remember` only for contributor-wide facts absent from docs, never personal
setup. Cross-repository identifiers carry a tag, e.g. `decision-23 (x86jit)`.

## Hard rules

- Never commit without an explicit maintainer request. Plan acceptance does not
  authorize committing. Push only when requested, only to configured `origin`.
- Write as if public: English, publish-safe, guest-agnostic code, docs, bead text,
  memories and commit messages. No proprietary guest names, private hosts/paths
  or personal setup. Never create, mirror or publish a repository; that is the
  maintainer's step.
- Add only requested features and required fixes; no speculative framework or
  opportunistic cleanup. Respect milestone order and deliberate deferrals.
- Derive documented behavior only from cited primary sources: document/revision,
  section/table and page. Pin every implemented fact with an independent witness.
  Undocumented facts need a revision/date-and-hash-pinned written A/B/C statement
  plus independent silicon confirmation (B or A of different origin), subject to the [second-source manufacturer licence in decision-18](<backlog/decisions/decision-18 - 8085-undocumented-instructions-and-V-K-flags.md#amendment-to-decision-17>); see
  [decision-17](<backlog/decisions/decision-17 - Two-key-evidence-rule-for-undocumented-behaviour.md>).
  Oracles can falsify; alone they cannot license a fact. Never read another
  emulator's source or documentation. Unknown facts stay explicitly unverified.
- Preserve the zero-dependency, guest-agnostic core. Machine behavior and x86
  register/segment conversion belong to the embedder. Hardware-derived code and
  witnesses cite both the design section and the primary source.
- Keep project knowledge under `backlog/`, apart from root entry points, license,
  provenance and bootstrap files. Open tasks belong in beads, not markdown lists;
  the requested draft plan's design questions/TODOs are an explicit exception.
- Never bypass hooks. Report unavailable gates with their limits; never claim a
  skipped check passed. Repository docs/memories override generic `bd prime`
  advice where they differ; explicit maintainer instructions take precedence.

## Code ownership

Maintainer / sole decider: Mikołaj Mikołajczyk.
