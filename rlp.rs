// The purpose of RLP is to {en|de}code arbitrarily nested arrays of binary data
// specs: https://github.com/ethereum/wiki/wiki/%5BEnglish%5D-RLP

use std::num;
use std::io::BufReader;
use std::io::extensions::u64_to_be_bytes;

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
            let lengthOfLength = (reader.read_u8().unwrap() - LIST_OFFSET - LENGTH_OFFSET) as uint;
            reader.read_be_uint_n(lengthOfLength).unwrap() as uint
          };
          counter += length;
          res.push(Rlp::decode_with_bufreader(reader, length));
        }
      }
    }
    if res.len() == 1 { res.pop().unwrap() } else { List(res) }
  }
}



fn main() {

}
