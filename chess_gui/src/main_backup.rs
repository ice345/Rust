use eframe::egui;
use egui::{Color32, Pos2, Rect, Sense, Vec2};
use std::time::Instant;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents the type of chess piece
enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents the color of a chess piece
enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents a chess piece with its type and color
struct Piece {
    piece_type: PieceType,
    color: Color,
}

#[derive(Debug, Clone)]
/// Represents the chess board with pieces and their positions(with kings and rooks moved flags)
struct Board {
    squares: [[Option<Piece>; 8]; 8],
    white_king_pos: (usize, usize),
    black_king_pos: (usize, usize),
    white_king_moved: bool,
    black_king_moved: bool,
    white_rook_a_moved: bool,
    white_rook_h_moved: bool,
    black_rook_a_moved: bool,
    black_rook_h_moved: bool,
    en_passant_target: Option<(usize, usize)>, // 过路兵目标位置
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents a move in chess, including promotion if applicable
struct Move {
    from: (usize, usize),
    to: (usize, usize),
    promotion: Option<PieceType>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents the state of the game
enum GameState {
    Playing,
    WhiteWins,
    BlackWins,
    Draw,
}

/// AI难度级别
#[derive(Debug, Clone, Copy, PartialEq)]
enum AIDifficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

impl AIDifficulty {
    fn get_depth(&self) -> u32 {
        match self {
            AIDifficulty::Easy => 2,
            AIDifficulty::Medium => 4,
            AIDifficulty::Hard => 6,
            AIDifficulty::Expert => 8,
        }
    }
    
    fn get_time_limit(&self) -> u64 {
        match self {
            AIDifficulty::Easy => 200,    // 0.2秒
            AIDifficulty::Medium => 800,  // 0.8秒
            AIDifficulty::Hard => 3000,   // 3秒
            AIDifficulty::Expert => 8000, // 8秒
        }
    }
    
    fn to_string(&self) -> &str {
        match self {
            AIDifficulty::Easy => "Easy",
            AIDifficulty::Medium => "Medium",
            AIDifficulty::Hard => "Hard",
            AIDifficulty::Expert => "Expert",
        }
    }
}

/// Main application structure that holds the board, AI, and game state
struct ChessApp {
    board: Board,
    ai: ChessAI,
    current_player: Color,
    selected_square: Option<(usize, usize)>,
    valid_moves: Vec<Move>,
    game_state: GameState,
    status_message: String,
    ai_thinking: bool,
    ai_move_start: Option<Instant>,
    ai_difficulty: AIDifficulty,
    promotion_pending: Option<Move>, // 待升变的走法
}

#[allow(dead_code)] // Allow unused code warnings for this file
impl Board {
    // coordinates origin at the bottom left corner (0, 0) for white pieces
    fn new() -> Self {
        let mut board = Board {
            squares: [[None; 8]; 8],
            white_king_pos: (7, 4),
            black_king_pos: (0, 4),
            white_king_moved: false,
            black_king_moved: false,
            white_rook_a_moved: false,
            white_rook_h_moved: false,
            black_rook_a_moved: false,
            black_rook_h_moved: false,
            en_passant_target: None,
        };

        board.setup_initial_position();
        board
    }

    fn setup_initial_position(&mut self) {
        // White pieces
        self.squares[7][0] = Some(Piece {
            piece_type: PieceType::Rook,
            color: Color::White,
        });
        self.squares[7][1] = Some(Piece {
            piece_type: PieceType::Knight,
            color: Color::White,
        });
        self.squares[7][2] = Some(Piece {
            piece_type: PieceType::Bishop,
            color: Color::White,
        });
        self.squares[7][3] = Some(Piece {
            piece_type: PieceType::Queen,
            color: Color::White,
        });
        self.squares[7][4] = Some(Piece {
            piece_type: PieceType::King,
            color: Color::White,
        });
        self.squares[7][5] = Some(Piece {
            piece_type: PieceType::Bishop,
            color: Color::White,
        });
        self.squares[7][6] = Some(Piece {
            piece_type: PieceType::Knight,
            color: Color::White,
        });
        self.squares[7][7] = Some(Piece {
            piece_type: PieceType::Rook,
            color: Color::White,
        });

        for col in 0..8 {
            self.squares[6][col] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: Color::White,
            });
        }

        // Black pieces
        self.squares[0][0] = Some(Piece {
            piece_type: PieceType::Rook,
            color: Color::Black,
        });
        self.squares[0][1] = Some(Piece {
            piece_type: PieceType::Knight,
            color: Color::Black,
        });
        self.squares[0][2] = Some(Piece {
            piece_type: PieceType::Bishop,
            color: Color::Black,
        });
        self.squares[0][3] = Some(Piece {
            piece_type: PieceType::Queen,
            color: Color::Black,
        });
        self.squares[0][4] = Some(Piece {
            piece_type: PieceType::King,
            color: Color::Black,
        });
        self.squares[0][5] = Some(Piece {
            piece_type: PieceType::Bishop,
            color: Color::Black,
        });
        self.squares[0][6] = Some(Piece {
            piece_type: PieceType::Knight,
            color: Color::Black,
        });
        self.squares[0][7] = Some(Piece {
            piece_type: PieceType::Rook,
            color: Color::Black,
        });

        for col in 0..8 {
            self.squares[1][col] = Some(Piece {
                piece_type: PieceType::Pawn,
                color: Color::Black,
            });
        }
    }

    fn get_piece(&self, pos: (usize, usize)) -> Option<Piece> {
        self.squares[pos.0][pos.1]
    }

    fn set_piece(&mut self, pos: (usize, usize), piece: Option<Piece>) {
        self.squares[pos.0][pos.1] = piece;
    }

