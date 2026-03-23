//! UCI engine integration.
//!
//! 本模块负责：
//!
//! - 启动/切换外部引擎（Stockfish/LC0）
//! - 发送 `bestmove` 与 `analysis` 请求
//! - 轮询异步结果并转为统一的 `EngineResult`

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use crate::ai::ChessAI;
use crate::board::Board;
use crate::types::*;

struct UciEngine {
    child: Child,
    stdin_tx: std::sync::mpsc::Sender<String>,
    stdout_rx: Receiver<String>,
}

impl UciEngine {
    fn new(path: &str) -> Result<Self, String> {
        let mut child = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn Stockfish: {}", e))?;

        let stdout = child.stdout.take().ok_or("No stdout")?;
        let mut stdin = child.stdin.take().ok_or("No stdin")?;

        let (line_tx, line_rx) = mpsc::channel::<String>();
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines().map_while(Result::ok) {
                if line_tx.send(line).is_err() {
                    break;
                }
            }
        });

        let (cmd_tx, cmd_rx) = mpsc::channel::<String>();
        thread::spawn(move || {
            while let Ok(cmd) = cmd_rx.recv() {
                if writeln!(stdin, "{}", cmd).is_err() {
                    break;
                }
                let _ = stdin.flush();
            }
        });

        let engine = UciEngine {
            child,
            stdin_tx: cmd_tx,
            stdout_rx: line_rx,
        };

        engine.send("uci");
        engine.wait_for("uciok", 5000)?;
        engine.send("isready");
        engine.wait_for("readyok", 5000)?;

        Ok(engine)
    }

    fn send(&self, cmd: &str) {
        let _ = self.stdin_tx.send(cmd.to_string());
    }

    fn wait_for(&self, token: &str, timeout_ms: u64) -> Result<(), String> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return Err(format!("timeout waiting for '{}'", token));
            }
            match self.stdout_rx.recv_timeout(remaining) {
                Ok(line) if line.contains(token) => return Ok(()),
                Ok(_) => continue,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    return Err(format!("timeout waiting for '{}'", token));
                }
                Err(_) => return Err("engine disconnected".into()),
            }
        }
    }

    fn drain_pending(&self) {
        while self.stdout_rx.try_recv().is_ok() {}
    }
}

impl Drop for UciEngine {
    fn drop(&mut self) {
        let _ = self.stdin_tx.send("quit".to_string());
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn engine_candidates(env_key: &str, default_path: &str, command_name: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    if let Ok(path) = std::env::var(env_key) {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            candidates.push(trimmed.to_string());
        }
    }
    candidates.push(default_path.to_string());
    candidates.push(command_name.to_string());
    candidates
}

fn open_first_engine(candidates: &[String]) -> Option<UciEngine> {
    for path in candidates {
        if let Ok(engine) = UciEngine::new(path) {
            return Some(engine);
        }
    }
    None
}

// Parse "score cp <N>" or "score mate <N>" from UCI info line
fn parse_score_cp(line: &str) -> Option<i32> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for i in 0..parts.len().saturating_sub(1) {
        if parts[i] == "score" {
            if parts.get(i + 1) == Some(&"cp") {
                return parts.get(i + 2).and_then(|s| s.parse().ok());
            }
            if parts.get(i + 1) == Some(&"mate") {
                let mate_in: i32 = parts.get(i + 2).and_then(|s| s.parse().ok())?;
                return Some(if mate_in > 0 {
                    30000 - mate_in
                } else {
                    -30000 - mate_in
                });
            }
        }
    }
    None
}

/// Engine asynchronous response payload.
pub enum EngineResult {
    /// Best move result for a move request.
    BestMove(Move),
    /// Position analysis result for a review request.
    Analysis(PositionAnalysis),
}

#[derive(Clone, Copy, PartialEq)]
enum PendingRequest {
    Move,
    Analysis,
}

/// Handles engine lifecycle, request dispatch, and result polling.
pub struct EngineManager {
    engine: Option<UciEngine>,
    engine_type: EngineType,
    fallback_ai: ChessAI,
    result_rx: Option<Receiver<EngineResult>>,
    pending: bool,
    pending_type: Option<PendingRequest>,
    pv_lines: std::collections::HashMap<u32, (i32, String)>,
    last_depth: u32,
    last_score_cp: Option<i32>,
    pending_terminal_eval_cp: Option<i32>,
}

