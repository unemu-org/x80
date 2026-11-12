---
id: decision-4
title: Keep no_std, no allocation and MSRV 1.95
date: '2026-11-11'
status: accepted
---

Decider: maintainer. Settles [plan](../docs/plan.md) §6 Q8; design in §2.5.

## Context

`x80-core` is `#![no_std]`, forbids unsafe code and declares `rust-version =
"1.95"`, inherited from x86jit without a test. The pinned development toolchain
is Rust 1.98.1. An embedder may build x80 and x86jit together with one toolchain.

## Choice

- `x80-core` stays `#![no_std]` and does not use `alloc`.
- The declared MSRV stays 1.95, the same as x86jit, so an embedder of both is
  not forced onto a newer toolchain.
- Rust 1.95 must build and test `x80-core` in the gates before the first public
  API lands (bead `x80-ecy.1`). Until then the MSRV is declared, not verified.
  The [MSRV gate](../docs/commands.md#msrv) now tests it.
- Host-side integration tests may use `std` without adding dependencies
  (unchanged from plan §2.5).

## Alternatives

- MSRV equal to the pinned toolchain (1.98.1). Rejected: stricter than x86jit
  with no feature that needs it.
- Allow `alloc`. Rejected: no demonstrated need; the interpreter state is fixed
  size.

## Consequences

- Code avoids language and library features newer than 1.95.
- A second toolchain enters the gates.

## Revisit when

A needed feature is newer than 1.95, x86jit changes its MSRV, or an embedder
demonstrates a need for allocation.
