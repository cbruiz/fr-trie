//! The Trie Matcher trait
use std::sync::Arc;
use serde::{Serialize, Deserialize};

///////////////////////////

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatchType {
    Literal,
    AnyOr
}

///////////////////////////

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum State {
    Accepting,
    Expecting,
    Accepted,
    Rejected,
    Beyond,
    Failure(String),
}

#[derive(Debug, Clone, Copy)]
pub enum Event {
    CharIn(char),
    EndOfStream
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StateSequence {
    pub match_type: MatchType,
    pub sequence: Vec<char>
}

///! This is the trait on which [Iterator] relies
///!
pub trait PushdownStateMachine {
    fn new() -> Self;

    fn step_in(&mut self, key: &Vec<Arc<StateSequence>>);
    fn step_out(&mut self);

    fn accepts_more(&self) -> bool;
    fn feed(&mut self, ev: Event);

    fn state(&self) -> State;
    fn is_sink(&self) -> bool;
}

pub enum Ahead {
    Exactly(char),
    AnyOr(char),
    Any,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matcher_test() {
        let ev1 = Event::CharIn(12 as char);
        let ev2 = Event::EndOfStream;
        let ev3 = ev2;
        println!("states = {:?}, {:?} {:?}", ev1.clone(), ev2, ev3);
    }
}