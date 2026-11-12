//! Input contract (decision-12 rules 1, 2 and 8).

use crate::Control;

/// The embedder's 8085 interrupt and serial lines, beside the host stop.
///
/// Line types: Intel 205775-002 (January 1983) §2.2.7 p. 2-5;
/// 231718-001 (September 1987), handbook p. 16-13 Table 2.
/// Defaults request no interrupt and return SID low; SOD notification is a no-op.
///
/// The execution contract polls `stop_requested`, `trap`, `rst7_5`, `rst6_5`,
/// `rst5_5`, `intr` once each, in that order, at every boundary that passes
/// the budget check, regardless of results or CPU state (rule 1). Polls should
/// read levels without changing the emulated device. Consecutive samples detect
/// TRAP/RST 7.5 edges; shorter pulses can be missed, so hold them until polled
/// (rule 2, an API choice). Sampling is not implemented yet.
///
/// RIM calls `sid`, `rst6_5`, `rst5_5` once each in that order inside the
/// instruction (rule 8; 205775-002 §2.3.8 p. 2-19 and RIM p. 5-17).
pub trait Inputs: Control {
    /// TRAP level: rising edge and sustained high (205775-002 p. 2-5).
    fn trap(&mut self) -> bool {
        false
    }

    /// RST 7.5 level: rising edge latched internally (same table).
    fn rst7_5(&mut self) -> bool {
        false
    }

    /// RST 6.5 level: high requests service (same table).
    fn rst6_5(&mut self) -> bool {
        false
    }

    /// RST 5.5 level: high requests service (same table).
    fn rst5_5(&mut self) -> bool {
        false
    }

    /// INTR level: high requests service (same table).
    fn intr(&mut self) -> bool {
        false
    }

    /// SID level, read only by RIM (205775-002 p. 5-17).
    fn sid(&mut self) -> bool {
        false
    }

    /// SIM with bit 6 set loads SOD from bit 7 (205775-002 p. 5-18).
    /// The execution contract notifies once per enabled write, even if the
    /// level is unchanged, in program order during SIM (rule 8). This timing
    /// simplifies p. 2-19's transfer during the instruction following SIM.
    /// Reset has no input parameter and makes no notification (rule 9).
    fn sod_changed(&mut self, _level: bool) {}
}
