// A modified Radix+Merkle tree to encode the state of all contracts
#![feature(struct_variant)]

use lib::leveldb::database;

type Key   = Vec<u8>;
type Value = Vec<u8>;
type Ref   = Vec<u8>;

enum Node {
  Blank,
  Leaf{ key: Key, value: Value },
  Extension{ key: Key, value: Ref },
  Branch{ refs: [Ref, ..16], value: Value },
}

impl Node {
  fn isTerminal(&self) -> bool {
    match *self {
      Blank | Leaf{..} => true,
      Extension{..} | Branch{..} => false
    }
  }
}

struct Trie {
  database: database,
  root_id: Ref
}

impl Trie {
  pub fn new(database: database, root_id: Ref) -> Trie {
    Trie {
      database: database,
      root_id: root_id
    }
  }

  pub fn get(&self, key: Key) -> Value {
    self.get_from_node(self.root_id, key)
  }

  pub fn set(&self, key: Key, value: Value) {
    self.set_from_node(self.root_id, key, value)
  }

  pub fn remove(&self, key: Key) {
    self.set_from_node(self.root_id, key, Value(vec![]))
  }



  fn get_node(&self, node_id: Ref) -> Node {
    self.database.get(node_id.unwrap())
  }

  fn get_from_node(&self, node_id: Ref, search: Key) -> Value {
    let node = self.get_node(node_id);
    let null_vec = vec![0u8, ..32];
    match node {
      Blank => null_vec,

      Leaf{key, value} => {
        if key == search { value }
        else { null_vec }
      },

      Extension{key, value} => {
        // XXX
        self.get_from_node(value, search.split())
      }

      Branch{refs, value} => {
        if search.len() == 0 {
          node.value
        } else {
          let child_id = refs[search[0]];
          self.get_from_node(child_id, search.slice_from(1).to_vec())
        }
      }
    }
  }

  fn set_from_node(&self, node_id: Ref, key: Key, value: Value) -> Node {
    let node = self.get_node(node_id);
    match node {
      Blank => {
        [pack_nibbles(with_terminator(key)), value.unwrap()]
      },

      Leaf => {

      },

      Extension => {

      },

      Branch => {
        if key == "" {
          node.value = value
        } else {
          let child = self.update_and_delete_storage()
        }
      }
    }
  }

  fn update_and_delete_storage(&self, node: Node, key: Key, value: Value) {

  }
}
