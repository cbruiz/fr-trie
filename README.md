# fr-trie

[![CI](https://github.com/cbruiz/fr-trie/actions/workflows/rust.yml/badge.svg)](https://github.com/cbruiz/fr-trie/actions/workflows/rust.yml)
[![CodeCov](https://codecov.io/gh/cbruiz/fr-trie/branch/main/graph/badge.svg)](https://codecov.io/gh/cbruiz/fr-trie/branch/main)
[![](https://img.shields.io/crates/v/fr-trie.svg)](https://crates.io/crates/fr-trie)

[![](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)

This is a generic fuzzy and compact Trie implementation focused on:
* Small memory footprint.
* Efficient caching.
  

Trie is keyed by lists of type `K`, which can be anything satisfying the `KeyPrefix` and `Clone` traits.

This structure is thought to be used in some particular scenarios where:
* The keys prefixes are string based and highly repeating.
* The volume of keys to store is not very big.
* A fuzzy and customizable key matching strategy is needed.

For more information, see the [API documentation][doc].

# Usage

Add `fr-trie` to your `Cargo.toml`.

```toml
[dependencies]
fr-trie = "*"
```
# Examples 
Glob matching with multiple results
```rust
use fr_trie::glob::acl::{Acl, AclTrie, Permissions};
use fr_trie::glob::GlobMatcher;

let mut trie = AclTrie::new();
trie.insert(Acl::new("/path/*"), Permissions::READ);
trie.insert(Acl::new("/path/to/resource"), Permissions::WRITE);

// Multiget example 1
let result = trie.get_merge::<GlobMatcher>(&Acl::new("/path/to/anything"));
if let Some(value) = result {
    if value == Permissions::READ {
        println!("Expecting /path/* wilcard key is accessed");
    }
}

// Multiget example 2
let result = trie.get_merge::<GlobMatcher>(&Acl::new("/path/to/anything"));
if let Some(value) = result {
    if value == (Permissions::READ | Permissions::WRITE) {
        println!("Expecting both /path/* wilcard key and /path/to/resource is accessed");
    }
}

// Dump trie structure
trie.foreach(|tup| {
    let indent= String::from_utf8(vec![b' '; tup.0 *3]).unwrap();
    println!("{} {} = {:?}", indent, tup.1, tup.2);
});
```

# Caveats
* Still not fully-productive

# Similar work
* [Radix Trie][radix-trie] – Fast generic radix trie implemented in Rust
* [Sequence Trie][sequence-trie] – Ergonomic trie data structure

# License

Licensed under [MIT license](http://opensource.org/licenses/MIT)

[doc]: https://docs.rs/fr-trie/
[radix-trie]: https://github.com/michaelsproul/rust_radix_trie
[sequence-trie]: https://github.com/michaelsproul/rust_sequence_trie
