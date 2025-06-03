use std::{fs, path::PathBuf};
use anyhow::Result;

use crate::png::Png;

/// 打印PNG文件中的所有chunk

pub fn print(
    file_path: PathBuf,
) -> Result<()> {
    // 读取PNG文件
    let file_data = fs::read(&file_path)?;
    
    // 创建Png对象
    let png = Png::try_from(file_data.as_slice()).unwrap();

    // 打印所有chunk的信息
    for chunk in png.chunks() {
        println!("Chunk Type: {:?}", chunk.chunk_type());
        println!("Chunk Data: {:?}", String::from_utf8_lossy(chunk.data()));
        println!("-----------------------------");
    }
    
    Ok(())
}