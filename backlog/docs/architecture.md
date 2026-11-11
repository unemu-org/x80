# Architecture — planned layout

The design sketch is [plan §2](plan.md#2-architecture-sketch). The only current
code is an empty module skeleton; names below do not promise implemented APIs.

```text
x80-core/
  src/lib.rs       no_std, unsafe forbidden, module declarations
  src/state.rs     planned registers and borrowed register view
  src/bus.rs       planned borrowed memory/port interface
  src/cpu.rs       planned interpreter and step/run budgets
  src/exit.rs      planned typed return reasons and progress
  src/interrupt.rs planned injection and stop controls
  tests/boundary.rs zero-dependency tripwire
backlog/docs/      design and guides
backlog/decisions/ durable decisions (none accepted yet)
```

Planned flow: embedder-owned register file + memory/port bus + explicit events →
step/run → state updates, bus effects, cycle totals and typed return. Persistent
execution latches survive each borrow. No core-owned machine, loader, OS or device.

Only `x80-core` is a workspace member. Future dependency-bearing oracle harnesses
will be separate and unpublished; 8085 and Z80 layout waits for those milestones.
The V20 adapter belongs to the embedder; x80 will expose only agreed boundaries.
Keep this map current when modules change.
