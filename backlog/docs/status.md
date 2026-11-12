# Status

**M0: done.** Tracking, the pinned shell, hooks, local gates and the first commit
are in place. Decisions 1-5 settle the M1 API, decision-6 the CPU model layout and
decision-7 an interim flag reading. The 8080 primary
sources are pinned (see [PROVENANCE.md](../../PROVENANCE.md)).

**M1: in progress.** `x80-core` has the passive types of decision-5: the
model-neutral `Bus`, `MemoryAccess`, `Control`, `Budget`, `Report`, `Exit` and
`Unsupported` at the crate root, and the `i8080` model with `Registers`, `Flags`,
`Latches`, `Inputs` and `I8080`. The BC, DE and HL pair and PSW accessors and the
PSW flag-byte layout are cited and witnessed. Nothing executes: there is no
decoder, no instruction behaviour and no `step`, `run` or `reset`, so no
instruction group is implemented yet (decision-5 item 7).

Two Intel 8080 manuals are pinned as fetch-only sources in the shared `oracles/`
submodule (pin ddef3b6). No ROMs or oracle corpora are pinned yet.
M2–M4 have not started. See [plan §5](plan.md#5-milestones).

This is a capability snapshot, not the task board. Consult beads for claims and
priorities. Update this file when a milestone advances.
