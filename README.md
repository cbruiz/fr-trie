# fr-trie

[![Rust](https://github.com/cbruiz/fr-trie/actions/workflows/rust.yml/badge.svg)](https://github.com/cbruiz/fr-trie/actions/workflows/rust.yml)
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
