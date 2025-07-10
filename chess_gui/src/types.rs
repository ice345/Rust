//! 国际象棋游戏的基础类型定义

#[derive(Debug, Clone, Copy, PartialEq)]
/// 表示棋子的类型
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// 表示棋子的颜色
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// 表示一个棋子，包含类型和颜色
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// 表示一步棋，包括起始位置、目标位置和可能的升变
pub struct Move {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub promotion: Option<PieceType>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// 表示游戏状态
pub enum GameState {
    Playing,
    WhiteWins,
    BlackWins,
    Draw,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// AI难度级别
pub enum AIDifficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl AIDifficulty {
    pub fn get_depth(&self) -> u32 {
        match self {
            AIDifficulty::Easy => 2,
            AIDifficulty::Medium => 4,
            AIDifficulty::Hard => 6,
            AIDifficulty::Expert => 8,
        }
    }
    
    pub fn get_time_limit(&self) -> u64 {
        match self {
            AIDifficulty::Easy => 200,    // 0.2秒
            AIDifficulty::Medium => 800,  // 0.8秒
            AIDifficulty::Hard => 3000,   // 3秒
            AIDifficulty::Expert => 8000, // 8秒
        }
    }
    
    pub fn to_string(&self) -> &str {
        match self {
            AIDifficulty::Easy => "Easy",
            AIDifficulty::Medium => "Medium",
            AIDifficulty::Hard => "Hard",
            AIDifficulty::Expert => "Expert",
        }
    }
}

impl Color {
    /// 获取相反的颜色
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Piece {
    /// 创建一个新的棋子
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Self { piece_type, color }
    }
}
