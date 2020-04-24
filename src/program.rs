use std::{collections::HashMap, fmt::Debug, hash::Hash};

pub trait TransitionFn<State, Alphabet>: Fn(&State, &Alphabet) -> Response<State, Alphabet> + Sized {}

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

pub struct Stimulus<State, Alphabet> {
    pub state: State,
    pub read: Alphabet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response<State, Alphabet> {
    pub goto: Goto<State>,
    pub write: Alphabet,
    pub mv: Option<Movement>,
}

impl<T, State, Alphabet> TransitionFn<State, Alphabet> for T
where
    T: Fn(&State, &Alphabet) -> Response<State, Alphabet>
{}

impl<State, Alphabet> From<(State, Alphabet)> for Stimulus<State, Alphabet> {
    fn from((state, read): (State, Alphabet)) -> Self {
        Self {
            state,
            read,
        }
    }
}

impl<State, Alphabet> From<(Goto<State>, Alphabet, Option<Movement>)> for Response<State, Alphabet> {
    fn from((goto, write, mv): (Goto<State>, Alphabet, Option<Movement>)) -> Self {
        Self {
            goto,
            write,
            mv,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgramBuilder<State, Alphabet>
where
    State: Hash + Eq,
    Alphabet: Hash + Eq,
{
    table: HashMap<State, HashMap<Alphabet, Response<State, Alphabet>>>,
}

impl<State, Alphabet> ProgramBuilder<State, Alphabet>
where
    State: Hash + Eq + Clone,
    Alphabet: Hash + Eq + Clone,
{
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn build(self) -> impl TransitionFn<State, Alphabet> {
        let table = self.table;
        move |state: &State, symbol: &Alphabet| match table
            .get(state)
            .and_then(|subtable| subtable.get(symbol))
        {
            Some(response) => return response.clone(),
            None => Response {
                goto: Goto::Halt(false),
                write: symbol.clone(),
                mv: None,
            },
        }
    }

    pub fn with_transition<S, R>(mut self, stimulus: S, response: R) -> Self
    where
        Stimulus<State, Alphabet>: From<S>,
        Response<State, Alphabet>: From<R>,
    {
        let stimulus: Stimulus<_, _> = stimulus.into();
        let response = response.into();
        self.table
            .entry(stimulus.state)
            .or_default()
            .insert(stimulus.read, response);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let prog = ProgramBuilder::new().build();
        assert_eq!(
            prog(&0, &0),
            Response {
                goto: Goto::Halt(false),
                write: 0,
                mv: None
            }
        );
    }

    #[test]
    fn nonempty() {
        let prog = ProgramBuilder::new()
            .with_transition((0, 0), (Goto::Run(1), 1, Some(Movement::Right)))
            .build();
        assert_eq!(
            prog(&0, &0),
            Response {
                goto: Goto::Run(1),
                write: 1,
                mv: Some(Movement::Right)
            }
        );
    }
}
