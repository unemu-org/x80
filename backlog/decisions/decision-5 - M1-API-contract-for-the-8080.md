---
id: decision-5
title: M1 API contract for the 8080
date: '2026-11-12'
status: accepted
---

Decider: maintainer (2026-11-12). Fixes what decision-1,
decision-2 and decision-3 leave to the API bead, within the model layout of
decision-6. Design in [plan](../docs/plan.md) §2.2-2.6. Bead `x80-ecy.4`.

## Context

decision-1 accepts a synchronous infallible bus but leaves its signatures open.
decision-2 accepts the budget rules but leaves exit precedence open. decision-3
accepts a typed unsupported exit but leaves its contents open. decision-6 makes
each CPU model a concrete type over a small model-neutral core. The 8080 facts
used here were checked against page images of 98-153B:

- p. 2-6, Table 2-1: the status word distinguishes instruction fetch (M1, "the
  fetch cycle for the first byte of an instruction"), memory read, memory
  write, stack read, stack write (STACK: "the address bus holds the pushdown
  stack address from the Stack Pointer"), input read, output write and
  interrupt acknowledge.
- p. 2-10, Table 2-2: in T1 "a memory address or I/O device number is placed on
  the Address Bus"; p. 4-3: "port: 8-bit address of an I/O device"; p. 4-14: IN
  and OUT carry a one-byte port operand.
- p. 2-11, "Interrupt Sequences" and Figure 2-8: INT is sampled with INTE in the
  last state of an instruction, so the instruction in progress completes. The
  interrupt machine cycle resembles an ordinary FETCH with the INTA status bit;
  the program counter is placed on the address lines but not incremented; the
  device "jams" an eight-bit interrupt instruction onto the data bus in T3. RST
  is the one-byte call provided for this.
- p. 2-13, "Halt Sequences" and "Start-up of the 8080 CPU": an interrupt exits
  halt only if INTE was set when halt was entered; RESET clears the program
  counter and has no effect on the flags, the working registers or the stack
  pointer.
- p. 5-14, 8080A pin definitions: INTE "is automatically reset (disabling
  further interrupts) at time T1 of the instruction fetch cycle (M1) when an
  interrupt is accepted and is also reset by the RESET signal"; INT is
  recognised "at the end of the current instruction or while halted"; RESET:
  "The INTE and HLDA flip/flops are also reset".

## Choice

Sketch below; `x80-ecy.5` fixes the exact names. Behaviour is fixed here.

```rust
// Model-neutral, crate root.
#[non_exhaustive]
pub enum MemoryAccess { OpcodeFetch, OperandFetch, Data, Stack, InterruptAcknowledge }
pub trait Bus {
    fn read(&mut self, address: u16, kind: MemoryAccess) -> u8;
    fn write(&mut self, address: u16, value: u8, kind: MemoryAccess); // Data or Stack
    fn input(&mut self, port: u8) -> u8;
    fn output(&mut self, port: u8, value: u8);
}
pub trait Control {                    // host stop, every model
    fn stop_requested(&mut self) -> bool { false }
}
pub struct Budget { pub steps: u64, pub cycles: u64 }
pub struct Report { pub steps: u64, pub cycles: u64, pub exit: Exit }
#[non_exhaustive]
pub enum Exit { Budget, Stop, Halt, Unsupported(Unsupported) }
#[non_exhaustive]
pub struct Unsupported { pub pc: u16, pub opcode: u8, pub acknowledge: bool }

// Model module.
pub mod i8080 {
    pub struct Registers { /* A, flags, B, C, D, E, H, L, SP, PC */ }
    pub struct Latches { /* interrupt enable, EI delay, halted */ }
    pub trait Inputs: super::Control {
        fn interrupt_requested(&mut self) -> bool { false } // INT line
    }
    pub struct I8080 { /* Registers, Latches */ }
    // I8080::new(Registers, Latches), registers(_mut), latches(_mut), reset,
    // run<S: Bus + Inputs>(&mut self, &mut S, Budget) -> Report, step(..).
}
```

1. **Model and ownership (decision-6).** `i8080::I8080` owns an
   `i8080::Registers` and an `i8080::Latches`. The borrowed register view of
   plan §2.2 is an M2 question (plan §6 Q4, `x80-pzp`).
2. **Construction.** `new` takes explicit registers and latches. `Default` is
   all zeros with interrupts disabled and not halted, a library choice, not a
   hardware claim (p. 2-13: register contents are indeterminate at power-up).
3. **Bus (extends decision-1).** Every memory access carries a kind:
   `OpcodeFetch` for the first byte of an instruction, `OperandFetch` for its
   following bytes, `Stack` for accesses at the stack pointer, `Data` for other
   memory data, and `InterruptAcknowledge` for the acknowledge read. Writes
   carry `Data` or `Stack`. Except `OperandFetch`, the kinds follow Table 2-1;
   the 8080 reports operand bytes as a memory read, so `OperandFetch` is an API
   distinction for embedders that place instruction-stream bytes differently
   (requested by x86jit for its V20/V30 adapter). Which accesses of each
   instruction are stack accesses follows the machine-cycle table (98-153B
   pp. 2-16 to 2-20) and is cited per instruction. The memory address is `u16`.
   The port is the 8-bit device number (pp. 4-3, 4-14). `MemoryAccess` is
   non-exhaustive; M4 revisits the port width. decision-1's other choices
   stand; on acceptance, decision-1 gets a pointer to this record.
   Z80 extensions for port width and `OpcodeFetch` wording: decision-15
   rules 4–5; this item's 8080 contract is unchanged.
4. **Input lines and acknowledge.** At each instruction boundary that
   passes the budget check, the core calls `stop_requested` and then
   `interrupt_requested`, exactly once each, whatever their results and the
   CPU state. Both should be side-effect-free level reads; the call pattern is
   part of the deterministic trace. An interrupt is eligible when INT is high,
   interrupts are enabled and the EI delay has expired. On acceptance the core
   clears the interrupt-enable latch (p. 5-14), leaves halt, reads
   `InterruptAcknowledge` at the current PC without incrementing PC, and
   executes the returned byte as that boundary's instruction (p. 2-11). M1
   implements and witnesses RST n only; any other byte gives `Unsupported`
   (item 6). Chosen over a one-shot request set before `run`, because a device
   can raise INT during a run and observes the acknowledge when it happens.
5. **Exit precedence.** At each instruction boundary, in this order:
   1. budget: either limit reached, including a zero budget, gives `Budget`
      with no poll and no bus access;
   2. poll the inputs (item 4); a host stop gives `Stop`;
   3. an eligible interrupt is accepted, and its acknowledged instruction
      executes in place of step 5; the boundary ends there;
   4. a halted CPU gives `Halt` without fetching;
   5. fetch and execute, which may give `Unsupported`.

   Worked pairs:
   - Zero budget with a stop request, a pending interrupt, a halted CPU or an
     undocumented opcode at PC: `Budget`, with no poll and no fetch. A halt is
     visible in `latches()`.
   - Stop with an eligible interrupt, while halted, or with an undocumented
     opcode at PC: `Stop`, with nothing fetched; the inputs are polled again on
     the next run.
   - Halted with an eligible interrupt: accepted, which leaves halt; the run
     continues.
   - Halted with INT high but interrupts disabled: `Halt`, never a spin
     (decision-2, p. 2-13).
   - Halted with an undocumented opcode at PC: `Halt`; nothing is fetched.
   - An eligible interrupt whose acknowledge returns RST n with an undocumented
     opcode at PC: the RST executes; the opcode at PC is not fetched in this
     boundary.
   - An eligible interrupt whose acknowledge returns another byte, halted or
     not: `Unsupported` with `acknowledge = true` (item 6). If the embedder
     keeps INT high and supplies the same byte, every later run acknowledges
     again and returns the same exit; it must lower INT or change the byte.
6. **Unsupported contents (completes decision-3).** `Unsupported` carries
   `pc`, `opcode` and `acknowledge`. For a fetched opcode, `pc` is the address
   of the instruction's first byte. For an acknowledge, `pc` is the boundary PC
   placed on the address bus and `opcode` is the byte the device returned. The
   exit contributes zero steps and zero cycles; earlier progress in the same
   run is reported as usual. Registers, flags, PC, SP, memory and the execution
   latches are as they were at that boundary: for an acknowledge this is a
   stated departure from p. 5-14, so that the embedder sees the state before
   the attempt. Only the opcode or acknowledge read has reached the bus. With
   the same bus responses, a repeated call returns the same exit. The struct is
   non-exhaustive so that prefixed opcodes of a later model can be added.
   Multi-byte payload and refusal-read wording: decision-14 and
   decision-15 rule 13; this item's 8080 contract is unchanged.
7. **Documented but not yet implemented (extends decision-3 for M1).** During
   M1, a documented opcode whose group has not landed returns `Unsupported`
   like an undocumented one. `status.md` lists the implemented groups.
   `x80-ecy.16` proves the final set and removes this interim path.
8. **`step` and `run`.** `step` is `run` with the budget `{ steps: 1, cycles:
   u64::MAX }`. An accepted interrupt's acknowledged instruction is the one
   instruction of that boundary and counts as one step, so `step` never runs
   two instructions. This settles the step half of decision-2's deferred
   interrupt accounting; the cycles of acceptance are fixed in `x80-ecy.14`
   from the cited tables.
9. **Reset.** `reset` clears PC, the interrupt-enable latch (p. 5-14) and the
   halt latch (p. 2-13). Flags, working registers and SP are unchanged
   (pp. 2-13, 5-14). Its effect on the EI delay is settled in `x80-ecy.13` from a
   cited source or recorded as unverified.
10. **Exit enum.** `Exit` is non-exhaustive. Handoff exits for an integration
    profile belong to M2; none is added now.
11. **Licensed diagnostic run.** M1 requires one if `x80-ecy.3` finds an
    artifact whose licence permits oracle use; otherwise M1 closes with "none
    qualifies" recorded and `x80-ecy.2` at its interim state. The harness is a
    separate unpublished crate (plan §2.1).

## Alternatives

- Interrupt request and acknowledge byte set before `run` (one-shot). Rejected:
  no delivery during a run, and the device cannot observe
  the acknowledge.
- Stop before budget in the precedence. Rejected: a zero budget would then
  make a poll call, and both exits return at once.
- Three access kinds only (decision-1's list plus the acknowledge). Rejected:
  an embedder that maps stack, operand and data accesses differently
  could not tell them apart, and adding kinds later breaks embedders.
- Clearing the interrupt-enable latch before an `Unsupported` acknowledge, as
  the hardware does. Rejected: the embedder would get a state it cannot retry
  from.
- A family-wide `Cpu` and `Registers` at the crate root. Rejected by decision-6.
- `u16` port carrying a guessed upper byte. Rejected: not documented in 98-153B.

## Consequences

- An embedder implements `Bus`, `Control` and, for interrupts, `i8080::Inputs`
  on the same value; the defaults request no stop and no interrupt.
- One pair of input calls per boundary that passes the budget check; with the
  default methods they inline to nothing.
- Bus traces in tests carry the access kind of every read and write, so
  witnesses can check stack, operand and acknowledge accesses against the
  machine-cycle table.

## Revisit when

The M2 contract with x86jit (`xj-50w.7` (x86jit), `x80-pzp`) needs a handoff
exit or a borrowed view, M3 or M4 adds a model, or a cited source contradicts an
item above.
