use eframe::egui;
use egui::{Color32, Pos2, Rect, Sense, Vec2};
use std::time::Instant;

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

        // Handle castling condition
        // (think the posibility of castling is checked before calling this function)
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
            if let Some(target) = self.get_piece(mv.to) {
                return target.color != color;
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
                    if let Some(target) = self.get_piece((new_row, new_col)) {
                        if target.color != color {
                            self.add_pawn_move(pos, (new_row, new_col), color, moves);
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

    /// Evaluates the board state and returns a score
    fn evaluate(&self) -> i32 {
        let mut score = 0;

        // Pawn Piece position tables for better evaluation
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

        // Knight Piece position tables for better evaluation
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

        // Iterate through the board and evaluate each piece(遍历棋盘评估每个棋子)
        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = self.get_piece((row, col)) {
                    // Base piece value(基础棋子价值)
                    let mut piece_value = match piece.piece_type {
                        PieceType::Pawn => 100,
                        PieceType::Knight => 320,
                        PieceType::Bishop => 330,
                        PieceType::Rook => 500,
                        PieceType::Queen => 900,
                        PieceType::King => 20000,
                    };

                    // Add position bonus(位置加成)
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
                        _ => 0, // Other pieces don't have position bonuses in this simple evaluation
                    };

                    piece_value += position_bonus;

                    // Adjust score based on piece color(根据棋子颜色调整分数)
                    match piece.color {
                        Color::White => score += piece_value,
                        Color::Black => score -= piece_value,
                    }
                }
            }
        }

        // Mobility bonus(机动性评估)
        let white_moves = self.generate_moves(Color::White).len() as i32;
        let black_moves = self.generate_moves(Color::Black).len() as i32;
        score += (white_moves - black_moves) * 10;

        // King safety
        if self.is_in_check(Color::White) {
            score -= 50;
        }
        if self.is_in_check(Color::Black) {
            score += 50;
        }

        score
    }
}

struct ChessAI {
    max_depth: u32,
}

impl ChessAI {
    fn new(depth: u32) -> Self {
        ChessAI { max_depth: depth }
    }

    /*
    基本思路:
    1. 评估当前棋盘状态，返回一个分数
    2. 生成所有可能的走法
    3. 对每个走法进行递归调用，直到达到最大深度
    4. 使用α-β剪枝优化搜索

    AI会假设对手会选择最优解, 并根据这一假设进行搜索寻找最佳走法,最大化AI的收益,最小化对手的收益

    剪枝原理：
    - Alpha (α): 最大化玩家已知的最好选择
    - Beta (β): 最小化玩家已知的最好选择
    - 当 β ≤ α 时，可以停止搜索该分支
     */
    /// Implements the minimax algorithm with alpha-beta pruning
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

                // Skip moves that leave king in check
                if new_board.is_in_check(color) {
                    continue;
                }

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

                // Skip moves that leave king in check
                if new_board.is_in_check(color) {
                    continue;
                }

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

    fn get_best_move(&self, board: &Board, color: Color) -> Option<Move> {
        let is_in_check = board.is_in_check(color);
        let mut moves = board.generate_moves(color);

        if moves.is_empty() {
            return None;
        }

        // If in check, we must find a move that gets out of check
        if is_in_check {
            moves.retain(|mv| {
                let mut new_board = board.clone();
                new_board.make_move(*mv);
                !new_board.is_in_check(color)
            });

            if moves.is_empty() {
                return None; // No legal moves available, checkmate or stalemate
            }
        }

        // Optimizing the best move
        self.order_moves(&mut moves, board);

        let mut best_move = moves[0];
        let mut best_value = if color == Color::White {
            i32::MIN
        } else {
            i32::MAX
        };

        for mv in moves {
            let mut new_board = board.clone();
            new_board.make_move(mv);

            // Skip moves that leave king in check
            if new_board.is_in_check(color) {
                continue;
            }

            let value = self.minimax(
                &new_board,
                self.max_depth - 1,
                i32::MIN,
                i32::MAX,
                color == Color::Black,
            );

            if (color == Color::White && value > best_value)
                || (color == Color::Black && value < best_value)
            {
                best_value = value;
                best_move = mv;
            }
        }

        Some(best_move)
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
        {
            return;
        }

        if let Some(selected) = self.selected_square {
            // Try to make a move
            let mut mv = Move {
                from: selected,
                to: (row, col),
                promotion: None,
            };

            // Check if this is pawn promotion
            if let Some(piece) = self.board.get_piece(selected) {
                if piece.piece_type == PieceType::Pawn
                    && ((piece.color == Color::White && row == 0)
                        || (piece.color == Color::Black && row == 7))
                {
                    mv.promotion = Some(PieceType::Queen);
                }
            }

            // Check if the move is in valid moves list
            let move_found = self
                .valid_moves
                .iter()
                .find(|valid_mv| valid_mv.from == mv.from && valid_mv.to == mv.to);

            if let Some(found_move) = move_found {
                self.board.make_move(*found_move);
                self.selected_square = None;
                self.valid_moves.clear();
                self.current_player = Color::Black;
                self.update_game_state();
                if self.game_state == GameState::Playing {
                    self.status_message = "AI is thinking...".to_string();
                    self.ai_thinking = true;
                    self.ai_move_start = Some(Instant::now());
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
                            .filter(|mv| {
                                if mv.from != (row, col) {
                                    return false;
                                }
                                let mut temp_board = self.board.clone();
                                temp_board.make_move(*mv);
                                !temp_board.is_in_check(Color::White)
                            })
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
                        .filter(|mv| {
                            if mv.from != (row, col) {
                                return false;
                            }
                            let mut temp_board = self.board.clone();
                            temp_board.make_move(*mv);
                            !temp_board.is_in_check(Color::White)
                        })
                        .collect();
                }
            }
        }
    }

    fn update_game_state(&mut self) {
        let moves = self
            .board
            .generate_moves(self.current_player)
            .into_iter()
            .filter(|mv| {
                let mut temp_board = self.board.clone();
                temp_board.make_move(*mv);
                !temp_board.is_in_check(self.current_player)
            })
            .collect::<Vec<_>>();

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
    }
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle AI moves
        if self.ai_thinking && self.current_player == Color::Black {
            if let Some(start_time) = self.ai_move_start {
                if start_time.elapsed().as_millis() > 500 {
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
                
                ui.label("AI Difficulty:");
                let old_difficulty = self.ai_difficulty;
                egui::ComboBox::from_label("")
                    .selected_text(self.ai_difficulty.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.ai_difficulty, AIDifficulty::Easy, "Easy");
                        ui.selectable_value(&mut self.ai_difficulty, AIDifficulty::Medium, "Medium");
                        ui.selectable_value(&mut self.ai_difficulty, AIDifficulty::Hard, "Hard");
                        ui.selectable_value(&mut self.ai_difficulty, AIDifficulty::Expert, "Expert");
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
                let rank_num = 8 - row;
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