impl EngineManager {
    /// Creates a manager with the given difficulty.
    ///
    /// Engine selection priority:
    ///
    /// 1. `CHESS_GUI_STOCKFISH_PATH`
    /// 2. Stockfish default candidates
    /// 3. `CHESS_GUI_LC0_PATH`
    /// 4. LC0 default candidates
    /// 5. Built-in fallback
    pub fn new(difficulty: &AIDifficulty) -> Self {
        // Resolve engine paths from env overrides first:
        // CHESS_GUI_STOCKFISH_PATH / CHESS_GUI_LC0_PATH
        let mut engine_type = EngineType::Stockfish;
        let stockfish_candidates = engine_candidates(
            "CHESS_GUI_STOCKFISH_PATH",
            "/usr/bin/stockfish",
            "stockfish",
        );
        let mut engine = open_first_engine(&stockfish_candidates);

        // If Stockfish not found but LC0 is, fallback to LC0 (or user can switch later)
        if engine.is_none() {
            let lc0_candidates = engine_candidates("CHESS_GUI_LC0_PATH", "/usr/bin/lc0", "lc0");
            if let Some(lc0) = open_first_engine(&lc0_candidates) {
                engine = Some(lc0);
                engine_type = EngineType::Lc0;
            } else {
                engine_type = EngineType::BuiltIn;
            }
        }

        let mut fallback_ai = ChessAI::new(difficulty.get_depth());
        fallback_ai.time_limit = difficulty.get_time_limit();

        EngineManager {
            engine,
            engine_type,
            fallback_ai,
            result_rx: None,
            pending: false,
            pending_type: None,
            pv_lines: std::collections::HashMap::new(),
            last_depth: 0,
            last_score_cp: None,
            pending_terminal_eval_cp: None,
        }
    }

    /// Returns whether an external engine process is available.
    pub fn has_stockfish(&self) -> bool {
        self.engine.is_some() // Represents either lc0 or stockfish
    }

    /// Switches engine type at runtime.
    ///
    /// If the requested external engine cannot be spawned, it gracefully
    /// falls back to `EngineType::BuiltIn`.
    pub fn set_engine(&mut self, new_engine: EngineType) {
        if self.engine_type == new_engine {
            return;
        }

        self.engine = match new_engine {
            EngineType::Stockfish => open_first_engine(&engine_candidates(
                "CHESS_GUI_STOCKFISH_PATH",
                "/usr/bin/stockfish",
                "stockfish",
            )),
            EngineType::Lc0 => open_first_engine(&engine_candidates(
                "CHESS_GUI_LC0_PATH",
                "/usr/bin/lc0",
                "lc0",
            )),
            EngineType::BuiltIn => None,
        };
        self.engine_type = new_engine;

        // If they requested Stockfish/Lc0 and it failed, fallback to builtin
        if new_engine != EngineType::BuiltIn && self.engine.is_none() {
            self.engine_type = EngineType::BuiltIn;
        }

        self.pending = false;
        self.pending_type = None;
        self.pv_lines.clear();
        self.last_depth = 0;
        self.last_score_cp = None;
        self.pending_terminal_eval_cp = None;
    }

    /// Current effective engine type.
    pub fn engine_type(&self) -> EngineType {
        self.engine_type
    }

