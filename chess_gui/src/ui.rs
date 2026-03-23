use eframe::egui;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use std::sync::Mutex;
use std::time::Instant;

use crate::ai::ChessAI;
use crate::board::Board;
use crate::pgn;
use crate::review::ReviewState;
use crate::types::*;
use crate::uci::{EngineManager, EngineResult};

mod review_panel;

const BG_DARK: Color32 = Color32::from_rgb(18, 18, 18);
const BG_PANEL: Color32 = Color32::from_rgb(26, 28, 33);
const TEXT_PRIMARY: Color32 = Color32::from_rgb(232, 238, 246);
const TEXT_SECONDARY: Color32 = Color32::from_rgb(169, 182, 198);
const TEXT_MUTED: Color32 = Color32::from_rgb(121, 137, 157);
const ACCENT_GREEN: Color32 = Color32::from_rgb(68, 197, 221);
const LIGHT_SQUARE: Color32 = Color32::from_rgb(231, 234, 239);
const DARK_SQUARE: Color32 = Color32::from_rgb(123, 142, 166);
const SELECTED_SQUARE: Color32 = Color32::from_rgb(145, 214, 205);
const VALID_TARGET: Color32 = Color32::from_rgb(109, 184, 173);

/// Main application structure that holds the board, AI, and game state.
pub struct ChessApp {
    pub board: Board,
    pub ai: ChessAI,
    pub engine_type: EngineType,
    pub engine_manager: EngineManager,
    pub human_name: String,
    pub opponent_name: String,
    pub current_player: Color,
    pub human_color: Color,
    pub selected_square: Option<(usize, usize)>,
    pub valid_moves: Vec<Move>,
    pub game_state: GameState,
    pub status_message: String,
    pub ai_thinking: bool,
    pub ai_move_start: Option<Instant>,
    pub ai_difficulty: AIDifficulty,
    pub promotion_pending: Option<Move>,
    pub board_history: Vec<(Board, Color)>,
    pub move_history: Vec<MoveRecord>,
    pub last_move: Option<Move>,
    pub app_mode: AppMode,
    pub review_state: Option<ReviewState>,
    pub review_hover_top_line: Option<usize>,
    pub playing_view_ply: Option<usize>,
    pub initial_board: Board,
    pub pgn_copied_msg: Option<Instant>,
    pub clipboard: Option<Mutex<arboard::Clipboard>>,
    pub dragging: Option<(usize, usize)>,
    pub show_setup_dialog: bool,
}

impl ChessApp {
    pub fn new() -> Self {
        let difficulty = AIDifficulty::Medium;
        let engine_manager = EngineManager::new(&difficulty);
        let engine_type = if engine_manager.has_stockfish() {
            EngineType::Stockfish
        } else {
            EngineType::BuiltIn
        };
        let board = Board::new();

        Self {
            board: board.clone(),
            ai: ChessAI::new(difficulty.get_depth()),
            engine_type,
            engine_manager,
            human_name: "Player".to_string(),
            opponent_name: "Computer".to_string(),
            current_player: Color::White,
            human_color: Color::White,
            selected_square: None,
            valid_moves: Vec::new(),
            game_state: GameState::Playing,
            status_message: "White to move".to_string(),
            ai_thinking: false,
            ai_move_start: None,
            ai_difficulty: difficulty,
            promotion_pending: None,
            board_history: Vec::new(),
            move_history: Vec::new(),
            last_move: None,
            app_mode: AppMode::Playing,
            review_state: None,
            review_hover_top_line: None,
            playing_view_ply: None,
            initial_board: board,
            pgn_copied_msg: None,
            clipboard: arboard::Clipboard::new().ok().map(Mutex::new),
            dragging: None,
            show_setup_dialog: true,
        }
    }

