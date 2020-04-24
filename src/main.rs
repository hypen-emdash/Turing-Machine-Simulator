pub mod program;
pub mod tape;
pub mod turing_machine;

use smol_str::SmolStr;

use program::{Goto, Movement, Response, TransitionFn};
use tape::Unbounded;
use turing_machine::TuringMachine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let program = printer();
    let mut machine =
        TuringMachine::new(SmolStr::from("Hello, world!\n"), program, Unbounded::new());

    let accept = machine.run();
    let output = machine.get_tape();
    for item in output {
        print!("{}", item);
    }
    eprintln!("{}", accept);
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
