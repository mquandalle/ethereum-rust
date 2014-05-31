// The purpose of RLP is to {en|de}code arbitrarily nested arrays of binary data
// specs: https://github.com/ethereum/wiki/wiki/%5BEnglish%5D-RLP

use std::num;
use std::io::extensions::{u64_from_be_bytes, u64_to_be_bytes};

pub enum RlpEncodable {
  Binary(Vec<u8>),
  List(Vec<RlpEncodable>)
}

pub struct Rlp(Vec<u8>);

static BINARY_OFFSET: u8 = 128;
static LIST_OFFSET: u8   = 192;
static LENGTH_OFFSET: u8 = 55;

impl RlpEncodable {
  pub fn encode(self) -> Rlp {
    Rlp(match self {
      Binary(v) => {
        if v.len() == 1 && v.get(0) < &BINARY_OFFSET { v }
        else { RlpEncodable::encode_length(v.len(), BINARY_OFFSET) + v }
      },

      List(v) => {
        let mut data:Vec<u8> = Vec::new();
        for item in v.move_iter() {
          data.push_all_move(item.encode().into_vec());
        }
        RlpEncodable::encode_length(data.len(), LIST_OFFSET) + data
      }
    })
  }

  fn encode_length(length: uint, offset: u8) -> Vec<u8> {
    if length < 56 {
      return vec![length as u8 + offset];
    }
    for lengthOfLength in range(1u, 9) {
      if length < num::pow(32u, lengthOfLength) {
        let mut data = vec![lengthOfLength as u8 + offset + LENGTH_OFFSET];
        u64_to_be_bytes(length as u64, lengthOfLength, |v| data.push_all(v));
        return data;
      }
    }
    fail!()
  }
}

impl Rlp {
  fn into_vec(self) -> Vec<u8> {
    let Rlp(vec) = self;
    vec
  }

  pub fn decode(self) -> RlpEncodable {
    let mut res:Vec<RlpEncodable> = Vec::new();
    let Rlp(vec) = self;
    let mut reader = vec.move_iter();
    loop {
      match reader.next() {
        None => break,
        Some(byte) => {
          let val = if byte < 0xb8 {
            let length = if byte <= 0x7f {
              0
            } else {
              (reader.next().unwrap() - BINARY_OFFSET) as uint
            };
            Binary(reader.take(length).collect())

          } else {
            let length = if byte <= 0xbf {
              (reader.next().unwrap() - LIST_OFFSET) as uint
            } else {
              let lengthOfLength = (reader.next().unwrap() - LIST_OFFSET - LENGTH_OFFSET) as uint;
              let lengthB = reader.take(lengthOfLength).collect::<Vec<u8>>().as_slice();
              u64_from_be_bytes(lengthB, 0, lengthOfLength) as uint
            };
            Rlp(reader.take(length).collect()).decode()
          };
          res.push(val);
        }
      }
    }

    return if res.len() == 1 { res.pop().unwrap() } else { List(res) };
  }
}

fn main() {

}