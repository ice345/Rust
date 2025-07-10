//! 国际象棋棋盘模块
//! 包含棋盘状态管理、走法生成、合法性检查等核心逻辑

use crate::types::*;

#[derive(Debug, Clone)]
/// 表示国际象棋棋盘，包含棋子位置和游戏状态
pub struct Board {
    pub squares: [[Option<Piece>; 8]; 8],
    pub white_king_pos: (usize, usize),
    pub black_king_pos: (usize, usize),
    pub white_king_moved: bool,
    pub black_king_moved: bool,
    pub white_rook_a_moved: bool,
    pub white_rook_h_moved: bool,
    pub black_rook_a_moved: bool,
    pub black_rook_h_moved: bool,
    pub en_passant_target: Option<(usize, usize)>, // 过路兵目标位置
}

impl Board {
    /// 创建一个新的棋盘并设置初始位置
    pub fn new() -> Self {
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

    /// 设置棋盘的初始位置
    fn setup_initial_position(&mut self) {
        // 白方棋子
        self.squares[7][0] = Some(Piece::new(PieceType::Rook, Color::White));
        self.squares[7][1] = Some(Piece::new(PieceType::Knight, Color::White));
        self.squares[7][2] = Some(Piece::new(PieceType::Bishop, Color::White));
        self.squares[7][3] = Some(Piece::new(PieceType::Queen, Color::White));
        self.squares[7][4] = Some(Piece::new(PieceType::King, Color::White));
        self.squares[7][5] = Some(Piece::new(PieceType::Bishop, Color::White));
        self.squares[7][6] = Some(Piece::new(PieceType::Knight, Color::White));
        self.squares[7][7] = Some(Piece::new(PieceType::Rook, Color::White));

        for col in 0..8 {
            self.squares[6][col] = Some(Piece::new(PieceType::Pawn, Color::White));
        }

        // 黑方棋子
        self.squares[0][0] = Some(Piece::new(PieceType::Rook, Color::Black));
        self.squares[0][1] = Some(Piece::new(PieceType::Knight, Color::Black));
        self.squares[0][2] = Some(Piece::new(PieceType::Bishop, Color::Black));
        self.squares[0][3] = Some(Piece::new(PieceType::Queen, Color::Black));
        self.squares[0][4] = Some(Piece::new(PieceType::King, Color::Black));
        self.squares[0][5] = Some(Piece::new(PieceType::Bishop, Color::Black));
        self.squares[0][6] = Some(Piece::new(PieceType::Knight, Color::Black));
        self.squares[0][7] = Some(Piece::new(PieceType::Rook, Color::Black));

        for col in 0..8 {
            self.squares[1][col] = Some(Piece::new(PieceType::Pawn, Color::Black));
        }
    }

    /// 获取指定位置的棋子
    pub fn get_piece(&self, pos: (usize, usize)) -> Option<Piece> {
        self.squares[pos.0][pos.1]
    }

    /// 设置指定位置的棋子
    pub fn set_piece(&mut self, pos: (usize, usize), piece: Option<Piece>) {
        self.squares[pos.0][pos.1] = piece;
    }

    /// 执行一步棋
    pub fn make_move(&mut self, mv: Move) -> bool {
        let piece = self.get_piece(mv.from);
        if piece.is_none() {
            return false;
        }

        let piece = piece.unwrap();

        // 清除之前的过路兵标记
        self.en_passant_target = None;

        // 处理王车易位
        if piece.piece_type == PieceType::King {
            let col_diff = mv.to.1 as i32 - mv.from.1 as i32;
            if col_diff.abs() == 2 {
                // 这是王车易位
                let (rook_from_col, rook_to_col) = if col_diff > 0 {
                    // 王翼易位
                    (7, 5)
                } else {
                    // 后翼易位
                    (0, 3)
                };

                // 移动车
                let rook = self.get_piece((mv.from.0, rook_from_col)).unwrap();
                self.set_piece((mv.from.0, rook_from_col), None);
                self.set_piece((mv.from.0, rook_to_col), Some(rook));
            }

            // 更新王的位置
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

        // 处理兵的移动
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

        // 更新车移动标记
        if piece.piece_type == PieceType::Rook {
            match (piece.color, mv.from) {
                (Color::White, (7, 0)) => self.white_rook_a_moved = true,
                (Color::White, (7, 7)) => self.white_rook_h_moved = true,
                (Color::Black, (0, 0)) => self.black_rook_a_moved = true,
                (Color::Black, (0, 7)) => self.black_rook_h_moved = true,
                _ => {}
            }
        }

        // 处理兵的升变
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

    /// 生成指定颜色的所有合法走法
    pub fn generate_moves(&self, color: Color) -> Vec<Move> {
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
    pub fn generate_raw_moves(&self, color: Color) -> Vec<Move> {
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

    /// 检查指定颜色的王是否被将军
    pub fn is_in_check(&self, color: Color) -> bool {
        let king_pos = match color {
            Color::White => self.white_king_pos,
            Color::Black => self.black_king_pos,
        };

        let opponent_color = color.opposite();

        // 检查对方骑士攻击
        let knight_moves = [
            (2, 1), (2, -1), (-2, 1), (-2, -1),
            (1, 2), (1, -2), (-1, 2), (-1, -2),
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

        // 检查各个方向的滑动攻击
        let directions = [
            (0, 1), (1, 0), (0, -1), (-1, 0),  // 水平和垂直方向
            (1, 1), (1, -1), (-1, 1), (-1, -1), // 对角线方向
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
                    break;
                }
                r += dr;
                c += dc;
            }
        }

        // 检查兵的攻击
        let pawn_dirs = if color == Color::White {
            [(-1, -1), (-1, 1)]
        } else {
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

        // 检查对方国王相邻的格子
        let king_dirs = [
            (1, 0), (-1, 0), (0, 1), (0, -1),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
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

    // 生成指定棋子的所有走法
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
                    (0, 1), (0, -1), (1, 0), (-1, 0),
                    (1, 1), (1, -1), (-1, 1), (-1, -1),
                ],
                moves,
            ),
            PieceType::Knight => self.generate_knight_moves(pos, moves),
            PieceType::King => self.generate_king_moves(pos, piece.color, moves),
        }
    }

    // 以下是各种棋子的走法生成方法...
    // (这里包含原来的所有走法生成逻辑，为了节省空间暂时省略具体实现)
    
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
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
