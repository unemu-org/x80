<!-- The release commit replaces "Unreleased" in the version heading with the release date (YYYY-MM-DD). -->

# Changelog

Notable changes to `x80-core`, in the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

## [0.1.0] - 2026-11-16

### Added

- Three CPU models: `i8080::I8080` (Intel 8080, all 244 documented opcodes), `i8085::I8085`
  (Intel 8085, all 256 opcodes, including RIM/SIM and the ten undocumented instructions) and
  `z80::Z80` (NMOS Zilog Z80, every opcode, undocumented ones included).
- Interrupts: RST on the 8080; TRAP, RST 7.5/6.5/5.5 and INTR on the 8085; NMI and interrupt
  modes 0, 1 and 2 on the Z80.
- `step` runs at most one instruction; `run` stops when its `Budget` is spent, the host asks it
  to stop, the CPU halts or it meets an unsupported byte, and its `Report` says which. Your
  program implements `Bus` (memory and I/O), `Control` (stop requests) and each model's `Inputs`.
- The NEC V20/V30 8080 mode runs as the `i8080` model under an integration profile
  ([decision-9](https://github.com/unemu-org/x80/blob/main/backlog/decisions/decision-9%20-%20NEC-V20-V30-8080-emulation-mode-is-the-i8080-model-with-a-V20-profile.md));
  the x86 side belongs to your program.
- No dependencies, `no_std`, no heap allocation and `forbid(unsafe_code)`; Rust 1.95 or newer.

### Known limits

- `Unsupported` for the 8080's twelve undocumented opcodes and some interrupt responses: a
  non-RST byte on the 8080, anything but RST or CALL on the 8085, a non-RST instruction in Z80
  interrupt mode 0 and an odd vector in Z80 interrupt mode 2.
- Cycle counts are instruction T-state totals, with no external wait states or bus-cycle
  timing; under the V20 profile they are Intel 8080 timings, not V20 clock counts.
