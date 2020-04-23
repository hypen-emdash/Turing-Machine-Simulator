use crate::program::{Goto, Movement, Response};
use std::{
    collections::VecDeque,
    default::Default,
    fmt,
    fmt::{Debug, Display},
    hash::Hash,
    iter::FromIterator,
};

#[derive(Debug)]
pub struct TuringMachine<State, Alphabet, Program>
where
    State: Eq + Hash + Clone + Debug,
    Alphabet: Eq + Hash + Default + Clone + Debug,
    Program: Fn(&State, &Alphabet) -> Response<State, Alphabet>,
{
    state: Goto<State>,
    prog: Program,
    tape: VecDeque<Alphabet>,
    idx: usize, // location on the tape.
}

impl<State, Alphabet, Program> TuringMachine<State, Alphabet, Program>
where
    State: Eq + Hash + Clone + Debug,
    Alphabet: Eq + Hash + Default + Clone + Debug,
    Program: Fn(&State, &Alphabet) -> Response<State, Alphabet>,
{
    pub fn new(start: State, prog: Program, input: impl Iterator<Item = Alphabet>) -> Self {
        let mut tape = VecDeque::from_iter(input);
        if tape.is_empty() {
            tape.push_back(Default::default());
        }
        Self {
            state: Goto::Run(start),
            prog,
            tape,
            idx: 0,
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

    /// If TM is in a halt state, returns Some(acceptance)
    /// If TM is in execution, computes one step and returns None
    pub fn step(&mut self) -> Option<bool> {
        match self.state {
            Goto::Halt(accept) => Some(accept),
            Goto::Run(ref state) => {
                let response = (self.prog)(state, &self.read_symbol());
                self.apply_response(response);
                None
            }
        }
    }

    fn apply_response(&mut self, response: Response<State, Alphabet>) {
        self.state = response.goto;
        self.write_symbol(response.write);
        response.mv.map(|mv| self.move_head(mv));
    }

    fn read_symbol(&self) -> Alphabet {
        self.tape[self.idx].clone()
    }

    fn write_symbol(&mut self, sym: Alphabet) {
        self.tape[self.idx] = sym;
    }

    fn move_head(&mut self, direction: Movement) {
        match direction {
            Movement::Left => self.move_left(),
            Movement::Right => self.move_right(),
        }
    }

    fn move_left(&mut self) {
        if self.idx == 0 {
            self.tape.push_front(Default::default());
        } else {
            self.idx -= 1;
        }
    }

    fn move_right(&mut self) {
        self.idx += 1;
        if self.tape.get(self.idx).is_none() {
            self.tape.push_back(Default::default());
        }
    }

    pub fn observe<'a>(&'a self, radius: usize) -> impl Iterator<Item = Alphabet> + 'a {
        let start = self.idx as isize - radius as isize;
        let end = self.idx as isize + radius as isize;

        (start..=end).map(move |i| self.simulate_read(i))
    }

    pub fn get_tape<'a>(&'a self) -> impl Iterator<Item = Alphabet> + 'a {
        self.tape.iter().cloned()
    }

    fn simulate_read(&self, idx: isize) -> Alphabet {
        if idx < 0 {
            Default::default()
        } else {
            match self.tape.get(idx as usize) {
                None => Default::default(),
                Some(a) => a.clone(),
            }
        }
    }
}

impl<State, Alphabet, Program> fmt::Display for TuringMachine<State, Alphabet, Program>
where
    State: Eq + Hash + Clone + Debug + Display,
    Alphabet: Eq + Hash + Default + Clone + Debug + Display,
    Program: Fn(&State, &Alphabet) -> Response<State, Alphabet>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "================")?;
        for (i, symbol) in self.observe(9).enumerate() {
            write!(f, "{}", symbol)?;
            if i == 9 {
                write!(f, "\t{:?}", self.state)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "================")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::program::{ProgramBuilder};

    #[test]
    fn construct() {
        let prog = ProgramBuilder::<bool, u8>::new().build();
        let m = TuringMachine::new(false, prog, std::iter::empty());
        assert_eq!(m.read_symbol(), 0);
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

        fn get_prog() -> impl Fn(&State, &Alphabet) -> Response<State, Alphabet> {
            use Alphabet::*;
            use State::*;
            use Movement::*;
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
            let mut m = TuringMachine::new(State::Scan, prog, std::iter::empty());
            assert!(m.run());
        }

        #[test]
        fn accept_nontrivial() {
            use Alphabet::*;
            let prog = get_prog();
            let tape = [Zero, Zero, One, Zero, One, One, Zero];
            let mut m = TuringMachine::new(State::Scan, prog, tape.iter().cloned());
            assert!(m.run())
        }

        #[test]
        fn reject_nontrivial() {
            use Alphabet::*;
            let prog = get_prog();
            let tape = [Zero, Zero, One, One, One, One, Zero];
            let mut m = TuringMachine::new(State::Scan, prog, tape.iter().cloned());
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
                    },
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
            let mut m = TuringMachine::new([0; 3], prog, std::iter::empty());
            assert!(m.run());
        }

        #[test]
        fn accept_small() {
            use Alphabet::*;

            let prog = get_prog();
            let input = [One, Three, Two];
            let mut m = TuringMachine::new([0; 3], prog, input.iter().cloned());
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
            let input = ones.chain(twos).chain(threes);
            let mut m = TuringMachine::new([0; 3], prog, input);
            assert!(m.run());
        }

        #[test]
        fn reject_no_ones() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let twos = repeat(Two).take(300);
            let threes = repeat(Three).take(300);
            let input = twos.chain(threes);
            let mut m = TuringMachine::new([0; 3], prog, input);
            assert!(!m.run());
        }

        #[test]
        fn reject_no_twos() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let ones = repeat(One).take(300);
            let threes = repeat(Three).take(300);
            let input = ones.chain(threes);
            let mut m = TuringMachine::new([0; 3], prog, input);
            assert!(!m.run());
        }

        #[test]
        fn reject_no_threes() {
            use std::iter::repeat;
            use Alphabet::*;

            let prog = get_prog();
            let twos = repeat(Two).take(300);
            let ones = repeat(One).take(300);
            let input = ones.chain(twos);
            let mut m = TuringMachine::new([0; 3], prog, input);
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
            let input = ones.chain(twos).chain(threes);
            let mut m = TuringMachine::new([0; 3], prog, input);
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
            let input = ones.chain(twos).chain(threes);
            let mut m = TuringMachine::new([0; 3], prog, input);
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
            let input = ones.chain(twos).chain(threes);
            let mut m = TuringMachine::new([0; 3], prog, input);
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
            let input = ones.chain(twos).chain(threes);
            let mut m = TuringMachine::new([0; 3], prog, input);
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
            let input = ones.chain(twos).chain(threes);
            let mut m = TuringMachine::new([0; 3], prog, input);
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
            let input = ones.chain(twos).chain(threes);
            let mut m = TuringMachine::new([0; 3], prog, input);
            assert!(!m.run());
        }
    }
}
