# Glossary

Orientation only; verify hardware rules and exact encodings against the pinned
manuals before implementation. No flag bit positions or timing constants are
established here. See [PROVENANCE.md](../../PROVENANCE.md).

| Term | Meaning |
| --- | --- |
| Guest / host | Emulated program / machine running the Rust library |
| Embedder | Consumer supplying memory, ports, devices and scheduling |
| Bus | Borrowed interface for memory and port transactions |
| State / T-state | CPU clock-period timing unit; distinct from the Rust CPU state |
| Machine cycle | A bus/control operation composed of states; an instruction may span several |
| Instruction cycle | The complete execution of an instruction; totals come from manual state tables |
| A / accumulator | Accumulator register |
| BC, DE, HL | Register-pair names used by the programming model |
| PC / SP | Program counter / stack pointer |
| PSW | Program Status Word, the accumulator/flag grouping in the 8080 programming model |
| S | Sign flag |
| Z | Zero flag |
| AC | Auxiliary carry flag |
| P | Parity flag |
| CY | Carry flag; exact update/preservation rules are instruction-specific |
| Reserved flag bits | Bits outside the modeled flag set; images require a manual citation |
| RST | Restart instruction / restart-vector terminology; distinguish it from a reset input |
| IN / OUT | Port I/O instruction terminology |
| INTR / acknowledgement | Interrupt request / the CPU's acceptance and acknowledgement sequence |
| EI / DI | Interrupt-enable/disable instruction terminology; delay rules need witnesses |
| HLT / halt | CPU halt state, distinct from an embedder stop request |
| READY / wait state | External readiness / added bus wait timing; support is unresolved |
| Budget | Explicit instruction-step and T-state limit for a run |
| Exit | Typed reason execution returns control to the embedder |
| Witness / oracle | Independent manual-derived assertion / comparison that can falsify a claim |
| BRKEM / RETEM / CALLN | NEC V20/V30 instructions that enter 8080 emulation (BRKEM), leave it (RETEM) and call a native routine from it (CALLN); see decision-9 and plan §3 |
