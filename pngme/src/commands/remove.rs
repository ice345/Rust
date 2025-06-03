use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::chunk_type::ChunkType;
use crate::png::Png;

/// 删除PNG文件中的指定chunk

pub fn remove(
    file_path: PathBuf,
    chunk_type: ChunkType,
) -> Result<()> {
    // 读取PNG文件
    let file_data = fs::read(&file_path)?;
    
    // 创建Png对象
    let mut png = Png::try_from(file_data.as_slice()).unwrap();

    // 转换chunk_type为&str
    let chunk_type_str = chunk_type.to_string();
    
    // 删除指定类型的chunk
    let _ =png.remove_first_chunk(&chunk_type_str);
    
    // 写回文件
    fs::write(file_path, png.as_bytes())?;
    
    Ok(())
}