#![feature(macro_rules)]
#[allow(dead_code)]

mod rlp;
mod trie;

#[cfg(not(test))]
fn main() {
  println!("Ethereum");
}
