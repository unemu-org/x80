//! Strict, dependency-free reader for the language-neutral format in README.md.
use super::adapter::{Adapter, Lines, Outcome};
use super::native::{Frame, Memory, Native};
use x80_core::MemoryAccess;

const TEXT: &str = include_str!("../../vectors/mode-switch.txt");

pub fn hex(s: &str) -> u16 {
    u16::from_str_radix(s, 16).expect("hexadecimal word")
}

fn words(tokens: &[&str]) -> Vec<u16> {
    tokens.iter().map(|s| hex(s)).collect()
}

fn state(tokens: &[&str]) -> Native {
    Native::from_words(&words(tokens))
}

fn outcome(token: &str) -> Outcome {
    match token {
        "budget" => Outcome::Budget,
        "halt" => Outcome::Halt,
        "retem" => Outcome::Retem,
        "serviced" => Outcome::Serviced,
        "wake" => Outcome::Wake,
        "reset" => Outcome::Reset,
        _ => panic!("unknown outcome {token}"),
    }
}

/// Parse initial memory independently of the mock's address/word helpers.
fn patch(bytes: &mut [u8], tokens: &[&str]) {
    let segment = usize::from(hex(tokens[0]));
    let offset = usize::from(hex(tokens[1]));
    for (i, token) in tokens[2..].iter().enumerate() {
        let physical = (segment * 16 + ((offset + i) % 65536)) % 1048576;
        bytes[physical] = u8::try_from(hex(token)).expect("byte");
    }
}

fn configuration(name: &str) -> (Native, Memory, u8) {
    let (common, section) = section(name);
    let mut memory = Memory::new();
    let mut initial = None;
    let mut vector = None;
    for line in common.into_iter().chain(section) {
        match line.as_slice() {
            ["format", "x80-v20-contract-1"] => {}
            ["initial", w @ ..] => {
                assert!(initial.replace(state(w)).is_none());
            }
            ["brkem", n] => {
                assert!(vector.replace(u8::try_from(hex(n)).unwrap()).is_none());
            }
            ["memory", data @ ..] => patch(&mut memory.bytes, data),
            _ => {}
        }
    }
    (initial.unwrap(), memory, vector.unwrap())
}

pub fn fixture(name: &str) -> Adapter {
    let (initial, memory, vector) = configuration(name);
    Adapter::enter(initial, memory, vector)
}

type Tokens<'a> = Vec<Vec<&'a str>>;

fn section(name: &str) -> (Tokens<'static>, Tokens<'static>) {
    let mut common = Vec::new();
    let mut selected = Vec::new();
    let mut current = None;
    let mut found = false;
    for line in TEXT.lines() {
        let tokens: Vec<_> = line.split('#').next().unwrap().split_whitespace().collect();
        match tokens.as_slice() {
            [] => {}
            ["case", id] => {
                current = Some(*id);
                found |= *id == name;
            }
            ["end"] => {
                assert!(current.is_some());
                current = Some("");
            }
            _ if current.is_none() => common.push(tokens),
            _ if current == Some(name) => selected.push(tokens),
            _ => {}
        }
    }
    assert!(found, "missing vector {name}");
    (common, selected)
}

pub fn run_vector(name: &str) {
    let (common, section) = section(name);
    let (initial, memory, vector) = configuration(name);
    let mut expected_memory = memory.bytes.clone();
    let mut adapter = Adapter::enter(initial, memory, vector);
    let mut frames = Vec::new();
    let mut services = Vec::new();
    let mut routine_psws = Vec::new();
    let mut final_seen = false;
    let mut access_seen = false;
    for tokens in common.into_iter().chain(section) {
        match tokens.as_slice() {
            ["format", "x80-v20-contract-1"] | ["initial", ..] | ["brkem", _] | ["memory", ..] => {}
            ["run", n] => assert_eq!(
                adapter.run(u64::from(hex(n)), Lines::default()),
                Ok(Outcome::Budget)
            ),
            ["until", expected] => adapter.until(outcome(expected)),
            ["emulation", psw] => {
                // NEC p. 12-168: witness RETI's restored PSW before RETEM
                // can replace it with the original native flags.
                let expected = hex(psw);
                assert!(!adapter.ended);
                assert_eq!(adapter.native.psw, expected);
                assert_eq!(adapter.cpu.registers().psw() & 0xFF, expected & 0xFF);
            }
            ["signal", int, nmi, hold, reset, vector, expected] => {
                let bit = |s| match s {
                    "0" => false,
                    "1" => true,
                    _ => panic!("line must be 0 or 1"),
                };
                let lines = Lines {
                    int: bit(int),
                    nmi: bit(nmi),
                    hold: bit(hold),
                    reset: bit(reset),
                    vector: u8::try_from(hex(vector)).unwrap(),
                };
                assert_eq!(adapter.run(1, lines), Ok(outcome(expected)));
            }
            ["final", w @ ..] => {
                assert!(!final_seen);
                final_seen = true;
                assert_eq!(adapter.native, state(w));
                assert!(adapter.ended);
            }
            ["changed", data @ ..] => patch(&mut expected_memory, data),
            ["frame", source, ss, sp, pc, ps, psw] => {
                let frame = Frame {
                    source: (*source).into(),
                    ss: hex(ss),
                    sp: hex(sp),
                    words: [hex(pc), hex(ps), hex(psw)],
                };
                // Literal expected words become bytes in increasing addresses,
                // independently of push_frame/put_word/physical in native.rs.
                let base = usize::from(frame.ss) * 16;
                for (i, word) in frame.words.iter().enumerate() {
                    for (j, value) in [(*word & 255) as u8, (*word / 256) as u8]
                        .into_iter()
                        .enumerate()
                    {
                        let address =
                            (base + ((usize::from(frame.sp) + i * 2 + j) % 65536)) % 1048576;
                        expected_memory[address] = value;
                    }
                }
                frames.push(frame);
            }
            ["service", w @ ..] => services.push(state(w)),
            ["routine", psw] => routine_psws.push(hex(psw)),
            ["access", ps, ds0, counts @ ..] => {
                assert!(!access_seen);
                access_seen = true;
                let mut actual = [0u16; 6];
                for a in &adapter.accesses {
                    let index = match (a.kind, a.write) {
                        (MemoryAccess::OpcodeFetch, false) => 0,
                        (MemoryAccess::OperandFetch, false) => 1,
                        (MemoryAccess::Data, false) => 2,
                        (MemoryAccess::Data, true) => 3,
                        (MemoryAccess::Stack, false) => 4,
                        (MemoryAccess::Stack, true) => 5,
                        _ => panic!("unexpected access {a:?}"),
                    };
                    assert_eq!(a.segment, hex(if index < 2 { ps } else { ds0 }));
                    actual[index] += 1;
                }
                assert_eq!(actual.as_slice(), words(counts));
            }
            _ => panic!("unknown record: {tokens:?}"),
        }
    }
    assert!(final_seen && access_seen);
    assert_eq!(adapter.memory.frames, frames);
    assert_eq!(adapter.services, services);
    assert_eq!(adapter.routine_psws, routine_psws);
    assert!(adapter.ports.is_empty());
    // Entire flat memory, including poison in untouched segments, is checked.
    // Avoid printing two MiB of bytes on a failure.
    assert_eq!(
        adapter
            .memory
            .bytes
            .iter()
            .zip(&expected_memory)
            .position(|(a, b)| a != b),
        None,
        "first differing physical memory address"
    );
}
