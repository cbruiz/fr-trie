//! The Trie Key trait
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::matcher::{MatchType, StateSequence};

/// The Trie Key prefix trait
pub trait KeyPrefix {

    fn key_chars(&self) -> Vec<char>;

    fn key_len(&self) -> usize;

    fn empty() -> Self;

    fn new_from_key_prefix(&self, index: usize) -> Self;

    fn new_from_postfix(&self, index: usize) -> Self;

    #[inline]
    fn compiled(&self) -> Vec<Arc<StateSequence>> {
        let mut state_seq = Vec::new();
        let mut buff: Vec<char> = Vec::new();
        for ch in self.key_chars() {
            buff.push(ch);
        }
        if !buff.is_empty() {
            state_seq.push(Arc::new(StateSequence {
                match_type: MatchType::Literal,
                sequence: buff,
            }));
        }
        state_seq
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrieKey<K> {
    pub(crate) key: K,
    pub(crate) seq: Vec<Arc<StateSequence>>,
}

impl <K: KeyPrefix> TrieKey<K> {

    #[inline]
    pub fn new(key: K) -> Self {
        Self {
            seq: key.compiled(),
            key,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.seq.is_empty()
    }

    #[inline]
    pub fn lcp(&self, other: &Self) -> (usize, bool, bool) {
        let mut lcp = 0;
        let w0l = self.key.key_len();
        let w1l = other.key.key_len();
        let mlen = std::cmp::min(w0l, w1l);
        let mut w0i = self.key.key_chars().into_iter();
        let mut w1i = other.key.key_chars().into_iter();
        let mut preceeding = true;
        for _ in 0 .. mlen {
            let char0 = w0i.next().unwrap();
            let char1 = w1i.next().unwrap();
            if char0.eq(&char1) {
                lcp += 1;
            }
            else {
                if char1.lt(&char0) {
                    preceeding = false;
                }
                break;
            }
        }
        let full_match = w0l == w1l && lcp == w0l;
        (lcp, preceeding, full_match)
    }
}

impl KeyPrefix for String {

    #[inline]
    fn key_chars(&self) -> Vec<char> {
        self.chars().into_iter().collect()
    }

    #[inline]
    fn key_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn empty() -> Self {
        String::new()
    }

    #[inline]
    fn new_from_key_prefix(&self, index: usize) -> Self {
        self[..index].to_string()
    }

    #[inline]
    fn new_from_postfix(&self, index: usize) -> Self {
        self[index..].to_string()
    }
}
