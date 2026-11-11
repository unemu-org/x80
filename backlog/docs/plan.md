# Draft plan — x80

Design sketch, not an accepted API or a task board. Open work belongs in beads.

## 1. Goals and non-goals

Build a guest-agnostic, pure-Rust interpreter library for the Intel 8080 family:
8080 first, then 8085, then Zilog Z80 in the same crate family. Start with zero
dependencies. Embedders own memory, port devices, scheduling and interrupt sources.
Potential consumers include CP/M environments, Altair-class machines, arcade boards
and later Z80 machines; none supplies special cases to the core.

No JIT, machine emulation, x86 execution, device models, file-format loaders,
operating-system services or bundled ROMs. V20/V30 mode switching belongs to
x86jit and its embedder; x80 executes only the emulation-mode instruction stream.

## 2. Architecture sketch

### 2.1. Crate layout

One workspace member now: `x80-core`, edition 2021, zero normal, build or dev
dependencies. `tests/boundary.rs` enforces that boundary, including optional and
target-specific dependencies. Only empty documented modules exist. Add a separate
unpublished harness crate later if oracle tooling needs dependencies; do not create
8085/Z80 crates or a generic decoder framework before their milestones.

### 2.2. CPU state and borrowing

Planned logical state: A, B, C, D, E, H, L, PC, SP, flags, interrupt-enable state,
pending enable delay, halt state and pending interrupt/stop state. Exact flag
images, reset values and timing latches require the pinned manual, not guessed
constants. Keep register-pair and flag manipulation centralized.

Prefer an owned register file for standalone use plus a small safe register-access
trait/view for borrowed storage. A run borrows registers and an embedder memory
view; it neither takes ownership nor assumes compatible Rust struct layouts.
Execution latches must persist across runs and budget exits, independently of the
register view. Avoid raw-pointer overlays and a frozen C ABI at this stage.

### 2.3. Bus interface

Draft choice: a synchronous, borrowed `Bus` trait for byte memory reads/writes and
port reads/writes, with access context sufficient to distinguish instruction fetch
and data access. This fits a small interpreter and avoids replaying I/O side
effects through an exit protocol. An embedder implements segmentation, mapping
and devices. No host allocation, filesystem access or global memory in the core.

Bus failures, wait-state accounting and observation points remain open. Do not
promise restartable bus faults until partial reads/writes and device side effects
have an explicit contract. Prefer instruction-boundary returns initially; bus-cycle
stepping is a separate requirement, not implied by instruction cycle totals.

### 2.4. Step, run, exits and timing

Proposed shapes only: `step(registers, bus, control)` executes at most one
instruction; `run(registers, bus, control, budget)` repeats it. Budgets carry
instruction steps and T-states; reports include actual progress and a typed return
reason (budget, halt, stop request, pending interrupt handoff, integration boundary,
unsupported encoding or bus failure). No callable API is implemented yet.

Draft budget policy: zero budget executes nothing; stop when either limit is
reached. Finish an instruction already started and report actual cycles, including
any overshoot of the cycle limit. Define overflow, interrupt-acknowledgement
accounting and halted progress before implementation. A strict no-overshoot limit
would need a different contract; conditional timing and bus waits complicate
preflight. Halt must return promptly rather than spin forever in `run`.

M1 targets documented per-instruction T-state totals, including taken/untaken
paths, from the manual's state tables. Do not advertise cycle-exact bus waveforms,
READY sampling or pin-level timing on that basis alone.

### 2.5. Interrupts, determinism and no_std

The embedder supplies interrupt requests and acknowledgement data explicitly.
Separate a cooperative host stop from an architectural interrupt. The standalone
8080 path will accept eligible interrupts according to cited enable/delay/halt
rules; an integration policy must hand a V20 interrupt request back to the
embedder before x80 services it, including when 8080 interrupts are disabled.
Settle priority and precise observation boundaries with the NEC contract.

Identical initial state, bus responses and ordered injected events must produce
identical state, accesses, exits and cycle counts. No wall clock, host randomness
or hidden worker threads. Timing and events advance only through explicit inputs.
The empty library already uses `#![no_std]` and forbids unsafe code; preserve a
no-allocation default unless a demonstrated requirement changes that decision.
Host-side integration tests may use `std` without adding crate dependencies.

## 3. V20/V30 integration contract sketch

First consumer: x86jit's planned NEC V20/V30 models. Settle this contract jointly
with x86jit's accepted **decision-23** (x86jit commit 0eb0d06, §3 "8080 emulation
mode on the V20/V30 is split between two cores") and its V20/V30 bead **C4
(xj-50w.7 in x86jit)**, which designs the typed `Exit` and depends on pinning the
NEC V20/V30 manual (xj-vao). Until then x86jit refuses BRKEM with a typed exit.
These are coordination references, not verified semantics.

