# Status

**M0: in progress.** Tracking, the pinned shell, hooks, local gates and the first
commit are done. Decisions 1-4 settle the initial API choices. Source pins remain.

`x80-core` contains empty documented modules and a zero-dependency boundary test.
There is no CPU state implementation, decoder, step/run API or instruction support.
No 8080-family manuals, ROMs or oracle corpora are pinned yet; the shared `oracles/`
submodule (pin 86e33db) holds only other projects' sources.
M1–M4 have not started. See [plan §5](plan.md#5-milestones).

This is a capability snapshot, not the task board. Consult beads for claims and
priorities. Update this file when a milestone advances.
