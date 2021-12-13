//! The Trie internal node implementation
use std::marker::PhantomData;
use std::slice::Iter;
use serde::{Serialize, Deserialize};
use crate::key::{TrieKey, KeyPrefix};
use crate::iterator::{TrieIterator};
use crate::matcher::PushdownStateMachine;

#[derive(Clone, Serialize, Deserialize)]
pub struct RFRNode<K: KeyPrefix + Clone, V: Clone> {
    pub node_key: TrieKey<K>,

    /// The key and value stored at this node.
    pub value: Option<V>,

    pub children: Vec<Box<RFRNode<K, V>>>,

    #[serde(skip)]
    _phantom_k: PhantomData<K>,

    #[serde(skip)]
    _phantom_v: PhantomData<V>,
}

impl<K, V> RFRNode<K, V> where K: KeyPrefix + Clone, V: Clone {

    #[inline]
    pub fn new() -> Self {
        Self {
            node_key: TrieKey::new(K::empty()),
            value: None,
            children: Vec::new(),
            _phantom_k: Default::default(),
            _phantom_v: Default::default(),
        }
    }

    #[inline]
    pub fn new_aux(node_key: TrieKey<K>) -> Self {
        Self {
            node_key,
            value: None,
            children: Vec::new(),
            _phantom_k: Default::default(),
            _phantom_v: Default::default(),
        }
    }

    #[inline]
    pub fn new_leaf_with_prefix(node_key: TrieKey<K>, value: V) -> Self {
        Self {
            node_key,
            value: Some(value),
            children: Vec::new(),
            _phantom_k: Default::default(),
            _phantom_v: Default::default(),
        }
    }

    #[inline]
    pub fn strip_prefix(&mut self, prefix_len: usize) {
        self.node_key = TrieKey::new(self.node_key.key.new_from_key_prefix(prefix_len));
    }

    #[inline]
    pub fn insert(&mut self, key: TrieKey<K>, value: Option<V>) -> Option<V>  {
        
        let mut prev_insert_index = 0 as usize;
        let mut insert_index = 0 as usize;
        let mut lcp = 0 as usize;
        let mut index = 0 as usize;
        let mut full_match = false;

        for child in self.children.iter() {
            let (item_lcp, item_is_preceeding, item_is_full_match) = key.lcp(&child.node_key);
            prev_insert_index = index;
            lcp = item_lcp;
            full_match = item_is_full_match;
            if !item_is_preceeding {
                index += 1;
                insert_index = index;
                if lcp > 0 {
                    break;
                }
            }
            else {
                break;
            }
        }

        if full_match {
            // Node already exists and is a full match
            // Alternatives:
            // 1. Colliding node is leaf -> Just replace
            // 2. Colliding node is aux -> TBD
            self.children.get_mut(prev_insert_index).unwrap().value = value.clone();
        }
        else {
            if lcp > 0 { // Partial collision
                // Child is leaf: Split
                let prev_node = self.children.get_mut(prev_insert_index).unwrap();

                let common_prefix = prev_node.node_key.key.new_from_key_prefix(lcp);
                let prev_node_postfix = prev_node.node_key.key.new_from_postfix(lcp);

                let new_node_postfix = key.key.new_from_postfix(lcp);
                let new_node_postfix_kl = new_node_postfix.key_len();
                let prev_node_postfix_kl = prev_node_postfix.key_len();
                if prev_node_postfix_kl != 0 {
                    let mut prev_node = self.children.remove(prev_insert_index);
                    prev_node.strip_prefix(lcp);
                    let mut aux: Box<RFRNode<K, V>> = if new_node_postfix_kl != 0 || value.is_none() {
                        Box::new(RFRNode::new_aux(
                            TrieKey::new(common_prefix)
                        ))
                    }
                    else {
                        Box::new(RFRNode::new_leaf_with_prefix(
                            TrieKey::new(common_prefix),
                            value.as_ref().unwrap().clone()
                            ))
                    };
                    aux.insert(TrieKey::new(prev_node_postfix), prev_node.value);
                    for _idx in 0..prev_node.children.len() {
                        let prev_node_child = prev_node.children.remove(0);
                        aux.children.get_mut(0).unwrap().children.push(prev_node_child);
                    }
                    if new_node_postfix_kl != 0 {
                        aux.insert(TrieKey::new(new_node_postfix), value.clone());
                    }
                    self.children.insert(prev_insert_index, aux);
                }
                else {
                    prev_node.insert(TrieKey::new(new_node_postfix), value.clone());
                }
            }
            else {
                // No collition -> Just insert as a leaf
                if value.is_some() {
                    self.children.insert(insert_index, Box::new(RFRNode::new_leaf_with_prefix(
                        key,
                        value.as_ref().unwrap().clone()
                    )));
                    return None;
                }
                else {
                    self.children.insert(insert_index, Box::new(RFRNode::new_aux(
                        key,
                    )));
                }
            }
        }
        return value;
    }

    #[inline]
    pub fn lookup<M: PushdownStateMachine + Clone>(&self, match_key: &K) -> TrieIterator<'_, K, V, M> {
        TrieIterator::new(self, match_key)
    }

    #[inline]
    pub fn get<M: PushdownStateMachine + Clone>(&self, key: &K) -> Option<V> {
        for value in self.lookup::<M>(key) {
            return Some(value);
        }
        None
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, Box<RFRNode<K, V>>> {
        self.children.iter()
    }

    pub fn foreach<F>(&self, level: usize, f: &F) -> ()
        where F: Fn((usize, &K, &Option<V>))
    {
        for item in self.iter() {
            f( (level, &item.node_key.key, &item.value) );
            item.foreach( level + 1, f);
        }
    }
}



