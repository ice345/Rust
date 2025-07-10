// Game logic module - handles game state management and game flow
use std::time::Instant;

use crate::types::*;
use crate::board::Board;
use crate::ai::ChessAI;

/// Game controller that manages the game state and flow
pub struct ChessGame {
    pub board: Board,
    pub ai: ChessAI,
    pub current_player: Color,
    pub game_state: GameState,
    pub ai_thinking: bool,
    pub ai_move_start: Option<Instant>,
    pub ai_difficulty: AIDifficulty,
}

impl ChessGame {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            ai: ChessAI::new(4),
            current_player: Color::White,
            game_state: GameState::Playing,
            ai_thinking: false,
            ai_move_start: None,
            ai_difficulty: AIDifficulty::Medium,
        }
    }

    pub fn new_game(&mut self) {
        self.board = Board::new();
        self.current_player = Color::White;
        self.game_state = GameState::Playing;
        self.ai_thinking = false;
        self.ai_move_start = None;
    }

    pub fn set_ai_difficulty(&mut self, difficulty: AIDifficulty) {
        self.ai_difficulty = difficulty;
        self.ai = ChessAI::new(difficulty.get_depth());
    }

    pub fn update_game_state(&mut self) -> String {
        let moves = self.board.generate_moves(self.current_player);

        if moves.is_empty() {
            if self.board.is_in_check(self.current_player) {
                self.game_state = match self.current_player {
                    Color::White => GameState::BlackWins,
                    Color::Black => GameState::WhiteWins,
                };
                format!(
                    "{:?} wins by checkmate!",
                    match self.current_player {
                        Color::White => Color::Black,
                        Color::Black => Color::White,
                    }
                )
            } else {
                self.game_state = GameState::Draw;
                "Draw by stalemate!".to_string()
            }
        } else if self.board.is_in_check(self.current_player) {
            format!("{:?} is in check!", self.current_player)
        } else {
            format!("{:?} to move", self.current_player)
        }
    }

    pub fn make_move(&mut self, mv: Move) -> bool {
        if self.game_state != GameState::Playing {
            return false;
        }

        let valid_moves = self.board.generate_moves(self.current_player);
        let is_valid = valid_moves.iter().any(|valid_mv| {
            valid_mv.from == mv.from && valid_mv.to == mv.to && valid_mv.promotion == mv.promotion
        });

        if is_valid {
            self.board.make_move(mv);
            self.current_player = match self.current_player {
                Color::White => Color::Black,
                Color::Black => Color::White,
            };
            true
        } else {
            false
        }
    }

    pub fn get_valid_moves_for_piece(&self, pos: (usize, usize)) -> Vec<Move> {
        if self.game_state != GameState::Playing {
            return Vec::new();
        }

        self.board
            .generate_moves(self.current_player)
            .into_iter()
            .filter(|mv| mv.from == pos)
            .collect()
    }

    pub fn start_ai_thinking(&mut self) {
        if self.current_player == Color::Black && self.game_state == GameState::Playing {
            self.ai_thinking = true;
            self.ai_move_start = Some(Instant::now());
        }
    }

    pub fn get_ai_move(&mut self) -> Option<Move> {
        if self.ai_thinking && self.current_player == Color::Black {
            if let Some(start_time) = self.ai_move_start {
                let elapsed = start_time.elapsed().as_millis();
                if elapsed > 500 {
                    let ai_move = self.ai.get_best_move(&self.board, Color::Black);
                    self.ai_thinking = false;
                    self.ai_move_start = None;
                    return ai_move;
                }
            }
        }
        None
    }

    pub fn get_thinking_progress(&self) -> f32 {
        if let Some(start_time) = self.ai_move_start {
            let elapsed = start_time.elapsed().as_millis();
            let time_limit = self.ai.time_limit as u128;
            (elapsed as f32 / time_limit as f32 * 100.0).min(100.0)
        } else {
            0.0
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.game_state != GameState::Playing
    }

    pub fn get_game_result(&self) -> GameState {
        self.game_state
    }

    pub fn can_promote(&self, mv: Move) -> bool {
        if let Some(piece) = self.board.get_piece(mv.from) {
            piece.piece_type == PieceType::Pawn
                && ((piece.color == Color::White && mv.to.0 == 0)
                    || (piece.color == Color::Black && mv.to.0 == 7))
        } else {
            false
        }
    }
}

impl Default for ChessGame {
    fn default() -> Self {
        Self::new()
    }
}