    fn make_move(&mut self, mv: Move) -> bool {
        let piece = self.get_piece(mv.from);
        if piece.is_none() {
            return false;
        }

        let piece = piece.unwrap();

        // 清除之前的过路兵标记
        self.en_passant_target = None;

        // Handle castling condition
        if piece.piece_type == PieceType::King {
            let col_diff = mv.to.1 as i32 - mv.from.1 as i32;
            if col_diff.abs() == 2 {
                // This is castling
                let (rook_from_col, rook_to_col) = if col_diff > 0 {
                    // King-side
                    (7, 5)
                } else {
                    // Queen-side
                    (0, 3)
                };

                // Move rook
                let rook = self.get_piece((mv.from.0, rook_from_col)).unwrap();
                self.set_piece((mv.from.0, rook_from_col), None);
                self.set_piece((mv.from.0, rook_to_col), Some(rook));
            }

            // Update king position
            match piece.color {
                Color::White => {
                    self.white_king_pos = mv.to;
                    self.white_king_moved = true;
                }
                Color::Black => {
                    self.black_king_pos = mv.to;
                    self.black_king_moved = true;
                }
            }
        }

        // Handle pawn moves
        if piece.piece_type == PieceType::Pawn {
            // 检查是否是双格移动（设置过路兵目标）
            let row_diff = (mv.to.0 as i32 - mv.from.0 as i32).abs();
            if row_diff == 2 {
                // 双格移动，设置过路兵目标位置
                let en_passant_row = if piece.color == Color::White {
                    mv.from.0 - 1
                } else {
                    mv.from.0 + 1
                };
                self.en_passant_target = Some((en_passant_row, mv.from.1));
            }

            // 检查是否是过路兵吃子
            if mv.from.1 != mv.to.1 && self.get_piece(mv.to).is_none() {
                // 这是过路兵吃子，移除被吃的兵
                let captured_pawn_row = mv.from.0;
                self.set_piece((captured_pawn_row, mv.to.1), None);
            }
        }

        // Update rook moved flags
        if piece.piece_type == PieceType::Rook {
            match (piece.color, mv.from) {
                (Color::White, (7, 0)) => self.white_rook_a_moved = true,
                (Color::White, (7, 7)) => self.white_rook_h_moved = true,
                (Color::Black, (0, 0)) => self.black_rook_a_moved = true,
                (Color::Black, (0, 7)) => self.black_rook_h_moved = true,
                _ => {}
            }
        }

        // Handle pawn promotion condition or not
        let final_piece = if piece.piece_type == PieceType::Pawn {
            match (piece.color, mv.to.0) {
                (Color::White, 0) | (Color::Black, 7) => Piece {
                    piece_type: mv.promotion.unwrap_or(PieceType::Queen),
                    color: piece.color,
                },
                _ => piece,
            }
        } else {
            piece
        };

        self.set_piece(mv.from, None);
        self.set_piece(mv.to, Some(final_piece));
        true
    }

    fn is_valid_move(&self, mv: Move, color: Color) -> bool {
        let piece = self.get_piece(mv.from);
        if piece.is_none() || piece.unwrap().color != color {
            return false;
        }

        let piece = piece.unwrap();

        if mv.to.0 >= 8 || mv.to.1 >= 8 {
            return false;
        }

        // Check if the destination square is occupied by a piece of the same color
        if let Some(dest_piece) = self.get_piece(mv.to) {
            if dest_piece.color == color {
                return false;
            }
        }

        match piece.piece_type {
            PieceType::Pawn => self.is_valid_pawn_move(mv, piece.color),
            PieceType::Rook => self.is_valid_rook_move(mv),
            PieceType::Knight => self.is_valid_knight_move(mv),
            PieceType::Bishop => self.is_valid_bishop_move(mv),
            PieceType::Queen => self.is_valid_queen_move(mv),
            PieceType::King => self.is_valid_king_move(mv),
        }
    }

    fn is_valid_pawn_move(&self, mv: Move, color: Color) -> bool {
        let (from_row, from_col) = mv.from;
        let (to_row, to_col) = mv.to;

        // Determine the direction of movement based on color
        let direction = if color == Color::White { -1i32 } else { 1i32 };
        let start_row = if color == Color::White { 6 } else { 1 };

        let row_diff = to_row as i32 - from_row as i32;
        let col_diff = (to_col as i32 - from_col as i32).abs();

        if col_diff == 0 {
            // 直线移动
            if row_diff == direction && self.get_piece(mv.to).is_none() {
                return true;
            }
            if from_row == start_row
                && row_diff == 2 * direction
                && self.get_piece(mv.to).is_none()
                && {
                    let intermediate_row = from_row as i32 + direction;
                    if (0..8).contains(&intermediate_row) {
                        self.get_piece((intermediate_row as usize, from_col))
                            .is_none()
                    } else {
                        false
                    }
                }
            {
                return true;
            }
        } else if col_diff == 1 && row_diff == direction {
            // 斜向攻击
            if let Some(target) = self.get_piece(mv.to) {
                // 普通吃子
                return target.color != color;
            } else if let Some(en_passant_pos) = self.en_passant_target {
                // 过路兵吃子
                return mv.to == en_passant_pos;
            }
        }

        false
    }

    fn is_valid_rook_move(&self, mv: Move) -> bool {
        let (from_row, from_col) = mv.from;
        let (to_row, to_col) = mv.to;

        if from_row != to_row && from_col != to_col {
            return false;
        }

        self.is_path_clear(mv.from, mv.to)
    }

    fn is_valid_knight_move(&self, mv: Move) -> bool {
        let row_diff = (mv.to.0 as i32 - mv.from.0 as i32).abs();
        let col_diff = (mv.to.1 as i32 - mv.from.1 as i32).abs();

        (row_diff == 2 && col_diff == 1) || (row_diff == 1 && col_diff == 2)
    }

    fn is_valid_bishop_move(&self, mv: Move) -> bool {
        let row_diff = (mv.to.0 as i32 - mv.from.0 as i32).abs();
        let col_diff = (mv.to.1 as i32 - mv.from.1 as i32).abs();

        if row_diff != col_diff {
            return false;
        }

        self.is_path_clear(mv.from, mv.to)
    }

    fn is_valid_queen_move(&self, mv: Move) -> bool {
        self.is_valid_rook_move(mv) || self.is_valid_bishop_move(mv)
    }

    fn is_valid_king_move(&self, mv: Move) -> bool {
        let row_diff = (mv.to.0 as i32 - mv.from.0 as i32).abs();
        let col_diff = (mv.to.1 as i32 - mv.from.1 as i32).abs();

        // Regular king move
        if row_diff <= 1 && col_diff <= 1 {
            return true;
        }

        // Check for castling
        if row_diff == 0 && col_diff == 2 {
            return self.is_valid_castling(mv);
        }

        false
    }