    /// Requests one best move for the current position.
    ///
    /// This method is non-blocking; use [`Self::poll_result`] to consume
    /// the asynchronous response.
    pub fn request_move(&mut self, board: &Board, color: Color, difficulty: &AIDifficulty) {
        if self.pending {
            return;
        }
        self.pending = true;
        self.pending_type = Some(PendingRequest::Move);
        self.pv_lines.clear();
        self.last_depth = 0;
        self.last_score_cp = None;
        self.pending_terminal_eval_cp = None;

        if let Some(ref engine) = self.engine {
            let fen = board.to_fen(color);
            engine.drain_pending();
            engine.send(&format!(
                "setoption name Skill Level value {}",
                difficulty.stockfish_skill_level()
            ));
            engine.send("isready");
            let _ = engine.wait_for("readyok", 2000);
            engine.send(&format!("position fen {}", fen));
            if self.engine_type == EngineType::Lc0 {
                // LC0 can hang forever if told to search "depth X" because MCTS plies are scaled differently
                // We use the time limit generated from AIDifficulty
                engine.send(&format!("go movetime {}", difficulty.get_time_limit()));
            } else {
                engine.send(&format!("go depth {}", difficulty.stockfish_depth()));
            }
        } else {
            // Fallback: run built-in AI on background thread
            let (tx, rx) = mpsc::channel();
            self.result_rx = Some(rx);
            let mut ai = self.fallback_ai.clone();
            let board = board.clone();
            thread::spawn(move || {
                if let Some(mv) = ai.get_best_move(&board, color) {
                    let _ = tx.send(EngineResult::BestMove(mv));
                }
            });
        }
    }

    /// Requests analysis lines for a FEN position.
    ///
    /// This method is non-blocking; use [`Self::poll_result`] to fetch
    /// `EngineResult::Analysis`.
    pub fn request_analysis(&mut self, fen: &str, depth: u32) {
        if self.pending {
            return;
        }
        self.pending = true;
        self.pending_type = Some(PendingRequest::Analysis);
        self.pv_lines.clear();
        self.last_depth = 0;
        self.last_score_cp = None;
        self.pending_terminal_eval_cp = Self::terminal_eval_cp_from_fen(fen);

        if let Some(ref engine) = self.engine {
            engine.drain_pending();
            engine.send("setoption name MultiPV value 3");
            engine.send("isready");
            let _ = engine.wait_for("readyok", 2000);
            engine.send(&format!("position fen {}", fen));
            if self.engine_type == EngineType::Lc0 {
                // MCTS scales poorly with depth; 2.5 seconds max limit per analysis round
                engine.send("go movetime 2500");
            } else {
                engine.send(&format!("go depth {}", depth));
            }
        } else {
            // Fallback: use built-in AI to provide at least one analysis line.
            let (board, active_color, _) = match Board::from_fen(fen) {
                Ok(parsed) => parsed,
                Err(_) => {
                    self.pending = false;
                    self.pending_type = None;
                    return;
                }
            };

            let (tx, rx) = mpsc::channel();
            self.result_rx = Some(rx);
            let mut ai = self.fallback_ai.clone();
            thread::spawn(move || {
                let best_move = ai.get_best_move(&board, active_color);
                let eval_white_pov = board.evaluate();
                let eval_side_to_move = if active_color == Color::White {
                    eval_white_pov
                } else {
                    -eval_white_pov
                };

                let lines = if let Some(mv) = best_move {
                    vec![AnalysisLine {
                        eval_cp: eval_side_to_move,
                        best_move_uci: Board::move_to_uci(&mv),
                    }]
                } else {
                    // Terminal nodes still need a meaningful eval for UI graph/status.
                    let terminal_eval = if board.generate_moves(active_color).is_empty() {
                        if board.is_in_check(active_color) {
                            -30000
                        } else {
                            0
                        }
                    } else {
                        eval_side_to_move
                    };
                    vec![AnalysisLine {
                        eval_cp: terminal_eval,
                        best_move_uci: "(none)".to_string(),
                    }]
                };

                let _ = tx.send(EngineResult::Analysis(PositionAnalysis {
                    lines,
                    depth,
                    verification_passes: 1,
                    stability_pct: 100,
                }));
            });
        }
    }

