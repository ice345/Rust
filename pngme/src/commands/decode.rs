use std::path::PathBuf;
use anyhow::Result;
use std::fs;

use crate::chunk_type::ChunkType;
use crate::png::Png;

/// 解码PNG文件中的指定chunk

pub fn decode(
    file_path: PathBuf,
    chunk_type: ChunkType,
) -> Result<()> {
    // 读取PNG文件
    let file_data = fs::read(&file_path)?;
    
    // 创建Png对象
    let png = Png::try_from(file_data.as_slice()).unwrap();

    // 转换chunk_type为&str
    let chunk_type_str = chunk_type.to_string();

    // 触发彩蛋的chunk类型
    if chunk_type_str == "bOOm" {
        println!("\n炸弹已激活！倒计时开始...");
        for i in (1..=3).rev() {
            println!("{i}...");
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
        println!("\n💥 轰！💥\n");
        println!("
          .-^^---....,,--       
     _--                  --_  
    <                        >)
    |                         | 
     \\._                   _./  
        ```--. . , ; .--'''       
              | |   |             
           .-=||  | |=-.   
           `-=#$%&%$#=-'   
              | ;  :|     
     _____.,-#%&$@%#&#~,._____
        ");
        return Ok(());
    }

    // 查找指定类型的chunk
    if let Some(chunk) = png.chunk_by_type(&chunk_type_str) {
        // 打印chunk的内容
        println!("Chunk Type: {:?}", chunk.chunk_type());
        println!("Chunk Data: {:?}", String::from_utf8_lossy(chunk.data()));
        
        // 根据chunk类型显示不同的ASCII艺术
        match chunk_type_str.as_str() {
            "ruSt" => println!("
        ╭──────────────────────────╮
        │  成功解码Rust秘密消息！  │
        ╰──────────────────────────╯
             \\
              \\
                 _~^~^~_
             \\) /  o o  \\ (/
               '_   -   _'
               / '-----' \\
            "),
                "pNgE" => println!("
          _____   _   _  _____  __  __  _____
         |  __ \\ | \\ | |/ ____||  \\/  ||  ___|
         | |__) ||  \\| || |  __ | \\  / || |__
         |  ___/ | . ` || | |_ || |\\/| ||  __|
         | |     | |\\  || |__| || |  | || |___
         |_|     |_| \\_| \\_____||_|  |_||_____|
            "),
            _ => {} // 其他chunk类型不显示特殊艺术
        }
    } else {
        println!("No chunk found with type {:?}", chunk_type);
    }
    
    Ok(())
}