    fn is_valid_castling(&self, mv: Move) -> bool {
        let (from_row, from_col) = mv.from;
        let (to_row, to_col) = mv.to;

        // Castling must be on the same rank
        if from_row != to_row {
            return false;
        }

        let piece = self.get_piece(mv.from);
        if piece.is_none() || piece.unwrap().piece_type != PieceType::King {
            return false;
        }

        let king = piece.unwrap();

        // Check if king has moved
        match king.color {
            Color::White => {
                if self.white_king_moved || from_row != 7 || from_col != 4 {
                    return false;
                }
            }
            Color::Black => {
                if self.black_king_moved || from_row != 0 || from_col != 4 {
                    return false;
                }
            }
        }

        // Determine castling side and check rook
        let (rook_col, rook_moved) = if to_col == 6 {
            // King-side castling
            (
                7,
                match king.color {
                    Color::White => self.white_rook_h_moved,
                    Color::Black => self.black_rook_h_moved,
                },
            )
        } else if to_col == 2 {
            // Queen-side castling
            (
                0,
                match king.color {
                    Color::White => self.white_rook_a_moved,
                    Color::Black => self.black_rook_a_moved,
                },
            )
        } else {
            return false;
        };

        if rook_moved {
            return false;
        }

        // Check if rook exists
        if let Some(rook) = self.get_piece((from_row, rook_col)) {
            if rook.piece_type != PieceType::Rook || rook.color != king.color {
                return false;
            }
        } else {
            return false;
        }

        // Check if path is clear between king and its destination
        let start = from_col.min(to_col);
        let end = from_col.max(to_col);
        for col in (start + 1)..end {
            if self.get_piece((from_row, col)).is_some() {
                return false;
            }
        }

        // For queen-side castling, also check if b-file is clear (rook path)
        if to_col == 2 && rook_col == 0 {
            // Check b1/b8 square is empty (between rook and king)
            if self.get_piece((from_row, 1)).is_some() {
                return false;
            }
        }

        // Check if king is in check or passes through check
        if self.is_in_check(king.color) {
            return false;
        }

        // Check intermediate square for check
        let intermediate_col = if to_col == 6 { 5 } else { 3 };
        let mut temp_board = self.clone();
        temp_board.set_piece(mv.from, None);
        temp_board.set_piece((from_row, intermediate_col), Some(king));
        if temp_board.is_in_check(king.color) {
            return false;
        }

        // Check final square for check
        let mut final_board = self.clone();
        final_board.set_piece(mv.from, None);
        final_board.set_piece((from_row, to_col), Some(king));
        if final_board.is_in_check(king.color) {
            return false;
        }

        true
    }

    fn is_path_clear(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let row_dir = (to.0 as i32 - from.0 as i32).signum();
        let col_dir = (to.1 as i32 - from.1 as i32).signum();

        let mut current_row = from.0 as i32 + row_dir;
        let mut current_col = from.1 as i32 + col_dir;

        while current_row != to.0 as i32 || current_col != to.1 as i32 {
            if self
                .get_piece((current_row as usize, current_col as usize))
                .is_some()
            {
                return false;
            }
            current_row += row_dir;
            current_col += col_dir;
        }

        true
    }

