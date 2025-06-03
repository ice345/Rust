use std::{convert::TryFrom, fmt::Display, string::FromUtf8Error};
use crc::CRC_32_ISO_HDLC;
use crate::chunk_type::ChunkType;

#[derive(Debug, PartialEq, Eq)]
pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        let crc_calculator = crc::Crc::<u32>::new(&CRC_32_ISO_HDLC);

        let mut crc_input: Vec<u8>= Vec::new();
        crc_input.extend_from_slice(&chunk_type.bytes());
        crc_input.extend_from_slice(&data);

        let crc = crc_calculator.checksum(&crc_input);

        let length = data.len() as u32;

        Chunk {
            length,
            chunk_type,
            data,
            crc,
        }
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn data_as_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.data.clone())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut chunk_bytes: Vec<u8> = Vec::new();

        let length_bytes = self.length.to_be_bytes();
        let chunk_type_bytes = self.chunk_type.bytes();
        let data_bytes = &self.data;
        let crc_bytes = self.crc.to_be_bytes();

        chunk_bytes.extend_from_slice(&length_bytes);
        chunk_bytes.extend_from_slice(&chunk_type_bytes);
        chunk_bytes.extend_from_slice(&data_bytes);
        chunk_bytes.extend_from_slice(&crc_bytes);

        chunk_bytes
    }
}

impl TryFrom<&Vec<u8>> for Chunk {
    type Error = &'static str;

    fn try_from(value: &Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() < 12 {
            return Err("Chunk data is too short");
        }

        let length = u32::from_be_bytes(value[0..4].try_into().unwrap());
        let type_value: [u8; 4] = value[4..8].try_into().expect("unable to try_into array slice in try_from function");
        let chunk_type = ChunkType::try_from(type_value)?;
        let data = value[8..(8 + length as usize)].to_vec();
        let crc = u32::from_be_bytes(value[(8 + length as usize)..].try_into().unwrap());

        let crc_calculator = crc::Crc::<u32>::new(&CRC_32_ISO_HDLC);
        
        let mut crc_input: Vec<u8> = Vec::new();
        crc_input.extend_from_slice(&chunk_type.bytes());
        crc_input.extend_from_slice(&data);
        let crc_check = crc_calculator.checksum(&crc_input);
        if crc == crc_check {
            Ok(Chunk {
                length,
                chunk_type,
                data,
                crc,
            })
        } else {
            Err("CRC mismatch")
        }
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
        "length:  {}
        chunk_type: {}
        data: {:#?}
        crc: {}",
        self.length,
        self.chunk_type,
        self.data,
        self.crc
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        
        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!".as_bytes().to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();
        
        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();
        
        let _chunk_string = format!("{}", chunk);
    }
}