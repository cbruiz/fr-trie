//! The Trie trait(s)
use std::marker::PhantomData;
use std::slice::Iter;
use serde::{Serialize, Deserialize};
use crate::node::RFRNode;
use crate::key::{TrieKey, KeyPrefix};
use crate::matcher::PushdownStateMachine;


#[derive(Clone, Serialize, Deserialize)]
pub struct Trie<K, V> where
    K: KeyPrefix + Clone, V: Clone
{
    size: usize,
    node: RFRNode<K, V>,
    _phantom_k: PhantomData<K>,
    _phantom_v: PhantomData<V>,
}

impl<K: KeyPrefix + Clone, V: Clone> Trie<K, V>
{
    /// Creates new trie
    pub fn new()  -> Self {
        Self {
            size: 0,
            node: RFRNode::new(),
            _phantom_k: Default::default(),
            _phantom_v: Default::default()
        }
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) -> Option<V>  {
        let result = self.node.insert(TrieKey::new(key), Some(value));
        if result.is_none() {
            self.size += 1;
        }
        result
    }

    #[inline]
    pub fn get<M: PushdownStateMachine + Clone>(&self, key: &K) -> Option<V>  {
        self.node.get::<M>(key)
    }

    #[inline]
    /// Inmutable slice iterator
    pub fn iter(&self) -> Iter<'_, Box<RFRNode<K, V>>> {
        self.node.children.iter()
    }

    #[inline]
    pub fn foreach<F>(&self, f: F) -> ()
        where F: Fn((usize, &K, &Option<V>))
    {
        for item in self.iter() {
            f( (1 as usize, &item.node_key.key, &item.value) );
            item.foreach(2, &f);
        }
    }
}
