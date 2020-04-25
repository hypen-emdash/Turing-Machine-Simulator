use serde::Deserialize;
use smol_str::SmolStr;

use std::{collections::HashMap, io::Read};

use crate::{
    program,
    program::{Goto, ProgramBuilder, TransitionFn},
};

#[derive(Debug, Deserialize)]
enum Movement {
    Stay,
    Left,
    Right,
}

#[derive(Debug, Deserialize)]
struct Program {
    initial: String,
    transitions: HashMap<(String, String), (String, String, Movement)>,
}

pub fn read_program<R>(
    rdr: R,
) -> Result<(SmolStr, impl TransitionFn<SmolStr, SmolStr>), ron::de::Error>
where
    R: Read,
{
    let prog: Program = ron::de::from_reader(rdr)?;
    let initial = SmolStr::from(prog.initial);
    let transitions = prog.transitions;

    let mut prog_builder = ProgramBuilder::new();
    for ((state, read), (goto, write, mv)) in transitions.into_iter() {
        let state = SmolStr::from(state);
        let read = SmolStr::from(read); // TODO: check grapheme count.
        let goto = match goto.as_str() {
            "accept" => Goto::Halt(true),
            "reject" => Goto::Halt(false),
            s => Goto::Run(SmolStr::from(s)),
        };
        let write = SmolStr::from(write);
        let mv = match mv {
            Movement::Stay => None,
            Movement::Left => Some(program::Movement::Left),
            Movement::Right => Some(program::Movement::Right),
        };

        prog_builder.add_transition((state, read), (goto, write, mv));
    }

    Ok((initial, prog_builder.build()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::program::Response;

    #[test]
    fn degenerate_program() {
        let code = r#"Program(
    initial: "main",
    transitions: {}
)"#;
        let (initial, tr_func) =
            read_program(code.as_bytes()).expect("Perfectly valid program with no transitions.");
        assert_eq!(initial.as_str(), "main");
        assert_eq!(
            tr_func(&SmolStr::from("main"), &SmolStr::from("8")),
            Response {
                goto: Goto::Halt(false),
                write: SmolStr::from("8"),
                mv: None
            }
        );
    }

    #[test]
    #[should_panic]
    fn empty_string() {
        read_program("".as_bytes()).expect("Empty program is not valid.");
    }

    #[test]
    #[should_panic]
    fn empty_program() {
        let code = r#"Program()"#;
        read_program(code.as_bytes()).unwrap();
    }

    #[test]
    #[should_panic]
    fn no_initial() {
        let code = r#"Program(transitions: {})"#;
        read_program(code.as_bytes()).unwrap();
    }

    #[test]
    fn valid_program() {
        let code = r#"Program(
    initial: "one",
    transitions: {
        ("one", "0"): ("two", "", Stay),
        ("two", "1"): ("reject", "0", Right),
        ("three", ""): ("accept", "0", Left),
    },
)"#;
        let (init, tr_func) = read_program(code.as_bytes()).expect("Perfectly valid program.");
        assert_eq!(init.as_str(), "one");
        assert_eq!(
            tr_func(&SmolStr::from("one"), &SmolStr::from("0")),
            Response {
                goto: Goto::Run(SmolStr::from("two")),
                write: SmolStr::from(""),
                mv: None,
            }
        );
        assert_eq!(
            tr_func(&SmolStr::from("two"), &SmolStr::from("1")),
            Response {
                goto: Goto::Halt(false),
                write: SmolStr::from("0"),
                mv: Some(program::Movement::Right),
            }
        );
        assert_eq!(
            tr_func(&SmolStr::from("three"), &SmolStr::from("")),
            Response {
                goto: Goto::Halt(true),
                write: SmolStr::from("0"),
                mv: Some(program::Movement::Left),
            }
        );
        assert_eq!(
            tr_func(&SmolStr::from("three"), &SmolStr::from("unexpected")),
            Response {
                goto: Goto::Halt(false),
                write: SmolStr::from("unexpected"),
                mv: None,
            }
        );
    }
}
