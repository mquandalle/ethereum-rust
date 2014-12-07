// A modified Radix+Merkle tree to encode the state of all contracts

extern crate "leveldb-rs" as leveldb;

use std::path::posix::Path;
use self::leveldb::DB;
use std::slice::bytes;
use rlp::{Rlpable, RlpEncodable, RlpDecodable};
use rlp::RlpEncodable::{Binary, List};
use self::TrieValue::{Reference, InlineValue};
use self::Node::{Blank, Extension, Branch};



#[deriving(PartialEq, Eq)]
struct Key {
  value: Vec<u8>,
  nibble: u8
}

// impl Rlpable for Key {
//   fn encode(self) -> RlpEncodable {
//     Binary(self.value)
//   }

//   fn decode(source: RlpDecodable) -> Key {
//     Key{value: source.to_vec(), nibble: 0}
//   }
// }

type Ref = [u8, ..32];

#[deriving(PartialEq, Eq)]
enum TrieValue {
  Reference(Ref),
  InlineValue(RlpDecodable)
}

// impl Rlpable for TrieValue {
//   fn encode(self) -> RlpEncodable {
//     match self {
//       Reference(r) => Binary(r.to_vec()),
//       InlineValue(r) => Binary(r.to_vec())
//     }
//   }

//   fn decode(source: RlpDecodable) -> TrieValue {
//     match source.to_vec().len() {
//       x if x < 32 => InlineValue(source),
//       x if x == 32 => Reference({
//         let mut x: [u8, ..32] = [0, ..32];
//         bytes::copy_memory(x.as_mut_slice(), source.to_vec().as_slice());
//         x
//       }),
//       _ => panic!("Too large trie value")
//     }
//   }
// }

#[deriving(PartialEq, Eq)]
enum Node {
  Blank,
  Extension {
    key: Key,
    value: TrieValue
  },
  Branch {
    branchs: [Option<TrieValue>, ..16],
    value: Option<Key> // Should be <InlineValue>
  },
}

// impl Rlpable for Node {
//   fn encode(self) -> RlpEncodable {
//     match self {
//       Blank => Binary(vec![]),
//       Extension{ key: k, value: v } => List(vec![k.encode(), v.encode()]),
//       Branch{ branchs: branchs, value: v} => {
//         let l:Vec<RlpEncodable> = Vec::new();
//         for b in branchs.iter() {
//           match *b {
//             Some(x) => { l.push(x.encode()); }
//             None => { l.push(Binary(vec![])); }
//           }
//         }
//         match v {
//           Some(x) => { l.push(x.encode()); }
//           None => { l.push(Binary(vec![])); }
//         }
//         List(l)
//       }
//     }
//   }

//   fn decode(source: RlpDecodable) -> Node {
//     Blank
//   }
// }

// impl Node {
//   pub fn from_vec(source: Vec<u8>) -> Node {
//     let rlp_data = Rlp::new(source).decode();
//       match rlp_data {
//         List(v) => {
//           match v.len() {
//             2 => {
//               let key = match v[0] {
//                 Binary(a) => Key::from_vec(a),
//                 List(_) => panic!("Unexpected value")
//               };
//               let value = match v[1] {

//               };

//               Extension{ key: key, value: TrieValue::from_vec(v[1])}
//             },
//             17 => Blank,
//             _ => panic!("Unexpected value")
//           }
//         }
//         Binary(_) => panic!("Unexpected value"),
//       }
//   }
// }

pub struct Trie {
  storage: DB,
  root_hash: Option<Ref>
}

impl Trie {
  fn new(path: &Path, root_hash: Option<Ref>) -> Trie {
    match DB::create(path) {
      Ok(db) => Trie { storage: db, root_hash: root_hash },
      Err(why) => panic!("Error creating DB: {}", why),
    }
  }

  fn get_root_node(&self) -> Node {
    self.get_node(&self.root_hash)
  }

  fn get_node(&self, key: &Option<Ref>) -> Node {
    match key {
      &None => Blank,
      &Some(ref key) => {
        match self.storage.get(key.as_slice()) {
          Err(why) => panic!("Error reading the DB for key {}, {}", key, why),
          Ok(None) => Blank,
          Ok(Some(raw)) => Blank
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::io::TempDir;
  use super::Trie;
  use super::Node::Blank;

  fn get_tmp_trie(name: &str) -> Trie {
    Trie::new(TempDir::new(name).unwrap().path(), None)
  }

  #[test]
  fn get_blank_root_node() {
    assert!(get_tmp_trie("empty").get_root_node() == Blank)
  }
}
