use crate::program::{Goto, Movement, Program, Transition};
use smol_str::SmolStr;
use std::str::FromStr;

// Problem - This accepts neither escape codes nor literals. Fix.

#[derive(Debug)]
pub enum ParseTransitionError {
    BadString(String),
}
impl FromStr for Transition<SmolStr, SmolStr> {
    type Err = ParseTransitionError;

    /*
     * <Transition> = <Stimulus> -> <Response>
     * <Stimulus>   = (<State>, <Symbol>)
     * <Response>   = (<Goto>, <Symbol>, <Direction>)
     * <Goto>       = <State> | ^ | v
     * <Direction>  = - | 0 | +
     * <State>      = [String]
     * <Symbol>     = "String"
     */

    // TODO: write this better.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ParseTransitionError::*;

        let mut s = s.chars();
        match s
            .by_ref()
            .filter(|c| *c != ' ')
            .take(2)
            .collect::<String>()
            .as_str()
        {
            "([" => {}
            _ => return Err(BadString("0".to_string())),
        }
        let mut old_state = String::new();
        while let Some(c) = s.next() {
            if c == ']' {
                break;
            }
            old_state.push(c);
        }
        match s
            .by_ref()
            .filter(|c| *c != ' ')
            .take(2)
            .collect::<String>()
            .as_str()
        {
            ",\"" => {}
            _ => return Err(BadString("1".to_string())),
        }
        let mut read_symbol = String::new();
        while let Some(c) = s.next() {
            if c == '"' {
                break;
            }
            read_symbol.push(c);
        }
        match s
            .by_ref()
            .filter(|c| *c != ' ')
            .take(4)
            .collect::<String>()
            .as_str()
        {
            ")->(" => {}
            bad => return Err(BadString(bad.to_string())),
        }
        let goto = match s.by_ref().find(|c| *c != ' ') {
            Some('^') => Goto::Halt(true),
            Some('v') => Goto::Halt(false),
            Some('[') => {
                let mut new_state = String::new();
                while let Some(c) = s.next() {
                    if c == ']' {
                        break;
                    }
                    new_state.push(c);
                }
                Goto::Run(new_state.into())
            }
            _ => return Err(BadString("3".to_string())),
        };
        match s
            .by_ref()
            .filter(|c| *c != ' ')
            .take(2)
            .collect::<String>()
            .as_str()
        {
            ",\"" => {}
            _ => return Err(BadString("4".to_string())),
        }
        let mut write_symbol = String::new();
        while let Some(c) = s.next() {
            if c == '"' {
                break;
            }
            write_symbol.push(c);
        }
        match s.by_ref().find(|c| *c != ' ') {
            Some(',') => {}
            _ => return Err(BadString("6".to_string())),
        }
        let movement = match s.by_ref().find(|c| *c != ' ') {
            Some('-') => Some(Movement::Left),
            Some('0') => None,
            Some('+') => Some(Movement::Right),
            _ => return Err(BadString("7".to_string())),
        };
        match s.by_ref().find(|c| *c != ' ') {
            Some(')') => {}
            _ => return Err(BadString("8".to_string())),
        }
        Ok(Transition::new(
            old_state.into(),
            read_symbol.replace("\\n", "\n").replace("\\t", "\t").into(),
            goto,
            write_symbol
                .replace("\\n", "\n")
                .replace("\\t", "\t")
                .into(),
            movement,
        ))
    }
}

impl FromStr for Program<SmolStr, SmolStr> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let transitions: Vec<Transition<SmolStr, SmolStr>> = s
            .lines()
            .map(|line| Transition::from_str(line))
            .filter(|result| result.is_ok())
            .map(|ok_result| ok_result.unwrap())
            .collect();
        let initial_state = match transitions.get(0) {
            None => return Err(()),
            Some(first_transition) => first_transition.get_stimulus().0.clone(),
        };
        Ok(Program::new(initial_state).with_transitions(transitions.into_iter()))
    }
}
