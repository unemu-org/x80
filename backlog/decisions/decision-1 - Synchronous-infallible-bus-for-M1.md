---
id: decision-1
title: Synchronous infallible bus for M1
date: '2026-11-11'
status: accepted
---

Decider: maintainer. Settles [plan](../docs/plan.md) §6 Q2; design in §2.3.
decision-5 fixes the signatures and extends the access kinds; the rest stands.

## Context

The core needs one way to reach embedder memory and port devices. M1 targets
documented 8080 instruction behaviour with instruction-level cycle totals, not
bus waveforms. Embedders range from CP/M environments to arcade boards and, in
M2, a NEC V20/V30 host that lends a DS-relative memory view (plan §3,
unverified).

## Choice

- The embedder implements a borrowed, synchronous `Bus` trait. The core calls it
  during an instruction and never takes ownership of it.
- The trait covers byte memory reads and writes and byte port input and output.
  Each access carries a kind that distinguishes an opcode fetch, another memory
  access and a port access. The API bead fixes the exact signatures.
- Accesses are infallible: they return a value, never an error. M1 has no bus
  faults, no wait states and no bus-cycle stepping.
- Timing is counted at instruction boundaries from the manual's state tables,
  once that source is pinned (plan §2.4, PROVENANCE.md). Bus accesses do not
  report cycles.
- Mapping, segmentation, mirroring and devices belong to the embedder. The core
  allocates nothing and keeps no global memory.

## Alternatives

- Fallible accesses returning a typed failure and a bus-failure exit. Rejected
  for M1: it needs a contract for partly executed instructions and device side
  effects before any instruction exists (plan §2.3).
- Port and memory accesses returned as exits for the embedder to replay.
  Rejected: replaying device side effects is harder for every embedder than a
  direct call.

## Consequences

- To stop on a device access, an embedder can record it in its bus and use the
  stop request planned in plan §2.4; the core then returns at the next
  instruction boundary.
- Restartable bus faults are not available.
- Adding failures or wait states later changes the trait and needs a new,
  superseding decision.

## Revisit when

An embedder needs bus faults, wait states, READY behaviour or bus-cycle
stepping, or the M2 contract with x86jit (`xj-50w.7` (x86jit)) requires a change.
