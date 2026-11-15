//! ASSUMPTIONS, never NEC hardware facts. Shared by every fixture operation.
//!
//! PROVENANCE.md §1, "NEC V20/V30 8080 emulation mode", to-confirm list:
//! IE and BRK after BRKEM and CALLN, CALLN's saved PC, the emulation EI delay
//! and CALLN's effect on it, the offset wrap inside the segment and the 8080
//! IN/OUT high address byte.

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
/// ASSUMPTION (decision-9 amendment, 2026-11-15): no EI delay survives a
/// CALLN excursion, and CALLN itself consumes it. x80 leaves `ei_delay` set
/// on the ED refusal, because only a completed instruction clears it. No NEC
/// page states this; x86jit's C4a makes the same choice in its CALLN unit and
/// witnesses it right after the CALLN step (`xj-50w.7` (x86jit)). On a native core the routine's own RETI would also
/// end the delay. The mock routine runs no native instruction, so here this
/// constant alone clears it.
pub const CALLN_CONSUMES_EI_DELAY: bool = true;
/// ASSUMPTION: zero 8080 IN/OUT high address byte. x80 passes only the 8-bit
/// port, so the adapter alone supplies it; PROVENANCE §1.
pub const IO_HIGH_BYTE: u8 = 0;
/// ASSUMPTION: the offset wraps inside the segment, FFFFh to 0000h. NEC
/// p. 10-1 gives a 16-bit offset added to segment x 16 but does not state
/// the wrap; x80's bus offset is `u16`, so it cannot express a carry into
/// the next segment. PROVENANCE §1.
pub const WRAP_OFFSETS: bool = true;

// Fixture policies for other open joint-contract issues (plan §3):
// - INT and NMI service keep the native model's EI delay across the
//   excursion. This is open (PROVENANCE §1, decision-9 amendment 2026-11-15):
//   an INT is accepted only after the delay, so only an NMI right after an
//   8080 EI reaches it, and on a native core the routine's instructions would
//   end the delay. Format 2 cannot express the case: its routines are a fixed
//   mock payload, and it has no record for the EI delay.
// - RETEM and RESET drop the 8080 latches, since no emulation state remains.
// - Events are explicitly ordered, not a proposed shared clock or
//   cancellation API.
// - RESET retains undefined register/flag values in this mock only (NEC
//   p. 7-1).
