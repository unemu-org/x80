//! Guest-agnostic Intel 8080-family interpreter core.
//!
//! The crate root holds the model-neutral contract and carries no hardware
//! fact: the [`Bus`] with its [`MemoryAccess`] kinds, the host stop
//! ([`Control`]), the run [`Budget`], the [`Report`] and the typed [`Exit`]
//! with its [`Unsupported`] payload. Each CPU model is a concrete type in its
//! own module (decision-6); the only model is the Intel 8080, [`i8080::I8080`].
//!
//! Status: [`i8080::I8080`] has `run` and `step` with the decision-2 budget,
//! the decision-5 boundary sequence and the documented 8080 opcode map. Only
//! NOP executes; every other opcode returns [`Exit::Unsupported`]. Interrupts
//! are polled but not accepted yet, and there is no `reset`.
//!
//! Memory, devices, interrupt sources, file formats and scheduling belong to
//! the embedder. The crate is `no_std`, allocates nothing and has no
//! dependencies (decision-4). Design: `backlog/docs/plan.md` §2; decisions:
//! `backlog/decisions/`.

#![no_std]
#![forbid(unsafe_code)]

mod budget;
mod bus;
mod control;
mod exit;
pub mod i8080;

pub use budget::{Budget, Report};
pub use bus::{Bus, MemoryAccess};
pub use control::Control;
pub use exit::{Exit, Unsupported};
