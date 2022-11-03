//! The Trie iterator based on a pushdown automata to perform lookup

use crate::key::KeyPrefix;
use crate::matcher::{Event, PushdownStateMachine, State};
use crate::node::RFRNode;

///! Tracks lookup
struct LookupState<'a, K: 'a + KeyPrefix + Clone, V: 'a + Clone> {
    node: &'a RFRNode<K, V>,
    current_child_idx: usize,
    key_char_pos: usize,
}

///! The iterator implementation
pub struct TrieIterator<'a, K: 'a + KeyPrefix + Clone, V: 'a + Clone, M: PushdownStateMachine + Clone> {
    stack: Vec<LookupState<'a, K, V>>,
    match_key_chars: Vec<char>,
    matcher_sm: M
}

impl <'a, K: KeyPrefix + Clone, V: Clone, M: PushdownStateMachine + Clone> TrieIterator<'a, K, V, M> {

    ///! Creates a new iterator
    pub fn new(root: &'a RFRNode<K, V>, match_key: &K) -> Self {
        let it = Self {
            stack: vec![LookupState {
                node: root,
                current_child_idx: 0,
                key_char_pos: 0,
            }],
            match_key_chars: match_key.key_chars(),
            matcher_sm: M::new(),
        };
        it
    }
}

impl <'a, K: 'a + KeyPrefix + Clone, V: 'a + Clone, M: PushdownStateMachine + Clone> Iterator for TrieIterator<'a, K, V, M> {
    type Item = V;

    /// Consume iterator
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.pop() {
                None => { // No more work to do
                    return None;
                }
                Some(ls) => {

                    if ls.current_child_idx >= ls.node.children.len() {
                        // No (more) children. Give up at this level
                        break;
                    }

                    let child = ls.node.children.get(ls.current_child_idx).unwrap();
                    self.matcher_sm.step_in(&child.node_key.seq);
                    let mut advanced = 0 as usize;
                    for ch in self.match_key_chars[ls.key_char_pos..].iter() {
                        if self.matcher_sm.is_sink() {
                            break;
                        }
                        self.matcher_sm.feed(Event::CharIn(*ch));
                        advanced += 1;
                        if !self.matcher_sm.accepts_more() {
                            break;
                        }
                    }
                    if ls.key_char_pos + advanced == self.match_key_chars.len() {
                        if !self.matcher_sm.is_sink() { // Flush (be always greedy)
                            self.matcher_sm.feed(Event::EndOfStream);
                        }

                    }
                    match self.matcher_sm.state() {
                        State::Accepting | State::Expecting => {
                            self.stack.push(LookupState {
                                node: child,
                                current_child_idx: 0,
                                key_char_pos: ls.key_char_pos + advanced
                            });
                        }
                        State::Accepted => {
                            self.matcher_sm.step_out();
                            self.stack.push(LookupState {
                                node: ls.node,
                                current_child_idx: ls.current_child_idx + 1,
                                key_char_pos: ls.key_char_pos
                            });
                            return match &child.value {
                                None => {
                                    None
                                },
                                Some(v) => {
                                    Some(v.clone())
                                }
                            }
                        }
                        State::Rejected => {
                            self.matcher_sm.step_out();
                            self.stack.push(LookupState {
                                node: ls.node,
                                current_child_idx: ls.current_child_idx + 1,
                                key_char_pos: ls.key_char_pos
                            });
                        }
                        State::Beyond => {
                            return None // Won't find anything beyond
                        }
                        State::Failure(reason) => {
                            panic!("Internal error: {}", reason);
                        }
                    }
                }
            }
        }
        return None;
    }
}