// A modified Radix+Merkle tree to encode the state of all contracts
#![feature(struct_variant)]

use db::DB;

struct Key(Vec<u8>);
struct Value(Vec<u8>);
struct Ref(Vec<u8>);

impl Ref {
  fn start_with(&self, search: Vec<u8>) -> bool {
    false
  }
  fn get_node(&self, db: &DB) -> Node {
    Blank
  }
}

enum Node {
  Blank,
  Leaf{ key: Key, val: Value },
  Extension{ key: Key, val: Ref },
  Branch{ refs: [Ref, ..16], val: Value },
}

impl Node {
  fn isTerminal(&self) -> bool {
    match *self {
      Blank | Leaf{..} => { true },
      Extension{..} | Branch{..} => { false }
    }
  }
}

struct Trie {
  database: DB,
  root_hash: Ref
}

impl Trie {
  pub fn get(&self, key: Key) -> Value {
    Trie::get_from_node(self.root_hash.get_node(), key)
  }

  pub fn set(&self, key: Key, val: Value) {

  }

  pub fn update(&self, key: Key, new_val: Value) {

  }

  pub fn delete(&self, key: Key) {

  }

  fn get_from_node(db: &DB, node: &Node, search: Key) -> Value {
    let node = node_id.get_node();
    match *node {
      Leaf{key, val} if key == search => {
        val
      },
      Extension{key, val} if key.start_with(search) {
        Trie::get_from_node(val.get_node(db))
      }
      Branch{refs, val} {
        // TODO
      }
      _ => {
        vec![0u8, ..32]
      }
    }
  }
}

fn main() {

}
