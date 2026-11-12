# Architecture — planned layout

The design sketch is [plan §2](plan.md#2-architecture-sketch). The only current
code is an empty module skeleton; names below do not promise implemented APIs.

```text
x80-core/
  src/lib.rs       no_std, unsafe forbidden, module declarations
  src/state.rs     planned 8080 registers and execution latches
  src/bus.rs       planned borrowed memory/port interface
  src/cpu.rs       planned interpreter and step/run budgets
  src/exit.rs      planned typed return reasons and progress
  src/interrupt.rs planned input lines and host stop
  tests/boundary.rs zero-dependency tripwire
backlog/docs/      design and guides
backlog/decisions/ durable decisions
```

Planned flow: model type (owned registers and latches) + embedder bus and input
lines → step/run → state updates, bus effects, cycle totals and typed return.
Execution latches persist across runs. No core-owned machine, loader, OS or device.

Model layout (decision-6, plan §2.6): model-neutral bus, budget, report and exit
types stay at the crate root; each CPU model gets its own module and type,
starting with `i8080::I8080`, which `x80-ecy.5` introduces. The planned state,
cpu and interrupt modules above move under that model.

Only `x80-core` is a workspace member. Future dependency-bearing oracle harnesses
will be separate and unpublished; 8085 and Z80 modules wait for those milestones.
The V20 adapter belongs to the embedder; x80 will expose only agreed boundaries.
Keep this map current when modules change.
