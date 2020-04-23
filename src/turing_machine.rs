use crate::program::{Goto, Movement, Program};
use std::{
    collections::VecDeque,
    default::Default,
    fmt,
    fmt::{Debug, Display},
    hash::Hash,
    iter::FromIterator,
};

#[derive(Debug)]
pub struct TuringMachine<State, Alphabet>
where
    State: Eq + Hash + Clone + Debug,
    Alphabet: Eq + Hash + Default + Clone + Debug,
{
    state: Goto<State>,
    prog: Program<State, Alphabet>,
    tape: VecDeque<Alphabet>,
    idx: usize, // location on the tape.
    counter: u64,
}

impl<State, Alphabet> TuringMachine<State, Alphabet>
where
    State: Eq + Hash + Clone + Debug,
    Alphabet: Eq + Hash + Default + Clone + Debug,
{
    pub fn new(prog: Program<State, Alphabet>, input: impl Iterator<Item = Alphabet>) -> Self {
        let mut tape = VecDeque::from_iter(input);
        if tape.is_empty() {
            tape.push_back(Default::default());
        }
        Self {
            state: Goto::Run(prog.get_start().clone()),
            prog,
            tape,
            idx: 0,
            counter: 0,
        }
    }

    // May not return - halting problem is hard, yo.
    pub fn run(&mut self) -> Option<bool> {
        let mut ret = None;
        for temp in self {
            ret = temp;
        }
        ret
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

impl<State, Alphabet> Iterator for TuringMachine<State, Alphabet>
where
    State: Eq + Hash + Clone + Debug,
    Alphabet: Eq + Hash + Default + Clone + Debug,
{
    type Item = Option<bool>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.state {
            Goto::Halt(_) => None,
            Goto::Run(ref state) => {
                self.counter += 1;

                let to_do = self.prog.get_response(state.clone(), self.read_symbol())?;
                self.state = to_do.0;
                self.write_symbol(to_do.1);
                if let Some(direction) = to_do.2 {
                    self.move_head(direction);
                }
                match &self.state {
                    Goto::Run(_) => Some(None),
                    Goto::Halt(ref b) => Some(Some(*b)),
                }
            }
        }
    }
}

impl<State, Alphabet> fmt::Display for TuringMachine<State, Alphabet>
where
    State: Eq + Hash + Clone + Debug + Display,
    Alphabet: Eq + Hash + Default + Clone + Debug + Display,
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
        writeln!(f, "================")?;
        write!(f, "{}", self.counter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::program::{Program, Transition};

    #[test]
    fn construct() {
        let prog = Program::<bool, u8>::new(false);
        let m = TuringMachine::new(prog, std::iter::empty());
        assert_eq!(m.read_symbol(), 0);
    }

    #[test]
    fn even_zeros() {
        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        enum State {
            Scan,
            Flip,
            Even,
            Odd,
        }

        #[derive(Debug, Clone, Hash, PartialEq, Eq)]
        enum Alphabet {
            Zero,
            One,
            Blank,
        }

        impl Default for Alphabet {
            fn default() -> Self {
                Alphabet::Blank
            }
        }

        use Alphabet::*;
        use State::*;

        // Accepts strings with an even number of Zeros.
        let p = Program::<State, Alphabet>::new(State::Scan).with_transitions(
            [
                // Scan to the right.
                Transition::cont(Scan, Zero, Scan, Zero).right(),
                Transition::cont(Scan, One, Scan, One).right(),
                Transition::cont(Scan, Blank, Flip, Blank).left(),
                // Scan to the left, flipping each cell.
                Transition::cont(Flip, Zero, Flip, One).left(),
                Transition::cont(Flip, One, Flip, Zero).left(),
                Transition::cont(Flip, Blank, Even, Blank).right(),
                // Count the parity.
                Transition::cont(Even, Zero, Even, Zero).right(),
                Transition::cont(Even, One, Odd, One).right(),
                Transition::cont(Odd, Zero, Odd, Zero).right(),
                Transition::cont(Odd, One, Even, One).right(),
                // Check the parity.
                Transition::accept(Even, Blank, Blank),
                Transition::reject(Odd, Blank, Blank),
            ]
            .iter()
            .cloned(),
        );

        let input = [Zero, One, One, Zero, One, Zero, One, Zero, One];
        let mut m = TuringMachine::new(p, input.iter().cloned());
        assert_eq!(m.run(), Some(true));
    }

    mod ones_twos_threes {
        use super::*;

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        enum State {
            Begin,
            SkipOnes,
            SkipTwos,
            SkipThrees,
            CheckThrees,
            CheckTwos,
            CheckOnes,
            CheckSuccess,
        }
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        enum Alphabet {
            Blank,
            One,
            Two,
            Three,
            Seen,
        }
        use Alphabet::*;
        use State::*;

        impl Default for Alphabet {
            fn default() -> Self {
                Alphabet::Blank
            }
        }

        fn get_prog() -> Program<State, Alphabet> {
            // T(n) = 5n^2 + 3n
            // (I think)
            Program::new(Begin).with_transitions(
                [
                    // EraseOne
                    Transition::cont(Begin, One, SkipOnes, Seen).right(),
                    Transition::accept(Begin, Blank, Blank),
                    Transition::reject(Begin, Two, Two),
                    Transition::reject(Begin, Three, Three),
                    // SkipOnes
                    Transition::cont(SkipOnes, One, SkipOnes, One).right(),
                    Transition::cont(SkipOnes, Seen, SkipOnes, Seen).right(),
                    Transition::cont(SkipOnes, Two, SkipTwos, Seen).right(),
                    Transition::reject(SkipOnes, Three, Three),
                    Transition::reject(SkipOnes, Blank, Blank),
                    // SkipTwos
                    Transition::cont(SkipTwos, Two, SkipTwos, Two).right(),
                    Transition::cont(SkipTwos, Seen, SkipTwos, Seen).right(),
                    Transition::cont(SkipTwos, Three, SkipThrees, Seen).right(),
                    Transition::reject(SkipTwos, One, One),
                    Transition::reject(SkipTwos, Blank, Blank),
                    // SkipThrees
                    Transition::cont(SkipThrees, Three, SkipThrees, Three).right(),
                    Transition::cont(SkipThrees, Seen, SkipThrees, Seen).right(),
                    Transition::cont(SkipThrees, Blank, CheckSuccess, Blank).left(),
                    Transition::reject(SkipThrees, One, One),
                    Transition::reject(SkipThrees, Two, Two),
                    // CheckSuccess
                    Transition::cont(CheckSuccess, Seen, CheckSuccess, Seen).left(),
                    Transition::cont(CheckSuccess, Three, CheckThrees, Three).left(),
                    Transition::accept(CheckSuccess, Blank, Blank),
                    Transition::reject(CheckSuccess, One, One),
                    Transition::reject(CheckSuccess, Two, Two),
                    // CheckThrees
                    Transition::cont(CheckThrees, Three, CheckThrees, Three).left(),
                    Transition::cont(CheckThrees, Seen, CheckThrees, Seen).left(),
                    Transition::cont(CheckThrees, Two, CheckTwos, Two).left(),
                    Transition::reject(CheckThrees, One, One),
                    Transition::reject(CheckThrees, Blank, Blank),
                    // CheckTwos
                    Transition::cont(CheckTwos, Two, CheckTwos, Two).left(),
                    Transition::cont(CheckTwos, Seen, CheckTwos, Seen).left(),
                    Transition::cont(CheckTwos, One, CheckOnes, One).left(),
                    Transition::reject(CheckTwos, Three, Three),
                    Transition::reject(CheckTwos, Blank, Blank),
                    // CheckOnes
                    Transition::cont(CheckOnes, One, CheckOnes, One).left(),
                    Transition::cont(CheckOnes, Seen, Begin, Seen).right(),
                    Transition::reject(CheckOnes, Three, Three),
                    Transition::reject(CheckOnes, Two, Two),
                ]
                .iter()
                .cloned(),
            )
        }

        #[test]
        fn accept_empty() {
            let input = [];
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(true));
        }

        #[test]
        fn accept_small() {
            let input = [One, Two, Three];
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(true));
        }

        #[test]
        fn accept_large() {
            let input = {
                let mut input = Vec::new();
                let n = 500;
                for _ in 0..n {
                    input.push(One);
                }
                for _ in 0..n {
                    input.push(Two);
                }
                for _ in 0..n {
                    input.push(Three);
                }
                input
            };
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(true));
        }

        #[test]
        fn reject_no_ones() {
            let input = [Two, Three];
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(false));
        }

        #[test]
        fn reject_no_twos() {
            let input = [One, Three];
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(false));
        }

        #[test]
        fn reject_no_threes() {
            let input = [One, Two];
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(false));
        }

        #[test]
        fn reject_out_of_order() {
            let input = [Two, One, Three];
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(false));
        }

        #[test]
        fn reject_too_many_ones() {
            let input = {
                let mut input = Vec::new();
                let n = 12;
                for _ in 0..n + 1 {
                    input.push(One);
                }
                for _ in 0..n {
                    input.push(Two);
                }
                for _ in 0..n {
                    input.push(Three);
                }
                input
            };
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(false));
        }

        #[test]
        fn reject_too_many_twos() {
            let input = {
                let mut input = Vec::new();
                let n = 12;
                for _ in 0..n {
                    input.push(One);
                }
                for _ in 0..n + 1 {
                    input.push(Two);
                }
                for _ in 0..n {
                    input.push(Three);
                }
                input
            };
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(false));
        }

        #[test]
        fn reject_too_many_threes() {
            let input = {
                let mut input = Vec::new();
                let n = 12;
                for _ in 0..n {
                    input.push(One);
                }
                for _ in 0..n {
                    input.push(Two);
                }
                for _ in 0..n + 1 {
                    input.push(Three);
                }
                input
            };
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(false));
        }

        #[test]
        fn reject_no_enough_ones() {
            let input = {
                let mut input = Vec::new();
                let n = 12;
                for _ in 0..n - 1 {
                    input.push(One);
                }
                for _ in 0..n {
                    input.push(Two);
                }
                for _ in 0..n {
                    input.push(Three);
                }
                input
            };
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(false));
        }

        #[test]
        fn reject_no_enough_twos() {
            let input = {
                let mut input = Vec::new();
                let n = 12;
                for _ in 0..n {
                    input.push(One);
                }
                for _ in 0..n - 1 {
                    input.push(Two);
                }
                for _ in 0..n {
                    input.push(Three);
                }
                input
            };
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(false));
        }

        #[test]
        fn reject_no_enough_threes() {
            let input = {
                let mut input = Vec::new();
                let n = 12;
                for _ in 0..n {
                    input.push(One);
                }
                for _ in 0..n {
                    input.push(Two);
                }
                for _ in 0..n - 1 {
                    input.push(Three);
                }
                input
            };
            let mut m = TuringMachine::new(get_prog(), input.iter().cloned());
            assert_eq!(m.run(), Some(false));
        }
    }

    mod repeated_string {
        use super::*;
        use smol_str::SmolStr;
        use std::str::FromStr;

        fn get_prog() -> Program<SmolStr, SmolStr> {
            Program::<SmolStr, SmolStr>::new("read".into()).with_transitions(
                [
                    // read - degenerate case
                    Transition::from_str("([read], \"\") -> (v, \"\", 0)").unwrap(),
                    // read - final case
                    Transition::from_str("([read], \"#\") -> ([end], \"#\", +)").unwrap(),
                    // read - good cases
                    Transition::from_str("([read], \"0\")->([search 0 left], \"x\", +)").unwrap(),
                    Transition::from_str("([read], \"1\")->([search 1 left], \"x\", +)").unwrap(),
                    // search-0-left
                    Transition::from_str("([search 0 left], \"0\") -> ([search 0 left], \"0\", +)")
                        .unwrap(),
                    Transition::from_str("([search 0 left], \"1\") -> ([search 0 left], \"1\", +)")
                        .unwrap(),
                    Transition::from_str(
                        "([search 0 left], \"#\") -> ([search 0 right], \"#\", +)",
                    )
                    .unwrap(),
                    Transition::from_str("([search 0 left], \"\") -> (v, \"\", 0)").unwrap(),
                    // search-1-left
                    Transition::from_str("([search 1 left], \"0\") -> ([search 1 left], \"0\", +)")
                        .unwrap(),
                    Transition::from_str("([search 1 left], \"1\") -> ([search 1 left], \"1\", +)")
                        .unwrap(),
                    Transition::from_str(
                        "([search 1 left], \"#\") -> ([search 1 right], \"#\", +)",
                    )
                    .unwrap(),
                    Transition::from_str("([search 1 left], \"\") -> (v, \"\", 0)").unwrap(),
                    // search-0-right
                    Transition::from_str(
                        "([search 0 right], \"0\") -> ([back up right], \"x\", -)",
                    )
                    .unwrap(),
                    Transition::from_str(
                        "([search 0 right], \"x\") -> ([search 0 right], \"x\", +)",
                    )
                    .unwrap(),
                    Transition::from_str("([search 0 right], \"1\") -> (v, \"1\", 0)").unwrap(),
                    Transition::from_str("([search 0 right], \"#\") -> (v, \"#\", 0)").unwrap(),
                    Transition::from_str("([search 0 right], \"\") -> (v, \"\", 0)").unwrap(),
                    // search-1-right
                    Transition::from_str(
                        "([search 1 right], \"1\") -> ([back up right], \"x\", -)",
                    )
                    .unwrap(),
                    Transition::from_str(
                        "([search 1 right], \"x\") -> ([search 1 right], \"x\", +)",
                    )
                    .unwrap(),
                    Transition::from_str("([search 1 right], \"0\") -> (v, \"0\", 0)").unwrap(),
                    Transition::from_str("([search 1 right], \"#\") -> (v, \"#\", 0)").unwrap(),
                    Transition::from_str("([search 1 right], \"\") -> (v, \"\", 0)").unwrap(),
                    // back-up-right
                    Transition::from_str("([back up right], \"x\") -> ([back up right], \"x\", -)")
                        .unwrap(),
                    Transition::from_str("([back up right], \"#\") -> ([back up left], \"#\", -)")
                        .unwrap(),
                    // back-up-left
                    Transition::from_str("([back up left], \"0\") -> ([back up left], \"0\", -)")
                        .unwrap(),
                    Transition::from_str("([back up left], \"1\") -> ([back up left], \"1\", -)")
                        .unwrap(),
                    Transition::from_str("([back up left], \"x\") -> ([read], \"x\", +)").unwrap(),
                    // check at the end.
                    Transition::from_str("([end], \"x\") -> ([end], \"x\", +)").unwrap(),
                    Transition::from_str("([end], \"\") -> (^, \"\", 0)").unwrap(),
                    Transition::from_str("([end], \"0\") -> (v, \"0\", 0)").unwrap(),
                    Transition::from_str("([end], \"1\") -> (v, \"1\", 0)").unwrap(),
                ]
                .iter()
                .cloned(),
            )
        }

        #[test]
        fn accept_normal() {
            use rand::prelude::*;

            let mut rng = thread_rng();
            let word: Vec<SmolStr> = (0..7)
                .map(|_| if rng.gen() { "1".into() } else { "0".into() })
                .collect();
            let mut input = word.clone();
            input.push("#".into());
            input.extend(word.into_iter());

            let mut m = TuringMachine::new(get_prog(), input.into_iter());
            assert_eq!(m.run(), Some(true));
        }
    }
}
