use eframe::egui;
use egui::{Color32, Pos2, Rect, Sense, Vec2};
use std::time::Instant;

use crate::ai::ChessAI;
use crate::board::Board;
use crate::types::*;

/// Main application structure that holds the board, AI, and game state
pub struct ChessApp {
    pub board: Board,
    pub ai: ChessAI,
    pub current_player: Color,
    pub selected_square: Option<(usize, usize)>,
    pub valid_moves: Vec<Move>,
    pub game_state: GameState,
    pub status_message: String,
    pub ai_thinking: bool,
    pub ai_move_start: Option<Instant>,
    pub ai_difficulty: AIDifficulty,
    pub promotion_pending: Option<Move>, // 待升变的走法
}

impl ChessApp {
    pub fn new() -> Self {
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

    pub fn piece_to_unicode(&self, piece: Piece) -> &str {
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

    pub fn handle_square_click(&mut self, row: usize, col: usize) {
        if self.game_state != GameState::Playing
            || self.current_player != Color::White
            || self.ai_thinking
            || self.promotion_pending.is_some()
        // 如果正在等待升变选择，不处理点击
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
        } else if self.board.is_in_check(self.current_player) {
            self.status_message = format!("{:?} is in check!", self.current_player);
        } else {
            self.status_message = format!("{:?} to move", self.current_player);
        }
    }

    pub fn new_game(&mut self) {
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

    pub fn set_ai_difficulty(&mut self, difficulty: AIDifficulty) {
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
                        if ui
                            .add_sized([60.0, 60.0], egui::Button::new("♕\nQueen"))
                            .clicked()
                        {
                            self.handle_promotion_choice(PieceType::Queen);
                        }
                        ui.add_space(10.0);
                        // 车
                        if ui
                            .add_sized([60.0, 60.0], egui::Button::new("♖\nRook"))
                            .clicked()
                        {
                            self.handle_promotion_choice(PieceType::Rook);
                        }
                        ui.add_space(10.0);
                        // 象
                        if ui
                            .add_sized([60.0, 60.0], egui::Button::new("♗\nBishop"))
                            .clicked()
                        {
                            self.handle_promotion_choice(PieceType::Bishop);
                        }
                        ui.add_space(10.0);
                        // 马
                        if ui
                            .add_sized([60.0, 60.0], egui::Button::new("♘\nKnight"))
                            .clicked()
                        {
                            self.handle_promotion_choice(PieceType::Knight);
                        }
                    });

                    ui.add_space(10.0);
                    ui.label("Click on the piece you want to promote to");
                });
            });
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
}

impl Default for ChessApp {
    fn default() -> Self {
        Self::new()
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
                    .selected_text(format!(
                        "{} (depth:{})",
                        self.ai_difficulty.to_string(),
                        self.ai_difficulty.get_depth()
                    ))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.ai_difficulty,
                            AIDifficulty::Easy,
                            "Easy (depth:2)",
                        );
                        ui.selectable_value(
                            &mut self.ai_difficulty,
                            AIDifficulty::Medium,
                            "Medium (depth:4)",
                        );
                        ui.selectable_value(
                            &mut self.ai_difficulty,
                            AIDifficulty::Hard,
                            "Hard (depth:6)",
                        );
                        ui.selectable_value(
                            &mut self.ai_difficulty,
                            AIDifficulty::Expert,
                            "Expert (depth:8)",
                        );
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
