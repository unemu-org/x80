//! Guest-agnostic Intel 8080-family interpreter core: module skeleton only.
//!
//! Planned contracts live in `backlog/docs/plan.md` §§2–4. Memory, devices,
//! file formats and scheduling belong to the embedder. No CPU executes yet.

#![no_std]
#![forbid(unsafe_code)]

pub mod bus;
pub mod cpu;
pub mod exit;
pub mod interrupt;
pub mod state;
