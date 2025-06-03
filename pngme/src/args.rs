use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::chunk_type::ChunkType;

#[derive(Debug, Parser)]
#[command(
    author = "LJB",
    version = "0.0.1",
    about = "A simple png paser",
    long_about = None
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Encode {
        #[arg(short, long)]
        file_path: PathBuf,

        #[arg(short, long)]
        chunk_type: ChunkType,

        #[arg(short, long)]
        message: String,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Decode {
        #[arg(short, long)]
        file_path: PathBuf,

        #[arg(short, long)]
        chunk_type: ChunkType,
    },
    Remove {
        #[arg(short, long)]
        file_path: PathBuf,

        #[arg(short, long)]
        chunk_type: ChunkType,
    },
    Print {
        #[arg(short, long)]
        file_path: PathBuf,
    }
}