//! 国际象棋AI模块
//! 包含AI搜索算法、评估函数和移动排序

use crate::board::Board;
use crate::types::*;
use std::collections::HashMap;
use std::time::Instant;

/// 置换表条目
#[derive(Clone)]
struct TranspositionEntry {
    depth: u32,
    score: i32,
    best_move: Option<Move>,
    node_type: NodeType,
}

#[derive(Clone)]
enum NodeType {
    Exact,      // 精确值
    LowerBound, // α截断
    UpperBound, // β截断
}

/// 优化后的AI结构
#[derive(Clone)]
pub struct ChessAI {
    max_depth: u32,
    transposition_table: HashMap<u64, TranspositionEntry>,
    pub time_limit: u64,
    pub nodes_searched: u64,
    zobrist_pieces: [[[u64; 2]; 6]; 64], // [square][piece_type][color]
    zobrist_turn: u64,
    zobrist_castling: [u64; 4], // [white_king, white_queen, black_king, black_queen]
}

impl ChessAI {
    pub fn new(depth: u32) -> Self {
        let mut ai = ChessAI {
            max_depth: depth,
            transposition_table: HashMap::new(),
            time_limit: match depth {
                2 => 200,
                4 => 800,
                6 => 3000,
                8 => 8000,
                _ => 1000,
            },
            nodes_searched: 0,
            zobrist_pieces: [[[0u64; 2]; 6]; 64],
            zobrist_turn: 0,
            zobrist_castling: [0u64; 4],
        };

        // 初始化Zobrist哈希表
        ai.init_zobrist();
        ai
    }

    fn init_zobrist(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // 为每个棋子位置生成随机数
        for square in 0..64 {
            for piece_type in 0..6 {
                for color in 0..2 {
                    (square * 12 + piece_type * 2 + color).hash(&mut hasher);
                    self.zobrist_pieces[square][piece_type][color] = hasher.finish();
                    hasher = DefaultHasher::new();
                }
            }
        }

        // 生成其他哈希值
        999999u64.hash(&mut hasher);
        self.zobrist_turn = hasher.finish();

        for i in 0..4 {
            hasher = DefaultHasher::new();
            (888888u64 + i as u64).hash(&mut hasher);
            self.zobrist_castling[i] = hasher.finish();
        }
    }

