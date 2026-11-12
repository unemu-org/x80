# Status

**M0: done.** Tracking, the pinned shell, hooks, local gates and the first commit
are in place. Decisions 1-5 settle the M1 API, decision-6 the CPU model layout and
decision-7 an interim flag reading. The 8080 primary
sources are pinned (see [PROVENANCE.md](../../PROVENANCE.md)).

`x80-core` contains empty documented modules and a zero-dependency boundary test.
There is no CPU state implementation, decoder, step/run API or instruction support.
Two Intel 8080 manuals are pinned as fetch-only sources in the shared `oracles/`
submodule (pin ddef3b6). No ROMs or oracle corpora are pinned yet.
M1–M4 have not started. See [plan §5](plan.md#5-milestones).

This is a capability snapshot, not the task board. Consult beads for claims and
priorities. Update this file when a milestone advances.
