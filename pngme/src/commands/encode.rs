use std::path::PathBuf;
use std::fs;
use std::convert::TryFrom;
use anyhow::Result;

use crate::chunk_type::ChunkType;
use crate::chunk::Chunk;
use crate::png::Png;

pub fn encode(
    file_path: PathBuf,
    chunk_type: ChunkType,
    message: String,
    output_path: Option<PathBuf>
) -> Result<()> {
    // 读取PNG文件
    let file_data = fs::read(&file_path)?;
    let mut png = Png::try_from(file_data.as_slice()).unwrap();
    
    // 创建新的chunk
    let chunk = Chunk::new(chunk_type, message.as_bytes().to_vec());
    
    // 添加chunk到PNG
    png.append_chunk(chunk);
    
    // 确定输出路径
    let out_path = match output_path {
        Some(path) => path,
        None => file_path
    };
    
    // 写回文件
    fs::write(out_path, png.as_bytes())?;
    
    Ok(())
}