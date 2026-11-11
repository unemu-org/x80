---
id: decision-2
title: Run budget stops at instruction boundaries with overshoot
date: '2026-11-11'
status: accepted
---

Decider: maintainer. Settles [plan](../docs/plan.md) §6 Q3; design in §2.4.

## Context

`run` repeats `step` under a budget so an embedder can interleave the CPU with
devices and other cores. The cost of the next instruction may not be known
before it starts: plan §2.4 expects separate taken and untaken cycle totals, to
be confirmed from the manual once pinned. decision-23 (x86jit) keeps steps as
completed instructions and REP iterations and accepts opt-in cycles beside
them, not yet implemented in x86jit (Timing model, Run API).

## Choice

- A budget carries an instruction-step limit and a cycle (T-state) limit. A
  step is one instruction executed by `step` (plan §2.4). Joint step and cycle
  accounting with x86jit is settled in the M2 contract (`x80-pzp`).
- A zero budget executes nothing and returns at once.
- Before each instruction, `run` returns a budget exit if either limit is
  reached. An instruction that has started always completes.
- The report carries the actual steps and cycles. Actual cycles may exceed the
  cycle limit (overshoot).
- A halted CPU makes `run` return at once with a halt exit; it never spins.
- Cycles spent accepting an interrupt count against the cycle limit. Exact step
  and cycle accounting for interrupt acceptance is fixed with the interrupt
  implementation, from the manual once pinned.
- Step and cycle counters are `u64` and saturate.
- Exit precedence when several conditions hold at once (for example a halted
  CPU and a zero budget) is not decided here; the API bead proposes it for
  maintainer approval.

## Alternatives

- Strict limit: never start an instruction that could exceed the cycle limit.
  Rejected: it needs a preflight of conditional timings, and later bus wait
  states would make the preflight unreliable.
- Cycle-only budget. Rejected: x86jit and the M2 handoff count steps.

## Consequences

- A scheduler carries the overshoot forward as debt for the next slice.
- Identical inputs give identical step and cycle reports.

## Revisit when

An embedder needs exact cycle deadlines, or the joint V20/V30 accounting with
x86jit (`xj-50w.7` (x86jit), bead `x80-pzp`) requires a change.
