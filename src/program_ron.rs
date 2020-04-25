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

