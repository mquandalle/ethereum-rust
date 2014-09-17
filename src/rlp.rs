// The purpose of RLP is to {en|de}code arbitrarily nested arrays of binary data
// specs: https://github.com/ethereum/wiki/wiki/%5BEnglish%5D-RLP

use std::num;
use std::io::BufReader;
use std::io::extensions::u64_to_be_bytes;

pub enum RlpEncodable {
  Binary(Vec<u8>),
  List(Vec<RlpEncodable>)
}

#[deriving(PartialEq)]
#[deriving(Eq)]
pub struct Rlp(Vec<u8>);

static BINARY_OFFSET: u8 = 128;
static LIST_OFFSET: u8   = 192;
static LENGTH_OFFSET: u8 = 55;



impl RlpEncodable {
  pub fn encode(self) -> Rlp {
    Rlp(match self {
      Binary(v) => {
        if v.len() == 1 && v[0] < BINARY_OFFSET { v }
        else { RlpEncodable::encode_length(v.len(), BINARY_OFFSET) + v }
      },

      List(v) => {
        let mut data:Vec<u8> = Vec::new();
        for item in v.into_iter() {
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
    for length_of_length in range(1u, 9) {
      if length < num::pow(32u, length_of_length) {
        let mut data = vec![length_of_length as u8 + offset + LENGTH_OFFSET];
        u64_to_be_bytes(length as u64, length_of_length, |v| data.push_all(v));
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
    let vec = self.into_vec();
    let mut reader = BufReader::new(vec.as_slice());
    Rlp::decode_with_bufreader(&mut reader, vec.len())
  }

  fn decode_with_bufreader(reader: &mut BufReader, limit: uint) -> RlpEncodable {
    let mut counter: uint = 0;
    let mut res:Vec<RlpEncodable> = Vec::new();
    while counter < limit {
      match reader.read_byte() {
        Err(_) => break,

        // Binary
        Ok(byte) if byte < 0xb8 => {
          let length = if byte <= 0x7f {
            0u
          } else {
            (reader.read_u8().unwrap() - BINARY_OFFSET) as uint
          };
          counter += length;
          res.push(Binary(reader.read_exact(length).unwrap()));
        }

        // List
        Ok(byte) => {
          let length = if byte <= 0xbf {
            (reader.read_u8().unwrap() - LIST_OFFSET) as uint
          } else {
            let length_of_length = (reader.read_u8().unwrap() - LIST_OFFSET - LENGTH_OFFSET) as uint;
            reader.read_be_uint_n(length_of_length).unwrap() as uint
          };
          counter += length;
          res.push(Rlp::decode_with_bufreader(reader, length));
        }
      }
    }
    if res.len() == 1 { res.pop().unwrap() } else { List(res) }
  }
}



#[test]
fn rlp_encodage() {
  assert!(Binary(vec![]).encode() == Rlp(vec![0x80]));
  assert!(Binary(String::from_str("dog").into_bytes()).encode() == Rlp(vec![0x83, 0x64, 0x6f, 0x67]));
  assert!(List(vec![Binary(String::from_str("cat").into_bytes()), Binary(String::from_str("dog").into_bytes())]).encode() == Rlp(vec![0xc8, 0x83, 0x63, 0x61, 0x74, 0x83, 0x64, 0x6f, 0x67]));
  assert!(List(vec![]).encode() == Rlp(vec![0xc0]));
  assert!(List(vec![List(vec![]), List(vec![List(vec![])]), List(vec![List(vec![]), List(vec![List(vec![])])])]).encode() == Rlp(vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0]));
}
