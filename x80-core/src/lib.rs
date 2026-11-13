//! Guest-agnostic Intel 8080-family interpreter core.
//!
//! The crate root holds the model-neutral contract and carries no hardware
//! fact: the [`Bus`] with its [`MemoryAccess`] kinds, the host stop
//! ([`Control`]), the run [`Budget`], the [`Report`] and the typed [`Exit`]
//! with its [`Unsupported`] payload. Each CPU model is a concrete type in its
//! own module (decisions 6/13): [`i8080::I8080`], [`i8085::I8085`] and
//! [`z80::Z80`].
//!
//! Model-specific execution, timing, interrupt and reset behavior is
//! documented in the model module.
//!
//! Memory, devices, interrupt sources, file formats and scheduling belong to
//! the embedder. The crate is `no_std`, allocates nothing and has no
//! dependencies (decision-4). Design: `backlog/docs/plan.md` §2; decisions:
//! `backlog/decisions/`.
//!
//! Model types are available through their model module:
//!
//! ```
//! use x80_core::i8080::{I8080, Registers, Latches, Flags, Inputs};
//! ```
//!
//! ```
//! use x80_core::i8085::{I8085, Registers, Latches, Flags, Inputs};
//! ```
//!
//! ```
//! use x80_core::z80::{Z80, Registers, Latches, Flags, Inputs, InterruptMode};
//! ```
//!
//! The crate root keeps the model-neutral boundary (decision-6):
//!
//! ```compile_fail,E0432
//! use x80_core::I8080;
//! ```
//!
//! ```compile_fail,E0432
//! use x80_core::I8085;
//! ```
//!
//! ```compile_fail,E0432
//! use x80_core::Z80;
//! ```
//!
//! ```compile_fail,E0432
//! use x80_core::InterruptMode;
//! ```
//!
//! ```compile_fail,E0432
//! use x80_core::Registers;
//! ```
//!
//! ```compile_fail,E0432
//! use x80_core::Latches;
//! ```
//!
//! ```compile_fail,E0432
//! use x80_core::Flags;
//! ```
//!
//! ```compile_fail,E0432
//! use x80_core::Inputs;
//! ```

// These doctests guard accidental coupling (decision-6), not adversarial code.
// A re-export under a different name is not covered.

#![no_std]
#![forbid(unsafe_code)]

mod budget;
mod bus;
mod control;
mod decode;
mod exit;
pub mod i8080;
pub mod i8085;
mod progress;
pub mod z80;

pub use budget::{Budget, Report};
pub use bus::{Bus, MemoryAccess};
pub use control::Control;
pub use exit::{Exit, Unsupported};
