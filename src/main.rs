#[warn(rust_2018_idioms)]
pub mod program;
pub mod program_ron;
pub mod tape;
pub mod turing_machine;

use std::{fmt, fs::File, io, io::Read, path::PathBuf};

use smol_str::SmolStr;
use structopt::StructOpt;
use unicode_segmentation::UnicodeSegmentation;

use program::{Goto, Movement, Response, TransitionFn};
use tape::Unbounded;
use turing_machine::TuringMachine;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    #[structopt(short, long)]
    debug: bool,
}

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Ron(ron::de::Error),
    StrRead(std::string::FromUtf8Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "{}", err),
            Error::Ron(err) => write!(f, "{}", err),
            Error::StrRead(err) => write!(f, "{}", err.utf8_error()),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<ron::de::Error> for Error {
    fn from(err: ron::de::Error) -> Self {
        Error::Ron(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error::StrRead(err)
    }
}

impl std::error::Error for Error {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    Ok(run(opt)?)
}

fn run(opt: Opt) -> Result<(), Error> {
    let (init, tr_func) = program_ron::read_program(File::open(opt.file)?)?;

    let mut input_buf = Vec::new();
    io::stdin().read_to_end(&mut input_buf)?;
    let input = String::from_utf8(input_buf)?;
    let graphemes = UnicodeSegmentation::graphemes(input.as_str(), true);
    let tape: Unbounded<SmolStr> = graphemes.map(|g| SmolStr::from(g)).collect();

    let mut machine = TuringMachine::new(init, tr_func, tape);

    let accept = if opt.debug {
        machine.run_debug()?
    } else {
        machine.run()
    };
    let output = machine.get_tape();
    for item in output {
        print!("{}", item);
    }
    println!("{}", accept);
    Ok(())
}

#[allow(dead_code)]
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