    fn color_name(color: Color) -> &'static str {
        match color {
            Color::White => "White",
            Color::Black => "Black",
        }
    }

    fn engine_name(&self) -> &'static str {
        match self.engine_type {
            EngineType::Stockfish => "Stockfish",
            EngineType::Lc0 => "Leela Chess Zero",
            EngineType::BuiltIn => "Built-in AI",
        }
    }

    fn piece_symbol(piece_type: PieceType) -> &'static str {
        match piece_type {
            PieceType::Pawn => "♟",
            PieceType::Rook => "♜",
            PieceType::Knight => "♞",
            PieceType::Bishop => "♝",
            PieceType::Queen => "♛",
            PieceType::King => "♚",
        }
    }

    fn piece_symbol_for_color(piece_type: PieceType, color: Color) -> &'static str {
        match (piece_type, color) {
            (PieceType::Pawn, Color::White) => "♙",
            (PieceType::Rook, Color::White) => "♖",
            (PieceType::Knight, Color::White) => "♘",
            (PieceType::Bishop, Color::White) => "♗",
            (PieceType::Queen, Color::White) => "♕",
            (PieceType::King, Color::White) => "♔",
            (PieceType::Pawn, Color::Black) => "♟",
            (PieceType::Rook, Color::Black) => "♜",
            (PieceType::Knight, Color::Black) => "♞",
            (PieceType::Bishop, Color::Black) => "♝",
            (PieceType::Queen, Color::Black) => "♛",
            (PieceType::King, Color::Black) => "♚",
        }
    }

    fn human_name_display(&self) -> String {
        let name = self.human_name.trim();
        if name.is_empty() {
            "Player".to_string()
        } else {
            name.to_string()
        }
    }

    fn opponent_name_display(&self) -> String {
        let base = self.opponent_name.trim();
        let base = if base.is_empty() { "Computer" } else { base };
        format!("{} ({})", base, self.engine_name())
    }

    fn names_by_color(&self) -> (String, String) {
        if self.human_color == Color::White {
            (self.human_name_display(), self.opponent_name_display())
        } else {
            (self.opponent_name_display(), self.human_name_display())
        }
    }

    fn display_name_for_color(&self, color: Color) -> String {
        let (white, black) = self.names_by_color();
        match color {
            Color::White => white,
            Color::Black => black,
        }
    }

    fn side_captured_pieces(board: &Board, capturer: Color) -> Vec<PieceType> {
        fn start_count(piece_type: PieceType) -> i32 {
            match piece_type {
                PieceType::Pawn => 8,
                PieceType::Knight | PieceType::Bishop | PieceType::Rook => 2,
                PieceType::Queen => 1,
                PieceType::King => 1,
            }
        }

        let mut list = Vec::new();
        let target_color = capturer.opposite();
        for piece_type in [
            PieceType::Queen,
            PieceType::Rook,
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::Pawn,
        ] {
            let mut on_board = 0i32;
            for row in &board.squares {
                for sq in row {
                    if let Some(piece) = sq
                        && piece.color == target_color
                        && piece.piece_type == piece_type
                    {
                        on_board += 1;
                    }
                }
            }
            let captured = (start_count(piece_type) - on_board).max(0);
            for _ in 0..captured {
                list.push(piece_type);
            }
        }
        list
    }

    fn side_material_advantage(board: &Board, side: Color) -> i32 {
        let mut white_material = 0i32;
        let mut black_material = 0i32;
        for row in &board.squares {
            for piece in row.iter().flatten() {
                let value = piece.piece_type.material_value();
                match piece.color {
                    Color::White => white_material += value,
                    Color::Black => black_material += value,
                }
            }
        }
        match side {
            Color::White => (white_material - black_material).max(0),
            Color::Black => (black_material - white_material).max(0),
        }
    }

    fn board_eval_cp_white_pov(&self, review: &ReviewState) -> Option<i32> {
        let idx = review.current_index;
        let mut cp = review
            .analyses
            .get(idx)
            .and_then(|a| a.as_ref())
            .and_then(|a| a.lines.first())
            .map(|line| line.eval_cp)?;
        if idx % 2 == 1 {
            cp = -cp;
        }
        Some(cp)
    }

    fn board_eval_bar_white_share(&self, review: &ReviewState) -> f32 {
        if let Some(cp) = self.board_eval_cp_white_pov(review) {
            if cp.abs() > 29000 {
                return if cp > 0 { 1.0 } else { 0.0 };
            }
            return (0.5 + 0.5 * (cp as f32 / 420.0).tanh()).clamp(0.0, 1.0);
        }

        let color_to_move = if review.current_index.is_multiple_of(2) {
            Color::White
        } else {
            Color::Black
        };
        let board = review.current_board();
        if board.generate_moves(color_to_move).is_empty() && board.is_in_check(color_to_move) {
            return if color_to_move == Color::White {
                0.0
            } else {
                1.0
            };
        }
        0.5
    }

    fn board_eval_text(&self, review: &ReviewState) -> String {
        if let Some(cp) = self.board_eval_cp_white_pov(review) {
            if cp.abs() > 29000 {
                let mate_ply = 30000 - cp.abs();
                if cp > 0 {
                    return format!("M{}", mate_ply);
                }
                return format!("-M{}", mate_ply);
            }
            return format!("{:+.2}", cp as f32 / 100.0);
        }

        let color_to_move = if review.current_index.is_multiple_of(2) {
            Color::White
        } else {
            Color::Black
        };
        let board = review.current_board();
        if board.generate_moves(color_to_move).is_empty() && board.is_in_check(color_to_move) {
            if color_to_move == Color::White {
                return "-M0".to_string();
            }
            return "M0".to_string();
        }
        "0.00".to_string()
    }

    fn board_for_ply(&self, ply: usize) -> Board {
        let mut board = self.initial_board.clone();
        for record in self.move_history.iter().take(ply) {
            board.make_move(record.mv);
        }
        board
    }

    fn last_move_for_ply(&self, ply: usize) -> Option<Move> {
        if ply == 0 {
            None
        } else {
            self.move_history.get(ply - 1).map(|r| r.mv)
        }
    }

    fn is_viewing_history_in_playing(&self) -> bool {
        self.playing_view_ply.is_some()
    }

    fn normalize_position_key(fen: &str) -> String {
        fen.split_whitespace().take(4).collect::<Vec<_>>().join(" ")
    }

    fn is_threefold_repetition_draw(&self) -> bool {
        let current_key = self.board.position_key(self.current_player);
        let mut repeats = 0usize;

        let initial_key = self.initial_board.position_key(Color::White);
        if initial_key == current_key {
            repeats += 1;
        }

        for rec in &self.move_history {
            if Self::normalize_position_key(&rec.fen_after) == current_key {
                repeats += 1;
                if repeats >= 3 {
                    return true;
                }
            }
        }
        false
    }

    fn record_move(&mut self, mv: Move) {
        let san = pgn::move_to_san(&self.board, &mv, self.current_player);
        self.board.make_move(mv);
        let next_player = self.current_player.opposite();
        let fen_after = self.board.to_fen(next_player);
        self.move_history.push(MoveRecord {
            mv,
            san,
            fen_after,
            color: self.current_player,
        });
        self.last_move = Some(mv);
        self.playing_view_ply = None;
    }

    fn game_result_pgn(&self) -> &str {
        match self.game_state {
            GameState::WhiteWins => "1-0",
            GameState::BlackWins => "0-1",
            GameState::Draw => "1/2-1/2",
            GameState::Playing => "*",
        }
    }

    fn do_pgn_copy(&mut self) {
        let date = chrono::Local::now().format("%Y.%m.%d").to_string();
        let (white_name, black_name) = self.names_by_color();
        let pgn_text = pgn::generate_pgn(
            &self.move_history,
            &white_name,
            &black_name,
            self.game_result_pgn(),
            &date,
        );
        if self.clipboard.is_none() {
            self.clipboard = arboard::Clipboard::new().ok().map(Mutex::new);
        }
        if let Some(ref clipboard_mutex) = self.clipboard
            && let Ok(mut clipboard) = clipboard_mutex.lock()
            && clipboard.set_text(pgn_text).is_ok()
        {
            self.pgn_copied_msg = Some(Instant::now());
        }
    }

    fn review_analysis_depth_plan(&self) -> Vec<u32> {
        match self.engine_type {
            EngineType::Stockfish => vec![12, 16, 20],
            EngineType::Lc0 => vec![18],
            EngineType::BuiltIn => vec![self.ai_difficulty.get_depth().max(4)],
        }
    }

    fn enter_review_mode(&mut self) {
        if self.move_history.is_empty() {
            return;
        }
        let mut review = ReviewState::new(self.initial_board.clone(), self.move_history.clone());
        review.set_analysis_depth_plan(self.review_analysis_depth_plan());
        review.start_analysis();
        self.review_state = Some(review);
        self.review_hover_top_line = None;
        self.app_mode = AppMode::Review;
        self.playing_view_ply = None;
        self.dragging = None;
        self.show_setup_dialog = false;
    }

    fn review_best_tip_text(review: &ReviewState) -> String {
        if let Some(Some(a)) = review.analyses.get(review.current_index)
            && let Some(line) = a.lines.first()
            && let Ok(mv) = Board::uci_to_move(&line.best_move_uci)
        {
            let color = if review.current_index.is_multiple_of(2) {
                Color::White
            } else {
                Color::Black
            };
            let san = pgn::move_to_san(review.current_board(), &mv, color);
            if !san.is_empty() {
                return format!("{} is the best move", san);
            }
            return format!("{} is the best move", line.best_move_uci);
        }
        "No best move tip yet".to_string()
    }

    fn opening_name_from_records(records: &[MoveRecord]) -> String {
        let first = records.first().map(|r| r.san.as_str()).unwrap_or("");
        let second = records.get(1).map(|r| r.san.as_str()).unwrap_or("");
        match (first, second) {
            ("e4", "e5") => "Open Game".to_string(),
            ("d4", "d5") => "Closed Game".to_string(),
            ("e4", "c5") => "Sicilian Defence".to_string(),
            ("d4", "Nf6") => "Indian Defence".to_string(),
            _ => "Custom Opening".to_string(),
        }
    }

    fn view_to_board_coords(square: (usize, usize), flipped: bool) -> (usize, usize) {
        if flipped {
            (7 - square.0, 7 - square.1)
        } else {
            square
        }
    }

    fn board_to_view_coords(square: (usize, usize), flipped: bool) -> (usize, usize) {
        if flipped {
            (7 - square.0, 7 - square.1)
        } else {
            square
        }
    }

    fn start_ai_if_needed(&mut self) {
        if self.game_state == GameState::Playing && self.current_player != self.human_color {
            self.start_ai_move();
        }
    }

    fn start_ai_move(&mut self) {
        self.status_message = format!("{} is thinking...", self.engine_name());
        self.ai_thinking = true;
        self.ai_move_start = Some(Instant::now());
        self.engine_manager
            .request_move(&self.board, self.current_player, &self.ai_difficulty);
    }

    pub fn undo_move(&mut self) {
        if self.ai_thinking || self.promotion_pending.is_some() || self.board_history.is_empty() {
            return;
        }

        let mut pop_count = 0usize;
        while let Some((prev_board, prev_color)) = self.board_history.pop() {
            pop_count += 1;
            self.board = prev_board;
            self.current_player = prev_color;
            if self.current_player == self.human_color || self.board_history.is_empty() {
                break;
            }
        }

        if pop_count > 0 {
            let new_len = self.move_history.len().saturating_sub(pop_count);
            self.move_history.truncate(new_len);
            self.last_move = self.move_history.last().map(|r| r.mv);
        }

        self.selected_square = None;
        self.valid_moves.clear();
        self.promotion_pending = None;
        self.ai_thinking = false;
        self.ai_move_start = None;
        self.dragging = None;
        self.playing_view_ply = None;
        self.update_game_state();
        self.start_ai_if_needed();
    }

    pub fn piece_to_unicode(&self, piece: Piece) -> &str {
        let _ = piece.color;
        Self::piece_symbol(piece.piece_type)
    }

    pub fn handle_square_click(&mut self, row: usize, col: usize) {
        if self.game_state != GameState::Playing
            || self.current_player != self.human_color
            || self.ai_thinking
            || self.promotion_pending.is_some()
        {
            return;
        }

        if let Some(selected) = self.selected_square {
            let mv = Move {
                from: selected,
                to: (row, col),
                promotion: None,
            };

            let is_promotion = if let Some(piece) = self.board.get_piece(selected) {
                piece.piece_type == PieceType::Pawn
                    && ((piece.color == Color::White && row == 0)
                        || (piece.color == Color::Black && row == 7))
            } else {
                false
            };

            let move_found = self
                .valid_moves
                .iter()
                .any(|valid_mv| valid_mv.from == mv.from && valid_mv.to == mv.to);

            if move_found {
                if is_promotion {
                    self.promotion_pending = Some(mv);
                    self.status_message = "Choose piece for promotion".to_string();
                } else {
                    self.board_history
                        .push((self.board.clone(), self.current_player));
                    self.record_move(mv);
                    self.selected_square = None;
                    self.valid_moves.clear();
                    self.dragging = None;
                    self.current_player = self.current_player.opposite();
                    self.update_game_state();
                    self.start_ai_if_needed();
                }
            } else if let Some(piece) = self.board.get_piece((row, col)) {
                if piece.color == self.current_player {
                    self.selected_square = Some((row, col));
                    self.valid_moves = self
                        .board
                        .generate_moves(self.current_player)
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
        } else if let Some(piece) = self.board.get_piece((row, col))
            && piece.color == self.current_player
        {
            self.selected_square = Some((row, col));
            self.valid_moves = self
                .board
                .generate_moves(self.current_player)
                .into_iter()
                .filter(|mv| mv.from == (row, col))
                .collect();
        }
    }

    pub fn handle_review_square_click(&mut self, row: usize, col: usize) {
        if self.review_state.is_none() {
            return;
        }
        let mut review = self.review_state.take().unwrap();
        let board = review.current_board().clone();
        let current_color = if review.current_index.is_multiple_of(2) {
            Color::White
        } else {
            Color::Black
        };

        if let Some(selected) = review.selected_square {
            let tentative = Move {
                from: selected,
                to: (row, col),
                promotion: None,
            };
            if let Some(actual_move) = review
                .valid_moves
                .iter()
                .find(|m| m.from == tentative.from && m.to == tentative.to)
                .copied()
            {
                let san = pgn::move_to_san(&board, &actual_move, current_color);
                review.line_apply_existing_or_branch_move(actual_move, san, current_color);
                review.selected_square = None;
                review.valid_moves.clear();
                self.dragging = None;
            } else if let Some(piece) = board.get_piece((row, col)) {
                if piece.color == current_color {
                    review.selected_square = Some((row, col));
                    review.valid_moves = board
                        .generate_moves(current_color)
                        .into_iter()
                        .filter(|mv| mv.from == (row, col))
                        .collect();
                } else {
                    review.selected_square = None;
                    review.valid_moves.clear();
                    self.dragging = None;
                }
            } else {
                review.selected_square = None;
                review.valid_moves.clear();
                self.dragging = None;
            }
        } else if let Some(piece) = board.get_piece((row, col))
            && piece.color == current_color
        {
            review.selected_square = Some((row, col));
            review.valid_moves = board
                .generate_moves(current_color)
                .into_iter()
                .filter(|mv| mv.from == (row, col))
                .collect();
        }

        self.review_state = Some(review);
    }

    pub fn update_game_state(&mut self) {
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
        } else if self.board.is_fifty_move_rule_draw() {
            self.game_state = GameState::Draw;
            self.status_message = "Draw by 50-move rule".to_string();
        } else if self.is_threefold_repetition_draw() {
            self.game_state = GameState::Draw;
            self.status_message = "Draw by threefold repetition".to_string();
        } else if self.board.has_insufficient_material() {
            self.game_state = GameState::Draw;
            self.status_message = "Draw by insufficient material".to_string();
        } else if self.board.is_in_check(self.current_player) {
            self.game_state = GameState::Playing;
            self.status_message = format!("{:?} is in check!", self.current_player);
        } else {
            self.game_state = GameState::Playing;
            self.status_message = format!("{:?} to move", self.current_player);
        }
    }

    pub fn new_game(&mut self) {
        let board = Board::new();
        self.board = board.clone();
        self.initial_board = board;
        self.current_player = Color::White;
        self.selected_square = None;
        self.valid_moves.clear();
        self.game_state = GameState::Playing;
        self.status_message = "White to move".to_string();
        self.ai_thinking = false;
        self.ai_move_start = None;
        self.promotion_pending = None;
        self.board_history.clear();
        self.move_history.clear();
        self.last_move = None;
        self.app_mode = AppMode::Playing;
        self.review_state = None;
        self.review_hover_top_line = None;
        self.playing_view_ply = None;
        self.dragging = None;

        self.start_ai_if_needed();
    }

    pub fn set_ai_difficulty(&mut self, difficulty: AIDifficulty) {
        self.ai_difficulty = difficulty;
        self.ai = ChessAI::new(difficulty.get_depth());
        self.ai.time_limit = difficulty.get_time_limit();
        self.engine_manager.update_difficulty(&difficulty);
    }

    fn handle_promotion_choice(&mut self, piece_type: PieceType) {
        if let Some(mut mv) = self.promotion_pending {
            self.board_history
                .push((self.board.clone(), self.current_player));
            mv.promotion = Some(piece_type);
            self.record_move(mv);
            self.selected_square = None;
            self.valid_moves.clear();
            self.promotion_pending = None;
            self.dragging = None;
            self.current_player = self.current_player.opposite();
            self.update_game_state();
            self.start_ai_if_needed();
        }
    }

    fn show_settings_window(&mut self, ctx: &egui::Context) {
        if !self.show_setup_dialog {
            return;
        }

        egui::Window::new("Game Setup")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_min_width(320.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Start a new game with your preferences")
                            .color(Color32::LIGHT_GRAY),
                    );
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Play as:");
                        egui::ComboBox::from_id_salt("setup_human_color")
                            .selected_text(Self::color_name(self.human_color))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.human_color, Color::White, "White");
                                ui.selectable_value(&mut self.human_color, Color::Black, "Black");
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Your name:");
                        ui.add_sized(
                            [220.0, 28.0],
                            egui::TextEdit::singleline(&mut self.human_name).hint_text("Player"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Opponent:");
                        ui.add_sized(
                            [220.0, 28.0],
                            egui::TextEdit::singleline(&mut self.opponent_name)
                                .hint_text("Computer"),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Difficulty:");
                        let old_difficulty = self.ai_difficulty;
                        egui::ComboBox::from_id_salt("setup_ai_difficulty")
                            .selected_text(self.ai_difficulty.to_string())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.ai_difficulty,
                                    AIDifficulty::Easy,
                                    "Easy",
                                );
                                ui.selectable_value(
                                    &mut self.ai_difficulty,
                                    AIDifficulty::Medium,
                                    "Medium",
                                );
                                ui.selectable_value(
                                    &mut self.ai_difficulty,
                                    AIDifficulty::Hard,
                                    "Hard",
                                );
                                ui.selectable_value(
                                    &mut self.ai_difficulty,
                                    AIDifficulty::Expert,
                                    "Expert",
                                );
                            });
                        if old_difficulty != self.ai_difficulty {
                            self.set_ai_difficulty(self.ai_difficulty);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Engine:");
                        let old_engine = self.engine_type;
                        egui::ComboBox::from_id_salt("setup_engine")
                            .selected_text(self.engine_name())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.engine_type,
                                    EngineType::Stockfish,
                                    "Stockfish",
                                );
                                ui.selectable_value(
                                    &mut self.engine_type,
                                    EngineType::Lc0,
                                    "Leela Chess Zero",
                                );
                                ui.selectable_value(
                                    &mut self.engine_type,
                                    EngineType::BuiltIn,
                                    "Built-in AI",
                                );
                            });
                        if old_engine != self.engine_type {
                            self.engine_manager.set_engine(self.engine_type);
                            self.engine_type = self.engine_manager.engine_type();
                            self.ai_thinking = false;
                            self.ai_move_start = None;
                            self.start_ai_if_needed();
                        }
                    });

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized([140.0, 32.0], egui::Button::new("Start / Restart"))
                            .clicked()
                        {
                            self.new_game();
                            self.show_setup_dialog = false;
                        }

                        if ui
                            .add_sized([120.0, 32.0], egui::Button::new("Close"))
                            .clicked()
                        {
                            self.show_setup_dialog = false;
                        }
                    });
                });
            });
    }

    fn draw_player_strip(
        &self,
        ui: &mut egui::Ui,
        board: &Board,
        side: Color,
        width: f32,
        active: bool,
    ) {
        let name = self.display_name_for_color(side);
        let captured = Self::side_captured_pieces(board, side);
        let material_adv = Self::side_material_advantage(board, side);
        let side_text = match side {
            Color::White => "White",
            Color::Black => "Black",
        };
        let chip_bg = if active {
            Color32::from_rgb(36, 48, 63)
        } else {
            Color32::from_rgb(28, 36, 48)
        };
        let chip_border = if active {
            Color32::from_rgb(96, 138, 188)
        } else {
            Color32::from_rgb(64, 88, 118)
        };
        let side_indicator = if side == Color::White {
            Color32::from_rgb(236, 240, 245)
        } else {
            Color32::from_rgb(42, 48, 58)
        };

        egui::Frame::none()
            .fill(chip_bg)
            .stroke(Stroke::new(1.0, chip_border))
            .rounding(7.0)
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .show(ui, |ui| {
                ui.set_width(width);
                ui.horizontal(|ui| {
                    let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                    ui.painter()
                        .circle_filled(dot_rect.center(), 5.0, side_indicator);

                    ui.label(
                        egui::RichText::new(name)
                            .size(16.0)
                            .color(TEXT_PRIMARY)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!("({})", side_text))
                            .size(12.0)
                            .color(TEXT_MUTED),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if material_adv > 0 {
                            ui.label(
                                egui::RichText::new(format!("+{}", material_adv))
                                    .size(15.0)
                                    .color(ACCENT_GREEN)
                                    .strong(),
                            );
                        }
                        if captured.is_empty() {
                            ui.label(egui::RichText::new("—").size(13.0).color(TEXT_MUTED));
                        } else {
                            ui.horizontal(|ui| {
                                for piece_type in captured.iter().take(12) {
                                    ui.label(
                                        egui::RichText::new(Self::piece_symbol_for_color(
                                            *piece_type,
                                            side.opposite(),
                                        ))
                                        .size(15.0)
                                        .color(TEXT_SECONDARY),
                                    );
                                }
                            });
                        }
                    });
                });
            });
    }

    fn draw_review_eval_bar(&self, ctx: &egui::Context) {
        let Some(ref review) = self.review_state else {
            return;
        };

        let white_share = self.board_eval_bar_white_share(review);
        let eval_text = self.board_eval_text(review);
        egui::SidePanel::left("review_eval_bar")
            .resizable(false)
            .default_width(58.0)
            .min_width(50.0)
            .max_width(74.0)
            .frame(
                egui::Frame::none()
                    .fill(BG_DARK)
                    .inner_margin(egui::Margin::symmetric(8.0, 8.0)),
            )
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("B")
                            .size(12.0)
                            .color(TEXT_SECONDARY)
                            .strong(),
                    );
                    ui.add_space(3.0);
                    let bar_height = (ui.available_height() - 54.0).max(100.0);
                    let bar_width = (ui.available_width() - 8.0).clamp(22.0, 34.0);
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(bar_width, bar_height), Sense::hover());
                    let painter = ui.painter_at(rect);
                    let rounding = 7.0;

                    painter.rect_filled(rect, rounding, Color32::from_rgb(20, 24, 30));

                    let white_h = rect.height() * white_share;
                    let white_rect = Rect::from_min_max(
                        Pos2::new(rect.min.x, rect.max.y - white_h),
                        Pos2::new(rect.max.x, rect.max.y),
                    );
                    painter.rect_filled(white_rect, rounding, Color32::from_rgb(232, 237, 243));
                    painter.rect_stroke(
                        rect,
                        rounding,
                        Stroke::new(1.0, Color32::from_rgb(70, 84, 104)),
                    );

                    let split_y = rect.max.y - white_h;
                    painter.line_segment(
                        [
                            Pos2::new(rect.min.x, split_y),
                            Pos2::new(rect.max.x, split_y),
                        ],
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(88, 122, 176, 190)),
                    );

                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(eval_text)
                            .size(13.0)
                            .color(ACCENT_GREEN)
                            .strong(),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("W")
                            .size(12.0)
                            .color(TEXT_SECONDARY)
                            .strong(),
                    );
                });
            });
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_board_widget(
        &mut self,
        ui: &mut egui::Ui,
        board: &Board,
        interactive: bool,
        last_move: Option<&Move>,
        best_moves: Option<&Vec<Move>>,
        last_move_class: Option<MoveClassification>,
    ) -> Option<(usize, usize)> {
        let coordinate_size = 20.0;
        let available = ui.available_size();
        let board_size = (available.x - coordinate_size - 8.0)
            .min(available.y - coordinate_size - 96.0)
            .clamp(320.0, 880.0);
        let square_size = (board_size / 8.0).floor();
        let board_size = square_size * 8.0;
        let total_size = Vec2::new(board_size + coordinate_size, board_size + coordinate_size);
        let strip_height = 36.0;
        let strip_gap = 6.0;
        let total_height = total_size.y + strip_height * 2.0 + strip_gap * 2.0;

        let mut clicked_square = None;
        let flipped = self.human_color == Color::Black;
        let (top_side, bottom_side) = if flipped {
            (Color::White, Color::Black)
        } else {
            (Color::Black, Color::White)
        };
        let active_side = if self.app_mode == AppMode::Review {
            self.review_state.as_ref().map(|r| {
                if r.current_index.is_multiple_of(2) {
                    Color::White
                } else {
                    Color::Black
                }
            })
        } else if self.game_state == GameState::Playing {
            Some(self.current_player)
        } else {
            None
        };

        ui.add_space(((ui.available_height() - total_height).max(0.0)) * 0.5);
        ui.horizontal(|ui| {
            let pad = (ui.available_width() - total_size.x).max(0.0) * 0.5;
            if pad > 0.0 {
                ui.add_space(pad);
            }

            ui.vertical(|ui| {
                self.draw_player_strip(
                    ui,
                    board,
                    top_side,
                    total_size.x,
                    active_side == Some(top_side),
                );
                ui.add_space(strip_gap);

                let (response, painter) = ui.allocate_painter(
                    total_size,
                    if interactive {
                        Sense::click_and_drag()
                    } else {
                        Sense::hover()
                    },
                );
                let board_rect = Rect::from_min_size(
                    Pos2::new(response.rect.min.x + coordinate_size, response.rect.min.y),
                    Vec2::new(board_size, board_size),
                );

                for view_row in 0..8 {
                    for view_col in 0..8 {
                        let (row, col) = Self::view_to_board_coords((view_row, view_col), flipped);
                        let square_rect = Rect::from_min_size(
                            Pos2::new(
                                board_rect.min.x + view_col as f32 * square_size,
                                board_rect.min.y + view_row as f32 * square_size,
                            ),
                            Vec2::splat(square_size),
                        );
                        let is_light = (row + col) % 2 == 0;
                        let base = if is_light { LIGHT_SQUARE } else { DARK_SQUARE };
                        painter.rect_filled(square_rect, 0.0, base);

                        if let Some(lm) = last_move {
                            if (row, col) == lm.from {
                                painter.rect_filled(
                                    square_rect,
                                    0.0,
                                    Color32::from_rgba_unmultiplied(252, 219, 98, 118),
                                );
                            } else if (row, col) == lm.to {
                                painter.rect_filled(
                                    square_rect,
                                    0.0,
                                    Color32::from_rgba_unmultiplied(110, 185, 232, 118),
                                );
                            }
                        }

                        let is_selected = if self.app_mode == AppMode::Review {
                            self.review_state
                                .as_ref()
                                .map(|r| r.selected_square == Some((row, col)))
                                .unwrap_or(false)
                        } else {
                            self.selected_square == Some((row, col))
                        };
                        if is_selected {
                            painter.rect_filled(
                                square_rect,
                                0.0,
                                Color32::from_rgba_unmultiplied(
                                    SELECTED_SQUARE.r(),
                                    SELECTED_SQUARE.g(),
                                    SELECTED_SQUARE.b(),
                                    120,
                                ),
                            );
                        }

                        if let Some(piece) = board.get_piece((row, col)) {
                            if self.dragging == Some((row, col)) {
                                continue;
                            }
                            if piece.piece_type == PieceType::King && board.is_in_check(piece.color)
                            {
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

                            let symbol = self.piece_to_unicode(piece);
                            let font = egui::FontId::proportional(square_size * 0.42);
                            let center = square_rect.center();
                            let (piece_color, outline_color) = match piece.color {
                                Color::White => (
                                    Color32::from_rgb(242, 242, 242),
                                    Color32::from_rgb(32, 32, 32),
                                ),
                                Color::Black => (
                                    Color32::from_rgb(22, 22, 22),
                                    Color32::from_rgb(210, 210, 210),
                                ),
                            };
                            for (dx, dy) in
                                [(-0.7f32, -0.7f32), (0.7, -0.7), (-0.7, 0.7), (0.7, 0.7)]
                            {
                                painter.text(
                                    center + egui::vec2(dx, dy),
                                    egui::Align2::CENTER_CENTER,
                                    symbol,
                                    font.clone(),
                                    outline_color,
                                );
                            }
                            painter.text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                symbol,
                                font,
                                piece_color,
                            );
                        }

                        let is_valid_target = if self.app_mode == AppMode::Review {
                            self.review_state
                                .as_ref()
                                .map(|r| r.valid_moves.iter().any(|mv| mv.to == (row, col)))
                                .unwrap_or(false)
                        } else {
                            self.valid_moves.iter().any(|mv| mv.to == (row, col))
                        };
                        if is_valid_target {
                            let center = square_rect.center();
                            if board.get_piece((row, col)).is_some() {
                                painter.circle_stroke(
                                    center,
                                    square_size * 0.44,
                                    egui::Stroke::new(square_size * 0.06, VALID_TARGET),
                                );
                            } else {
                                painter.circle_filled(center, square_size * 0.13, VALID_TARGET);
                            }
                        }
                    }
                }

                if let Some(lines) = best_moves {
                    let colors = [
                        Color32::from_rgba_unmultiplied(45, 213, 173, 230),
                        Color32::from_rgba_unmultiplied(81, 184, 255, 220),
                        Color32::from_rgba_unmultiplied(175, 153, 255, 210),
                    ];
                    let focused = if self.app_mode == AppMode::Review {
                        self.review_hover_top_line
                    } else {
                        None
                    };
                    for (i, mv) in lines.iter().enumerate().take(3) {
                        let is_primary = focused.map(|idx| idx == i).unwrap_or(true);
                        let dim_scale = if focused.is_some() && !is_primary {
                            0.28
                        } else {
                            1.0
                        };
                        let base = colors[i];
                        let color = Color32::from_rgba_unmultiplied(
                            base.r(),
                            base.g(),
                            base.b(),
                            ((base.a() as f32) * dim_scale).round().clamp(0.0, 255.0) as u8,
                        );
                        let from = Self::board_to_view_coords(mv.from, flipped);
                        let to = Self::board_to_view_coords(mv.to, flipped);
                        let start = Pos2::new(
                            board_rect.min.x + from.1 as f32 * square_size + square_size * 0.5,
                            board_rect.min.y + from.0 as f32 * square_size + square_size * 0.5,
                        );
                        let end = Pos2::new(
                            board_rect.min.x + to.1 as f32 * square_size + square_size * 0.5,
                            board_rect.min.y + to.0 as f32 * square_size + square_size * 0.5,
                        );
                        let vec = end - start;
                        let len = vec.length();
                        if len < 1.0 {
                            continue;
                        }
                        let dir = vec / len;
                        let normal = egui::Vec2::new(-dir.y, dir.x);
                        let shaft_start = start + dir * (square_size * 0.18);
                        let tip = end - dir * (square_size * 0.14);
                        let shaft_end = tip - dir * (square_size * 0.20);
                        let shaft_width = square_size * (0.14 - i as f32 * 0.02).max(0.08);

                        painter.line_segment(
                            [shaft_start + normal * 0.6, shaft_end + normal * 0.6],
                            Stroke::new(
                                shaft_width + if is_primary { 2.4 } else { 1.9 },
                                Color32::from_rgba_unmultiplied(
                                    0,
                                    0,
                                    0,
                                    if is_primary { 75 } else { 45 },
                                ),
                            ),
                        );
                        painter.line_segment(
                            [shaft_start, shaft_end],
                            Stroke::new(shaft_width, color),
                        );
                        painter.line_segment(
                            [shaft_start + normal * 0.6, shaft_end + normal * 0.6],
                            Stroke::new(
                                shaft_width * 0.34,
                                Color32::from_rgba_unmultiplied(255, 255, 255, 58),
                            ),
                        );

                        let head_back = tip - dir * (square_size * 0.30);
                        let head_w = shaft_width * 1.25;
                        let head_left = head_back + normal * head_w;
                        let head_right = head_back - normal * head_w;
                        painter.add(egui::Shape::convex_polygon(
                            vec![tip, head_left, head_right],
                            color,
                            Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
                        ));

                        painter.circle_stroke(
                            start,
                            square_size * 0.19,
                            Stroke::new(square_size * 0.045, color),
                        );
                        painter.circle_filled(
                            start,
                            square_size * 0.07,
                            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 220),
                        );

                        let rank_badge_pos =
                            tip + normal * (square_size * 0.22) - dir * (square_size * 0.05);
                        painter.circle_filled(
                            rank_badge_pos,
                            square_size * 0.14,
                            Color32::from_rgba_unmultiplied(
                                11,
                                16,
                                24,
                                if is_primary { 230 } else { 190 },
                            ),
                        );
                        painter.circle_stroke(
                            rank_badge_pos,
                            square_size * 0.14,
                            Stroke::new(
                                1.1,
                                Color32::from_rgba_unmultiplied(
                                    color.r(),
                                    color.g(),
                                    color.b(),
                                    210,
                                ),
                            ),
                        );
                        painter.text(
                            rank_badge_pos,
                            egui::Align2::CENTER_CENTER,
                            (i + 1).to_string(),
                            egui::FontId::proportional(square_size * 0.18),
                            Color32::WHITE,
                        );
                    }
                }

                if let Some(lm) = last_move
                    && let Some(class) = last_move_class
                {
                    let to = Self::board_to_view_coords(lm.to, flipped);
                    let square = Rect::from_min_size(
                        Pos2::new(
                            board_rect.min.x + to.1 as f32 * square_size,
                            board_rect.min.y + to.0 as f32 * square_size,
                        ),
                        Vec2::splat(square_size),
                    );
                    let [r, g, b] = class.color32();
                    let color = Color32::from_rgb(r, g, b);
                    let badge = Pos2::new(
                        square.max.x - square_size * 0.17,
                        square.min.y + square_size * 0.17,
                    );
                    painter.circle_filled(badge, square_size * 0.2, color);
                    painter.text(
                        badge,
                        egui::Align2::CENTER_CENTER,
                        class.icon(),
                        egui::FontId::proportional(square_size * 0.24),
                        Color32::WHITE,
                    );
                }

                painter.rect_stroke(
                    board_rect,
                    0.0,
                    egui::Stroke::new(2.0, Color32::from_rgb(62, 76, 97)),
                );

                for view_col in 0..8 {
                    let (_, board_col) = Self::view_to_board_coords((7, view_col), flipped);
                    let file_char = (b'a' + board_col as u8) as char;
                    let x = board_rect.min.x + view_col as f32 * square_size + square_size / 2.0;
                    let y = board_rect.max.y + coordinate_size / 2.0;
                    painter.text(
                        Pos2::new(x, y),
                        egui::Align2::CENTER_CENTER,
                        file_char.to_string(),
                        egui::FontId::proportional(16.0),
                        Color32::from_rgb(176, 190, 210),
                    );
                }
                for view_row in 0..8 {
                    let (board_row, _) = Self::view_to_board_coords((view_row, 0), flipped);
                    let rank_num = 8 - board_row;
                    let x = board_rect.min.x - coordinate_size / 2.0;
                    let y = board_rect.min.y + view_row as f32 * square_size + square_size / 2.0;
                    painter.text(
                        Pos2::new(x, y),
                        egui::Align2::CENTER_CENTER,
                        rank_num.to_string(),
                        egui::FontId::proportional(16.0),
                        Color32::from_rgb(176, 190, 210),
                    );
                }

                if let Some((drag_r, drag_c)) = self.dragging
                    && let Some(piece) = board.get_piece((drag_r, drag_c))
                    && let Some(pointer_pos) = ui.ctx().pointer_latest_pos()
                {
                    let symbol = self.piece_to_unicode(piece);
                    let font_size = square_size * 0.48;
                    let (piece_color, outline_color) = match piece.color {
                        Color::White => (
                            Color32::from_rgb(242, 242, 242),
                            Color32::from_rgba_unmultiplied(32, 32, 32, 220),
                        ),
                        Color::Black => (
                            Color32::from_rgb(22, 22, 22),
                            Color32::from_rgba_unmultiplied(210, 210, 210, 180),
                        ),
                    };
                    for (dx, dy) in [(-0.9f32, -0.9f32), (0.9, -0.9), (-0.9, 0.9), (0.9, 0.9)] {
                        painter.text(
                            pointer_pos + egui::vec2(dx, dy),
                            egui::Align2::CENTER_CENTER,
                            symbol,
                            egui::FontId::proportional(font_size),
                            outline_color,
                        );
                    }
                    painter.text(
                        pointer_pos,
                        egui::Align2::CENTER_CENTER,
                        symbol,
                        egui::FontId::proportional(font_size),
                        piece_color,
                    );
                }

                if interactive {
                    if response.drag_started()
                        || (response.hovered()
                            && ui.input(|i| {
                                i.pointer.button_pressed(egui::PointerButton::Primary)
                                    && !response.dragged()
                            }))
                    {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let relative_pos = pos - board_rect.min;
                            let view_col = (relative_pos.x / square_size) as usize;
                            let view_row = (relative_pos.y / square_size) as usize;
                            if view_row < 8 && view_col < 8 {
                                let square =
                                    Self::view_to_board_coords((view_row, view_col), flipped);
                                if board.get_piece(square).is_some() {
                                    self.dragging = Some(square);
                                }
                                clicked_square = Some(square);
                            }
                        }
                    } else if response.drag_stopped() {
                        if let Some(pos) = ui.ctx().pointer_interact_pos() {
                            let relative_pos = pos - board_rect.min;
                            let view_col = (relative_pos.x / square_size) as usize;
                            let view_row = (relative_pos.y / square_size) as usize;
                            if view_row < 8 && view_col < 8 {
                                clicked_square =
                                    Some(Self::view_to_board_coords((view_row, view_col), flipped));
                            }
                        }
                        self.dragging = None;
                    } else if !response.dragged()
                        && self.dragging.is_some()
                        && !ui.input(|i| i.pointer.primary_down())
                    {
                        self.dragging = None;
                    }
                }
                ui.add_space(strip_gap);
                self.draw_player_strip(
                    ui,
                    board,
                    bottom_side,
                    total_size.x,
                    active_side == Some(bottom_side),
                );
            });
        });

        clicked_square
    }

    fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("status_bar")
            .resizable(false)
            .frame(egui::Frame::none().fill(BG_PANEL).inner_margin(6.0))
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let you_name = self.human_name_display();
                    let opp_name = self.opponent_name_display();
                    let status_text = if let Some(ply) = self.playing_view_ply {
                        format!(
                            "{} | Browsing ply {}/{} (click board to return live)",
                            self.status_message,
                            ply,
                            self.move_history.len()
                        )
                    } else {
                        self.status_message.clone()
                    };
                    ui.label(
                        egui::RichText::new("Chess Game")
                            .color(TEXT_PRIMARY)
                            .strong(),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("Status: {}", status_text)).color(TEXT_PRIMARY),
                    );
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!(
                            "You: {} ({})",
                            you_name,
                            Self::color_name(self.human_color)
                        ))
                        .color(TEXT_SECONDARY),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "Opponent: {} ({})",
                            opp_name,
                            Self::color_name(self.human_color.opposite())
                        ))
                        .color(TEXT_SECONDARY),
                    );
                    ui.label(
                        egui::RichText::new(format!("Engine: {}", self.engine_name()))
                            .color(ACCENT_GREEN),
                    );
                    if self.ai.nodes_searched > 0 {
                        ui.separator();
                        ui.label(
                            egui::RichText::new(format!("Nodes: {}", self.ai.nodes_searched))
                                .color(TEXT_SECONDARY),
                        );
                    }
                });
            });
    }

    fn draw_match_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("match_side_panel")
            .default_width(320.0)
            .min_width(280.0)
            .max_width(460.0)
            .resizable(true)
            .frame(
                egui::Frame::none()
                    .fill(BG_DARK)
                    .inner_margin(egui::Margin::same(10.0)),
            )
            .show(ctx, |ui| {
                let can_undo = !self.ai_thinking
                    && self.promotion_pending.is_none()
                    && !self.board_history.is_empty();
                let can_review = !self.move_history.is_empty()
                    && !self.ai_thinking
                    && self.promotion_pending.is_none();
                let can_copy_pgn = !self.move_history.is_empty();

                egui::Frame::none()
                    .fill(Color32::from_rgb(24, 33, 45))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(60, 84, 116)))
                    .rounding(10.0)
                    .inner_margin(egui::Margin::symmetric(12.0, 10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Match Control")
                                    .size(19.0)
                                    .color(TEXT_PRIMARY)
                                    .strong(),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(self.engine_name())
                                            .size(12.0)
                                            .color(ACCENT_GREEN),
                                    );
                                },
                            );
                        });
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Play, review and configure from one place")
                                .size(12.0)
                                .color(TEXT_MUTED),
                        );
                    });
                ui.add_space(10.0);

                egui::ScrollArea::vertical()
                    .id_salt("match_panel_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Frame::none()
                            .fill(BG_PANEL)
                            .stroke(Stroke::new(1.0, Color32::from_rgb(54, 71, 95)))
                            .rounding(9.0)
                            .inner_margin(egui::Margin::symmetric(10.0, 9.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("Live Status")
                                        .size(15.0)
                                        .color(TEXT_SECONDARY)
                                        .strong(),
                                );
                                ui.add_space(6.0);
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Turn: {} ({})",
                                        self.display_name_for_color(self.current_player),
                                        Self::color_name(self.current_player)
                                    ))
                                    .size(14.0)
                                    .color(TEXT_PRIMARY)
                                    .strong(),
                                );
                                ui.add_space(3.0);
                                ui.label(
                                    egui::RichText::new(format!("State: {:?}", self.game_state))
                                        .size(13.0)
                                        .color(TEXT_SECONDARY),
                                );
                                if self.ai.nodes_searched > 0 {
                                    ui.add_space(3.0);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "Nodes searched: {}",
                                            self.ai.nodes_searched
                                        ))
                                        .size(12.0)
                                        .color(TEXT_MUTED),
                                    );
                                }
                            });

                        ui.add_space(10.0);

                        egui::Frame::none()
                            .fill(BG_PANEL)
                            .stroke(Stroke::new(1.0, Color32::from_rgb(54, 71, 95)))
                            .rounding(9.0)
                            .inner_margin(egui::Margin::symmetric(10.0, 10.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("Actions")
                                        .size(15.0)
                                        .color(TEXT_SECONDARY)
                                        .strong(),
                                );
                                ui.add_space(8.0);
                                let gap = 8.0;
                                let btn_w = (ui.available_width() - gap).max(10.0) * 0.5;
                                let btn_h = 36.0;
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_sized(
                                            [btn_w, btn_h],
                                            egui::Button::new(
                                                egui::RichText::new("New Game").size(14.0).strong(),
                                            )
                                            .fill(Color32::from_rgb(45, 81, 128)),
                                        )
                                        .clicked()
                                    {
                                        self.new_game();
                                    }

                                    if ui
                                        .add_enabled(
                                            can_undo,
                                            egui::Button::new(
                                                egui::RichText::new("Undo").size(14.0).strong(),
                                            )
                                            .min_size(Vec2::new(btn_w, btn_h))
                                            .fill(Color32::from_rgb(67, 88, 111)),
                                        )
                                        .clicked()
                                    {
                                        self.undo_move();
                                    }
                                });
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    if ui
                                        .add_enabled(
                                            can_review,
                                            egui::Button::new(
                                                egui::RichText::new("Review").size(14.0).strong(),
                                            )
                                            .min_size(Vec2::new(btn_w, btn_h))
                                            .fill(Color32::from_rgb(42, 105, 120)),
                                        )
                                        .clicked()
                                    {
                                        self.enter_review_mode();
                                    }

                                    if ui
                                        .add_enabled(
                                            can_copy_pgn,
                                            egui::Button::new(
                                                egui::RichText::new("Copy PGN").size(14.0).strong(),
                                            )
                                            .min_size(Vec2::new(btn_w, btn_h))
                                            .fill(Color32::from_rgb(77, 93, 118)),
                                        )
                                        .clicked()
                                    {
                                        self.do_pgn_copy();
                                    }
                                });
                                ui.add_space(6.0);
                                if ui
                                    .add_sized(
                                        [ui.available_width(), btn_h],
                                        egui::Button::new(
                                            egui::RichText::new("Open Setup Window")
                                                .size(14.0)
                                                .strong(),
                                        )
                                        .fill(Color32::from_rgb(58, 76, 99)),
                                    )
                                    .clicked()
                                {
                                    self.show_setup_dialog = true;
                                }
                                if let Some(t) = self.pgn_copied_msg {
                                    if t.elapsed().as_secs() < 3 {
                                        ui.add_space(4.0);
                                        ui.label(
                                            egui::RichText::new("PGN copied")
                                                .size(12.0)
                                                .color(ACCENT_GREEN)
                                                .strong(),
                                        );
                                    } else {
                                        self.pgn_copied_msg = None;
                                    }
                                }
                            });

                        ui.add_space(10.0);

                        egui::Frame::none()
                            .fill(BG_PANEL)
                            .stroke(Stroke::new(1.0, Color32::from_rgb(54, 71, 95)))
                            .rounding(9.0)
                            .inner_margin(egui::Margin::symmetric(10.0, 10.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("Quick Settings")
                                        .size(15.0)
                                        .color(TEXT_SECONDARY)
                                        .strong(),
                                );
                                ui.add_space(8.0);

                                let old_side = self.human_color;
                                egui::ComboBox::from_id_salt("quick_human_color")
                                    .width(ui.available_width())
                                    .selected_text(format!(
                                        "Play as {}",
                                        Self::color_name(self.human_color)
                                    ))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.human_color,
                                            Color::White,
                                            "White",
                                        );
                                        ui.selectable_value(
                                            &mut self.human_color,
                                            Color::Black,
                                            "Black",
                                        );
                                    });

                                ui.add_space(6.0);
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.human_name)
                                        .hint_text("Your name")
                                        .desired_width(ui.available_width()),
                                );
                                ui.add_space(4.0);
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.opponent_name)
                                        .hint_text("Opponent name")
                                        .desired_width(ui.available_width()),
                                );

                                ui.add_space(6.0);
                                let old_difficulty = self.ai_difficulty;
                                egui::ComboBox::from_id_salt("quick_ai_difficulty")
                                    .width(ui.available_width())
                                    .selected_text(format!(
                                        "Difficulty: {}",
                                        self.ai_difficulty.to_string()
                                    ))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.ai_difficulty,
                                            AIDifficulty::Easy,
                                            "Easy",
                                        );
                                        ui.selectable_value(
                                            &mut self.ai_difficulty,
                                            AIDifficulty::Medium,
                                            "Medium",
                                        );
                                        ui.selectable_value(
                                            &mut self.ai_difficulty,
                                            AIDifficulty::Hard,
                                            "Hard",
                                        );
                                        ui.selectable_value(
                                            &mut self.ai_difficulty,
                                            AIDifficulty::Expert,
                                            "Expert",
                                        );
                                    });
                                if old_difficulty != self.ai_difficulty {
                                    self.set_ai_difficulty(self.ai_difficulty);
                                }

                                ui.add_space(6.0);
                                let old_engine = self.engine_type;
                                egui::ComboBox::from_id_salt("quick_engine_type")
                                    .width(ui.available_width())
                                    .selected_text(format!("Engine: {}", self.engine_name()))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.engine_type,
                                            EngineType::Stockfish,
                                            "Stockfish",
                                        );
                                        ui.selectable_value(
                                            &mut self.engine_type,
                                            EngineType::Lc0,
                                            "Leela Chess Zero",
                                        );
                                        ui.selectable_value(
                                            &mut self.engine_type,
                                            EngineType::BuiltIn,
                                            "Built-in AI",
                                        );
                                    });
                                if old_engine != self.engine_type {
                                    self.engine_manager.set_engine(self.engine_type);
                                    self.engine_type = self.engine_manager.engine_type();
                                    self.ai_thinking = false;
                                    self.ai_move_start = None;
                                    self.start_ai_if_needed();
                                }

                                if old_side != self.human_color {
                                    self.new_game();
                                }
                            });

                        ui.add_space(10.0);

                        let selected_ply = self.playing_view_ply.unwrap_or(self.move_history.len());
                        let mut jump_to_ply: Option<usize> = None;
                        egui::Frame::none()
                            .fill(BG_PANEL)
                            .stroke(Stroke::new(1.0, Color32::from_rgb(54, 71, 95)))
                            .rounding(9.0)
                            .inner_margin(egui::Margin::symmetric(10.0, 10.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("Moves")
                                            .size(15.0)
                                            .color(TEXT_SECONDARY)
                                            .strong(),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if self.is_viewing_history_in_playing() {
                                                ui.label(
                                                    egui::RichText::new("Browsing")
                                                        .size(12.0)
                                                        .color(Color32::from_rgb(230, 188, 96))
                                                        .strong(),
                                                );
                                            } else {
                                                ui.label(
                                                    egui::RichText::new("Live")
                                                        .size(12.0)
                                                        .color(ACCENT_GREEN)
                                                        .strong(),
                                                );
                                            }
                                        },
                                    );
                                });
                                ui.add_space(6.0);

                                if self.move_history.is_empty() {
                                    ui.label(
                                        egui::RichText::new("No moves yet.")
                                            .size(12.0)
                                            .color(TEXT_MUTED),
                                    );
                                } else {
                                    egui::ScrollArea::vertical()
                                        .id_salt("playing_moves_list")
                                        .max_height(220.0)
                                        .show(ui, |ui| {
                                            let mut i = 0usize;
                                            while i < self.move_history.len() {
                                                let move_num = i / 2 + 1;
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(format!("{:>2}.", move_num))
                                                            .size(12.0)
                                                            .color(TEXT_MUTED)
                                                            .monospace(),
                                                    );

                                                    let white_ply = i + 1;
                                                    let white_san = self.move_history[i].san.clone();
                                                    let white_selected = selected_ply == white_ply;
                                                    let white_color = if white_selected {
                                                        ACCENT_GREEN
                                                    } else {
                                                        TEXT_PRIMARY
                                                    };
                                                    let w_resp = ui.selectable_label(
                                                        white_selected,
                                                        egui::RichText::new(white_san)
                                                            .size(13.0)
                                                            .color(white_color)
                                                            .monospace(),
                                                    );
                                                    if w_resp.clicked() {
                                                        jump_to_ply = Some(white_ply);
                                                    }

                                                    if i + 1 < self.move_history.len() {
                                                        let black_ply = i + 2;
                                                        let black_san =
                                                            self.move_history[i + 1].san.clone();
                                                        let black_selected =
                                                            selected_ply == black_ply;
                                                        let black_color = if black_selected {
                                                            ACCENT_GREEN
                                                        } else {
                                                            TEXT_PRIMARY
                                                        };
                                                        let b_resp = ui.selectable_label(
                                                            black_selected,
                                                            egui::RichText::new(black_san)
                                                                .size(13.0)
                                                                .color(black_color)
                                                                .monospace(),
                                                        );
                                                        if b_resp.clicked() {
                                                            jump_to_ply = Some(black_ply);
                                                        }
                                                    }
                                                });
                                                i += 2;
                                            }
                                        });
                                }

                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new("Tip: click board while browsing to jump back to live instantly.")
                                        .size(11.0)
                                        .color(TEXT_MUTED),
                                );
                            });

                        if let Some(ply) = jump_to_ply {
                            let ply = ply.min(self.move_history.len());
                            if ply == self.move_history.len() {
                                self.playing_view_ply = None;
                            } else {
                                self.playing_view_ply = Some(ply);
                            }
                            self.selected_square = None;
                            self.valid_moves.clear();
                            self.dragging = None;
                        }
                    });
            });
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

                    ui.horizontal(|ui| {
                        ui.add_space(20.0);
                        for (symbol, name, pt) in [
                            ("♕", "Queen", PieceType::Queen),
                            ("♖", "Rook", PieceType::Rook),
                            ("♗", "Bishop", PieceType::Bishop),
                            ("♘", "Knight", PieceType::Knight),
                        ] {
                            if ui
                                .add_sized(
                                    [60.0, 60.0],
                                    egui::Button::new(format!("{}\n{}", symbol, name)),
                                )
                                .clicked()
                            {
                                self.handle_promotion_choice(pt);
                            }
                            ui.add_space(10.0);
                        }
                    });

                    ui.add_space(10.0);
                    ui.label("Click on the piece you want to promote to");
                });
            });
    }

    fn show_game_over_screen(&mut self, ctx: &egui::Context) {
        egui::Area::new("game_over_overlay".into())
            .fixed_pos(egui::pos2(0.0, 0.0))
            .show(ctx, |ui| {
                let screen_rect = ctx.screen_rect();
                ui.allocate_ui_with_layout(
                    screen_rect.size(),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        let painter = ui.painter();
                        painter.rect_filled(
                            screen_rect,
                            0.0,
                            Color32::from_rgba_unmultiplied(0, 0, 0, 180),
                        );
                    },
                );
            });

        egui::Window::new("")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_min_width(400.0);
                ui.set_min_height(280.0);

                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

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
                        }
                        _ => {}
                    }

                    ui.add_space(15.0);
                    ui.label(
                        egui::RichText::new(&self.status_message)
                            .size(16.0)
                            .color(Color32::WHITE),
                    );

                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized(
                                [120.0, 40.0],
                                egui::Button::new(
                                    egui::RichText::new("New Game")
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

                        if ui
                            .add_enabled(
                                !self.move_history.is_empty(),
                                egui::Button::new(
                                    egui::RichText::new("Review")
                                        .size(16.0)
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(52, 90, 140))
                                .min_size(Vec2::new(120.0, 40.0)),
                            )
                            .clicked()
                        {
                            self.enter_review_mode();
                        }

                        ui.add_space(10.0);

                        if ui
                            .add_sized(
                                [120.0, 40.0],
                                egui::Button::new(
                                    egui::RichText::new("Exit").size(16.0).color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(150, 0, 0)),
                            )
                            .clicked()
                        {
                            std::process::exit(0);
                        }
                    });
                });
            });
    }
}