    /// Polls one asynchronous engine result, if available.
    pub fn poll_result(&mut self) -> Option<EngineResult> {
        if !self.pending {
            return None;
        }

        // Check fallback AI channel
        if let Some(ref rx) = self.result_rx {
            match rx.try_recv() {
                Ok(result) => {
                    self.pending = false;
                    self.pending_type = None;
                    self.result_rx = None;
                    return Some(result);
                }
                Err(TryRecvError::Disconnected) => {
                    self.pending = false;
                    self.pending_type = None;
                    self.result_rx = None;
                    return None;
                }
                Err(TryRecvError::Empty) => return None,
            }
        }

        // Poll Stockfish engine
        if let Some(ref engine) = self.engine {
            loop {
                match engine.stdout_rx.try_recv() {
                    Ok(line) => {
                        if line.starts_with("info") {
                            let mut current_multipv = 1;
                            let mut current_score = self.last_score_cp.unwrap_or(0);
                            let mut current_pv = None;

                            if let Some(cp) = parse_score_cp(&line) {
                                current_score = cp;
                                self.last_score_cp = Some(cp);
                            }
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            for i in 0..parts.len().saturating_sub(1) {
                                if parts[i] == "depth" {
                                    if let Ok(d) = parts[i + 1].parse() {
                                        self.last_depth = d;
                                    }
                                } else if parts[i] == "multipv" {
                                    if let Ok(m) = parts[i + 1].parse() {
                                        current_multipv = m;
                                    }
                                } else if parts[i] == "pv" {
                                    current_pv = parts.get(i + 1).map(|s| s.to_string());
                                }
                            }
                            if let Some(pv) = current_pv {
                                self.pv_lines.insert(current_multipv, (current_score, pv));
                            }
                        }
                        if line.starts_with("bestmove") {
                            self.pending = false;
                            let request_type = self.pending_type.take();
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if let Some(&bm) = parts.get(1) {
                                match request_type {
                                    Some(PendingRequest::Analysis) => {
                                        let mut lines = Vec::new();
                                        // Collect in order of multipv index
                                        for i in 1..=10 {
                                            if let Some((eval, pv)) = self.pv_lines.get(&i) {
                                                lines.push(AnalysisLine {
                                                    eval_cp: *eval,
                                                    best_move_uci: pv.clone(),
                                                });
                                            }
                                        }
                                        if lines.is_empty() {
                                            let fallback_eval = self
                                                .last_score_cp
                                                .or(self.pending_terminal_eval_cp)
                                                .unwrap_or(0);
                                            lines.push(AnalysisLine {
                                                eval_cp: fallback_eval,
                                                best_move_uci: bm.to_string(),
                                            });
                                        }
                                        self.pending_terminal_eval_cp = None;
                                        self.last_score_cp = None;
                                        return Some(EngineResult::Analysis(PositionAnalysis {
                                            lines,
                                            depth: self.last_depth,
                                            verification_passes: 1,
                                            stability_pct: 100,
                                        }));
                                    }
                                    Some(PendingRequest::Move) | None => {
                                        self.pending_terminal_eval_cp = None;
                                        self.last_score_cp = None;
                                        if let Ok(mv) = Board::uci_to_move(bm) {
                                            return Some(EngineResult::BestMove(mv));
                                        }
                                    }
                                }
                            }
                            return None;
                        }
                    }
                    Err(TryRecvError::Empty) => return None,
                    Err(TryRecvError::Disconnected) => {
                        self.pending = false;
                        self.pending_type = None;
                        self.pending_terminal_eval_cp = None;
                        self.last_score_cp = None;
                        return None;
                    }
                }
            }
        }

        None
    }

    /// Returns whether there is an in-flight engine request.
    pub fn is_pending(&self) -> bool {
        self.pending
    }

    /// Applies a new difficulty to the built-in fallback engine.
    pub fn update_difficulty(&mut self, difficulty: &AIDifficulty) {
        self.fallback_ai = ChessAI::new(difficulty.get_depth());
        self.fallback_ai.time_limit = difficulty.get_time_limit();
    }

    fn terminal_eval_cp_from_fen(fen: &str) -> Option<i32> {
        let (board, active_color, _) = Board::from_fen(fen).ok()?;
        if board.generate_moves(active_color).is_empty() {
            return Some(if board.is_in_check(active_color) {
                -30000
            } else {
                0
            });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_eval_detects_checkmate() {
        // Fool's mate after 1.f3 e5 2.g4 Qh4#
        let fen = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";
        assert_eq!(EngineManager::terminal_eval_cp_from_fen(fen), Some(-30000));
    }

    #[test]
    fn test_terminal_eval_detects_stalemate() {
        // Black to move, no legal moves and not in check.
        let fen = "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1";
        assert_eq!(EngineManager::terminal_eval_cp_from_fen(fen), Some(0));
    }

    #[test]
    fn test_terminal_eval_non_terminal_returns_none() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        assert_eq!(EngineManager::terminal_eval_cp_from_fen(fen), None);
    }
}