Provisional flow supplied by the brief: x86jit returns a typed entry exit at
BRKEM; the embedder lends the mapped registers and shared memory to x80; a
RETEM-like boundary returns control for native resumption. CALLN requires a native
call handoff, and an external interrupt forces native mode. Verify every detail
against a pinned NEC manual before implementing any part of this flow.

| 8080 register/view | Proposed V20/V30 backing; unverified |
| --- | --- |
| A | AL |
| BC | CX |
| DE | DX |
| HL | BX |
| SP | BP |
| Memory | DS-relative embedder bus view |

TODO: verify PC/IP entry and return positions, flag mapping and preserved native
bits, address/segment wrapping, stack frames, high-byte preservation, and the
ownership of native-mode context. The embedder owns all x86 register conversion.

RETEM/CALLN recognition must be an explicit integration profile or extension
boundary, not a change to ordinary 8080 opcode behavior. Define fetched/consumed
bytes, committed state and cycle accounting on each return; choose whether the
embedder recognizes boundaries before `step` or x80 exposes typed extension
exits. No dependency on x86jit and no x86 execution inside x80.

TODO: define interrupt request versus acceptance, masked interrupts, interrupted
halt, event priority, native return frames, CALLN continuation, cancellation and
budget precedence. Witness one complete entry/exit round trip and interrupted
round trip using shared memory and a borrowed register view on both projects.

## 4. Testing strategy and provenance

Pin sources first using [PROVENANCE.md](../../PROVENANCE.md). For each opcode,
derive independent expected values and cycle totals from document, revision,
section/table and printed page. Witness flags that change and flags that remain,
carry/auxiliary carry boundaries, decimal adjustment, register/memory variants,
stack and address wrapping, taken/untaken paths, halt and delayed interrupt enable.
Exercise all documented encodings and record the chosen unsupported/undocumented
opcode policy; never infer one from another emulator's implementation.

Use exhaustive small input spaces where practical for byte ALU operations, plus
small instruction sequences for stateful behavior. Add budget/event boundary
tests, deterministic bus traces, borrowed-versus-owned state comparisons and
instruction-cycle totals checked against the manual's state tables. Expected
values must not merely call the implementation's own flag/timing helpers.

Candidate oracles: 8080 exercisers/diagnostics (for example CPU diagnostic and
8080EXM-family programs), Z80 ZEX-family exercisers, and hardware-captured
single-step corpora where available. Exact artifacts, origin, licenses, pins and
redistribution permissions are **to check before vendoring**. Oracles can falsify
a claim but never license a fact; record measured divergences and consult primary
sources. Manuals and downloaded corpora must never be required for ordinary
offline witness tests. No 8080-family corpus or manual is pinned yet; the shared
`oracles/` checkout currently holds only other projects' sources.

## 5. Milestones

| Milestone | Completion criterion |
| --- | --- |
| M0 — skeleton and tooling | Review scaffold, initialize tracking, verify the pinned shell and local gates, settle source pins and initial API decisions. Tracked as epic `x80-2g7`. |
| M1 — 8080 | Complete documented instruction behavior, interrupts and halt, manual-derived cycle counts, per-opcode witnesses and licensed diagnostic runs. |
| M2 — V20 integration | Agree the shared contract with x86jit; borrowed-state/memory entry, RETEM/CALLN and interrupt handoffs pass joint witnesses. |
| M3 — 8085 | Pin 8085 sources; implement and test model-specific instructions, flags, interrupts and timing without weakening 8080 behavior. |
| M4 — Z80 | Pin Zilog sources; extend state, decoding, interrupts and timing with model-specific witnesses and licensed oracles. |

These are planning landmarks; after setup, beads holds priorities, dependencies
and claims. Do not turn this draft into a second task tracker.

## 6. Open questions for the maintainer

1. Which exact manual editions, order numbers and public scans should be pinned,
   (Storage is decided: the shared `unemu-org/oracles` repository as x80's own submodule.)
2. Confirm the synchronous bus choice: infallible access or typed failures, wait
   states, and instruction-boundary timing versus bus-cycle accuracy for M1?
3. Accept cycle-budget overshoot at completed instruction boundaries, and how
   should halt, interrupt acknowledgement and counter overflow consume budgets?
4. Is a safe borrowed-register trait sufficient for V20, and which execution
   latches must the embedder retain across native/emulation transitions?
5. The joint register/flag/PC/segment/stack and native-context mapping is settled in
   x86jit's C4 bead (xj-50w.7) against the pinned NEC manual; x80's side must be
   reviewed in the same change. Who owns the shared test fixture?
6. Who recognizes RETEM/CALLN, and what exact bytes, state, timing and event
   precedence apply to each native handoff, including masked external interrupts?
7. What is the policy for undocumented opcodes/flags and silicon variants across
   8080, 8085 and Z80? Which diagnostic/capture licenses permit oracle use?
8. Keep the no_std/no-allocation default and provisional Rust 1.95 MSRV inherited
   from x86jit, or choose a different tested MSRV before the first API lands?
