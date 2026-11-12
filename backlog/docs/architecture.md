# Architecture — layout

The design sketch is [plan §2](plan.md#2-architecture-sketch). The passive
types of decision-5 exist; nothing executes yet. Items marked planned are not
implemented APIs.

```text
x80-core/
  src/lib.rs             no_std, unsafe forbidden, root re-exports
  src/bus.rs             Bus trait and MemoryAccess kinds
  src/control.rs         Control: the host stop request
  src/budget.rs          Budget and Report
  src/exit.rs            Exit and its Unsupported payload
  src/i8080/mod.rs       the 8080 model module and its re-exports
  src/i8080/state.rs     Registers (BC, DE, HL and PSW accessors) and Latches
  src/i8080/flags.rs     Flags and the cited flag byte of the PSW
  src/i8080/interrupt.rs Inputs: the INT line
  src/i8080/cpu.rs       I8080, owning registers and latches; planned step/run/reset
  tests/boundary.rs      zero-dependency tripwire
  tests/support/         recording test bus shared by integration tests
  tests/*.rs             witnesses and API tests
backlog/docs/            design and guides
backlog/decisions/       durable decisions
```

Model split (decision-6, plan §2.6): the crate root holds the model-neutral
types, re-exported as `x80_core::{Bus, MemoryAccess, Control, Budget, Report,
Exit, Unsupported}`. They carry no hardware fact and cite decisions, not
manuals. Each CPU model is its own module and type; the only one is
`x80_core::i8080`, with `I8080`, `Registers`, `Flags`, `Latches` and `Inputs`.
Hardware facts live in the model and cite the pinned manuals by page. There is
no family-wide CPU or register type and no model trait.

Planned flow: `I8080` (owned registers and latches) + an embedder value that
implements `Bus` and `i8080::Inputs` → step/run → state updates, bus effects,
cycle totals and a typed `Report`. Execution latches persist across runs. No
core-owned machine, loader, OS or device.

An integration test that needs a bus includes `tests/support` with
`mod support;`. Its `RecordingBus` logs every access and input poll for trace
witnesses. Witness expectations are literals derived from the cited pages,
never crate helpers.

Only `x80-core` is a workspace member. Future dependency-bearing oracle harnesses
will be separate and unpublished; 8085 and Z80 modules wait for those milestones.
The V20 adapter belongs to the embedder; x80 will expose only agreed boundaries.
Keep this map current when modules change.
