use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

/**# 说明
```
位置	字符	意义	解释
第1字节	I	是否为关键块（Critical）	大写表示关键块，必须被解析才能理解图片
第2字节	H	是否为公开块（Public）	大写表示这是 PNG 标准公开定义的类型，小写是私有块
第3字节	D	是否被保留（Reserved）	必须是大写，供将来标准扩展使用（当前应为大写）
第4字节	R	是否安全复制（Safe-to-copy）	小写表示可以安全复制，即使解码器不理解这个块
```
 */

/**# 合法性
```
根据 PNG 规范，一个 chunk type 是合法的前提包括：
正好是 4 个 ASCII 字符。
每个字符必须是英文字母：A-Z 或 a-z。
第 3 个字符（保留位）必须是大写字母。
其他位的大小写有意义，但不影响是否合法（只影响语义）。
```
 */

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ChunkType([u8; 4]); 


impl ChunkType {
    /// 返回4个字节的ascii序列
    pub fn bytes(&self) -> [u8; 4] {
        self.0
    }

    /// 判断是否符合全是ascii的规则
    pub fn is_valid(&self) -> bool {
        self.0.iter().all(|&x| x.is_ascii()) && self.0[2].is_ascii_uppercase()
            && self.0[0].is_ascii_alphabetic()
            && self.0[1].is_ascii_alphabetic()
            && self.0[3].is_ascii_alphabetic()
    }

    /// 大写则表示是关键块
    pub fn is_critical(&self) -> bool {
        self.0[0].is_ascii_uppercase()
    }


    /// 大写则表示是公开块
    pub fn is_public(&self) -> bool {
        self.0[1].is_ascii_uppercase()
    }

    /// 大写则表示是被保留
    pub fn is_reserved_bit_valid(&self) -> bool {
        self.0[2].is_ascii_uppercase()
    }

    /// 小写则表示是可以安全复制
    pub fn is_safe_to_copy(&self) -> bool {
        self.0[3].is_ascii_lowercase()
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = &'static str;
    // type Error = Box<dyn std::error::Error>;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        if value.iter().all(|&x| x.is_ascii()) {
            Ok(ChunkType(value))
        } else {
            Err("Invalid byte array")
        }
    }
}

impl FromStr for ChunkType {
    type Err = &'static str;
    // type Err = Box<dyn std::error::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 || !s.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err("Invalid string length or non-alphabetic characters");
        }

        let bytes = s.as_bytes();
        let mut arr = [0; 4];
        arr.copy_from_slice(bytes);

        ChunkType::try_from(arr)
    }
}

impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}