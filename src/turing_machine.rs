use crate::{
    program::{Goto, Response, TransitionFn},
    tape::Tape,
};
use std::{
    fmt,
    fmt::Debug,
    io::{BufRead, Write},
    marker::PhantomData,
};

#[derive(Debug)]
pub struct TuringMachine<State, Alphabet, TapeImpl, Program> {
    state: Goto<State>,
    prog: Program,
    tape: TapeImpl,
    phantom: PhantomData<Alphabet>,
}

impl<State, Alphabet, TapeImpl, Program> TuringMachine<State, Alphabet, TapeImpl, Program>
where
    Alphabet: Clone,
    TapeImpl: Tape<Alphabet>,
    Program: TransitionFn<State, Alphabet>,
{
    pub fn new(start: State, prog: Program, input: TapeImpl) -> Self {
        Self {
            state: Goto::Run(start),
            prog,
            tape: input,
            phantom: PhantomData,
        }
    }

    /// May not return - halting problem is hard, yo.
    pub fn run(&mut self) -> bool {
        loop {
            if let Some(accept) = self.step() {
                return accept;
            }
        }
    }

    pub fn run_debug<R, W>(&mut self, mut rdr: R, mut wtr: W) -> Result<bool, std::io::Error>
    where
        R: BufRead,
        W: Write,
        State: Debug,
        Alphabet: Debug,
    {
        loop {
            writeln!(wtr, "{}", self)?;
            rdr.read_line(&mut String::new())?;
            if let Some(accept) = self.step() {
                return Ok(accept);
            }
        }
    }

    /// If TM is in a halt state, returns Some(acceptance)
    /// If TM is in execution, computes one step and returns None
    pub fn step(&mut self) -> Option<bool> {
        match self.state {
            Goto::Halt(accept) => Some(accept),
            Goto::Run(ref state) => {
                let response = (self.prog)(state, self.tape.get());
                self.apply_response(response);
                None
            }
        }
    }

    pub fn get_tape(self) -> impl Iterator<Item = Alphabet> {
        self.tape.get_all()
    }

    fn apply_response(&mut self, response: Response<State, Alphabet>) {
        self.state = response.goto;
        *self.tape.get_mut() = response.write;
        response.mv.map(|mv| self.tape.move_head(mv));
    }
}

impl<State, Alphabet, TapeImpl, Program> fmt::Display
    for TuringMachine<State, Alphabet, TapeImpl, Program>
