use std::{collections::HashMap, fmt::Debug, hash::Hash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Movement {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Goto<State> {
    Run(State),
    Halt(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition<State, Alphabet> {
    old_state: State,
    read: Alphabet,
    goto: Goto<State>,
    write: Alphabet,
    movement: Option<Movement>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Program<State, Alphabet>
where
    State: Eq + Hash + Clone + Debug,
    Alphabet: Eq + Hash + Default + Clone + Debug,
{
    initial_state: State,
    transitions: HashMap<(State, Alphabet), (Goto<State>, Alphabet, Option<Movement>)>,
}

impl<State, Alphabet> Transition<State, Alphabet> {
    pub fn new(
        old_state: State,
        read: Alphabet,
        goto: Goto<State>,
        write: Alphabet,
        movement: Option<Movement>,
    ) -> Self {
        Self {
            old_state,
            read,
            goto,
            write,
            movement,
        }
    }

    pub fn cont(old_state: State, read: Alphabet, new_state: State, write: Alphabet) -> Self {
        Self {
            old_state,
            read,
            goto: Goto::Run(new_state),
            write,
            movement: None,
        }
    }

    pub fn halt(old_state: State, read: Alphabet, result: bool, write: Alphabet) -> Self {
        Self {
            old_state,
            read,
            goto: Goto::Halt(result),
            write,
            movement: None,
        }
    }

    pub fn accept(old_state: State, read: Alphabet, write: Alphabet) -> Self {
        Self::halt(old_state, read, true, write)
    }

    pub fn reject(old_state: State, read: Alphabet, write: Alphabet) -> Self {
        Self::halt(old_state, read, false, write)
    }

    pub fn and_move(mut self, movement: Movement) -> Self {
        self.movement = Some(movement);
        self
    }

    pub fn left(self) -> Self {
        self.and_move(Movement::Left)
    }

    pub fn right(self) -> Self {
        self.and_move(Movement::Right)
    }

    pub fn get_stimulus(&self) -> (&State, &Alphabet) {
        (&self.old_state, &self.read)
    }
}

impl<State, Alphabet> Program<State, Alphabet>
where
    State: Eq + Hash + Clone + Debug,
    Alphabet: Eq + Hash + Default + Clone + Debug,
{
    pub fn new(initial_state: State) -> Self {
        Self {
            initial_state,
            transitions: HashMap::new(),
        }
    }

    pub fn with_transition(mut self, t: Transition<State, Alphabet>) -> Self {
        assert!(self
            .transitions
            .insert((t.old_state, t.read), (t.goto, t.write, t.movement))
            .is_none());
        self
    }

    pub fn with_transitions(
        mut self,
        ts: impl Iterator<Item = Transition<State, Alphabet>>,
    ) -> Self {
        for t in ts {
            self = self.with_transition(t);
        }
        self
    }

    pub fn get_start(&self) -> &State {
        &self.initial_state
    }

    pub fn get_response(
        &self,
        current_state: State,
        symbol: Alphabet,
    ) -> Option<(Goto<State>, Alphabet, Option<Movement>)> {
        self.transitions.get(&(current_state, symbol)).cloned()
    }
}