impl Default for ChessApp {
    fn default() -> Self {
        Self::new()
    }
}

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BG_DARK;
        visuals.window_fill = BG_DARK;
        ctx.set_visuals(visuals);
        match self.app_mode {
            AppMode::Playing => self.update_playing(ctx),
            AppMode::Review => self.update_review(ctx),
        }
    }
}

impl ChessApp {
    fn update_playing(&mut self, ctx: &egui::Context) {
        if self.ai_thinking {
            if let Some(EngineResult::BestMove(ai_move)) = self.engine_manager.poll_result() {
                let legal_moves = self.board.generate_moves(self.current_player);
                let is_legal = legal_moves.iter().any(|m| {
                    m.from == ai_move.from && m.to == ai_move.to && m.promotion == ai_move.promotion
                });
                if is_legal {
                    self.board_history
                        .push((self.board.clone(), self.current_player));
                    self.record_move(ai_move);
                    self.dragging = None;
                    self.current_player = self.current_player.opposite();
                    self.ai_thinking = false;
                    self.ai_move_start = None;
                    self.update_game_state();
                } else {
                    self.ai_thinking = false;
                    self.ai_move_start = None;
                    self.status_message = "Engine returned invalid move".to_string();
                }
            } else if let Some(start_time) = self.ai_move_start {
                let elapsed = start_time.elapsed().as_millis();
                let time_limit = self.ai_difficulty.get_time_limit() as u128;
                let progress = (elapsed as f32 / time_limit as f32 * 100.0).min(100.0);
                self.status_message =
                    format!("{} thinking... ({:.0}%)", self.engine_name(), progress);
            }
        }

        self.draw_status_bar(ctx);
        self.draw_match_side_panel(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_DARK).inner_margin(8.0))
            .show(ctx, |ui| {
                let browsing_ply = self
                    .playing_view_ply
                    .map(|p| p.min(self.move_history.len()));
                let (board, last_move) = if let Some(ply) = browsing_ply {
                    (self.board_for_ply(ply), self.last_move_for_ply(ply))
                } else {
                    (self.board.clone(), self.last_move)
                };
                let interactive = self.game_state == GameState::Playing && !self.show_setup_dialog;
                if let Some((row, col)) =
                    self.draw_board_widget(ui, &board, interactive, last_move.as_ref(), None, None)
                {
                    if browsing_ply.is_some() {
                        self.playing_view_ply = None;
                        self.selected_square = None;
                        self.valid_moves.clear();
                        self.dragging = None;
                    } else {
                        self.handle_square_click(row, col);
                    }
                }
            });

