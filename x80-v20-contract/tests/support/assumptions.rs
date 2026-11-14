//! ASSUMPTIONS, never NEC hardware facts. Shared by every fixture operation.
//!
//! PROVENANCE.md §1, "NEC V20/V30 8080 emulation mode", to-confirm list:
//! IE and BRK after BRKEM and CALLN, CALLN's saved PC, emulation EI delay and
//! IN/OUT high byte. Offset wrap is also unverified in PROVENANCE.md §3 and
//! plan §3's joint contract questions (it is not in §1's to-confirm list).

/// ASSUMPTION, ruling R2 (decision-9 amendment, 2026-11-14): BRKEM and CALLN
/// both clear IE and BRK, as the general interrupt entry on NEC p. 6-1 does.
/// Their own pages (12-169, 12-176) list only MD, and a blank flag entry
/// means "No change" (Table 12-4, p. 12-3). INT and NMI entry clearing IE
/// and BRK is read (pp. 6-1, 8-4), so this constant does not govern it.
pub const BRKEM_CALLN_CLEAR_IE_BRK: bool = true;
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
