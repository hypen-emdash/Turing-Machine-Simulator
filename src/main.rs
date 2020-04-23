use smol_str::SmolStr;
use std::{io, io::Read, str::FromStr};
use unicode_segmentation::UnicodeSegmentation;

pub mod parse;
pub mod program;
pub mod turing_machine;

use program::Program;
use turing_machine::TuringMachine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let prog = Program::from_str(
        "
read - degenerate case
    ([read], \"\") -> (v, \"\", 0)
    ([read], \"#\") -> ([end], \"#\", +)

read - good cases
    ([read], \"0\")->([search 0 left], \"x\", +)
    ([read], \"1\")->([search 1 left], \"x\", +)

search 0 left
    ([search 0 left], \"0\") -> ([search 0 left], \"0\", +)
    ([search 0 left], \"1\") -> ([search 0 left], \"1\", +)
    ([search 0 left], \"#\") -> ([search 0 right], \"#\", +)
    ([search 0 left], \"\") -> (v, \"\", 0)

search 1 left
    ([search 1 left], \"0\") -> ([search 1 left], \"0\", +)
    ([search 1 left], \"1\") -> ([search 1 left], \"1\", +)
    ([search 1 left], \"#\") -> ([search 1 right], \"#\", +)
    ([search 1 left], \"\") -> (v, \"\", 0)

search 0 right
    ([search 0 right], \"0\") -> ([back up right], \"x\", -)
    ([search 0 right], \"x\") -> ([search 0 right], \"x\", +)
    ([search 0 right], \"1\") -> (v, \"1\", 0)
    ([search 0 right], \"#\") -> (v, \"#\", 0)
    ([search 0 right], \"\") -> (v, \"\", 0)

search 1 right
    ([search 1 right], \"1\") -> ([back up right], \"x\", -)
    ([search 1 right], \"x\") -> ([search 1 right], \"x\", +)
    ([search 1 right], \"0\") -> (v, \"0\", 0)
    ([search 1 right], \"#\") -> (v, \"#\", 0)
    ([search 1 right], \"\") -> (v, \"\", 0)

back up
    ([back up right], \"x\") -> ([back up right], \"x\", -)
    ([back up right], \"#\") -> ([back up left], \"#\", -)
    ([back up left], \"0\") -> ([back up left], \"0\", -)
    ([back up left], \"1\") -> ([back up left], \"1\", -)
    ([back up left], \"x\") -> ([read], \"x\", +)

check at the end
    ([end], \"x\") -> ([end], \"x\", +)
    ([end], \"\") -> (^, \"\", 0)
    ([end], \"0\") -> (v, \"0\", 0)
    ([end], \"1\") -> (v, \"1\", 0)

",
    )
    .unwrap();

    let mut buffer = Vec::<u8>::new();
    io::stdin().read_to_end(&mut buffer)?;
    let input = String::from_utf8(buffer)?;
    let tape = UnicodeSegmentation::graphemes(input.as_str(), true).map(|s| SmolStr::from(s));

    let mut m = TuringMachine::new(prog, tape);

    let status = m.run();
    if let Some(accept) = status {
        for c in m.get_tape() {
            print!("{}", c);
        }
        std::process::exit(!accept as i32);
    };
    Ok(())
}