        if self.promotion_pending.is_some() {
            self.show_promotion_dialog(ctx);
        }
        self.show_settings_window(ctx);
        if self.game_state != GameState::Playing {
            self.show_game_over_screen(ctx);
        }
        if self.ai_thinking {
            ctx.request_repaint();
        }
    }

    fn update_review(&mut self, ctx: &egui::Context) {
        if self.engine_manager.is_pending()
            && let Some(EngineResult::Analysis(analysis)) = self.engine_manager.poll_result()
            && let Some(ref mut review) = self.review_state
            && let Some((fen, depth)) = review.ingest_analysis_result(analysis)
        {
            self.engine_manager.request_analysis(&fen, depth);
        }
        if !self.engine_manager.is_pending()
            && let Some(ref mut review) = self.review_state
            && let Some((_idx, fen, depth)) = review.next_analysis_request()
        {
            self.engine_manager.request_analysis(&fen, depth);
        }

        self.draw_review_eval_bar(ctx);
        self.draw_review_side_panel(ctx);

        let (board, last_mv, best_moves, last_class) = if let Some(ref review) = self.review_state {
            let board = review.current_board().clone();
            let last_mv = if review.current_index > 0 {
                review
                    .move_records
                    .get(review.current_index - 1)
                    .map(|r| r.mv)
            } else {
                None
            };
            let best_moves = review
                .analyses
                .get(review.current_index)
                .and_then(|a| a.as_ref())
                .map(|a| {
                    a.lines
                        .iter()
                        .filter_map(|l| Board::uci_to_move(&l.best_move_uci).ok())
                        .collect::<Vec<_>>()
                });
            let last_class = if review.current_index > 0 {
                review
                    .classifications
                    .get(review.current_index - 1)
                    .and_then(|c| *c)
            } else {
                None
            };
            (board, last_mv, best_moves, last_class)
        } else {
            (Board::new(), None, None, None)
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_DARK).inner_margin(8.0))
            .show(ctx, |ui| {
                if let Some((row, col)) = self.draw_board_widget(
                    ui,
                    &board,
                    true,
                    last_mv.as_ref(),
                    best_moves.as_ref(),
                    last_class,
                ) {
                    self.handle_review_square_click(row, col);
                }
            });

        if self
            .review_state
            .as_ref()
            .map(|r| r.analyzing)
            .unwrap_or(false)
        {
            ctx.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threefold_repetition_detected_in_playing_state() {
        let mut app = ChessApp::new();
        let mut mover = Color::White;
        for uci in [
            "g1f3", "g8f6", "f3g1", "f6g8", "g1f3", "g8f6", "f3g1", "f6g8",
        ] {
            let mv = Board::uci_to_move(uci).expect("valid move");
            app.board.make_move(mv);
            let next = mover.opposite();
            let fen_after = app.board.to_fen(next);
            app.move_history.push(MoveRecord {
                mv,
                san: uci.to_string(),
                fen_after,
                color: mover,
            });
            mover = next;
        }
        app.current_player = mover;
        assert!(app.is_threefold_repetition_draw());
    }
}