    fn get_board_hash(&self, board: &Board) -> u64 {
        let mut hash = 0u64;

        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = board.get_piece((row, col)) {
                    let square = row * 8 + col;
                    let piece_type = match piece.piece_type {
                        PieceType::Pawn => 0,
                        PieceType::Rook => 1,
                        PieceType::Knight => 2,
                        PieceType::Bishop => 3,
                        PieceType::Queen => 4,
                        PieceType::King => 5,
                    };
                    let color = match piece.color {
                        Color::White => 0,
                        Color::Black => 1,
                    };
                    hash ^= self.zobrist_pieces[square][piece_type][color];
                }
            }
        }

        // 添加其他状态到哈希
        if !board.white_king_moved {
            hash ^= self.zobrist_castling[0];
        }
        if !board.white_rook_a_moved {
            hash ^= self.zobrist_castling[1];
        }
        if !board.black_king_moved {
            hash ^= self.zobrist_castling[2];
        }
        if !board.black_rook_a_moved {
            hash ^= self.zobrist_castling[3];
        }

        hash
    }

    /// 获取最佳走法
    pub fn get_best_move(&mut self, board: &Board, color: Color) -> Option<Move> {
        self.iterative_deepening(board, color)
    }

    /// 迭代深化搜索
    fn iterative_deepening(&mut self, board: &Board, color: Color) -> Option<Move> {
        let start_time = Instant::now();
        let mut best_move = None;

        // 清空置换表以避免内存过多使用
        if self.transposition_table.len() > 100000 {
            self.transposition_table.clear();
        }

        // 从深度1开始，逐步加深
        for depth in 1..=self.max_depth {
            if start_time.elapsed().as_millis() > self.time_limit as u128 {
                break;
            }

            self.nodes_searched = 0;
            let result = self.search_depth(board, depth, color, start_time);

            if let Some(mv) = result {
                best_move = Some(mv);

                // 如果剩余时间不足，提前结束
                if start_time.elapsed().as_millis() > (self.time_limit / 2) as u128 {
                    break;
                }
            }
        }

        best_move
    }

    /// 在指定深度搜索
    fn search_depth(
        &mut self,
        board: &Board,
        depth: u32,
        color: Color,
        start_time: Instant,
    ) -> Option<Move> {
        let mut moves = board.generate_moves(color);
        if moves.is_empty() {
            return None;
        }

        // 移动排序
        self.advanced_move_ordering(&mut moves, board);

        let mut best_move = moves[0];
        let mut best_score = if color == Color::White {
            i32::MIN
        } else {
            i32::MAX
        };

        for mv in moves {
            // 检查时间限制
            if start_time.elapsed().as_millis() > self.time_limit as u128 {
                break;
            }

            let mut new_board = board.clone();
            new_board.make_move(mv);

            let score = self.minimax_with_tt(
                &new_board,
                depth - 1,
                i32::MIN,
                i32::MAX,
                color == Color::Black,
                start_time,
            );

            if (color == Color::White && score > best_score)
                || (color == Color::Black && score < best_score)
            {
                best_score = score;
                best_move = mv;
            }
        }

        Some(best_move)
    }

    /// 带置换表的minimax搜索
    fn minimax_with_tt(
        &mut self,
        board: &Board,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
        maximizing: bool,
        start_time: Instant,
    ) -> i32 {
        // 时间检查
        if start_time.elapsed().as_millis() > self.time_limit as u128 {
            return board.evaluate();
        }

        self.nodes_searched += 1;

        if depth == 0 {
            return board.evaluate();
        }

        let board_hash = self.get_board_hash(board);

        // 查找置换表
        if let Some(entry) = self.transposition_table.get(&board_hash) {
            if entry.depth >= depth {
                match entry.node_type {
                    NodeType::Exact => return entry.score,
                    NodeType::LowerBound => alpha = alpha.max(entry.score),
                    NodeType::UpperBound => beta = beta.min(entry.score),
                }
                if alpha >= beta {
                    return entry.score;
                }
            }
        }

        let color = if maximizing {
            Color::White
        } else {
            Color::Black
        };
        let mut moves = board.generate_moves(color);

        if moves.is_empty() {
            if board.is_in_check(color) {
                return if maximizing {
                    -100000 + depth as i32
                } else {
                    100000 - depth as i32
                };
            } else {
                return 0; // 和棋
            }
        }

        // 移动排序
        self.advanced_move_ordering(&mut moves, board);

        let original_alpha = alpha;
        let mut best_score = if maximizing { i32::MIN } else { i32::MAX };
        let mut best_move = None;

        for mv in moves {
            let mut new_board = board.clone();
            new_board.make_move(mv);

            let score =
                self.minimax_with_tt(&new_board, depth - 1, alpha, beta, !maximizing, start_time);

            if maximizing {
                if score > best_score {
                    best_score = score;
                    best_move = Some(mv);
                }
                alpha = alpha.max(score);
            } else {
                if score < best_score {
                    best_score = score;
                    best_move = Some(mv);
                }
                beta = beta.min(score);
            }

            if beta <= alpha {
                break; // Alpha-beta剪枝
            }
        }

        // 存储到置换表
        let node_type = if best_score <= original_alpha {
            NodeType::UpperBound
        } else if best_score >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };

        self.transposition_table.insert(
            board_hash,
            TranspositionEntry {
                depth,
                score: best_score,
                best_move,
                node_type,
            },
        );

        best_score
    }

    /// 高级移动排序
    fn advanced_move_ordering(&self, moves: &mut [Move], board: &Board) {
        moves.sort_by_cached_key(|mv| {
            let mut score = 0;

            // 1. 置换表中的最佳移动
            let board_hash = self.get_board_hash(board);
            if let Some(entry) = self.transposition_table.get(&board_hash) {
                if entry.best_move == Some(*mv) {
                    score += 10000;
                }
            }

            // 2. 吃子移动 (MVV-LVA)
            if let Some(victim) = board.get_piece(mv.to) {
                let victim_value = self.piece_value(victim.piece_type);
                let attacker_value = self.piece_value(board.get_piece(mv.from).unwrap().piece_type);
                score += victim_value * 10 - attacker_value;
            }

            // 3. 将军移动
            let mut temp_board = board.clone();
            temp_board.make_move(*mv);
            let opponent_color = match board.get_piece(mv.from).unwrap().color {
                Color::White => Color::Black,
                Color::Black => Color::White,
            };
            if temp_board.is_in_check(opponent_color) {
                score += 500;
            }

            // 4. 城堡移动
            if let Some(piece) = board.get_piece(mv.from) {
                if piece.piece_type == PieceType::King
                    && (mv.to.1 as i32 - mv.from.1 as i32).abs() == 2
                {
                    score += 300;
                }
            }

            // 5. 中心控制
            let center_bonus = match mv.to {
                (3, 3) | (3, 4) | (4, 3) | (4, 4) => 50,
                (2, 2)
                | (2, 3)
                | (2, 4)
                | (2, 5)
                | (3, 2)
                | (3, 5)
                | (4, 2)
                | (4, 5)
                | (5, 2)
                | (5, 3)
                | (5, 4)
                | (5, 5) => 20,
                _ => 0,
            };
            score += center_bonus;

            -score // 降序排列
        });
    }

    fn piece_value(&self, piece_type: PieceType) -> i32 {
        match piece_type {
            PieceType::Pawn => 100,
            PieceType::Knight => 320,
            PieceType::Bishop => 330,
            PieceType::Rook => 500,
            PieceType::Queen => 900,
            PieceType::King => 20000,
        }
    }
}

