# Architecture — layout

The design sketch is [plan §2](plan.md#2-architecture-sketch). The `i8080` and
`i8085` models execute every documented instruction through `step` and budgeted
`run`, with their own interrupt paths (decisions 5/10/12;
`tests/i8080_coverage.rs`, `tests/i8085_coverage.rs`; [status](status.md)).
The `z80` model also executes every documented instruction, with model-owned
state, flag images, inputs, reset, prefix tables and NMI/INT acceptance
(decisions 13–15; `tests/z80_coverage.rs`, `tests/z80_determinism.rs`).

```text
x80-core/
  src/lib.rs                    no_std, unsafe forbidden, root re-exports
  src/bus.rs                    Bus trait and MemoryAccess kinds
  src/control.rs                Control: the host stop request
  src/budget.rs                 Budget and Report
  src/exit.rs                   Exit and its Unsupported payload
  src/decode.rs                 private pattern matcher, generic opcode-map builder and cost slots
  src/progress.rs               private run counters, saturation and budget check
  src/intel/mod.rs              private borrowed View and condition-only FlagRules (decision-16)
  src/intel/access.rs           shared operand/location/pair/condition/address and push-and-jump helpers
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
  src/i8085/mod.rs              the 8085 model module and its re-exports (decisions 10–12)
  src/i8085/state.rs            Registers, pair/PSW accessors, interrupt and sampler Latches
  src/i8085/flags.rs            five flags and the decision-11 library PSW image
  src/i8085/interrupt.rs        Inputs: five interrupt lines, SID and SOD notifications
  src/i8085/cpu.rs              I8085: run/step, reset, boundary sampling, priority and dispatch
  src/i8085/opcodes.rs          model groups, field restrictions and shared-builder map assembly
  src/i8085/opcodes/tests.rs    independent map witness (205775-002 Tables 5-1/5-2 pp. 5-19/5-20)
  src/i8085/data_transfer.rs    Data Transfer encodings and executor (205775-002 pp. 5-4 to 5-6)
  src/i8085/alu.rs              model-owned arithmetic primitives and result flags
  src/i8085/arithmetic.rs       Arithmetic encodings and executor (pp. 5-6 to 5-9)
  src/i8085/logical.rs          Logical encodings and executor (pp. 5-9 to 5-12)
  src/i8085/branch.rs           Branch encodings and executor (pp. 5-13 to 5-15)
  src/i8085/stack_io_control.rs Stack, I/O and Machine Control, excluding RIM/SIM (pp. 5-15 to 5-17)
  src/i8085/rim_sim.rs          RIM/SIM encodings and executor (pp. 5-17/5-18; decision-12 rule 8)
  src/i8085/vectored.rs         TRAP and RST 5.5/6.5/7.5 acceptance (decision-12 rule 4)
  src/i8085/acknowledge.rs      INTR RST/CALL acceptance (decision-12 rule 5)
  src/z80/mod.rs                the Z80 model module and its re-exports (decisions 13–15)
  src/z80/state.rs              main/alternate Registers, pair accessors, InterruptMode and Latches
  src/z80/flags.rs              six flags and the decision-14 library F/F' image
  src/z80/alu.rs                model-owned arithmetic primitives and flag rules
  src/z80/interrupt.rs          Inputs: asserted-true NMI and INT requests
  src/z80/cpu.rs                Z80: run/step, reset, boundary sampling, prefix dispatch and executor context
  src/z80/nmi.rs                NMI discarded fetch, stack save and 0066h entry
  src/z80/acknowledge.rs        IM 0/1/2 acceptance and acknowledge restrictions
  src/z80/opcodes.rs            eleven model groups and five private maps via decode::build
  src/z80/opcodes/tests.rs      independent literal whole-map and source-disagreement witnesses
  src/z80/load8.rs              8-Bit Load encodings and executor (UM pp. 70–97)
  src/z80/load16.rs             16-Bit Load encodings and executor (pp. 98–122)
  src/z80/exchange_block_search.rs Exchange, Block Transfer, and Search executor (pp. 123–143)
  src/z80/arithmetic8.rs        8-Bit Arithmetic encodings and executor (pp. 144–171)
  src/z80/general_arithmetic_cpu_control.rs General-Purpose Arithmetic and CPU Control (pp. 172–186)
  src/z80/arithmetic16.rs       16-Bit Arithmetic encodings and executor (pp. 187–203)
  src/z80/rotate_shift.rs       Rotate and Shift encodings and executor (pp. 204–241)
  src/z80/bit_set_reset_test.rs Bit Set, Reset, and Test encodings and executor (pp. 242–260)
  src/z80/jump.rs               Jump encodings and executor (pp. 261–279)
  src/z80/call_return.rs        Call and Return encodings and executor (pp. 280–293)
  src/z80/input_output.rs       Input and Output encodings and executor (pp. 294–316)
  tests/boundary.rs             zero-dependency, fact-free root and Intel-family tripwires
  tests/root_contract.rs        full-width port defaults/overrides and preceding-byte diagnostics
  tests/support/                recording test bus, sample state, unsupported check
  tests/z80_run.rs              boundary, refusal, landed execution, refresh and budget-split witnesses
  tests/z80_coverage.rs         independent literal execution/refusal coverage for all seven logical tables
  tests/z80_determinism.rs      prefixed mixed program, block/interrupt resume and budget replay
  tests/z80_<group>.rs          independent instruction semantics, flags, timing and bus witnesses
  tests/z80_interrupts_nmi.rs   NMI acceptance, priority, edge and halt witnesses
  tests/z80_interrupts_int.rs   IM 0/1/2 acceptance, refusal and halt witnesses
  tests/z80_timing.rs           independent whole-set/path and interrupt T-state witnesses
  tests/z80_arithmetic_reference/ independent byte/pair ALU and flag witnesses
  tests/z80_model.rs            ownership, construction, reset, input defaults and recording polls
  tests/z80_registers.rs        main/alternate register pairs and F/F' image witnesses
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
  tests/i8085_<group>.rs        independent per-group execution witnesses, including RIM/SIM
  tests/i8085_arithmetic_reference/ independent 8085 arithmetic reference
  tests/i8085_daa.rs            decimal-adjust witnesses
  tests/i8085_timing.rs         independent whole-set and interrupt T-state witnesses
  tests/i8085_interrupts_intr.rs INTR acknowledge and priority witnesses
  tests/i8085_interrupts_vectored.rs vectored acceptance, edge, mask and priority witnesses
  tests/i8085_machine_control.rs EI/DI inhibition, HLT and reset witnesses
  tests/i8085_run.rs            boundary sequence, sampling, budgets and exit precedence
  tests/i8085_coverage.rs       literal whole-set execution and unsupported witnesses
  tests/i8085_determinism.rs    mixed-program replay and budget-split witnesses
  tests/i8085_model.rs          construction, ownership and reset
  tests/i8085_registers.rs      register pairs and PSW layout
backlog/docs/                   design and guides
backlog/decisions/              durable decisions
x80-v20-contract/               unpublished test-only V20 vectors and mock native mode switches (decision-9)
```

Model split (decision-6, plan §2.6): the crate root holds the model-neutral
types, re-exported as `x80_core::{Bus, MemoryAccess, Control, Budget, Report,
Exit, Unsupported}`. They carry no hardware fact and cite decisions, not
manuals. Each CPU model is its own module and type: `x80_core::i8080`,
`x80_core::i8085` and `x80_core::z80`, each with its CPU, `Registers`, `Flags`,
`Latches` and `Inputs`.
Hardware facts live in the model or the private Intel-family access tier and
cite the pinned manuals by page. There is no family-wide public CPU or register
type and no model trait.

Intel-family tier ([decision-16](<../decisions/decision-16 - Intel-family-shared-executors-for-the-8080-and-8085.md>),
accepted 2026-11-13; View and access helpers implemented in `x80-8sg.2`):

```text
x80-core/src/intel/             private to the crate
  mod.rs                       View and condition-only FlagRules contract
  access.rs                    operand/location/pair/condition/address and push-and-jump helpers
  data_transfer.rs             planned shared Data Transfer body
  branch.rs                    planned shared Branch body, per-model untaken-read constant
  stack_io.rs                  planned shared PUSH/POP, XTHL, SPHL, IN and OUT subset
  logical.rs                   planned shared Logical body, per-model AND rule
  arithmetic.rs                planned shared Arithmetic body including qualified DAA rules
```

Each Intel model builds a concrete borrowed `View` from its own registers in a
private `view` function. Its existing executors call the shared access helpers;
executor bodies remain per model at this step. Later extractions introduce thin
wrappers handling model-owned arms before the shared fallback.
Public types, ALU/flag rules and `FlagRules` implementations, encoding lists,
state totals, map witnesses, run loops, reset, interrupts, EI/DI/HLT/NOP and
RIM/SIM stay per model. `FlagRules` currently has only `sign`, `zero`, `parity`
and `carry` accessors, implemented in each model's `flags.rs`. Later steps may
introduce the remaining flag/ALU methods listed in decision-16; methods beyond
that list need an amendment, and latch, interrupt, timing and bus members are
forbidden. `BranchFacts` is planned for the Branch extraction; untaken operand
reads and PC advancement currently remain in each model's executor. Facts that
differ or lack statements in both primaries remain model-owned rules/constants,
including qualified DAA carry and XTHL order; matching code does not upgrade
evidence.

This tier permits a hardware literal only when both pinned primaries
state the fact, with 98-153B and 205775-002 citations at that fact and independent
witnesses per model. The root, `decode` and `progress` stay fact-free. Z80's
sharing boundary stays at decode/progress/test support. The boundary test
already checks both family identifiers, excludes Z80 identifiers and concrete
model paths, and now checks both family source files non-vacuously. Extraction
stops for a maintainer checkpoint after Stack/IO, before Logical/Arithmetic, to
assess interim duplication, oracle runtime and undocumented-8085 flag research.

M4 closure duplication (2026-11-13, `x80-bi2.27`, base `37571c9`),
**MEASURED** using decision-13 rule 7's comment-stripped, nonblank, trimmed-line
LCS, exact and with CPU type/module names normalised: the unchanged five-group
8080/8085 baseline reproduces 570/556 lines, LCS 488 exact / 498 normalised;
complete `execute` functions (signatures included) have 319/310 lines, LCS
279 / 284. Z80 groups split the Intel groups differently, so semantic module
pairs are measured separately, without summing overlapping comparisons.
For example, Z80 `load8` versus either Intel `data_transfer` has 133/116 lines,
LCS 44 / 46; `call_return` versus 8080/8085 `branch` has 100/121 and 100/117
lines, LCS 62 / 64 and 56 / 58. Register-only read/write helpers match 10 lines
against the Intel 12-line register/M helpers; this is similarity evidence, not
an extraction decision. `bit_set_reset_test` has no Intel instruction-group
counterpart; block/search, alternate-register exchange, indexed/CB/ED forms,
refresh and NMI-specific helpers also lack direct counterparts. The worker's
closure report supplies full per-pair and private-helper numbers for the
supervisor to record on `x80-bi2`.

State/flag images, ALU rules (including AND auxiliary/half carry, subtraction
and parity versus overflow), encoding restrictions, T-state totals, conditional
operand reads, interrupt sources and EI/DI rules, reset, Z80 prefix/R rules,
full-width ports and block iteration semantics must remain cited and witnessed
per model. That measurement alone authorises no extraction; decision-16 amends
decision-13 rule 7 to permit the private Intel-family tier described above.
Z80 still shares only fact-free `decode`, `progress` and test support. A crate
run-time wrapper still needs an embedder request.

Root-contract extensions (decision-14 rule 3; decision-15 rules 4/5/13):
`Bus::input16` and `output16` accept the full A0-A15 port address, defaulting
to the existing 8-bit methods with A0-A7. Only models whose decisions specify
them call these methods: Z80 does, while 8080/8085 use the 8-bit methods.
Models document which instruction bytes carry `MemoryAccess::OpcodeFetch`,
potentially more than one per instruction; the 8080/8085 first-byte mapping
is unchanged. `Unsupported` reports the refused byte in `opcode` and every
instruction-stream byte read before it, in read order, in the used prefix of
`preceding: [u8; 3]` with `preceding_len` in `0..=3` (displacement included,
unused elements zero). Its `preceding()` accessor returns the used bytes and
clamps invalid lengths to capacity. `pc` remains the first-byte or acknowledge
address. Only the reads of the reported bytes have reached the bus; CPU state
is restored and read-side device effects cannot be undone. 8080/8085
report zero preceding bytes for fetched and acknowledge refusals; Z80 uses
the preceding bytes for prefixed instruction refusals.

Private machinery is shared under decision-10: `decode.rs` matches bit patterns
using model-supplied field restrictions and builds maps generic over the model's
group type. It stores lengths and two cost slots, selecting a slot by the
executor's reported path. `progress.rs` counts completed steps and supplied costs,
saturates the counters and checks the budget. Neither module carries hardware
facts; both cite decisions only. The 8080 keeps its group enum, encoding lists,
timing values, executors, ALU, flags, state, interrupts, reset and map witness.
The 8085 owns the same categories independently, plus `rim_sim`, `vectored`
and `acknowledge` (decision-10; `i8085/mod.rs`). Both Intel models use the
private access helpers under decision-16, without changing their public types.

Flow for each model: CPU (owned registers and latches) + an embedder value
that implements `Bus` and that model's `Inputs` → `step`/`run` → state updates,
bus effects, cycle totals and a typed `Report` (decisions 5/12/15). Execution
latches persist across runs. No core-owned machine, loader, OS or device.

8080 dispatch: `run` checks the budget, polls the inputs, accepts an eligible
interrupt or checks halt, then fetches an opcode. `cpu.rs` looks it up in the
opcode map. Each group module owns its `ENCODINGS` (bit patterns as printed in
98-153B Ch. 4, lengths and required state totals) and its `execute`, which
returns the completed timing path. `opcodes.rs` supplies the five lists and
its cited field restrictions to the shared builder at compile time. In
particular, excluding 110 from DDD/SSS remains 8080 data, not a matcher rule.
Only undocumented fetched bytes or non-RST acknowledge bytes return
`Unsupported`, with no architectural state change.
There is no documented-but-unimplemented dispatch path (decision-5 item 7).

8085 dispatch follows decision-12: budget; six polls in fixed order; stop;
edge sampling and highest eligible interrupt; halt; opcode fetch and group
execution. `opcodes.rs` supplies six encoding lists to `decode.rs`, separating
RIM/SIM from the five manual groups; executors report paths and `cpu.rs`
selects the model-owned costs. `vectored.rs` and `acknowledge.rs`, like the
8080 acknowledge path, use `intel::access::push_and_jump` with explicit return
PC and target for stack writes and transfer of control (decision-16 rule 5;
decision-12 rules 4/5). Stop preserves state,
Unsupported restores pre-sampling latches, and Halt retains samples; the
boundary and replay witnesses are `tests/i8085_run.rs` and
`tests/i8085_determinism.rs`. Residual qualifications are in PROVENANCE.md §3.

Z80 map data (decision-13 rules 2–5; decision-14 rule 1): eleven group
modules each supply only their applicable `MAIN`, `CB`, `ED`, `INDEX` and
`INDEX_CB` slices. `opcodes.rs` calls the unchanged `decode::build` separately
for each table, with model-owned field restrictions. The key is respectively
the first byte, second byte after CB/ED/DD/FD, or fourth byte after DD/FD CB d.
All lengths and both cost slots describe the complete instruction. MAIN's four
prefix slots and INDEX's CB slot have no terminal entry. INDEX and INDEX_CB
serve both IX and IY, so the five stored counts 252/248/58/39/31 represent 698
logical terminal encodings. Repeating blocks store terminating/repeating totals
as untaken/taken (decision-15 rule 11). The independent literal witness checks
every key, group, length, cost slot and complement against the rendered primary
UM008011-0816 tables/entries and 03-0029-01 Tables 7.0-1–7.0-11; source
reconciliations are pinned separately and recorded in PROVENANCE.md §1.

Z80 dispatch and executor handoff (`x80-bi2.9`; decisions 13–15): `run` checks
budget; polls stop, NMI, INT exactly once in that order; honors stop; samples
NMI; selects a fresh NMI edge before eligible INT; checks halt; fetches and
executes. Stop precedes sampling. A refusal restores the complete CPU state
saved after polling and before sampling, including R and sampler history;
Halt retains samples. Interrupt retirement uses the same counters as ordinary
instructions. Every documented encoding dispatches directly to its group.
Only table holes return Unsupported, using the exact decision-14 rule 4 reads
and payload. `tests/z80_run.rs` pins boundary and refusal traces;
`tests/z80_coverage.rs` checks every literal documented/undocumented key.
`tests/z80_determinism.rs` checks complete state, bus trace and poll-sequence
equality across unlimited, one-step and cycle-budget execution, including
interrupts between repeating-block iterations and after HALT.

Every group executor has this final signature, using `cpu::Decoded`,
`opcodes::Instruction`, and `decode::Path`:

```rust
pub(super) fn execute<S: Bus>(
    cpu: &mut Z80,
    system: &mut S,
    instruction: &Instruction,
    decoded: &Decoded,
) -> Path
```

`Decoded` holds `table: Table` (`Main`, `Cb`, `Ed`, `Index`, `IndexCb`),
`opcode: u8` (table key), `index: Option<IndexRegister>` (`Ix`, `Iy`),
`displacement: Option<u8>` (only `IndexCb`) and `start_pc: u16`. The dispatcher
leaves PC **after the fetched prefix/key bytes**, including d/op for indexed CB.
`instruction.length` and `states` describe the whole instruction. Executors
must not add that length to the already advanced PC. They read remaining
operands in stream order through `cpu.fetch_operand(system)`, which returns
one `OperandFetch` byte and wraps PC forward without changing R. For ordinary
INDEX memory forms the executor reads d first, then any immediate byte; INDEX
pair/index-only forms have no d. Indexed-CB executors use the saved d and never
read it again. The dispatcher makes no data/stack/port access.

All eleven group executors are complete; there is no staging switch or
documented-but-unimplemented dispatch path. The dispatcher selects the returned
Path's complete cost and retires once.
A block executor rewinds PC by two and returns `Path::Taken` when repeating;
termination returns `Path::Untaken`. The next iteration naturally polls and
fetches both bytes again (decision-15 rule 11; UM LDIR p. 132, TM Table 7.0-3
p. 46). The loop has no repeat-specific machinery.

R advances once per `OpcodeFetch`, prefixes included, keeping bit 7 and
wrapping the low seven bits. Indexed CB reads DD/FD and CB as `OpcodeFetch`,
then d and op as `OperandFetch`, so it advances R twice. This fourth-byte
classification remains the **unverified** reading of decision-15 rule 5;
per-M1 counting is **derived** (rule 6; UM Registers p. 3, TM §§2/3 pp. 4/7).
No refresh bus call or halted R advance occurs (API choices). The EI delay
and parity-note pending latch expire before ordinary execution, allowing EI
and LD A,I/R to re-arm them; refusal restores both. EI-delay edge cases and
the parity note's boundary interpretation remain unverified under rules 7/10.

The final interrupt seams are `nmi::accept<S: Bus>(&mut Z80, &mut S) ->
Option<u8>` and `acknowledge::accept<S: Bus>(&mut Z80, &mut S) ->
Option<Result<u8, Unsupported>>`. None means not accepted and must have no
CPU or bus effect; Some completes one interrupt step or refuses it. The loop
retires returned states and skips handler execution, retaining the EI delay
through NMI acceptance. INT refusal restores the entire pre-sampling snapshot.
NMI saves PC and enters 0066h. INT accepts RST only in IM 0, uses 0038h in
IM 1, and saves PC before reading the even-addressed vector pair in IM 2.
Other IM 0 bytes and odd IM 2 vectors are refused before writes/table reads
(decision-15 rules 8–9). The acceptance paths call
`cpu.increment_r()` once after their discarded fetch/acknowledge respectively,
and `cpu.consume_parity_note()` only on successful acceptance, after refusal
validation and before service. That helper clears P/V only when a note is
pending, consumes the note and leaves the EI delay alone. Module comments
pin the access order, mode restrictions, costs and primary/second citations
of decision-15 rules 8–10 for each implementation.

An integration test that needs a bus includes `tests/support` with
`mod support;`. Its `RecordingBus` logs every access and input poll for trace
witnesses. Witness expectations are literals derived from the cited pages,
never crate helpers.

`x80-oracle/` is an unpublished workspace member with no dependencies beyond
`x80-core`: a CP/M-style host that runs the pinned exercisers on the
selected 8080/8085/Z80 model and gives a verdict only for the pinned SHA-256 digests
(commands.md, "Oracle runs"; PROVENANCE.md §2). Z80 prelim is judged and
ZEXDOC runs the 58 audited documented groups; nine groups are not run under
decision-14. Its private ED 70 BDOS rendezvous and digest-keyed in-memory table
filtering belong to the host, outside core instruction semantics.

Any future dependency-bearing oracle harness will also be separate and unpublished.
The Z80 layout follows decision-13 (accepted): [Z80 model, prefix tables and sharing](<../decisions/decision-13 - z80-model-layout-prefix-tables-and-sharing.md>).
The V20 adapter belongs to the embedder; x80 will expose only agreed boundaries.
Keep this map current when modules change.