where
    State: Debug,
    Alphabet: Clone + Debug,
    TapeImpl: Tape<Alphabet>,
    Program: TransitionFn<State, Alphabet>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (head_idx, items) = self.tape.get_radius(8);

        writeln!(f, "================")?;
        for (i, item) in items.enumerate() {
            write!(f, "{:?}", item)?;
            if i == head_idx {
                writeln!(f, "\t\t{:?}", self.state)?;
            } else {
                writeln!(f, "")?;
            }
        }
        writeln!(f, "================")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        program::{Movement, Movement::*, ProgramBuilder},
        tape::Unbounded,
    };

    #[test]
    fn construct() {
        let prog = ProgramBuilder::<bool, u8>::new().build();
        let m = TuringMachine::new(false, prog, Unbounded::new());
        assert_eq!(m.tape.get(), &0);
    }

    // Based around a program that accepts strings with an even number of zeros.
    mod even_zeros {
        use super::*;

        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum State {
            Scan,
            Flip,
            Even,
            Odd,
        }

        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum Alphabet {
            Blank,
            Zero,
            One,
        }

        impl Default for Alphabet {
            fn default() -> Self {
                Alphabet::Blank
            }
        }

        use Alphabet::*;
        use State::*;

        fn get_prog() -> impl Fn(&State, &Alphabet) -> Response<State, Alphabet> {
            ProgramBuilder::new()
                // Scan to the end of the string uselessly.
                .with_transition((Scan, Blank), (Goto::Run(Flip), Blank, Some(Left)))
                .with_transition((Scan, Zero), (Goto::Run(Scan), Zero, Some(Right)))
                .with_transition((Scan, One), (Goto::Run(Scan), One, Some(Right)))
                // Go back to the beginning, flipping each bit.
                .with_transition((Flip, Blank), (Goto::Run(Even), Blank, Some(Right)))
                .with_transition((Flip, Zero), (Goto::Run(Flip), One, Some(Left)))
                .with_transition((Flip, One), (Goto::Run(Flip), Zero, Some(Left)))
                // Count parity of ones.
                .with_transition((Even, Blank), (Goto::Halt(true), Blank, None))
                .with_transition((Even, Zero), (Goto::Run(Even), Zero, Some(Right)))
                .with_transition((Even, One), (Goto::Run(Odd), One, Some(Right)))
                .with_transition((Odd, Blank), (Goto::Halt(false), Blank, None))
                .with_transition((Odd, Zero), (Goto::Run(Odd), Zero, Some(Right)))
                .with_transition((Odd, One), (Goto::Run(Even), One, Some(Right)))
                .build()
        }

        #[test]
        fn accept_empty() {
            let prog = get_prog();
            let mut m = TuringMachine::new(State::Scan, prog, Unbounded::new());
            assert!(m.run());
        }

        #[test]
        fn accept_nontrivial() {
            use Alphabet::*;
            let prog = get_prog();
            let tape = vec![Zero, Zero, One, Zero, One, One, Zero];
            let mut m = TuringMachine::new(State::Scan, prog, Unbounded::from(tape));
            assert!(m.run())
        }

        #[test]
        fn reject_nontrivial() {
            use Alphabet::*;
            let prog = get_prog();
            let tape = vec![Zero, Zero, One, One, One, One, Zero];
            let mut m = TuringMachine::new(State::Scan, prog, Unbounded::from(tape));
            assert!(!m.run())
        }
    }

    // Based around a program that checks if a string has an equal number of ones, twos, and threes.
    mod ones_twos_threes {
        use super::*;
        type State = [usize; 3];
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        enum Alphabet {
            Blank,
            One,
            Two,
            Three,
        }

        impl Default for Alphabet {
            fn default() -> Self {
                Alphabet::Blank
            }
        }

        // This program technically violates the finite nature of turing machines, but if you're being even more
        // technical, it doesn't.
        fn get_prog() -> impl Fn(&State, &Alphabet) -> Response<State, Alphabet> {
            use Alphabet::*;
            |count, read| {
                let mut new_count = count.clone();
                match read {
                    Blank => {
                        return Response {
                            goto: Goto::Halt(count[0] == count[1] && count[1] == count[2]),
                            write: read.clone(),
                            mv: None,
                        }
                    }
                    One => new_count[0] += 1,
                    Two => new_count[1] += 1,
                    Three => new_count[2] += 1,
                }
                Response {
                    goto: Goto::Run(new_count),
                    write: read.clone(),
                    mv: Some(Movement::Right),
                }
            }
        }

        #[test]
        fn accept_empty() {
            let prog = get_prog();
            let mut m = TuringMachine::new([0; 3], prog, Unbounded::new());
            assert!(m.run());
        }

        #[test]
        fn accept_small() {
            use Alphabet::*;

            let prog = get_prog();
            let input = vec![One, Three, Two];
            let mut m = TuringMachine::new([0; 3], prog, Unbounded::from(input));
            assert!(m.run());
        }

        #[test]
        fn accept_large() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let ones = repeat(One).take(300);
            let twos = repeat(Two).take(300);
            let threes = repeat(Three).take(300);
            let input = ones.chain(twos).chain(threes).collect::<Vec<_>>();
            let mut m = TuringMachine::new([0; 3], prog, Unbounded::from(input));
            assert!(m.run());
        }

        #[test]
        fn reject_no_ones() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let twos = repeat(Two).take(300);
            let threes = repeat(Three).take(300);
            let input = twos.chain(threes).collect::<Vec<_>>();
            let mut m = TuringMachine::new([0; 3], prog, Unbounded::from(input));
            assert!(!m.run());
        }

        #[test]
        fn reject_no_twos() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let ones = repeat(One).take(300);
            let threes = repeat(Three).take(300);
            let input = ones.chain(threes).collect::<Vec<_>>();
            let mut m = TuringMachine::new([0; 3], prog, Unbounded::from(input));
            assert!(!m.run());
        }

        #[test]
        fn reject_no_threes() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let twos = repeat(Two).take(300);
            let ones = repeat(One).take(300);
            let input = ones.chain(twos).collect::<Vec<_>>();
            let mut m = TuringMachine::new([0; 3], prog, Unbounded::from(input));
            assert!(!m.run());
        }

        #[test]
        fn reject_too_many_ones() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let ones = repeat(One).take(301);
            let twos = repeat(Two).take(300);
            let threes = repeat(Three).take(300);
            let input = ones.chain(twos).chain(threes).collect::<Vec<_>>();
            let mut m = TuringMachine::new([0; 3], prog, Unbounded::from(input));
            assert!(!m.run());
        }

        #[test]
        fn reject_too_many_twos() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let ones = repeat(One).take(300);
            let twos = repeat(Two).take(301);
            let threes = repeat(Three).take(300);
            let input = ones.chain(twos).chain(threes).collect::<Vec<_>>();
            let mut m = TuringMachine::new([0; 3], prog, Unbounded::from(input));
            assert!(!m.run());
        }

        #[test]
        fn reject_too_many_threes() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let ones = repeat(One).take(300);
            let twos = repeat(Two).take(300);
            let threes = repeat(Three).take(301);
            let input = ones.chain(twos).chain(threes).collect::<Vec<_>>();
            let mut m = TuringMachine::new([0; 3], prog, Unbounded::from(input));
            assert!(!m.run());
        }

        #[test]
        fn reject_no_enough_ones() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let ones = repeat(One).take(299);
            let twos = repeat(Two).take(300);
            let threes = repeat(Three).take(300);
            let input = ones.chain(twos).chain(threes).collect::<Vec<_>>();
            let mut m = TuringMachine::new([0; 3], prog, Unbounded::from(input));
            assert!(!m.run());
        }

        #[test]
        fn reject_no_enough_twos() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let ones = repeat(One).take(300);
            let twos = repeat(Two).take(299);
            let threes = repeat(Three).take(300);
            let input = ones.chain(twos).chain(threes).collect::<Vec<_>>();
            let mut m = TuringMachine::new([0; 3], prog, Unbounded::from(input));
            assert!(!m.run());
        }

        #[test]
        fn reject_no_enough_threes() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let ones = repeat(One).take(300);
            let twos = repeat(Two).take(300);
            let threes = repeat(Three).take(299);
            let input = ones.chain(twos).chain(threes).collect::<Vec<_>>();
            let mut m = TuringMachine::new([0; 3], prog, Unbounded::from(input));
            assert!(!m.run());
        }
    }
}
