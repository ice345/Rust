//! 国际象棋棋盘模块
//! 包含棋盘状态管理、走法生成、合法性检查等核心逻辑

use crate::types::*;

#[derive(Debug, Clone)]
/// 表示国际象棋棋盘，包含棋子位置和游戏状态
/// 坐标原点是在左上角,但是这又和在数组中的表示不一样.坐标系上的x不是对应于数组的行
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
    pub en_passant_target: Option<(usize, usize)>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
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
            halfmove_clock: 0,
            fullmove_number: 1,
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
        let captured_on_target = self.get_piece(mv.to);
        let mut is_capture = captured_on_target.is_some();

        // 清除之前的过路兵标记
        self.en_passant_target = None;

        // 如果吃掉了初始位置的车，移除相应易位权
        if let Some(captured_piece) = captured_on_target
            && captured_piece.piece_type == PieceType::Rook
        {
            match (captured_piece.color, mv.to) {
                (Color::White, (7, 0)) => self.white_rook_a_moved = true,
                (Color::White, (7, 7)) => self.white_rook_h_moved = true,
                (Color::Black, (0, 0)) => self.black_rook_a_moved = true,
                (Color::Black, (0, 7)) => self.black_rook_h_moved = true,
                _ => {}
            }
        }

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
                is_capture = true;
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

        // 50回合规则计数
        if piece.piece_type == PieceType::Pawn || is_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock = self.halfmove_clock.saturating_add(1);
        }

        // 全回合数：黑方走完后 +1
        if piece.color == Color::Black {
            self.fullmove_number = self.fullmove_number.saturating_add(1);
        }

        true
    }

    /// 生成指定颜色的所有合法走法
    pub fn generate_moves(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);

        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = self.get_piece((row, col))
                    && piece.color == color
                {
                    self.generate_piece_moves((row, col), piece, &mut moves);
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
                if let Some(piece) = self.get_piece((row, col))
                    && piece.color == color
                {
                    self.generate_piece_moves((row, col), piece, &mut moves);
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
            ) && r < 8
                && c < 8
                && let Some(piece) = self.get_piece((r, c))
                && piece.color == opponent_color
                && piece.piece_type == PieceType::Knight
            {
                return true;
            }
        }

        // 检查各个方向的滑动攻击
        let directions = [
            (0, 1),
            (1, 0),
            (0, -1),
            (-1, 0), // 水平和垂直方向
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1), // 对角线方向
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
            ) && r < 8
                && c < 8
                && let Some(piece) = self.get_piece((r, c))
                && piece.color == opponent_color
                && piece.piece_type == PieceType::Pawn
            {
                return true;
            }
        }

        // 检查对方国王相邻的格子
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
            ) && r < 8
                && c < 8
                && let Some(piece) = self.get_piece((r, c))
                && piece.color == opponent_color
                && piece.piece_type == PieceType::King
            {
                return true;
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

    // 以下是各种棋子的走法生成方法...
    // (这里包含原来的所有走法生成逻辑，为了节省空间暂时省略具体实现)

    fn generate_pawn_moves(&self, pos: (usize, usize), color: Color, moves: &mut Vec<Move>) {
        let (row, col) = pos;
        let direction = if color == Color::White { -1i32 } else { 1i32 };
        let start_row = if color == Color::White { 6 } else { 1 };

        // Forward move
        if let Ok(new_row) = (row as i32 + direction).try_into()
            && new_row < 8
            && self.get_piece((new_row, col)).is_none()
        {
            self.add_pawn_move(pos, (new_row, col), color, moves);

            // Double forward from start
            if row == start_row
                && let Ok(double_row) = (row as i32 + 2 * direction).try_into()
                && double_row < 8
                && self.get_piece((double_row, col)).is_none()
            {
                self.add_pawn_move(pos, (double_row, col), color, moves);
            }
        }

        // Captures
        for &col_offset in &[-1i32, 1i32] {
            if let (Ok(new_row), Ok(new_col)) = (
                (row as i32 + direction).try_into(),
                (col as i32 + col_offset).try_into(),
            ) && new_row < 8
                && new_col < 8
            {
                // 普通吃子
                if let Some(target) = self.get_piece((new_row, new_col)) {
                    if target.color != color {
                        self.add_pawn_move(pos, (new_row, new_col), color, moves);
                    }
                }
                // 过路兵吃子
                else if let Some(en_passant_pos) = self.en_passant_target
                    && (new_row, new_col) == en_passant_pos
                {
                    moves.push(Move {
                        from: pos,
                        to: (new_row, new_col),
                        promotion: None,
                    });
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
                && new_row < 8
                && new_col < 8
            {
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
                && new_row < 8
                && new_col < 8
            {
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

        // Check if all squares between king and rook are clear.
        // This includes king destination square.
        if rook_col > from_col {
            for col in (from_col + 1)..rook_col {
                if self.get_piece((from_row, col)).is_some() {
                    return false;
                }
            }
        } else {
            for col in (rook_col + 1)..from_col {
                if self.get_piece((from_row, col)).is_some() {
                    return false;
                }
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
        match king.color {
            Color::White => temp_board.white_king_pos = (from_row, intermediate_col),
            Color::Black => temp_board.black_king_pos = (from_row, intermediate_col),
        }
        if temp_board.is_in_check(king.color) {
            return false;
        }

        // Check final square for check
        let mut final_board = self.clone();
        final_board.set_piece(mv.from, None);
        final_board.set_piece((from_row, to_col), Some(king));
        match king.color {
            Color::White => final_board.white_king_pos = (from_row, to_col),
            Color::Black => final_board.black_king_pos = (from_row, to_col),
        }
        if final_board.is_in_check(king.color) {
            return false;
        }

        true
    }

    /// 50回合规则：半回合计数达到100（双方各50步无兵动且无吃子）
    pub fn is_fifty_move_rule_draw(&self) -> bool {
        self.halfmove_clock >= 100
    }

    /// FIDE 简化版子力不足判和检测（覆盖常见可判定场景）
    pub fn has_insufficient_material(&self) -> bool {
        let mut white_bishops = Vec::new();
        let mut black_bishops = Vec::new();
        let mut white_knights = 0usize;
        let mut black_knights = 0usize;

        for row in 0..8 {
            for col in 0..8 {
                let Some(piece) = self.squares[row][col] else {
                    continue;
                };

                match piece.piece_type {
                    PieceType::Pawn | PieceType::Rook | PieceType::Queen => return false,
                    PieceType::Bishop => {
                        if piece.color == Color::White {
                            white_bishops.push(Self::is_light_square(row, col));
                        } else {
                            black_bishops.push(Self::is_light_square(row, col));
                        }
                    }
                    PieceType::Knight => {
                        if piece.color == Color::White {
                            white_knights += 1;
                        } else {
                            black_knights += 1;
                        }
                    }
                    PieceType::King => {}
                }
            }
        }

        let white_minor = white_bishops.len() + white_knights;
        let black_minor = black_bishops.len() + black_knights;

        // K vs K
        if white_minor == 0 && black_minor == 0 {
            return true;
        }

        // K+B vs K, K+N vs K
        if white_minor == 1 && black_minor == 0 {
            return true;
        }
        if black_minor == 1 && white_minor == 0 {
            return true;
        }

        // K+NN vs K and symmetric
        if white_minor == 2 && white_knights == 2 && black_minor == 0 {
            return true;
        }
        if black_minor == 2 && black_knights == 2 && white_minor == 0 {
            return true;
        }

        // K+B vs K+B (同色象)
        if white_knights == 0
            && black_knights == 0
            && white_bishops.len() == 1
            && black_bishops.len() == 1
            && white_bishops[0] == black_bishops[0]
        {
            return true;
        }

        false
    }

    /// 用于三次重复检测的局面键（忽略 halfmove/fullmove）
    pub fn position_key(&self, active_color: Color) -> String {
        self.to_fen(active_color)
            .split_whitespace()
            .take(4)
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn is_light_square(row: usize, col: usize) -> bool {
        (row + col).is_multiple_of(2)
    }

    // ---- FEN / notation helpers ----

    pub fn square_to_algebraic(row: usize, col: usize) -> String {
        let file = (b'a' + col as u8) as char;
        let rank = (b'1' + (7 - row) as u8) as char;
        format!("{}{}", file, rank)
    }

    pub fn algebraic_to_square(s: &str) -> Result<(usize, usize), String> {
        let bytes = s.as_bytes();
        if bytes.len() < 2 {
            return Err("too short".into());
        }
        let file = bytes[0];
        let rank = bytes[1];
        if !(b'a'..=b'h').contains(&file) || !(b'1'..=b'8').contains(&rank) {
            return Err(format!("invalid square: {}", s));
        }
        let col = (file - b'a') as usize;
        let row = 7 - (rank - b'1') as usize;
        Ok((row, col))
    }

    pub fn move_to_uci(mv: &Move) -> String {
        let from = Self::square_to_algebraic(mv.from.0, mv.from.1);
        let to = Self::square_to_algebraic(mv.to.0, mv.to.1);
        let promo = match mv.promotion {
            Some(PieceType::Queen) => "q",
            Some(PieceType::Rook) => "r",
            Some(PieceType::Bishop) => "b",
            Some(PieceType::Knight) => "n",
            _ => "",
        };
        format!("{}{}{}", from, to, promo)
    }

    pub fn uci_to_move(s: &str) -> Result<Move, String> {
        if s.len() < 4 {
            return Err("too short".into());
        }
        let from = Self::algebraic_to_square(&s[0..2])?;
        let to = Self::algebraic_to_square(&s[2..4])?;
        let promotion = if s.len() > 4 {
            match s.as_bytes()[4] {
                b'q' => Some(PieceType::Queen),
                b'r' => Some(PieceType::Rook),
                b'b' => Some(PieceType::Bishop),
                b'n' => Some(PieceType::Knight),
                _ => None,
            }
        } else {
            None
        };
        Ok(Move {
            from,
            to,
            promotion,
        })
    }

    fn piece_to_fen_char(piece: Piece) -> char {
        let c = match piece.piece_type {
            PieceType::Pawn => 'p',
            PieceType::Rook => 'r',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        };
        match piece.color {
            Color::White => c.to_uppercase().next().unwrap(),
            Color::Black => c,
        }
    }

    fn fen_char_to_piece(c: char) -> Option<Piece> {
        let color = if c.is_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        let piece_type = match c.to_lowercase().next()? {
            'p' => PieceType::Pawn,
            'r' => PieceType::Rook,
            'n' => PieceType::Knight,
            'b' => PieceType::Bishop,
            'q' => PieceType::Queen,
            'k' => PieceType::King,
            _ => return None,
        };
        Some(Piece::new(piece_type, color))
    }

    pub fn to_fen(&self, active_color: Color) -> String {
        let mut fen = String::new();
        for row in 0..8 {
            let mut empty = 0;
            for col in 0..8 {
                if let Some(piece) = self.squares[row][col] {
                    if empty > 0 {
                        fen.push_str(&empty.to_string());
                        empty = 0;
                    }
                    fen.push(Self::piece_to_fen_char(piece));
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                fen.push_str(&empty.to_string());
            }
            if row < 7 {
                fen.push('/');
            }
        }

        // Active color
        fen.push(' ');
        fen.push(match active_color {
            Color::White => 'w',
            Color::Black => 'b',
        });

        // Castling
        fen.push(' ');
        let mut castling = String::new();
        if !self.white_king_moved && !self.white_rook_h_moved {
            castling.push('K');
        }
        if !self.white_king_moved && !self.white_rook_a_moved {
            castling.push('Q');
        }
        if !self.black_king_moved && !self.black_rook_h_moved {
            castling.push('k');
        }
        if !self.black_king_moved && !self.black_rook_a_moved {
            castling.push('q');
        }
        if castling.is_empty() {
            fen.push('-');
        } else {
            fen.push_str(&castling);
        }

        // En passant
        fen.push(' ');
        if let Some((r, c)) = self.en_passant_target {
            fen.push_str(&Self::square_to_algebraic(r, c));
        } else {
            fen.push('-');
        }

        // Halfmove clock and fullmove number
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }

    pub fn from_fen(fen: &str) -> Result<(Board, Color, u32), String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("FEN must have at least 4 fields".into());
        }

        let mut board = Board {
            squares: [[None; 8]; 8],
            white_king_pos: (0, 0),
            black_king_pos: (0, 0),
            white_king_moved: true,
            black_king_moved: true,
            white_rook_a_moved: true,
            white_rook_h_moved: true,
            black_rook_a_moved: true,
            black_rook_h_moved: true,
            en_passant_target: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        };

        // Parse piece placement
        let rows: Vec<&str> = parts[0].split('/').collect();
        if rows.len() != 8 {
            return Err("FEN piece placement must have 8 ranks".into());
        }
        for (row, rank_str) in rows.iter().enumerate() {
            let mut col = 0usize;
            for ch in rank_str.chars() {
                if let Some(digit) = ch.to_digit(10) {
                    col += digit as usize;
                } else if let Some(piece) = Self::fen_char_to_piece(ch) {
                    if col >= 8 {
                        return Err("rank overflow".into());
                    }
                    board.squares[row][col] = Some(piece);
                    if piece.piece_type == PieceType::King {
                        match piece.color {
                            Color::White => board.white_king_pos = (row, col),
                            Color::Black => board.black_king_pos = (row, col),
                        }
                    }
                    col += 1;
                } else {
                    return Err(format!("invalid FEN char: {}", ch));
                }
            }
        }

        // Active color
        let active_color = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("invalid active color".into()),
        };

        // Castling availability
        let castling = parts[2];
        if castling != "-" {
            for ch in castling.chars() {
                match ch {
                    'K' => {
                        board.white_king_moved = false;
                        board.white_rook_h_moved = false;
                    }
                    'Q' => {
                        board.white_king_moved = false;
                        board.white_rook_a_moved = false;
                    }
                    'k' => {
                        board.black_king_moved = false;
                        board.black_rook_h_moved = false;
                    }
                    'q' => {
                        board.black_king_moved = false;
                        board.black_rook_a_moved = false;
                    }
                    _ => {}
                }
            }
        }

        // En passant target
        if parts[3] != "-" {
            board.en_passant_target = Some(Self::algebraic_to_square(parts[3])?);
        }

        // Halfmove clock
        if parts.len() > 4 {
            board.halfmove_clock = parts[4].parse().unwrap_or(0);
        }

        // Fullmove number
        let fullmove = if parts.len() > 5 {
            parts[5].parse().unwrap_or(1)
        } else {
            1
        };
        board.fullmove_number = fullmove;

        Ok((board, active_color, fullmove))
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Color, PieceType};

    #[test]
    fn test_initial_board_setup() {
        let board = Board::new();
        // Test some white pieces
        assert_eq!(
            board.get_piece((7, 4)),
            Some(Piece::new(PieceType::King, Color::White))
        );
        assert_eq!(
            board.get_piece((7, 0)),
            Some(Piece::new(PieceType::Rook, Color::White))
        );
        // Test some black pieces
        assert_eq!(
            board.get_piece((0, 4)),
            Some(Piece::new(PieceType::King, Color::Black))
        );
        // Test some pawns
        assert_eq!(
            board.get_piece((6, 0)),
            Some(Piece::new(PieceType::Pawn, Color::White))
        );
        assert_eq!(
            board.get_piece((1, 7)),
            Some(Piece::new(PieceType::Pawn, Color::Black))
        );
        // Test some empty squares
        assert!(board.get_piece((3, 3)).is_none());
    }

    #[test]
    fn test_make_move() {
        let mut board = Board::new();
        let mv = Move {
            from: (6, 4),
            to: (5, 4),
            promotion: None,
        };
        board.make_move(mv);
        assert!(board.get_piece((6, 4)).is_none());
        assert_eq!(
            board.get_piece((5, 4)),
            Some(Piece::new(PieceType::Pawn, Color::White))
        );
    }

    #[test]
    fn test_pawn_initial_double_move() {
        let board = Board::new();
        let moves = board.generate_moves(Color::White);
        let double_move_e4 = Move {
            from: (6, 4),
            to: (4, 4),
            promotion: None,
        };
        assert!(moves.contains(&double_move_e4));
    }

    #[test]
    fn test_is_in_check() {
        let mut board = Board::new();
        // Clear board except for kings and a threatening piece
        board.squares = [[None; 8]; 8];
        board.set_piece((0, 4), Some(Piece::new(PieceType::King, Color::Black)));
        board.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));
        board.black_king_pos = (0, 4);
        board.white_king_pos = (7, 4);

        // Place a white rook to check the black king
        board.set_piece((0, 0), Some(Piece::new(PieceType::Rook, Color::White)));

        assert!(board.is_in_check(Color::Black));
        assert!(!board.is_in_check(Color::White));
    }

    #[test]
    fn test_castling_generation() {
        let mut board = Board::new();
        // Clear path for white king-side castling
        board.set_piece((7, 5), None);
        board.set_piece((7, 6), None);

        let moves = board.generate_moves(Color::White);
        let castling_move = Move {
            from: (7, 4),
            to: (7, 6),
            promotion: None,
        };

        // Note: The is_valid_castling is complex. If this fails, it might be due to the logic
        // not perfectly handling all intermediate checks. For now, we assert it's generated.
        assert!(
            moves.contains(&castling_move),
            "Castling move should be generated"
        );
    }

    #[test]
    fn test_castling_blocked_when_destination_occupied() {
        let mut board = Board::new();
        // Clear only f1. Keep g1 occupied by white knight.
        board.set_piece((7, 5), None);

        let moves = board.generate_moves(Color::White);
        let castling_move = Move {
            from: (7, 4),
            to: (7, 6),
            promotion: None,
        };
        assert!(
            !moves.contains(&castling_move),
            "Castling should be illegal when destination square is occupied"
        );
    }

    #[test]
    fn test_castling_blocked_when_passing_square_attacked() {
        let mut board = Board::new();
        board.squares = [[None; 8]; 8];

        // White king and rook ready to castle king-side.
        board.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));
        board.set_piece((7, 7), Some(Piece::new(PieceType::Rook, Color::White)));
        board.white_king_pos = (7, 4);
        board.white_king_moved = false;
        board.white_rook_h_moved = false;
        board.white_rook_a_moved = true;

        // Black king (to keep board valid enough for check logic).
        board.set_piece((0, 4), Some(Piece::new(PieceType::King, Color::Black)));
        board.black_king_pos = (0, 4);

        // Black rook attacks f1 (square the king passes through).
        board.set_piece((0, 5), Some(Piece::new(PieceType::Rook, Color::Black)));

        let castling_move = Move {
            from: (7, 4),
            to: (7, 6),
            promotion: None,
        };
        let moves = board.generate_moves(Color::White);
        assert!(
            !moves.contains(&castling_move),
            "Castling should be illegal when king passes through attacked square"
        );
    }

    #[test]
    fn test_en_passant_move() {
        let mut board = Board::new();
        board.squares = [[None; 8]; 8]; // Clear board

        // Set up an en passant scenario
        let white_pawn = Piece::new(PieceType::Pawn, Color::White);
        let black_pawn = Piece::new(PieceType::Pawn, Color::Black);
        board.set_piece((3, 4), Some(white_pawn)); // White pawn on e5
        board.set_piece((1, 3), Some(black_pawn)); // Black pawn on d7
        board.white_king_pos = (7, 7); // Place kings somewhere safe
        board.black_king_pos = (0, 0);

        // 1. Black moves d7 to d5, creating an en passant target at d6
        let mut board_after_black_move = board.clone();
        board_after_black_move.make_move(Move {
            from: (1, 3),
            to: (3, 3),
            promotion: None,
        });

        assert_eq!(board_after_black_move.en_passant_target, Some((2, 3)));

        // 2. Check if white can legally perform en passant
        let white_moves = board_after_black_move.generate_moves(Color::White);
        let en_passant_move = Move {
            from: (3, 4),
            to: (2, 3),
            promotion: None,
        };
        assert!(white_moves.contains(&en_passant_move));

        // 3. Perform the en passant move
        board_after_black_move.make_move(en_passant_move);

        // 4. Verify the board state
        assert!(board_after_black_move.get_piece((2, 3)).is_some()); // White pawn moved to d6
        assert_eq!(
            board_after_black_move.get_piece((2, 3)).unwrap().piece_type,
            PieceType::Pawn
        );
        assert!(board_after_black_move.get_piece((3, 3)).is_none()); // Black pawn was captured
        assert!(board_after_black_move.get_piece((3, 4)).is_none()); // White pawn moved from e5
    }

    #[test]
    fn test_fen_initial_position() {
        let board = Board::new();
        let fen = board.to_fen(Color::White);
        assert_eq!(
            fen,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn test_fen_roundtrip() {
        let board = Board::new();
        let fen = board.to_fen(Color::White);
        let (parsed, color, _fullmove) = Board::from_fen(&fen).unwrap();
        assert_eq!(color, Color::White);
        assert_eq!(parsed.to_fen(Color::White), fen);
    }

    #[test]
    fn test_fen_after_e4() {
        let mut board = Board::new();
        board.make_move(Move {
            from: (6, 4),
            to: (4, 4),
            promotion: None,
        });
        let fen = board.to_fen(Color::Black);
        assert_eq!(
            fen,
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
        );
    }

    #[test]
    fn test_square_algebraic_roundtrip() {
        for row in 0..8 {
            for col in 0..8 {
                let alg = Board::square_to_algebraic(row, col);
                let (r, c) = Board::algebraic_to_square(&alg).unwrap();
                assert_eq!((r, c), (row, col));
            }
        }
    }

    #[test]
    fn test_uci_roundtrip() {
        let mv = Move {
            from: (6, 4),
            to: (4, 4),
            promotion: None,
        };
        let uci = Board::move_to_uci(&mv);
        assert_eq!(uci, "e2e4");
        let parsed = Board::uci_to_move(&uci).unwrap();
        assert_eq!(parsed.from, mv.from);
        assert_eq!(parsed.to, mv.to);
    }

    #[test]
    fn test_uci_promotion() {
        let mv = Move {
            from: (1, 4),
            to: (0, 4),
            promotion: Some(PieceType::Queen),
        };
        let uci = Board::move_to_uci(&mv);
        assert_eq!(uci, "e7e8q");
        let parsed = Board::uci_to_move(&uci).unwrap();
        assert_eq!(parsed.promotion, Some(PieceType::Queen));
    }

    #[test]
    fn test_halfmove_and_fullmove_counters() {
        let mut board = Board::new();
        // 1. Nf3
        board.make_move(Move {
            from: (7, 6),
            to: (5, 5),
            promotion: None,
        });
        assert_eq!(board.halfmove_clock, 1);
        assert_eq!(board.fullmove_number, 1);

        // ... d5 (pawn move resets halfmove, black move increments fullmove)
        board.make_move(Move {
            from: (1, 3),
            to: (3, 3),
            promotion: None,
        });
        assert_eq!(board.halfmove_clock, 0);
        assert_eq!(board.fullmove_number, 2);
    }

    #[test]
    fn test_fifty_move_rule_draw_detection() {
        let mut board = Board::new();
        board.halfmove_clock = 99;
        assert!(!board.is_fifty_move_rule_draw());
        board.halfmove_clock = 100;
        assert!(board.is_fifty_move_rule_draw());
    }

    #[test]
    fn test_insufficient_material_king_vs_king() {
        let mut board = Board::new();
        board.squares = [[None; 8]; 8];
        board.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));
        board.set_piece((0, 4), Some(Piece::new(PieceType::King, Color::Black)));
        board.white_king_pos = (7, 4);
        board.black_king_pos = (0, 4);
        assert!(board.has_insufficient_material());
    }

    #[test]
    fn test_insufficient_material_king_bishop_vs_king() {
        let mut board = Board::new();
        board.squares = [[None; 8]; 8];
        board.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));
        board.set_piece((0, 4), Some(Piece::new(PieceType::King, Color::Black)));
        board.set_piece((6, 3), Some(Piece::new(PieceType::Bishop, Color::White)));
        board.white_king_pos = (7, 4);
        board.black_king_pos = (0, 4);
        assert!(board.has_insufficient_material());
    }

    #[test]
    fn test_not_insufficient_material_king_bishop_knight_vs_king() {
        let mut board = Board::new();
        board.squares = [[None; 8]; 8];
        board.set_piece((7, 4), Some(Piece::new(PieceType::King, Color::White)));
        board.set_piece((0, 4), Some(Piece::new(PieceType::King, Color::Black)));
        board.set_piece((6, 3), Some(Piece::new(PieceType::Bishop, Color::White)));
        board.set_piece((6, 5), Some(Piece::new(PieceType::Knight, Color::White)));
        board.white_king_pos = (7, 4);
        board.black_king_pos = (0, 4);
        assert!(!board.has_insufficient_material());
    }

    #[test]
    fn test_position_key_ignores_move_counters() {
        let mut board = Board::new();
        let key1 = board.position_key(Color::White);
        board.halfmove_clock = 88;
        board.fullmove_number = 42;
        let key2 = board.position_key(Color::White);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_threefold_repetition_pattern_detection_by_key_count() {
        let mut board = Board::new();
        let mut active = Color::White;
        let mut keys = vec![board.position_key(active)];

        let cycle = [
            Move {
                from: (7, 6),
                to: (5, 5),
                promotion: None,
            }, // Nf3
            Move {
                from: (0, 6),
                to: (2, 5),
                promotion: None,
            }, // ...Nf6
            Move {
                from: (5, 5),
                to: (7, 6),
                promotion: None,
            }, // Ng1
            Move {
                from: (2, 5),
                to: (0, 6),
                promotion: None,
            }, // ...Ng8
        ];

        for _ in 0..2 {
            for mv in cycle {
                assert!(board.make_move(mv));
                active = active.opposite();
                keys.push(board.position_key(active));
            }
        }

        let current = board.position_key(active);
        let count = keys.iter().filter(|k| *k == &current).count();
        assert_eq!(count, 3);
    }
}
