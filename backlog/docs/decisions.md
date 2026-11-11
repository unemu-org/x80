# Where to capture a decision

| Surface | Use |
| --- | --- |
| `backlog/decisions/` | Durable architecture, public API, source policy or cross-cutting workflow choices |
| Bead append-note | A choice bound to one task |

```sh
backlog decision create "<summary>"
ls backlog/decisions/
bd update <id> --append-notes "Decided: <choice> over <alternative> — <reason>; revisit if <trigger>."
```

Use `backlog <command> --help` for installed-version flags. There is no separate
ADR directory or task board. Never overwrite an existing bead's notes with `--notes`.

A decision records status, date, decider, context, choice, alternatives,
consequences and trigger to revisit. Supersede an accepted decision in a new
record; do not silently change its substance. Tag cross-repository references,
e.g. `decision-23 (x86jit)`. Accepted records live in
[`backlog/decisions/`](../decisions/); the plan stays a draft outside them.
