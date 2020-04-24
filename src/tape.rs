// Possible TODO: turn this into a multi-file module.

use crate::program::Movement;
use std::collections::VecDeque;

pub trait Tape<Alphabet> {
    fn move_left(&mut self);
    fn move_right(&mut self);

    fn move_head(&mut self, dir: Movement) {
        match dir {
            Movement::Left => self.move_left(),
            Movement::Right => self.move_right(),
        }
    }

    fn get(&self) -> &Alphabet;
    fn get_mut(&mut self) -> &mut Alphabet;

    // rustc complains when we don't box the return type. Not sure why.
    fn get_all(self) -> Box<dyn Iterator<Item=Alphabet>>;
}

#[derive(Debug)]
pub struct Unbounded<Alphabet> {
    idx: usize,
    tape: VecDeque<Alphabet>,
}

impl<Alphabet> Unbounded<Alphabet>
where
    Alphabet: Default,
{
    pub fn new() -> Self {
        Default::default()
    }
}

impl<Alphabet> Tape<Alphabet> for Unbounded<Alphabet>
where
    Alphabet: Default + 'static,
{
    fn move_left(&mut self) {
        match self.idx.checked_sub(1) {
            Some(new_idx) => self.idx = new_idx,
            None => self.tape.push_front(Default::default()),
        }
    }
    fn move_right(&mut self) {
        self.idx += 1;
        if self.tape.get(self.idx).is_none() {
            self.tape.push_back(Default::default());
        }
    }

    fn get(&self) -> &Alphabet {
        self.tape
            .get(self.idx)
            .expect("Unbounded tape must have R/W head over initialised cell.")
    }
    fn get_mut(&mut self) -> &mut Alphabet {
        self.tape
            .get_mut(self.idx)
            .expect("Unbounded tape must have R/W head over initialised cell.")
    }

    fn get_all(self) -> Box<dyn Iterator<Item=Alphabet>> {
        Box::new(self.tape.into_iter())
    }
}

impl<T, Alphabet> From<T> for Unbounded<Alphabet>
where
    Alphabet: Default,
    VecDeque<Alphabet>: From<T>,
{
    fn from(src: T) -> Self {
        let mut ret = Self {
            tape: VecDeque::from(src),
            idx: 0,
        };
        if ret.tape.is_empty() {
            ret = Self::new();
        }
        ret
    }
}

impl<Alphabet> Default for Unbounded<Alphabet>
where
    Alphabet: Default,
{
    fn default() -> Self {
        Self {
            idx: 0,
            tape: VecDeque::from(vec![Default::default()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut tape = Unbounded::new();
        assert_eq!(tape.get(), &0);
        *tape.get_mut() = 4;
        assert_eq!(tape.get(), &4);
        tape.move_left();
        assert_eq!(tape.get(), &0);
        *tape.get_mut() = -900;
        assert_eq!(tape.get(), &-900);
        tape.move_right();
        assert_eq!(tape.get(), &4);
        tape.move_head(Movement::Left);
        assert_eq!(tape.get(), &-900);
        tape.move_head(Movement::Right);
        assert_eq!(tape.get(), &4);
        tape.move_head(Movement::Right);
        assert_eq!(tape.get(), &0);
        tape.move_head(Movement::Right);
        assert_eq!(tape.get(), &0);
        *tape.get_mut() = 5;
        assert_eq!(tape.get(), &5);
        tape.move_left();
        assert_eq!(tape.get(), &0);
        tape.move_left();
        assert_eq!(tape.get(), &4);
        tape.move_left();
        assert_eq!(tape.get(), &-900);
    }
}
