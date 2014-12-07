// The purpose of RLP is to {en|de}code arbitrarily nested arrays of binary data
// specs: https://github.com/ethereum/wiki/wiki/RLP

use std::num::Int;
use std::io::BufReader;
use std::io::extensions::u64_to_be_bytes;
use self::RlpEncodable::{Binary, List};

#[deriving(PartialEq, Eq, Show)]
pub enum RlpEncodable {
  Binary(Vec<u8>),
  List(Vec<RlpEncodable>)
}

#[deriving(PartialEq, Eq, Show)]
pub struct RlpDecodable(Vec<u8>);

pub trait Rlpable {
  fn encode(self) -> RlpEncodable;
  fn decode(from: RlpDecodable) -> Self;
}

static BINARY_OFFSET: u8 = 128;
static LIST_OFFSET: u8   = 192;
static LENGTH_RANGE: u8  = 55;

impl RlpEncodable {
  pub fn encode(self) -> RlpDecodable {
    RlpDecodable(
      match self {
        Binary(v) => {
          if v.len() == 1 && v[0] < BINARY_OFFSET { v }
          else { RlpEncodable::encode_next_length(v.len(), BINARY_OFFSET) + v }
        },

        List(v) => {
          let mut data:Vec<u8> = Vec::new();
          for item in v.into_iter() {
            data.push_all(item.encode().to_vec().as_slice());
          }
          RlpEncodable::encode_next_length(data.len(), LIST_OFFSET) + data
        }
      }
    )
  }

  fn encode_next_length(length: uint, offset: u8) -> Vec<u8> {
    if length <= LENGTH_RANGE as uint {
      return vec![length as u8 + offset];
    }
    for length_of_length in range(0u, 8) {
      if length < 32u.pow(length_of_length + 1) {
        let mut data = vec![length_of_length as u8 + offset + LENGTH_RANGE];
        u64_to_be_bytes(length as u64, length_of_length, |v| data.push_all(v));
        return data;
      }
    }
    panic!()
  }
}



impl RlpDecodable {
  pub fn new(vec: Vec<u8>) -> RlpDecodable {
    RlpDecodable(vec)
  }

  pub fn to_vec(self) -> Vec<u8> {
    let RlpDecodable(vec) = self;
    vec
  }

  pub fn decode(self) -> RlpEncodable {
    let vec = self.to_vec();
    let mut reader = BufReader::new(vec.as_slice());
    RlpDecodable::decode_with_bufreader(&mut reader)
  }

  fn decode_with_bufreader(reader: &mut BufReader) -> RlpEncodable {
    match reader.read_byte() {
      Ok(byte) if byte < BINARY_OFFSET => {
        Binary(vec![byte])
      },

      Ok(byte) if byte < LIST_OFFSET => {
        let length = RlpDecodable::decode_next_length(reader, byte, BINARY_OFFSET);
        Binary(reader.read_exact(length).unwrap())
      },

      Ok(byte) => {
        let mut res:Vec<RlpEncodable> = Vec::new();
        let length = RlpDecodable::decode_next_length(reader, byte, LIST_OFFSET);
        let initial_pos = reader.tell().unwrap() as uint;
        while (reader.tell().unwrap() as uint) < initial_pos + length {
          res.push(RlpDecodable::decode_with_bufreader(reader));
        }
        List(res)
      }

      Err(_) => {
        panic!()
      },
    }
  }

  fn decode_next_length(reader: &mut BufReader, byte:u8, offset: u8) -> uint {
    if byte <= (offset + LENGTH_RANGE) {
      (byte - offset) as uint
    } else {
      let length_of_length = (byte - offset - LENGTH_RANGE) as uint;
      reader.read_be_uint_n(length_of_length).unwrap() as uint
    }
  }
}

#[cfg(test)]
mod tests {
  use std::vec;
  use super::{RlpEncodable, RlpDecodable};
  use super::RlpEncodable::{Binary, List};

  macro_rules! s(($s:expr) => (Binary(String::from_str($s).into_bytes())))

  macro_rules! l(($($e:expr),*) => (List(vec!($($e),*))))

  fn generate_pairs() -> vec::MoveItems<(RlpEncodable, RlpDecodable)> {
    let lorem = "Lorem ipsum dolor sit amet, consectetur adipisicing elit";
    (vec![
      (s!(""),                   RlpDecodable(vec![0x80])),
      (s!("\x0f"),               RlpDecodable(vec![0x0f])),
      (s!("\x04\x00"),           RlpDecodable(vec![0x82, 0x04, 0x00])),
      (s!("dog"),                RlpDecodable(vec![0x83, 0x64, 0x6f, 0x67])),
      (l![],                     RlpDecodable(vec![0xc0])),
      (l![s!("cat"), s!("dog")], RlpDecodable(vec![0xc8, 0x83, 0x63, 0x61, 0x74, 0x83, 0x64, 0x6f, 0x67])),
      (s!(lorem),                RlpDecodable(vec![0xb8, 0x38] + String::from_str(lorem).into_bytes())),
      (
        l![l![], l![l![]], l![l![], l![l![]]]],
        RlpDecodable(vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0])
      ),
    ]).into_iter()
  }

  #[test]
  fn rlp_encodage() {
    for (a, b) in generate_pairs() {
      assert!(a.encode() == b);
    }
  }

  #[test]
  fn rlp_decodage() {
    for (b, a) in generate_pairs() {
      assert!(a.decode() == b);
    }
  }
}
