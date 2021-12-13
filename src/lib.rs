//! A generic fuzzy compressed radix trie implementation.

pub mod trie;
pub mod key;
pub mod node;
pub mod matcher;
pub mod iterator;
pub mod glob;

#[cfg(test)]
mod tests {
    use crate::trie::Trie;
    use crate::glob::GlobMatcher;
    use crate::glob::acl::{AclTrie, Acl, Permissions};

    #[test]
    fn functional_test() {

        let mut trie = Trie::new();
        let a = String::from("a");
        let aa = String::from("aa");
        let aaaa = String::from("aaaa");
        let aabb = String::from("aabb");
        let aacc = String::from("aacc");
        let z = String::from("z");

        trie.insert(z.clone(), z.clone());
        trie.insert(aaaa.clone(), aaaa.clone());
        trie.insert(aabb.clone(), aabb.clone());
        trie.insert(aacc.clone(), aacc.clone());
        trie.insert(a.clone(), a.clone());

        trie.foreach(|tup| {
            let indent= String::from_utf8(vec![b' '; tup.0 *3]).unwrap();
            println!("{} {:?} = {:?}", indent, tup.1, tup.2);
        });

        trie.insert(aa.clone(), aa.clone());

        trie.foreach(|tup| {
            let indent= String::from_utf8(vec![b' '; tup.0 *3]).unwrap();
            println!("{} {:?} = {:?}", indent, tup.1, tup.2);
        });

        assert_eq!(trie.get::<GlobMatcher>(&a).unwrap(), a);
        assert_eq!(trie.get::<GlobMatcher>(&aaaa).unwrap(), aaaa);
        assert_eq!(trie.get::<GlobMatcher>(&aabb).unwrap(), aabb);
        assert_eq!(trie.get::<GlobMatcher>(&aacc).unwrap(), aacc);
        assert_eq!(trie.get::<GlobMatcher>(&z).unwrap(), z);

        let mut trie = AclTrie::new();

        trie.insert(Acl::new("de"), Permissions::READ);
        trie.insert(Acl::new("df"), Permissions::READ);

        trie.insert(Acl::new("a"), Permissions::READ);
        trie.insert(Acl::new("z"), Permissions::READ);
        trie.insert(Acl::new("b"), Permissions::WRITE);
        let x = trie.get::<GlobMatcher>(&Acl::new("b"));
        assert_eq!(Permissions::WRITE, x.unwrap());

        trie.insert(Acl::new("b"), Permissions::OWNER);
        trie.insert(Acl::new("ab"), Permissions::READ);
        trie.insert(Acl::new("aaa0a"), Permissions::READ);
        trie.insert(Acl::new("aaa0b"), Permissions::READ);
        trie.insert(Acl::new("ac"), Permissions::READ);
        trie.insert(Acl::new("j1*"), Permissions::WRITE);
        trie.insert(Acl::new("j0*t"), Permissions::all());
        let x = trie.get::<GlobMatcher>(&Acl::new("j01t"));
        assert_eq!(Permissions::all(), x.unwrap());
        let x = trie.get::<GlobMatcher>(&Acl::new("j1zz"));
        assert_eq!(Permissions::WRITE, x.unwrap());

        let x = trie.get::<GlobMatcher>(&Acl::new("b"));
        assert_eq!(Permissions::OWNER, x.unwrap());
        trie.insert(Acl::new("be"), Permissions::READ);
        trie.insert(Acl::new("bf"), Permissions::READ);

        let x = trie.get::<GlobMatcher>(&Acl::new("b"));
        assert_eq!(Permissions::OWNER, x.unwrap());

        trie.insert(Acl::new("ba"), Permissions::READ);
        trie.insert(Acl::new("aaaab"), Permissions::WRITE);

        trie.foreach(|tup| -> () {
            let indent= String::from_utf8(vec![b' '; tup.0 *3]).unwrap();
            println!("{} {} = {:?}", indent, tup.1, tup.2);
        });

        let x = trie.get::<GlobMatcher>(&Acl::new("aaaab"));
        assert_eq!(Permissions::WRITE, x.unwrap());

        let x = trie.get::<GlobMatcher>(&Acl::new("aaaabb"));
        assert!(x.is_none());
    }

    #[test]
    fn serde_test() {
        let mut trie = AclTrie::new();
        trie.insert(Acl::new("aaaa"), Permissions::empty());
        trie.insert(Acl::new("aaaa"), Permissions::all());
        trie.insert(Acl::new("aabb"), Permissions::READ);
        trie.insert(Acl::new("t"), Permissions::READ);
        assert!(trie.get::<GlobMatcher>(&Acl::new("a")).is_none());
        assert!(trie.get::<GlobMatcher>(&Acl::new("z")).is_none());
        assert!(trie.get::<GlobMatcher>(&Acl::new("aaaaaaaaaa")).is_none());

        let serialized_string = serde_json::ser::to_string_pretty(&trie).unwrap();
        let other_trie = serde_json::de::from_str::<AclTrie>(&serialized_string).unwrap();
        match other_trie.get::<GlobMatcher>(&Acl::new("aaaa")) {
            None => assert!(false, "key not found after deserialization"),
            Some(result) => {
                assert_eq!(result, Permissions::all());
            }
        }
        let serialized_bytes = bincode::serialize(&other_trie).unwrap();
        let another_trie = bincode::deserialize::<AclTrie>(&serialized_bytes).unwrap();
        assert!(another_trie.get::<GlobMatcher>(&Acl::new("aabbcc")).is_none());
    }
}
