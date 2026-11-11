---
name: backlog
description: >-
  Docs and decision records with the Backlog.md CLI (backlog doc, backlog decision).
  Use for project knowledge under backlog/. Not for open work: x80 uses beads (bd),
  prefix x80, after supervisor initialization.
---

# Backlog.md

Adapted from x86jit's vendored backlog skill. Use only for docs (`backlog doc`)
and decisions (`backlog decision`). Never use `backlog task create`.
Open work belongs in beads; see `backlog/docs/working-on-tasks.md`.

[Backlog.md](https://github.com/MrLesk/Backlog.md) stores knowledge as markdown
inside the repository. `backlog <command> --help` supplies installed-version flags.

## Layout

```text
backlog/
  config.yml    # docs/decisions config; task fields remain unused
  docs/         # guides and reference
  decisions/    # durable architecture and workflow records
```

## Docs and decisions

```sh
backlog doc create "<title>" [-p <subpath>]
backlog doc list
backlog decision create "<title>"
ls backlog/decisions/
```

There is no `decision list` subcommand. Cross-cutting choices become decisions;
bead-bound choices use `bd update <id> --append-notes "Decided: …"`. Never overwrite
existing notes with `--notes`. See `backlog/docs/decisions.md`.

No legacy task archive or migration history exists in x80. Tracking setup belongs
to the supervisor; do not initialize bd just to create a documentation file.
