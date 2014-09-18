// The purpose of RLP is to {en|de}code arbitrarily nested arrays of binary data
// specs: https://github.com/ethereum/wiki/wiki/RLP

use std::num;
use std::io::BufReader;
use std::io::extensions::u64_to_be_bytes;

#[cfg(test)]
use std::vec;

#[deriving(PartialEq, Eq, Show)]
pub enum RlpEncodable {
  Binary(Vec<u8>),
  List(Vec<RlpEncodable>)
}

#[deriving(PartialEq, Eq, Show)]
pub struct Rlp(Vec<u8>);

static BINARY_OFFSET: u8 = 128;
static LIST_OFFSET: u8   = 192;
static LENGTH_RANGE: u8  = 55;



impl RlpEncodable {
  pub fn encode(self) -> Rlp {
    Rlp(
      match self {
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
      }
    )
  }

  fn encode_length(length: uint, offset: u8) -> Vec<u8> {
    if length <= LENGTH_RANGE as uint {
      return vec![length as u8 + offset];
    }
    for length_of_length in range(0u, 8) {
      if length < num::pow(32u, length_of_length + 1) {
        let mut data = vec![length_of_length as u8 + offset + LENGTH_RANGE];
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
    Rlp::decode_with_bufreader(&mut reader)
  }

  fn decode_with_bufreader(reader: &mut BufReader) -> RlpEncodable {
    match reader.read_byte() {
      Ok(byte) if byte < BINARY_OFFSET => {
        Binary(vec![byte])
      },

      Ok(byte) if byte < LIST_OFFSET => {
        let length = Rlp::get_next_length(reader, byte, BINARY_OFFSET);
        Binary(reader.read_exact(length).unwrap())
      },

      Ok(byte) => {
        let mut res:Vec<RlpEncodable> = Vec::new();
        let length = Rlp::get_next_length(reader, byte, LIST_OFFSET);
        let initial_pos = reader.tell().unwrap() as uint;
        while (reader.tell().unwrap() as uint) < initial_pos + length {
          res.push(Rlp::decode_with_bufreader(reader));
        }
        List(res)
      }

      Err(_) => {
        fail!()
      },
    }
  }

  fn get_next_length(reader: &mut BufReader, byte:u8, offset: u8) -> uint {
    if byte <= (offset + LENGTH_RANGE) {
      (byte - offset) as uint
    } else {
      let length_of_length = (byte - offset - LENGTH_RANGE) as uint;
      reader.read_be_uint_n(length_of_length).unwrap() as uint
    }
  }
}

#[cfg(test)]
macro_rules! s(($s:expr) => (Binary(String::from_str($s).into_bytes())))

#[cfg(test)]
macro_rules! l(($($e:expr),*) => (List(vec!($($e),*))))

#[cfg(test)]
fn generate_pairs() -> vec::MoveItems<(RlpEncodable, Rlp)> {
  let lorem = "Lorem ipsum dolor sit amet, consectetur adipisicing elit";
  (vec![
    (s!(""),                     Rlp(vec![0x80])),
    (s!("\x0f"),                 Rlp(vec![0x0f])),
    (s!("\x04\x00"),             Rlp(vec![0x82, 0x04, 0x00])),
    (s!("dog"),                  Rlp(vec![0x83, 0x64, 0x6f, 0x67])),
    (l![],                       Rlp(vec![0xc0])),
    (l![s!("cat"), s!("dog")],   Rlp(vec![0xc8, 0x83, 0x63, 0x61, 0x74, 0x83, 0x64, 0x6f, 0x67])),
    (s!(lorem),                  Rlp(vec![0xb8, 0x38] + String::from_str(lorem).into_bytes())),
    (
      l![l![], l![l![]], l![l![], l![l![]]]],
      Rlp(vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0])
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
