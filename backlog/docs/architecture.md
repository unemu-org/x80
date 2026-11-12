# Architecture — layout

The design sketch is [plan §2](plan.md#2-architecture-sketch). The types of
decision-5 exist, and `step` and `run` execute under a budget; only NOP
executes so far ([status](status.md)). Items marked planned are not
implemented APIs.

```text
x80-core/
  src/lib.rs                    no_std, unsafe forbidden, root re-exports
  src/bus.rs                    Bus trait and MemoryAccess kinds
  src/control.rs                Control: the host stop request
  src/budget.rs                 Budget and Report
  src/exit.rs                   Exit and its Unsupported payload
  src/i8080/mod.rs              the 8080 model module and its re-exports
  src/i8080/state.rs            Registers (BC, DE, HL and PSW accessors) and Latches
  src/i8080/flags.rs            Flags and the cited flag byte of the PSW
  src/i8080/interrupt.rs        Inputs: the INT line
  src/i8080/cpu.rs              I8080: run/step, boundary sequence, fetch and dispatch; planned reset
  src/i8080/opcodes.rs          private opcode map: documented, length, group, state totals
  src/i8080/opcodes/tests.rs    map witness against 98-153B p. 4-15 and 9800301-04 p. A-5
  src/i8080/data_transfer.rs    Data Transfer Group: encodings and executor (pp. 4-4, 4-5)
  src/i8080/arithmetic.rs       Arithmetic Group (pp. 4-6 to 4-8)
  src/i8080/logical.rs          Logical Group (pp. 4-8 to 4-11)
  src/i8080/branch.rs           Branch Group (pp. 4-11, 4-12)
  src/i8080/stack_io_control.rs Stack, I/O and Machine Control Group (pp. 4-13, 4-14)
  tests/boundary.rs             zero-dependency tripwire
  tests/support/                recording test bus, sample state, unsupported check
  tests/i8080_run.rs            budget, boundary sequence, exit precedence, step, determinism
  tests/i8080_<group>.rs        one witness file per instruction group
  tests/*.rs                    other witnesses and API tests
backlog/docs/                   design and guides
backlog/decisions/              durable decisions
```

Model split (decision-6, plan §2.6): the crate root holds the model-neutral
types, re-exported as `x80_core::{Bus, MemoryAccess, Control, Budget, Report,
Exit, Unsupported}`. They carry no hardware fact and cite decisions, not
manuals. Each CPU model is its own module and type; the only one is
`x80_core::i8080`, with `I8080`, `Registers`, `Flags`, `Latches` and `Inputs`.
Hardware facts live in the model and cite the pinned manuals by page. There is
no family-wide CPU or register type and no model trait.

Flow: `I8080` (owned registers and latches) + an embedder value that implements
`Bus` and `i8080::Inputs` → `step`/`run` → state updates, bus effects, cycle
totals and a typed `Report`. Execution latches persist across runs. No
core-owned machine, loader, OS or device.

Dispatch: `run` checks the budget, polls the inputs and checks halt, then
`cpu.rs` reads the opcode and looks it up in the opcode map. Each group module
owns its `ENCODINGS` (bit patterns as printed in 98-153B Ch. 4, lengths and,
once cited, state totals) and its `execute`; `opcodes.rs` builds the 256-entry
map from the five lists at compile time. An opcode returns `Unsupported` with
nothing changed when it is undocumented, when its state totals are not cited
yet, or when its group's `execute` returns `None` (not implemented yet). A
group bead changes its own module and its own `tests/i8080_<group>.rs`.

An integration test that needs a bus includes `tests/support` with
`mod support;`. Its `RecordingBus` logs every access and input poll for trace
witnesses. Witness expectations are literals derived from the cited pages,
never crate helpers.

Only `x80-core` is a workspace member. Future dependency-bearing oracle harnesses
will be separate and unpublished; 8085 and Z80 modules wait for those milestones.
The V20 adapter belongs to the embedder; x80 will expose only agreed boundaries.
Keep this map current when modules change.