    fn generate_moves(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);

        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = self.get_piece((row, col)) {
                    if piece.color == color {
                        self.generate_piece_moves((row, col), piece, &mut moves);
                    }
                }
            }
        }

        // 过滤掉会让己方国王陷入危险的走法
        moves.retain(|&mv| {
            let mut temp_board = self.clone();
            temp_board.make_move(mv);
            !temp_board.is_in_check(color)
        });

        moves
    }

    /// 生成所有原始走法（不过滤安全性，用于AI搜索）
    fn generate_raw_moves(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);

        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = self.get_piece((row, col)) {
                    if piece.color == color {
                        self.generate_piece_moves((row, col), piece, &mut moves);
                    }
                }
            }
        }

        moves
    }

    fn generate_piece_moves(&self, pos: (usize, usize), piece: Piece, moves: &mut Vec<Move>) {
        match piece.piece_type {
            PieceType::Pawn => self.generate_pawn_moves(pos, piece.color, moves),
            PieceType::Rook => {
                self.generate_sliding_moves(pos, &[(0, 1), (0, -1), (1, 0), (-1, 0)], moves)
            }
            PieceType::Bishop => {
                self.generate_sliding_moves(pos, &[(1, 1), (1, -1), (-1, 1), (-1, -1)], moves)
            }
            PieceType::Queen => self.generate_sliding_moves(
                pos,
                &[
                    (0, 1),
                    (0, -1),
                    (1, 0),
                    (-1, 0),
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1),
                ],
                moves,
            ),
            PieceType::Knight => self.generate_knight_moves(pos, moves),
            PieceType::King => self.generate_king_moves(pos, piece.color, moves),
        }
    }

    fn generate_pawn_moves(&self, pos: (usize, usize), color: Color, moves: &mut Vec<Move>) {
        let (row, col) = pos;
        let direction = if color == Color::White { -1i32 } else { 1i32 };
        let start_row = if color == Color::White { 6 } else { 1 };

        // Forward move
        if let Ok(new_row) = (row as i32 + direction).try_into() {
            if new_row < 8 && self.get_piece((new_row, col)).is_none() {
                self.add_pawn_move(pos, (new_row, col), color, moves);

                // Double forward from start
                if row == start_row {
                    if let Ok(double_row) = (row as i32 + 2 * direction).try_into() {
                        if double_row < 8 && self.get_piece((double_row, col)).is_none() {
                            self.add_pawn_move(pos, (double_row, col), color, moves);
                        }
                    }
                }
            }
        }

        // Captures
        for &col_offset in &[-1i32, 1i32] {
            if let (Ok(new_row), Ok(new_col)) = (
                (row as i32 + direction).try_into(),
                (col as i32 + col_offset).try_into(),
            ) {
                if new_row < 8 && new_col < 8 {
                    // 普通吃子
                    if let Some(target) = self.get_piece((new_row, new_col)) {
                        if target.color != color {
                            self.add_pawn_move(pos, (new_row, new_col), color, moves);
                        }
                    }
                    // 过路兵吃子
                    else if let Some(en_passant_pos) = self.en_passant_target {
                        if (new_row, new_col) == en_passant_pos {
                            moves.push(Move {
                                from: pos,
                                to: (new_row, new_col),
                                promotion: None,
                            });
                        }
                    }
                }
            }
        }
    }

    /// Adds a pawn move to the list of moves, handling promotion if applicable
    fn add_pawn_move(
        &self,
        from: (usize, usize),
        to: (usize, usize),
        color: Color,
        moves: &mut Vec<Move>,
    ) {
        // check promotion condition
        if (color == Color::White && to.0 == 0) || (color == Color::Black && to.0 == 7) {
            // Promotion
            for &promotion in &[
                PieceType::Queen,
                PieceType::Rook,
                PieceType::Bishop,
                PieceType::Knight,
            ] {
                moves.push(Move {
                    from,
                    to,
                    promotion: Some(promotion),
                });
            }
        } else {
            moves.push(Move {
                from,
                to,
                promotion: None,
            });
        }
    }

    /// Generates all sliding moves for a piece (rook, bishop, queen)
    fn generate_sliding_moves(
        &self,
        pos: (usize, usize),
        directions: &[(i32, i32)],
        moves: &mut Vec<Move>,
    ) {
        let (row, col) = pos;
        let piece_color = self.get_piece(pos).unwrap().color;

        for &(dr, dc) in directions {
            let mut r = row as i32 + dr;
            let mut c = col as i32 + dc;

            while (0..8).contains(&r) && (0..8).contains(&c) {
                let target_pos = (r as usize, c as usize);

                if let Some(target) = self.get_piece(target_pos) {
                    if target.color != piece_color {
                        moves.push(Move {
                            from: pos,
                            to: target_pos,
                            promotion: None,
                        });
                    }
                    break;
                } else {
                    moves.push(Move {
                        from: pos,
                        to: target_pos,
                        promotion: None,
                    });
                }

                r += dr;
                c += dc;
            }
        }
    }

    fn generate_knight_moves(&self, pos: (usize, usize), moves: &mut Vec<Move>) {
        let (row, col) = pos;
        let piece_color = self.get_piece(pos).unwrap().color;
        let knight_moves = [
            (2, 1),
            (2, -1),
            (-2, 1),
            (-2, -1),
            (1, 2),
            (1, -2),
            (-1, 2),
            (-1, -2),
        ];

        for &(dr, dc) in &knight_moves {
            if let (Ok(new_row), Ok(new_col)) =
                ((row as i32 + dr).try_into(), (col as i32 + dc).try_into())
            {
                if new_row < 8 && new_col < 8 {
                    let target_pos = (new_row, new_col);
                    if let Some(target) = self.get_piece(target_pos) {
                        if target.color != piece_color {
                            moves.push(Move {
                                from: pos,
                                to: target_pos,
                                promotion: None,
                            });
                        }
                    } else {
                        moves.push(Move {
                            from: pos,
                            to: target_pos,
                            promotion: None,
                        });
                    }
                }
            }
        }
    }

    fn generate_king_moves(&self, pos: (usize, usize), color: Color, moves: &mut Vec<Move>) {
        let (row, col) = pos;
        let king_moves = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];

        // Regular king moves
        for &(dr, dc) in &king_moves {
            if let (Ok(new_row), Ok(new_col)) =
                ((row as i32 + dr).try_into(), (col as i32 + dc).try_into())
            {
                if new_row < 8 && new_col < 8 {
                    let target_pos = (new_row, new_col);
                    if let Some(target) = self.get_piece(target_pos) {
                        if target.color != color {
                            moves.push(Move {
                                from: pos,
                                to: target_pos,
                                promotion: None,
                            });
                        }
                    } else {
                        moves.push(Move {
                            from: pos,
                            to: target_pos,
                            promotion: None,
                        });
                    }
                }
            }
        }

        // Castling moves
        if !self.is_in_check(color) {
            match color {
                Color::White if !self.white_king_moved && row == 7 && col == 4 => {
                    // King-side castling
                    if !self.white_rook_h_moved {
                        let castling_move = Move {
                            from: pos,
                            to: (7, 6),
                            promotion: None,
                        };
                        if self.is_valid_castling(castling_move) {
                            moves.push(castling_move);
                        }
                    }
                    // Queen-side castling
                    if !self.white_rook_a_moved {
                        let castling_move = Move {
                            from: pos,
                            to: (7, 2),
                            promotion: None,
                        };
                        if self.is_valid_castling(castling_move) {
                            moves.push(castling_move);
                        }
                    }
                }
                Color::Black if !self.black_king_moved && row == 0 && col == 4 => {
                    // King-side castling
                    if !self.black_rook_h_moved {
                        let castling_move = Move {
                            from: pos,
                            to: (0, 6),
                            promotion: None,
                        };
                        if self.is_valid_castling(castling_move) {
                            moves.push(castling_move);
                        }
                    }
                    // Queen-side castling
                    if !self.black_rook_a_moved {
                        let castling_move = Move {
                            from: pos,
                            to: (0, 2),
                            promotion: None,
                        };
                        if self.is_valid_castling(castling_move) {
                            moves.push(castling_move);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn is_in_check(&self, color: Color) -> bool {
        let king_pos = match color {
            Color::White => self.white_king_pos,
            Color::Black => self.black_king_pos,
        };

        let opponent_color = match color {
            Color::White => Color::Black,
            Color::Black => Color::White,
        };

        // 更高效的检查方法

        // 1. 检查对方骑士攻击
        let knight_moves = [
            (2, 1),
            (2, -1),
            (-2, 1),
            (-2, -1),
            (1, 2),
            (1, -2),
            (-1, 2),
            (-1, -2),
        ];

        for &(dr, dc) in &knight_moves {
            if let (Ok(r), Ok(c)) = (
                (king_pos.0 as i32 + dr).try_into(),
                (king_pos.1 as i32 + dc).try_into(),
            ) {
                if r < 8 && c < 8 {
                    if let Some(piece) = self.get_piece((r, c)) {
                        if piece.color == opponent_color && piece.piece_type == PieceType::Knight {
                            return true;
                        }
                    }
                }
            }
        }

        // 2. 检查各个方向的攻击
        let directions = [
            // 水平和垂直方向 (车和皇后)
            (0, 1),
            (1, 0),
            (0, -1),
            (-1, 0),
            // 对角线方向 (象和皇后)
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];

        for &(dr, dc) in &directions {
            let mut r = king_pos.0 as i32 + dr;
            let mut c = king_pos.1 as i32 + dc;

            while (0..8).contains(&r) && (0..8).contains(&c) {
                if let Some(piece) = self.get_piece((r as usize, c as usize)) {
                    if piece.color == opponent_color {
                        let is_sliding_attack = match piece.piece_type {
                            PieceType::Queen => true,
                            PieceType::Rook => dr == 0 || dc == 0,
                            PieceType::Bishop => dr != 0 && dc != 0,
                            _ => false,
                        };

                        if is_sliding_attack {
                            return true;
                        }
                    }
                    // 遇到任何棋子就停止这个方向的检查
                    break;
                }
                r += dr;
                c += dc;
            }
        }

        // 3. 检查兵的攻击
        let pawn_dirs = if color == Color::White {
            // 检查白王是否被黑兵攻击 - 黑兵在白王上方，攻击方向向下
            [(-1, -1), (-1, 1)]
        } else {
            // 检查黑王是否被白兵攻击 - 白兵在黑王下方，攻击方向向上
            [(1, -1), (1, 1)]
        };

        for &(dr, dc) in &pawn_dirs {
            if let (Ok(r), Ok(c)) = (
                (king_pos.0 as i32 + dr).try_into(),
                (king_pos.1 as i32 + dc).try_into(),
            ) {
                if r < 8 && c < 8 {
                    if let Some(piece) = self.get_piece((r, c)) {
                        if piece.color == opponent_color && piece.piece_type == PieceType::Pawn {
                            return true;
                        }
                    }
                }
            }
        }

        // 4. 检查对方国王相邻的格子
        let king_dirs = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];

        for &(dr, dc) in &king_dirs {
            if let (Ok(r), Ok(c)) = (
                (king_pos.0 as i32 + dr).try_into(),
                (king_pos.1 as i32 + dc).try_into(),
            ) {
                if r < 8 && c < 8 {
                    if let Some(piece) = self.get_piece((r, c)) {
                        if piece.color == opponent_color && piece.piece_type == PieceType::King {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// 改进的评估函数
    fn evaluate(&self) -> i32 {
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
struct ChessAI {
    max_depth: u32,
    transposition_table: HashMap<u64, TranspositionEntry>,
    time_limit: u64,
    nodes_searched: u64,
    zobrist_pieces: [[[u64; 2]; 6]; 64], // [square][piece_type][color]
    zobrist_turn: u64,
    zobrist_castling: [u64; 4], // [white_king, white_queen, black_king, black_queen]
}

impl ChessAI {
    fn new(depth: u32) -> Self {
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
    fn search_depth(&mut self, board: &Board, depth: u32, color: Color, start_time: Instant) -> Option<Move> {
        let mut moves = board.generate_moves(color);
        if moves.is_empty() {
            return None;
        }
        
        // 移动排序
        self.advanced_move_ordering(&mut moves, board);
        
        let mut best_move = moves[0];
        let mut best_score = if color == Color::White { i32::MIN } else { i32::MAX };
        
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
                start_time
            );
            
            if (color == Color::White && score > best_score) || 
               (color == Color::Black && score < best_score) {
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
        start_time: Instant
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
        
        let color = if maximizing { Color::White } else { Color::Black };
        let mut moves = board.generate_moves(color);
        
        if moves.is_empty() {
            if board.is_in_check(color) {
                return if maximizing { -100000 + depth as i32 } else { 100000 - depth as i32 };
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
            
            let score = self.minimax_with_tt(&new_board, depth - 1, alpha, beta, !maximizing, start_time);
            
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
        
        self.transposition_table.insert(board_hash, TranspositionEntry {
            depth,
            score: best_score,
            best_move,
            node_type,
        });
        
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
                if piece.piece_type == PieceType::King && 
                   (mv.to.1 as i32 - mv.from.1 as i32).abs() == 2 {
                    score += 300;
                }
            }
            
            // 5. 中心控制
            let center_bonus = match mv.to {
                (3, 3) | (3, 4) | (4, 3) | (4, 4) => 50,
                (2, 2) | (2, 3) | (2, 4) | (2, 5) | 
                (3, 2) | (3, 5) | (4, 2) | (4, 5) | 
                (5, 2) | (5, 3) | (5, 4) | (5, 5) => 20,
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
    /// Implements the minimax algorithm with alpha-beta pruning
    #[allow(dead_code)]
    fn minimax(
        &self,
        board: &Board,
        depth: u32,
        mut alpha: i32,
        mut beta: i32,
        maximizing: bool, // true for AI's turn, false for opponent's turn
    ) -> i32 {
        if depth == 0 {
            return board.evaluate();
        }

        let color = if maximizing {
            Color::White
        } else {
            Color::Black
        };
        let mut moves = board.generate_moves(color);

        if moves.is_empty() {
            if board.is_in_check(color) {
                // Checkmate
                return if maximizing {
                    -100000 + depth as i32
                } else {
                    100000 - depth as i32
                };
            } else {
                // Stalemate
                return 0;
            }
        }

        // Move ordering for better alpha-beta pruning
        self.order_moves(&mut moves, board);

        if maximizing {
            // AI's turn (seek maximum score)
            let mut max_eval = i32::MIN;
            for mv in moves {
                let mut new_board = board.clone();
                new_board.make_move(mv);

                let eval = self.minimax(&new_board, depth - 1, alpha, beta, false);
                max_eval = max_eval.max(eval);
                alpha = alpha.max(eval);

                // Alpha-beta pruning
                if beta <= alpha {
                    break; // Beta pruning
                }
            }
            max_eval
        } else {
            // Opponent's turn (seek minimum score)
            let mut min_eval = i32::MAX;
            for mv in moves {
                let mut new_board = board.clone();
                new_board.make_move(mv);

                let eval = self.minimax(&new_board, depth - 1, alpha, beta, true);
                min_eval = min_eval.min(eval);
                beta = beta.min(eval);

                // Alpha-beta pruning
                if beta <= alpha {
                    break; // Alpha pruning
                }
            }
            min_eval
        }
    }

    /// Orders moves for better alpha-beta pruning
    #[allow(dead_code)]
    fn order_moves(&self, moves: &mut [Move], board: &Board) {
        // The sorting rules: captures first, then other moves
        moves.sort_by(|a, b| {
            let a_capture = board.get_piece(a.to).is_some();
            let b_capture = board.get_piece(b.to).is_some();

            match (a_capture, b_capture) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => std::cmp::Ordering::Equal,
            }
        });
    }

    fn get_best_move(&mut self, board: &Board, color: Color) -> Option<Move> {
        // 使用优化后的迭代深化搜索
        self.iterative_deepening(board, color)
    }
}

impl ChessApp {
    fn new() -> Self {
        Self {
            board: Board::new(),
            ai: ChessAI::new(4),
            current_player: Color::White,
            selected_square: None,
            valid_moves: Vec::new(),
            game_state: GameState::Playing,
            status_message: "White to move".to_string(),
            ai_thinking: false,
            ai_move_start: None,
            ai_difficulty: AIDifficulty::Medium,
            promotion_pending: None,
        }
    }

    fn piece_to_unicode(&self, piece: Piece) -> &str {
        match (piece.color, piece.piece_type) {
            (Color::White, PieceType::Pawn) => "♙ ",
            (Color::White, PieceType::Rook) => "♖ ",
            (Color::White, PieceType::Knight) => "♘ ",
            (Color::White, PieceType::Bishop) => "♗ ",
            (Color::White, PieceType::Queen) => "♕ ",
            (Color::White, PieceType::King) => "♔ ",
            (Color::Black, PieceType::Pawn) => "♟ ",
            (Color::Black, PieceType::Rook) => "♜ ",
            (Color::Black, PieceType::Knight) => "♞ ",
            (Color::Black, PieceType::Bishop) => "♝ ",
            (Color::Black, PieceType::Queen) => "♛ ",
            (Color::Black, PieceType::King) => "♚ ",
        }
    }

    fn handle_square_click(&mut self, row: usize, col: usize) {
        if self.game_state != GameState::Playing
            || self.current_player != Color::White
            || self.ai_thinking
            || self.promotion_pending.is_some() // 如果正在等待升变选择，不处理点击
        {
            return;
        }

        if let Some(selected) = self.selected_square {
            // Try to make a move
            let mv = Move {
                from: selected,
                to: (row, col),
                promotion: None,
            };

            // Check if this is pawn promotion
            let is_promotion = if let Some(piece) = self.board.get_piece(selected) {
                piece.piece_type == PieceType::Pawn
                    && ((piece.color == Color::White && row == 0)
                        || (piece.color == Color::Black && row == 7))
            } else {
                false
            };

            // Check if the move is in valid moves list (without considering promotion type)
            let move_found = self
                .valid_moves
                .iter()
                .find(|valid_mv| valid_mv.from == mv.from && valid_mv.to == mv.to);

            if move_found.is_some() {
                if is_promotion {
                    // 设置待升变的走法，等待用户选择
                    self.promotion_pending = Some(mv);
                    self.status_message = "Choose piece for promotion".to_string();
                } else {
                    // 普通走法，直接执行
                    self.board.make_move(mv);
                    self.selected_square = None;
                    self.valid_moves.clear();
                    self.current_player = Color::Black;
                    self.update_game_state();
                    if self.game_state == GameState::Playing {
                        self.status_message = "AI is thinking...".to_string();
                        self.ai_thinking = true;
                        self.ai_move_start = Some(Instant::now());
                    }
                }
            } else {
                // Select new piece or deselect
                if let Some(piece) = self.board.get_piece((row, col)) {
                    if piece.color == Color::White {
                        self.selected_square = Some((row, col));
                        self.valid_moves = self
                            .board
                            .generate_moves(Color::White)
                            .into_iter()
                            .filter(|mv| mv.from == (row, col))
                            .collect();
                    } else {
                        self.selected_square = None;
                        self.valid_moves.clear();
                    }
                } else {
                    self.selected_square = None;
                    self.valid_moves.clear();
                }
            }
        } else {
            // Select a piece
            if let Some(piece) = self.board.get_piece((row, col)) {
                if piece.color == Color::White {
                    self.selected_square = Some((row, col));
                    self.valid_moves = self
                        .board
                        .generate_moves(Color::White)
                        .into_iter()
                        .filter(|mv| mv.from == (row, col))
                        .collect();
                }
            }
        }
    }

    fn update_game_state(&mut self) {
        let moves = self.board.generate_moves(self.current_player);

        if moves.is_empty() {
            if self.board.is_in_check(self.current_player) {
                self.game_state = match self.current_player {
                    Color::White => GameState::BlackWins,
                    Color::Black => GameState::WhiteWins,
                };
                self.status_message = format!(
                    "{:?} wins by checkmate!",
                    match self.current_player {
                        Color::White => Color::Black,
                        Color::Black => Color::White,
                    }
                );
            } else {
                self.game_state = GameState::Draw;
                self.status_message = "Draw by stalemate!".to_string();
            }
        } else if self.board.is_in_check(self.current_player) {
            self.status_message = format!("{:?} is in check!", self.current_player);
        } else {
            self.status_message = format!("{:?} to move", self.current_player);
        }
    }

    fn new_game(&mut self) {
        self.board = Board::new();
        self.current_player = Color::White;
        self.selected_square = None;
        self.valid_moves.clear();
        self.game_state = GameState::Playing;
        self.status_message = "White to move".to_string();
        self.ai_thinking = false;
        self.ai_move_start = None;
        self.promotion_pending = None;
    }

    fn show_game_over_screen(&mut self, ctx: &egui::Context) {
        // Semi-transparent background overlay
        egui::Area::new("game_over_overlay".into())
            .fixed_pos(egui::pos2(0.0, 0.0))
            .show(ctx, |ui| {
                let screen_rect = ctx.screen_rect(); // Get the screen rectangle
                ui.allocate_ui_with_layout(
                    // Allocate UI with layout that you can use to draw
                    screen_rect.size(),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        // Semi-transparent background
                        let painter = ui.painter();
                        painter.rect_filled(
                            screen_rect,
                            0.0,
                            Color32::from_rgba_unmultiplied(0, 0, 0, 180),
                        );
                    },
                );
            });

        // Game over dialog
        egui::Window::new("")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_min_width(400.0);
                ui.set_min_height(300.0);

                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

                    // Game result icon and text
                    match self.game_state {
                        GameState::WhiteWins => {
                            ui.label(egui::RichText::new("👑").size(80.0).color(Color32::GOLD));
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new("WHITE WINS!")
                                    .size(32.0)
                                    .color(Color32::from_rgb(255, 215, 0))
                                    .strong(),
                            );
                            ui.add_space(5.0);
                            ui.label(
                                egui::RichText::new("❀ Congratulations! ❀")
                                    .size(18.0)
                                    .color(Color32::LIGHT_GRAY),
                            );
                        }
                        GameState::BlackWins => {
                            ui.label(
                                egui::RichText::new("👑")
                                    .size(80.0)
                                    .color(Color32::DARK_GRAY),
                            );
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new("BLACK WINS!")
                                    .size(32.0)
                                    .color(Color32::from_rgb(64, 64, 64))
                                    .strong(),
                            );
                            ui.add_space(5.0);
                            ui.label(
                                egui::RichText::new("☠ AI Victory! ☠")
                                    .size(18.0)
                                    .color(Color32::LIGHT_GRAY),
                            );
                        }
                        GameState::Draw => {
                            ui.label(
                                egui::RichText::new("🤝")
                                    .size(80.0)
                                    .color(Color32::LIGHT_BLUE),
                            );
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new("DRAW!")
                                    .size(32.0)
                                    .color(Color32::LIGHT_BLUE)
                                    .strong(),
                            );
                            ui.add_space(5.0);
                            ui.label(
                                egui::RichText::new("Well played by both sides!")
                                    .size(18.0)
                                    .color(Color32::LIGHT_GRAY),
                            );
                        }
                        _ => {}
                    }

                    ui.add_space(20.0);

                    // Game details
                    ui.separator();
                    ui.add_space(10.0);

                    match self.game_state {
                        GameState::WhiteWins | GameState::BlackWins => {
                            ui.label(
                                egui::RichText::new("Victory by Checkmate")
                                    .size(16.0)
                                    .color(Color32::WHITE),
                            );
                        }
                        GameState::Draw => {
                            ui.label(
                                egui::RichText::new("Game ended in Stalemate")
                                    .size(16.0)
                                    .color(Color32::WHITE),
                            );
                        }
                        _ => {}
                    }

                    ui.add_space(20.0);

                    // Buttons
                    ui.horizontal(|ui| {
                        // New Game button
                        if ui
                            .add_sized(
                                [120.0, 40.0],
                                egui::Button::new(
                                    egui::RichText::new("⚛ New Game")
                                        .size(16.0)
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(0, 150, 0)),
                            )
                            .clicked()
                        {
                            self.new_game();
                        }

                        ui.add_space(10.0);

                        // Exit button (you can implement this if needed)
                        if ui
                            .add_sized(
                                [120.0, 40.0],
                                egui::Button::new(
                                    egui::RichText::new("⚜  Exit")
                                        .size(16.0)
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(150, 0, 0)),
                            )
                            .clicked()
                        {
                            std::process::exit(0);
                        }
                    });

                    ui.add_space(10.0);
                });
            });
    }

    fn set_ai_difficulty(&mut self, difficulty: AIDifficulty) {
        self.ai_difficulty = difficulty;
        self.ai = ChessAI::new(difficulty.get_depth());
        // 更新AI的时间限制
        self.ai.time_limit = difficulty.get_time_limit();
    }

    fn handle_promotion_choice(&mut self, piece_type: PieceType) {
        if let Some(mut mv) = self.promotion_pending {
            mv.promotion = Some(piece_type);
            self.board.make_move(mv);
            self.selected_square = None;
            self.valid_moves.clear();
            self.promotion_pending = None;
            self.current_player = Color::Black;
            self.update_game_state();
            if self.game_state == GameState::Playing {
                self.status_message = "AI is thinking...".to_string();
                self.ai_thinking = true;
                self.ai_move_start = Some(Instant::now());
            }
        }
    }

    fn show_promotion_dialog(&mut self, ctx: &egui::Context) {
        if self.promotion_pending.is_none() {
            return;
        }

        egui::Window::new("Pawn Promotion")
            .title_bar(true)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_min_width(350.0);
                ui.set_min_height(250.0);

                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading("Choose promotion piece:");
                    ui.add_space(20.0);

                    // 创建一个2x2的网格布局来显示选择
                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        // 皇后
                        if ui.add_sized([60.0, 60.0], egui::Button::new("♕\nQueen")).clicked() {
                            self.handle_promotion_choice(PieceType::Queen);
                        }
                        ui.add_space(10.0);
                        // 车
                        if ui.add_sized([60.0, 60.0], egui::Button::new("♖\nRook")).clicked() {
                            self.handle_promotion_choice(PieceType::Rook);
                        }
                        ui.add_space(10.0);
                        // 象
                        if ui.add_sized([60.0, 60.0], egui::Button::new("♗\nBishop")).clicked() {
                            self.handle_promotion_choice(PieceType::Bishop);
                        }
                        ui.add_space(10.0);
                        // 马
                        if ui.add_sized([60.0, 60.0], egui::Button::new("♘\nKnight")).clicked() {
                            self.handle_promotion_choice(PieceType::Knight);
                        }
                    });

                    ui.add_space(10.0);
                    ui.label("Click on the piece you want to promote to");
                });
            });
    }
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle AI moves
        if self.ai_thinking && self.current_player == Color::Black {
            if let Some(start_time) = self.ai_move_start {
                let elapsed = start_time.elapsed().as_millis();
                let time_limit = self.ai.time_limit as u128;
                
                // 更新状态消息显示思考进度
                let progress = (elapsed as f32 / time_limit as f32 * 100.0).min(100.0);
                self.status_message = format!("AI thinking... ({:.1}%)", progress);

                if elapsed > 500 {
                    if let Some(ai_move) = self.ai.get_best_move(&self.board, Color::Black) {
                        self.board.make_move(ai_move);
                        self.current_player = Color::White;
                        self.ai_thinking = false;
                        self.ai_move_start = None;
                        self.update_game_state();
                    }
                }
            }
        }

        // Show promotion dialog if needed
        if self.promotion_pending.is_some() {
            self.show_promotion_dialog(ctx);
        }

        // Show game over screen if the game is finished
        if self.game_state != GameState::Playing {
            self.show_game_over_screen(ctx);
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Chess Game");

            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                if ui.button("New Game").clicked() {
                    self.new_game();
                }
                
                ui.separator();
                
                // 显示性能信息
                if self.ai.nodes_searched > 0 {
                    ui.label(format!("Search nodes: {}", self.ai.nodes_searched));
                }
                
                ui.separator();

                ui.label("AI Difficulty:");
                let old_difficulty = self.ai_difficulty;
                egui::ComboBox::from_label("")
                    .selected_text(format!("{} (depth:{})", self.ai_difficulty.to_string(), self.ai_difficulty.get_depth()))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.ai_difficulty, AIDifficulty::Easy, "Easy (depth:2)");
                        ui.selectable_value(&mut self.ai_difficulty, AIDifficulty::Medium, "Medium (depth:4)");
                        ui.selectable_value(&mut self.ai_difficulty, AIDifficulty::Hard, "Hard (depth:6)");
                        ui.selectable_value(&mut self.ai_difficulty, AIDifficulty::Expert, "Expert (depth:8)");
                    });
                
                // 当难度改变时立即更新AI
                if old_difficulty != self.ai_difficulty {
                    self.set_ai_difficulty(self.ai_difficulty);
                }
            });

            ui.add_space(20.0);

            // Draw the chess board
            let square_size = 100.0;
            let board_size = square_size * 8.0;
            let coordinate_size = 20.0; // 坐标标记的宽度/高度

            let (response, painter) = ui.allocate_painter(
                Vec2::new(board_size + coordinate_size, board_size + coordinate_size),
                Sense::click(),
            );

            let board_rect = Rect::from_min_size(
                Pos2::new(response.rect.min.x + coordinate_size, response.rect.min.y),
                Vec2::new(board_size, board_size),
            );

            // Draw board squares
            for row in 0..8 {
                for col in 0..8 {
                    let square_rect = Rect::from_min_size(
                        Pos2::new(
                            board_rect.min.x + col as f32 * square_size,
                            board_rect.min.y + row as f32 * square_size,
                        ),
                        Vec2::splat(square_size),
                    );

                    // Square color
                    let is_light = (row + col) % 2 == 0;
                    let mut square_color = if is_light {
                        Color32::from_rgb(240, 217, 181)
                    } else {
                        Color32::from_rgb(181, 136, 99)
                    };

                    // Highlight selected square
                    if Some((row, col)) == self.selected_square {
                        square_color = Color32::from_rgb(255, 255, 0);
                    }

                    // Highlight valid move squares
                    if self.valid_moves.iter().any(|mv| mv.to == (row, col)) {
                        square_color = Color32::from_rgb(0, 255, 0);
                    }

                    painter.rect_filled(square_rect, 0.0, square_color);
                    painter.rect_stroke(square_rect, 0.0, egui::Stroke::new(1.0, Color32::BLACK));

                    // Draw piece
                    if let Some(piece) = self.board.get_piece((row, col)) {
                        // Check if this piece is a king in check and highlight it
                        let is_king_in_check = piece.piece_type == PieceType::King
                            && self.board.is_in_check(piece.color);

                        if is_king_in_check {
                            // Draw red background for king in check
                            painter.rect_filled(
                                square_rect,
                                0.0,
                                Color32::from_rgba_unmultiplied(255, 0, 0, 100),
                            );
                            painter.rect_stroke(
                                square_rect,
                                0.0,
                                egui::Stroke::new(3.0, Color32::RED),
                            );
                        }

                        painter.text(
                            square_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            self.piece_to_unicode(piece),
                            egui::FontId::proportional(40.0),
                            Color32::BLACK,
                        );
                    }
                }
            }

            // Draw file labels (a-h) at the bottom
            for col in 0..8 {
                let file_char = (b'a' + col as u8) as char;
                let x = board_rect.min.x + col as f32 * square_size + square_size / 2.0;
                let y = board_rect.max.y + coordinate_size / 2.0;

                painter.text(
                    Pos2::new(x, y),
                    egui::Align2::CENTER_CENTER,
                    file_char.to_string(),
                    egui::FontId::proportional(16.0),
                    Color32::GOLD,
                );
            }

            // Draw rank labels (8-1) on the left side
            for row in 0..8 {
                let rank_num =  8 - row;
                let x = board_rect.min.x - coordinate_size / 2.0;
                let y = board_rect.min.y + row as f32 * square_size + square_size / 2.0;

                painter.text(
                    Pos2::new(x, y),
                    egui::Align2::CENTER_CENTER,
                    rank_num.to_string(),
                    egui::FontId::proportional(16.0),
                    Color32::GOLD,
                );
            }

            // Handle clicks
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    // 调整点击位置以适应新的坐标系统（减去坐标标记的偏移）
                    let relative_pos = pos - board_rect.min;
                    let col = (relative_pos.x / square_size) as usize;
                    let row = (relative_pos.y / square_size) as usize;

                    if row < 8 && col < 8 {
                        self.handle_square_click(row, col);
                    }
                }
            }

            ui.add_space(10.0);
        });

        // Request repaint for AI thinking animation
        if self.ai_thinking {
            ctx.request_repaint();
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 700.0])
            .with_title("Chess Game"),
        ..Default::default()
    };

    eframe::run_native(
        "Chess Game",
        options,
        Box::new(|_cc| Ok(Box::new(ChessApp::new()))),
    )
}