// 为Board实现评估函数
impl Board {
    /// 改进的评估函数
    pub fn evaluate(&self) -> i32 {
        let mut score = 0;

        // 1. 基础子力价值
        score += self.material_evaluation();

        // 2. 位置评估
        score += self.positional_evaluation();

        // 3. 国王安全
        score += self.king_safety_evaluation();

        // 4. 机动性评估
        score += self.mobility_evaluation();

        score
    }

    fn material_evaluation(&self) -> i32 {
        let mut score = 0;

        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = self.get_piece((row, col)) {
                    let value = match piece.piece_type {
                        PieceType::Pawn => 100,
                        PieceType::Knight => 320,
                        PieceType::Bishop => 330,
                        PieceType::Rook => 500,
                        PieceType::Queen => 900,
                        PieceType::King => 20000,
                    };

                    match piece.color {
                        Color::White => score += value,
                        Color::Black => score -= value,
                    }
                }
            }
        }

        score
    }

    fn positional_evaluation(&self) -> i32 {
        let mut score = 0;

        let pawn_table = [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [50, 50, 50, 50, 50, 50, 50, 50],
            [10, 10, 20, 30, 30, 20, 10, 10],
            [5, 5, 10, 25, 25, 10, 5, 5],
            [0, 0, 0, 20, 20, 0, 0, 0],
            [5, -5, -10, 0, 0, -10, -5, 5],
            [5, 10, 10, -20, -20, 10, 10, 5],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ];

        let knight_table = [
            [-50, -40, -30, -30, -30, -30, -40, -50],
            [-40, -20, 0, 0, 0, 0, -20, -40],
            [-30, 0, 10, 15, 15, 10, 0, -30],
            [-30, 5, 15, 20, 20, 15, 5, -30],
            [-30, 0, 15, 20, 20, 15, 0, -30],
            [-30, 5, 10, 15, 15, 10, 5, -30],
            [-40, -20, 0, 5, 5, 0, -20, -40],
            [-50, -40, -30, -30, -30, -30, -40, -50],
        ];

        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = self.get_piece((row, col)) {
                    let position_bonus = match piece.piece_type {
                        PieceType::Pawn => {
                            if piece.color == Color::White {
                                pawn_table[7 - row][col]
                            } else {
                                pawn_table[row][col]
                            }
                        }
                        PieceType::Knight => {
                            if piece.color == Color::White {
                                knight_table[7 - row][col]
                            } else {
                                knight_table[row][col]
                            }
                        }
                        _ => 0,
                    };

                    match piece.color {
                        Color::White => score += position_bonus,
                        Color::Black => score -= position_bonus,
                    }
                }
            }
        }

        score
    }

    fn king_safety_evaluation(&self) -> i32 {
        let mut score = 0;

        if self.is_in_check(Color::White) {
            score -= 50;
        }
        if self.is_in_check(Color::Black) {
            score += 50;
        }

        score
    }

    fn mobility_evaluation(&self) -> i32 {
        let white_moves = self.generate_moves(Color::White).len() as i32;
        let black_moves = self.generate_moves(Color::Black).len() as i32;

        (white_moves - black_moves) * 5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::types::{Color, Piece, PieceType};

    #[test]
    fn test_ai_new() {
        let ai = ChessAI::new(3);
        assert_eq!(ai.max_depth, 3);
        // Fix: Correct assertion syntax
        assert!(ai.zobrist_turn != 0, "Zobrist keys should be initialized");
    }

    #[test]
    fn test_initial_board_evaluation_is_zero() {
        let board = Board::new();
        assert_eq!(board.evaluate(), 0);
    }

    #[test]
    fn test_ai_finds_a_move_in_initial_position() {
        let mut ai = ChessAI::new(1); // Use depth 1 for speed
        let board = Board::new();
        let best_move = ai.get_best_move(&board, Color::White);
        assert!(best_move.is_some());
    }

    #[test]
    fn test_evaluation_for_checkmate() {
        let mut board = Board::new();
        board.squares = [[None; 8]; 8]; // Clear board

        // Fix: Set up a real checkmate position.
        // Black king at a8, White queen at a7, White king at b6 (protecting the queen).
        board.set_piece((0, 0), Some(Piece::new(PieceType::King, Color::Black)));
        board.set_piece((1, 0), Some(Piece::new(PieceType::Queen, Color::White)));
        board.set_piece((2, 1), Some(Piece::new(PieceType::King, Color::White))); // Moved from c6 to b6
        board.black_king_pos = (0, 0);
        board.white_king_pos = (2, 1);

        // Black is in checkmate, so there are no legal moves.
        let moves = board.generate_moves(Color::Black);
        assert!(moves.is_empty(), "In a checkmate position, there should be no legal moves.");
        assert!(board.is_in_check(Color::Black));

        // The evaluation for a checkmated position should be extremely low for the losing side.
        // The minimax function should return a value close to -100000.
        let mut ai = ChessAI::new(2);
        let score = ai.minimax_with_tt(&board, 2, i32::MIN, i32::MAX, false, std::time::Instant::now());

        // Since it's black's turn (minimizing player) and they are checkmated, the score
        // should be a large positive number (good for white).
        assert!(score > 90000, "Score was {}, expected > 90000 for a checkmated position", score);
    }
}
