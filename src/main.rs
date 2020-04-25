pub mod program;
pub mod program_ron;
pub mod tape;
pub mod turing_machine;

use std::{io, io::Read};

use smol_str::SmolStr;
use unicode_segmentation::UnicodeSegmentation;

use program::{Goto, Movement, Response, TransitionFn};
use tape::Unbounded;
use turing_machine::TuringMachine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = printer();

    let mut input_buf = Vec::new();
    io::stdin().read_to_end(&mut input_buf)?;
    let input = String::from_utf8(input_buf)?;
    let graphemes = UnicodeSegmentation::graphemes(input.as_str(), true);
    let tape: Unbounded<SmolStr> = graphemes.map(|g| SmolStr::from(g)).collect();

    let mut machine = TuringMachine::new(SmolStr::from("Hello, world!\n"), program, tape);

    let accept = machine.run();
    let output = machine.get_tape();
    for item in output {
        print!("{}", item);
    }
    println!("{}", accept);
    Ok(())
}

fn printer() -> impl TransitionFn<SmolStr, SmolStr> {
    |str_to_print: &SmolStr, current_symbol: &SmolStr| {
        let mut chars = str_to_print.chars();
        let char_to_print = chars.next();
        let remainder = chars.collect::<SmolStr>();
        match char_to_print {
            None => Response {
                goto: Goto::Halt(true),
                write: current_symbol.clone(),
                mv: None,
            },
            Some(c) => Response {
                goto: Goto::Run(remainder),
                write: SmolStr::from(format!("{}", c)),
                mv: Some(Movement::Right),
            },
        }
    }
}
