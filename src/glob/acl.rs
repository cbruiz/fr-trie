//! A specific Trie implementation for Access Control Lists supporting (limited) glob matching
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use bitflags::bitflags;
use serde::{Serialize, Deserialize};
use crate::trie::Trie;
use crate::key::{KeyPrefix};
use crate::matcher::{MatchType, StateSequence};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Acl {
    pub path: String,
}

impl Acl {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string()
        }
    }
}

///////////////////////////

impl Display for Acl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.path))
    }
}

impl KeyPrefix for Acl {

    #[inline]
    fn key_chars(&self) -> Vec<char> {
        self.path.chars().collect::<Vec<_>>()
    }

    #[inline]
    fn key_len(&self) -> usize {
        self.path.len()
    }

    #[inline]
    fn empty() -> Self {
        Self {
            path: String::new(),
        }
    }

    #[inline]
    fn new_from_key_prefix(&self, index: usize) -> Self {
        Self {
            path: self.path[..index].to_string()
        }
    }

    #[inline]
    fn new_from_postfix(&self, index: usize) -> Self {
        Self {
            path: self.path[index..].to_string()
        }
    }

    #[inline]
    fn compiled(&self) -> Vec<Arc<StateSequence>> {
        let mut compiled_seq = Vec::new();
        let mut buff: Vec<char> = Vec::new();
        let mut next_state: MatchType = MatchType::Literal;
        for ch in self.path.chars() {
            if ch == '*' {
                if buff.len() > 0 {
                    compiled_seq.push(Arc::new(StateSequence {
                        match_type: next_state.clone(),
                        sequence: buff,
                    }));
                    next_state = MatchType::AnyOr;
                }
                buff = Vec::new();
            }
            else {
                buff.push(ch);
            }
        }
        if !buff.is_empty() {
            compiled_seq.push(Arc::new(StateSequence {
                match_type: next_state,
                sequence: buff,
            }));
        }
        else if next_state == MatchType::AnyOr {
            compiled_seq.push(Arc::new(StateSequence {
                match_type: next_state,
                sequence: Vec::new(),
            }));
        }
        compiled_seq
    }

}

///////////////////
bitflags! {
    #[derive(Default, Serialize, Deserialize)]
    pub struct Permissions: u8 {
        const READ = 0b00000001;
        const WRITE = 0b00000010;
        const CREATE = 0b00000100;
        const DELETE = 0b00001000;
        const OWNER = Self::READ.bits | Self::WRITE.bits | Self::CREATE.bits | Self::DELETE.bits;
    }
}

pub type AclTrie = Trie<Acl,Permissions>;

#[cfg(test)]
mod tests {

    use crate::key::TrieKey;
    use crate::matcher::{Event, PushdownStateMachine};
    use crate::glob::GlobMatcher;
    use crate::glob::acl::*;


    #[test]
    fn acl_trie_test() {
        {
            let token1 = String::from("f20000000001XXXG");
            let acl1 = TrieKey::new(Acl::new("f20*1*F"));
            println!("Processing {:?}", token1);
            let mut machine = GlobMatcher::new();

            machine.step_in(&acl1.seq);
            for ch in token1.chars() {
                machine.feed(Event::CharIn(ch));
                if machine.is_sink() {
                    break;
                }
            }

            println!("Machine is at {:?} state", machine.state());

            let token2 = String::from("T");

            let acl2 = TrieKey::new(Acl::new("T"));
            machine.step_in(&acl2.seq);
            for ch in token2.chars() {
                machine.feed(Event::CharIn(ch));
                if machine.is_sink() {
                    break;
                }
            }
            machine.feed(Event::EndOfStream);
            println!("Machine stopped at {:?} state", machine.state());
            machine.step_out();
        }

        {
            let acl = vec![TrieKey::new(Acl::new("f1*t")), TrieKey::new(Acl::new("t"))];
            let token = String::from("f11tt");
            println!("Processing {:?}", token);
            let mut machine = GlobMatcher::new();

            let mut acl_iterator = acl.iter();
            machine.step_in(&acl_iterator.next().unwrap().seq);
            for ch in token.chars() {
                if machine.is_sink() {
                    break;
                }
                machine.feed(Event::CharIn(ch));
                if !machine.accepts_more() {
                    match acl_iterator.next() {
                        None => break,
                        Some(acl) => machine.step_in(&acl.seq),
                    }
                }
            }
            println!("Machine is at {:?} state", machine.state());


            machine.feed(Event::EndOfStream);
            println!("Machine stopped in {:?} state", machine.state());
        }

        {
            let acl = TrieKey::new(Acl::new("f1*t"));
            let token = String::from("f01t");
            println!("Processing {:?}", token);
            let mut machine = GlobMatcher::new();
            machine.step_in(&acl.seq);
            for ch in token.chars() {
                machine.feed(Event::CharIn(ch));
                if machine.is_sink() {
                    break;
                }
            }
            println!("Machine stopped in {:?} state", machine.state())
        }

        {
            let acl = TrieKey::new(Acl::new("f1*"));
            let token = String::from("f01000000000");
            println!("Processing {:?}", token);
            let mut machine = GlobMatcher::new();
            machine.step_in(&acl.seq);
            for ch in token.chars() {
                machine.feed(Event::CharIn(ch));
                if machine.is_sink() {
                    break;
                }
            }
            println!("Machine stopped in {:?} state", machine.state())
        }

        {
            let acl = TrieKey::new(Acl::new("f1*"));
            let token = String::from("f10000000000");
            println!("Processing {:?}", token);
            let mut machine = GlobMatcher::new();
            machine.step_in(&acl.seq);
            for ch in token.chars() {
                machine.feed(Event::CharIn(ch));
                if machine.is_sink() {
                    break;
                }
            }
            println!("Machine stopped in {:?} state", machine.state())
        }

        {
            let acl = TrieKey::new(Acl::new("f1*"));
            let token = String::from("f20000000000");
            println!("Processing {:?}", token);
            let mut machine = GlobMatcher::new();
            machine.step_in(&acl.seq);
            for ch in token.chars() {
                machine.feed(Event::CharIn(ch));
                if machine.is_sink() {
                    break;
                }
            }
            println!("Machine stopped in {:?} state", machine.state())
        }

        {
            let acl = TrieKey::new(Acl::new("f20*1*F"));
            let token = String::from("f20000000001XXXF");
            println!("Processing {:?}", token);
            let mut machine = GlobMatcher::new();
            machine.step_in(&acl.seq);
            for ch in token.chars() {
                machine.feed(Event::CharIn(ch));
                if machine.is_sink() {
                    break;
                }
            }
            println!("Machine stopped in {:?} state", machine.state())
        }
    }
}