//! A tiny and limited glob matcher implementation for FR Tries (optional)
pub mod acl;

use std::cell::RefCell;
use std::sync::Arc;
use crate::matcher::{Ahead, Event, MatchType, PushdownStateMachine, State, StateSequence};

#[derive(Debug)]
pub struct MachineInstance {
    tokens: Vec<Arc<StateSequence>>,
    state: State,
    glob_idx: usize,
    glob_char_idx: usize,
}

impl MachineInstance {

    #[inline]
    fn feed(&mut self, ev: Event) {
        match (&self.state, ev) {
            (State::Accepting | State::Expecting, Event::EndOfStream) => {
                if self.at_end() {
                    match &self.state {
                        State::Accepting => {
                            self.state = State::Accepted;
                        }
                        State::Expecting => {
                            self.state = State::Accepted;
                        }
                        State::Accepted => {
                            self.state = State::Accepted;
                        }
                        State::Rejected => {
                            self.state = State::Rejected;
                        }
                        State::Beyond => {
                            self.state = State::Beyond;
                        }
                        State::Failure(msg) => {
                            self.state = State::Failure(msg.clone());
                        }
                    }
                }
                else {
                    match self.look_ahead() {
                        None => {
                            self.state = State::Accepted;
                        }
                        Some(_ahead) => {
                            self.state = State::Rejected;
                        }
                    }

                }
            },
            (State::Accepting | State::Expecting, Event::CharIn(ch)) => {
                match self.look_ahead() {
                    None => {
                        self.state = State::Beyond;
                    }
                    Some(ahead) => {
                        match ahead {
                            Ahead::Exactly(current_char) => {
                                if current_char == ch {
                                    self.advance();
                                    if self.at_end() {
                                        //self.state = State::Accepted;
                                    }
                                    else {
                                        //self.state = State::Expecting;
                                    }
                                }
                                else if ch < current_char {
                                    self.state = State::Beyond;
                                }
                                else {
                                    self.state = State::Rejected;
                                }
                            }
                            Ahead::AnyOr(current_char) => {
                                if current_char == ch {
                                    self.advance();
                                    if self.at_end() {
                                        //self.state = State::Accepted;
                                    }
                                    self.state = State::Accepting;
                                }
                            }
                            Ahead::Any => {
                                self.state = State::Accepting;
                            }
                        }
                    }
                }
            }
            (s, e) => {
                self.state = State::Failure(format!("Wrong state, event combination: {:#?} {:#?}", s, e)
                    .to_string())
            }
        }
    }

    #[inline]
    fn look_ahead(&self) -> Option<Ahead> {
        match self.tokens.get(self.glob_idx) {
            None => {
                return None; // Exceeding limits
            }
            Some(current_glob) => {
                match current_glob.sequence.get(self.glob_char_idx) {
                    None => {
                        match current_glob.match_type {
                            MatchType::Literal => {
                                return None;
                            }
                            MatchType::AnyOr => {
                                return Some(Ahead::Any)
                            }
                        }
                    }
                    Some(current_char) => {
                        match current_glob.match_type {
                            MatchType::Literal => {
                                return Some(Ahead::Exactly(*current_char))
                            }
                            MatchType::AnyOr => {
                                return Some(Ahead::AnyOr(*current_char))
                            }
                        }
                    }
                }
            }
        }
    }

    #[inline]
    fn advance(&mut self) {
        match self.tokens.get(self.glob_idx) {
            None => {
                panic!("Cannot advance");
            }
            Some(current_glob) => {
                if self.glob_char_idx < current_glob.sequence.len() - 1 {
                    self.glob_char_idx += 1;
                }
                else if self.glob_idx < self.tokens.len() - 1 {
                    self.glob_char_idx = 0;
                    self.glob_idx += 1;
                }
                else { // Position at END
                    self.glob_char_idx = 0;
                    self.glob_idx = self.tokens.len();
                }
            }
        }
    }

    #[inline]
    fn at_end(&self) -> bool {
        match self.tokens.get(self.glob_idx) {
            None => {
                true
            }
            Some(current_glob) => {
                if self.glob_idx + 1 >= self.tokens.len() {
                    if self.glob_char_idx + 1 >= current_glob.sequence.len() {
                        return true;
                    }
                    else {
                        return false;
                    }
                }
                else {
                    return false;
                }
            }
        }
    }

    #[inline]
    fn is_expecting(&self) -> bool {
        match self.tokens.get(self.glob_idx) {
            None => {
                false
            }
            Some(_) => {
                if self.glob_idx == self.tokens.len() {
                    return false;
                }
                else {
                    return true;
                }
            }
        }
    }
}

