//! 国际象棋游戏的基础类型定义

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// 表示棋子的类型
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// 表示棋子的颜色
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// 表示一个棋子，包含类型和颜色
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
            AIDifficulty::Easy => 2000,    // 2秒
            AIDifficulty::Medium => 4000,  // 4秒
            AIDifficulty::Hard => 8000,    // 8秒
            AIDifficulty::Expert => 16000, // 16秒
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineType {
    BuiltIn,
    Stockfish,
    Lc0,
}

#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub mv: Move,
    pub san: String,
    pub fen_after: String,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Playing,
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReviewTab {
    Report,
    Analysis,
}

#[derive(Debug, Clone)]
pub struct AnalysisLine {
    pub eval_cp: i32,
    pub best_move_uci: String,
}

#[derive(Debug, Clone)]
pub struct PositionAnalysis {
    pub lines: Vec<AnalysisLine>,
    pub depth: u32,
    pub verification_passes: u8,
    pub stability_pct: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveClassification {
    Brilliant,
    Critical,
    Best,
    Excellent,
    Okay,
    Inaccuracy,
    Mistake,
    Blunder,
}

impl MoveClassification {
    #[allow(clippy::too_many_arguments)]
    pub fn from_engine_context(
        loss_cp: i32,
        gain_cp: i32,
        move_rank: Option<usize>,
        top2_gap_cp: Option<i32>,
        material_delta_cp: i32,
        mover_eval_before_cp: i32,
        mover_eval_after_cp: i32,
        remaining_material_cp: i32,
    ) -> Self {
        let rank = move_rank.unwrap_or(99);
        let is_best = rank == 0;
        let is_second = rank == 1;
        let is_top3 = rank <= 2;

        let winning_mate_before = mover_eval_before_cp >= 29000;
        let winning_mate_after = mover_eval_after_cp >= 29000;
        let losing_mate_before = mover_eval_before_cp <= -29000;
        let losing_mate_after = mover_eval_after_cp <= -29000;

        if !losing_mate_before && losing_mate_after {
            return MoveClassification::Blunder;
        }
        if winning_mate_before && !winning_mate_after {
            return if mover_eval_after_cp <= -28000 {
                MoveClassification::Blunder
            } else {
                MoveClassification::Mistake
            };
        }

        let in_opening = remaining_material_cp >= 6200;
        let in_endgame = remaining_material_cp <= 2900;
        let blunder_th = if in_endgame {
            240
        } else if in_opening {
            340
        } else {
            300
        };
        let mistake_th = if in_endgame {
            130
        } else if in_opening {
            185
        } else {
            160
        };
        let inaccuracy_th = if in_endgame {
            65
        } else if in_opening {
            95
        } else {
            80
        };

        if loss_cp >= blunder_th {
            return MoveClassification::Blunder;
        }
        if loss_cp >= mistake_th {
            return MoveClassification::Mistake;
        }
        if loss_cp >= inaccuracy_th {
            return MoveClassification::Inaccuracy;
        }

        let top2_gap = top2_gap_cp.unwrap_or(0);
        let is_sacrifice = material_delta_cp <= -180;
        let forcing_gap = if in_endgame { 110 } else { 140 };
        let is_forcing = top2_gap >= forcing_gap;
        let is_stable = loss_cp <= if in_endgame { 8 } else { 10 };
        let is_strong = loss_cp <= if in_opening { 24 } else { 20 };

        if !winning_mate_before && winning_mate_after && is_best {
            return if is_sacrifice {
                MoveClassification::Brilliant
            } else {
                MoveClassification::Critical
            };
        }
        if losing_mate_before && !losing_mate_after && is_best {
            return MoveClassification::Critical;
        }

        if is_best && is_sacrifice && is_strong && (gain_cp >= 25 || is_forcing) {
            return MoveClassification::Brilliant;
        }
        if is_best && is_stable && is_forcing && gain_cp >= -10 {
            return MoveClassification::Critical;
        }
        if is_best && loss_cp <= 14 {
            return MoveClassification::Best;
        }
        if (is_best && loss_cp <= 30) || (is_second && loss_cp <= 18 && gain_cp >= -35) {
            return MoveClassification::Excellent;
        }
        if (is_top3 && loss_cp <= 55) || loss_cp <= 70 {
            return MoveClassification::Okay;
        }

        MoveClassification::Inaccuracy
    }

