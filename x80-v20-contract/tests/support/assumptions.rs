//! ASSUMPTIONS, never NEC hardware facts. Shared by every fixture operation.
//!
//! PROVENANCE.md §1, "NEC V20/V30 8080 emulation mode", to-confirm list:
//! IE after BRKEM, CALLN's saved PC, emulation EI delay and IN/OUT high byte.
//! BRK after BRKEM has the same NEC pp. 6-1 / 12-169 conflict as IE.
//! Offset wrap is also unverified in PROVENANCE.md §3 and plan §3's joint
//! contract questions (it is not in §1's to-confirm list).

/// ASSUMPTION: BRKEM preserves IE; pp. 6-1 and 12-169 do not settle this.
pub const BRKEM_PRESERVES_IE: bool = true;
/// ASSUMPTION: BRKEM preserves BRK; NEC p. 6-1 clears BRK in the general
/// interrupt sequence, but p. 12-169 lists only MD for BRKEM.
pub const BRKEM_PRESERVES_BRK: bool = true;
/// ASSUMPTION: CALLN saves the address after ED ED nn (p. 12-176 unclear).
pub const CALLN_PC_ADVANCE: u16 = 3;
/// ASSUMPTION: honor x80's one-following-instruction EI delay in emulation.
/// PROVENANCE §1: native EI delay applicability remains to confirm.
pub const EMULATION_EI_DELAY: bool = true;
/// ASSUMPTION: zero high address byte for 8080 IN/OUT; PROVENANCE §1.
pub const IO_HIGH_BYTE: u8 = 0;
/// ASSUMPTION: FFFFh wraps to 0000h inside the same segment; see above.
pub const WRAP_OFFSETS: bool = true;

// Fixture policies for other open joint-contract issues (plan §3): preserve
// EI-delay across a synchronous native excursion; clear halt on wake. Events
// are explicitly ordered, not a proposed shared clock or cancellation API.
// RESET retains undefined register/flag values in this mock only (NEC p. 7-1).