/////////////////////////////
#[derive(Debug, Clone)]
pub struct GlobMatcher {
    stack: Vec<Arc<RefCell<MachineInstance>>>,
}

impl PushdownStateMachine for GlobMatcher {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
        }
    }

    #[inline]
    fn step_in(&mut self, sequence: &Vec<Arc<StateSequence>>) {
        let new_instance = match self.stack.last() {
            None => {
                let initial_state = match sequence.first() {
                    None => {
                        State::Failure(String::from("No regexp provided"))
                    }
                    Some(first_seq) => {
                        match first_seq.match_type {
                            MatchType::Literal => {
                                State::Expecting
                            },
                            MatchType::AnyOr => {
                                State::Accepting
                            },
                        }
                    }
                };

                Arc::new(RefCell::new(MachineInstance {
                    tokens: sequence.clone(),
                    state: initial_state,
                    glob_idx: 0,
                    glob_char_idx: 0,
                }))
            }
            Some(top) => {
                let machine = top.borrow();
                let mut tokens = machine.tokens.clone();
                for aditional_token in sequence {
                    tokens.push(aditional_token.clone());
                }
                Arc::new(RefCell::new(MachineInstance {
                    tokens,
                    state: machine.state.clone(),
                    glob_idx: machine.glob_idx,
                    glob_char_idx: machine.glob_char_idx,
                }))
            }
        };
        self.stack.push(new_instance);
    }

    #[inline]
    fn step_out(&mut self) {
        let _k = self.stack.pop();
        //println!("STEP OUT FROM KEY ({:?})", k.as_ref().unwrap().borrow().tokens);
    }

    #[inline]
    fn accepts_more(&self) -> bool {
        match self.stack.last() {
            Some(machine) => machine.borrow().is_expecting(),
            None => false
        }
    }

    #[inline]
    fn feed(&mut self, ev: Event) {
        match self.stack.last_mut() {
            Some(machine) => machine.borrow_mut().feed(ev),
            None => {}
        }
    }

    #[inline]
    fn state(&self) -> State {
        match self.stack.last() {
            Some(machine) => machine.borrow().state.clone(),
            None => State::Failure(String::from("Machine not initiallized"))
        }
    }

    #[inline]
    fn is_sink(&self) -> bool {
        match self.stack.last() {
            Some(machine) => match machine.borrow().state {
                State::Accepting => false,
                State::Expecting => false,
                State::Accepted => true,
                State::Rejected => true,
                State::Beyond => true,
                State::Failure(_) => true,
            },
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matcher_test() {

        let mut expected_token_sequence: Vec<Arc<StateSequence>> = Vec::new();
        expected_token_sequence.push(Arc::new(StateSequence {
            match_type: MatchType::Literal,
            sequence: String::from("123").chars().collect()
        }));
        let mut matcher = GlobMatcher::new();
        matcher.step_in(&expected_token_sequence);
        let str: Vec<char> = String::from("12345").chars().collect();
        let mut input = str.iter();
        matcher.feed(Event::CharIn(*input.next().unwrap()));
        matcher.feed(Event::CharIn(*input.next().unwrap()));
        matcher.feed(Event::CharIn(*input.next().unwrap()));

        matcher.feed(Event::EndOfStream);
        let _st = matcher.state();
        let _st = matcher.accepts_more();

        matcher.feed(Event::CharIn(*input.next().unwrap()));
        let _st = matcher.state();
        let _st = matcher.accepts_more();

        println!("{:?}", matcher.stack.last().unwrap());
        let mi = matcher.stack.last().unwrap();
        let _v = mi.as_ref().borrow().look_ahead();
        let _v = mi.as_ref().borrow().is_expecting();

        matcher.step_out();
        let _st = matcher.state();
        let _st = matcher.accepts_more();

        matcher.feed(Event::CharIn(*input.next().unwrap()));

        println!("{:?}", matcher.clone());

        expected_token_sequence.push(Arc::new(StateSequence {
            match_type: MatchType::AnyOr,
            sequence: Vec::new()
        }));
        let str: Vec<char> = String::from("12345").chars().collect();
        let mut input = str.iter();
        matcher.step_out();
        matcher.step_in(&expected_token_sequence);
        matcher.feed(Event::CharIn(*input.next().unwrap()));
        let _st = matcher.state();
        let _st = matcher.accepts_more();

        let mi = matcher.stack.last().unwrap();
        let _v = mi.as_ref().borrow().look_ahead();
        let _v = mi.as_ref().borrow().is_expecting();

    }
}
