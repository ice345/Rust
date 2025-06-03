mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

use anyhow::Result;
use clap::Parser;
use crate::args::Args;


fn main() -> Result<()> {
    // 解析命令行参数
    let args = Args::parse();
    
    // 执行相应的命令
    match args.command {
        args::Command::Encode { file_path, chunk_type, message, output } => {
            commands::encode::encode(file_path, chunk_type, message, output)?;
        }
        args::Command::Decode { file_path, chunk_type } => {
            commands::decode::decode(file_path, chunk_type)?;
        }
        args::Command::Remove { file_path, chunk_type } => {
            commands::remove::remove(file_path, chunk_type)?;
        }
        args::Command::Print { file_path } => {
            commands::print::print(file_path)?;
        }
    }

    // 返回成功
    Ok(())
}