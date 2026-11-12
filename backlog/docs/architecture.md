# Architecture — layout

The design sketch is [plan §2](plan.md#2-architecture-sketch). The types of
decision-5 exist, and `step` and budgeted `run` execute every documented 8080
instruction and accept RST interrupts ([status](status.md)).

```text
x80-core/
  src/lib.rs                    no_std, unsafe forbidden, root re-exports
  src/bus.rs                    Bus trait and MemoryAccess kinds
  src/control.rs                Control: the host stop request
  src/budget.rs                 Budget and Report
  src/exit.rs                   Exit and its Unsupported payload
  src/decode.rs                 private pattern matcher, generic opcode-map builder and cost slots
  src/progress.rs               private run counters, saturation and budget check
  src/i8080/mod.rs              the 8080 model module and its re-exports
  src/i8080/state.rs            Registers (BC, DE, HL and PSW accessors) and Latches
  src/i8080/flags.rs            Flags and the cited flag byte of the PSW
  src/i8080/interrupt.rs        Inputs: the INT line
  src/i8080/cpu.rs              I8080: run/step, reset, interrupt acceptance, fetch and dispatch
  src/i8080/opcodes.rs          private model groups, field restrictions and opcode-map assembly
  src/i8080/opcodes/tests.rs    map witness against 98-153B p. 4-15 and 9800301-04 p. A-5
  src/i8080/data_transfer.rs    Data Transfer Group: encodings and executor (pp. 4-4, 4-5)
  src/i8080/alu.rs              arithmetic primitives and shared flag calculation
  src/i8080/arithmetic.rs       Arithmetic Group (pp. 4-6 to 4-8)
  src/i8080/logical.rs          Logical Group (pp. 4-8 to 4-11)
  src/i8080/branch.rs           Branch Group (pp. 4-11, 4-12)
  src/i8080/stack_io_control.rs Stack, I/O and Machine Control Group (pp. 4-13, 4-14)
  src/i8085/                    model-owned state, six completing instruction groups and five interrupt sources (decisions 10–12)
  tests/boundary.rs             zero-dependency and model-neutral root tripwires
  tests/support/                recording test bus, sample state, unsupported check
  tests/i8080_run.rs            budget, boundary sequence, exit precedence, step, determinism
  tests/i8080_<group>.rs        one witness file per instruction group
  tests/i8080_daa.rs            decimal-adjust witnesses
  tests/i8080_timing.rs         independent whole-set T-state witness
  tests/i8080_interrupts.rs     interrupt acceptance, acknowledge and halt traces
  tests/i8080_machine_control.rs EI delay, DI, HLT and reset
  tests/i8080_coverage.rs       literal whole-set execution and unsupported witnesses
  tests/i8080_determinism.rs    mixed-program replay and budget-split witnesses
  tests/i8080_model.rs          construction and ownership
  tests/i8080_registers.rs      register pairs and PSW layout
  tests/recording_bus.rs        8080 test-bus and input-default witnesses
  tests/arithmetic_reference/  independent 8080 arithmetic reference
backlog/docs/                   design and guides
backlog/decisions/              durable decisions
x80-v20-contract/               unpublished test-only V20 vectors and mock native mode switches (decision-9)
```

Model split (decision-6, plan §2.6): the crate root holds the model-neutral
types, re-exported as `x80_core::{Bus, MemoryAccess, Control, Budget, Report,
Exit, Unsupported}`. They carry no hardware fact and cite decisions, not
manuals. Each CPU model is its own module and type: `x80_core::i8080` and
`x80_core::i8085`, each with its CPU, `Registers`, `Flags`, `Latches` and `Inputs`.
Hardware facts live in the model and cite the pinned manuals by page. There is
no family-wide CPU or register type and no model trait.

Private machinery is shared under decision-10: `decode.rs` matches bit patterns
using model-supplied field restrictions and builds maps generic over the model's
group type. It stores lengths and two cost slots, selecting a slot by the
executor's reported path. `progress.rs` counts completed steps and supplied costs,
saturates the counters and checks the budget. Neither module carries hardware
facts; both cite decisions only. The 8080 keeps its group enum, encoding lists,
timing values, executors, ALU, flags, state, interrupts, reset and map witness.

Flow: `I8080` (owned registers and latches) + an embedder value that implements
`Bus` and `i8080::Inputs` → `step`/`run` → state updates, bus effects, cycle
totals and a typed `Report`. Execution latches persist across runs. No
core-owned machine, loader, OS or device.

Dispatch: `run` checks the budget, polls the inputs, accepts an eligible
interrupt or checks halt, then fetches an opcode. `cpu.rs` looks it up in the
opcode map. Each group module owns its `ENCODINGS` (bit patterns as printed in
98-153B Ch. 4, lengths and required state totals) and its `execute`, which
returns the completed timing path. `opcodes.rs` supplies the five lists and
its cited field restrictions to the shared builder at compile time. In
particular, excluding 110 from DDD/SSS remains 8080 data, not a matcher rule.
Only undocumented fetched bytes or non-RST acknowledge bytes return
`Unsupported`, with no architectural state change.
There is no documented-but-unimplemented dispatch path (decision-5 item 7).

An integration test that needs a bus includes `tests/support` with
`mod support;`. Its `RecordingBus` logs every access and input poll for trace
witnesses. Witness expectations are literals derived from the cited pages,
never crate helpers.

`x80-oracle/` is a second, unpublished workspace member with no dependencies beyond
`x80-core`: a CP/M-style host that runs the pinned 8080 exercisers and gives a verdict
only for their pinned SHA-256 digests (commands.md, "Oracle runs").

Any future dependency-bearing oracle harness will also be separate and unpublished.
The Z80 module waits for its milestone beads.
The V20 adapter belongs to the embedder; x80 will expose only agreed boundaries.
Keep this map current when modules change.