    pub fn icon(&self) -> &'static str {
        match self {
            MoveClassification::Brilliant => "!!",
            MoveClassification::Critical => "!?",
            MoveClassification::Best => "★",
            MoveClassification::Excellent => "👍",
            MoveClassification::Okay => "=",
            MoveClassification::Inaccuracy => "?!",
            MoveClassification::Mistake => "?",
            MoveClassification::Blunder => "??",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MoveClassification::Brilliant => "Brilliant",
            MoveClassification::Critical => "Critical",
            MoveClassification::Best => "Best",
            MoveClassification::Excellent => "Excellent",
            MoveClassification::Okay => "Okay",
            MoveClassification::Inaccuracy => "Inaccuracy",
            MoveClassification::Mistake => "Mistake",
            MoveClassification::Blunder => "Blunder",
        }
    }

    pub fn color32(&self) -> [u8; 3] {
        match self {
            MoveClassification::Brilliant => [30, 170, 255],
            MoveClassification::Critical => [170, 120, 255],
            MoveClassification::Best => [0, 180, 0],
            MoveClassification::Excellent => [80, 190, 120],
            MoveClassification::Okay => [138, 170, 142],
            MoveClassification::Inaccuracy => [230, 180, 50],
            MoveClassification::Mistake => [230, 120, 50],
            MoveClassification::Blunder => [220, 40, 40],
        }
    }
}

impl AIDifficulty {
    pub fn stockfish_skill_level(&self) -> u32 {
        match self {
            AIDifficulty::Easy => 3,
            AIDifficulty::Medium => 8,
            AIDifficulty::Hard => 14,
            AIDifficulty::Expert => 20,
        }
    }

    pub fn stockfish_depth(&self) -> u32 {
        match self {
            AIDifficulty::Easy => 5,
            AIDifficulty::Medium => 10,
            AIDifficulty::Hard => 15,
            AIDifficulty::Expert => 20,
        }
    }
}

impl PieceType {
    pub fn material_value(&self) -> i32 {
        match self {
            PieceType::Pawn => 1,
            PieceType::Knight => 3,
            PieceType::Bishop => 3,
            PieceType::Rook => 5,
            PieceType::Queen => 9,
            PieceType::King => 0,
        }
    }

    /// Sort order for captured piece display (queen first, then rook, bishop, knight, pawn)
    pub fn capture_display_order(&self) -> u8 {
        match self {
            PieceType::Queen => 0,
            PieceType::Rook => 1,
            PieceType::Bishop => 2,
            PieceType::Knight => 3,
            PieceType::Pawn => 4,
            PieceType::King => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_classification_brilliant() {
        let class =
            MoveClassification::from_engine_context(9, 45, Some(0), Some(170), -220, 35, 80, 5600);
        assert_eq!(class, MoveClassification::Brilliant);
    }

    #[test]
    fn test_move_classification_critical() {
        let class =
            MoveClassification::from_engine_context(6, 0, Some(0), Some(180), 0, 20, 18, 5600);
        assert_eq!(class, MoveClassification::Critical);
    }

    #[test]
    fn test_move_classification_best() {
        let class =
            MoveClassification::from_engine_context(8, 5, Some(0), Some(40), 0, 15, 10, 5600);
        assert_eq!(class, MoveClassification::Best);
    }

    #[test]
    fn test_move_classification_excellent() {
        let class =
            MoveClassification::from_engine_context(18, -8, Some(1), Some(40), 0, 25, 7, 5600);
        assert_eq!(class, MoveClassification::Excellent);
    }

    #[test]
    fn test_move_classification_okay() {
        let class =
            MoveClassification::from_engine_context(52, -15, Some(2), Some(25), 0, 20, -32, 5600);
        assert_eq!(class, MoveClassification::Okay);
    }

    #[test]
    fn test_move_classification_blunder() {
        let class = MoveClassification::from_engine_context(
            420,
            -420,
            Some(4),
            Some(30),
            0,
            120,
            -300,
            5600,
        );
        assert_eq!(class, MoveClassification::Blunder);
    }

    #[test]
    fn test_move_classification_losing_mate_is_blunder() {
        let class = MoveClassification::from_engine_context(
            20,
            -20,
            Some(0),
            Some(40),
            0,
            80,
            -29500,
            5000,
        );
        assert_eq!(class, MoveClassification::Blunder);
    }

    #[test]
    fn test_move_classification_finding_mate_is_critical() {
        let class = MoveClassification::from_engine_context(
            2,
            32000,
            Some(0),
            Some(160),
            0,
            20,
            29550,
            5000,
        );
        assert_eq!(class, MoveClassification::Critical);
    }
}
