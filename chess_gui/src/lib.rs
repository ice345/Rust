//! `chess_gui` crate.
//!
//! 提供一个基于 `egui/eframe` 的桌面国际象棋实现，包含：
//!
//! - 棋盘规则与合法性校验（`board`）
//! - 内置 AI（`ai`）
//! - UCI 引擎接入（`uci`）
//! - PGN/SAN 支持（`pgn`）
//! - 赛后复盘状态机（`review`）
//! - UI 与交互层（`ui`）

/// 内置 AI 搜索模块（minimax + alpha-beta + 置换表）。
pub mod ai;
/// 棋盘状态、规则判定与走法生成。
pub mod board;
/// PGN/SAN 编解码能力。
pub mod pgn;
/// 复盘状态机、分支与分析缓存。
pub mod review;
/// 公共类型定义（棋子、走法、状态、分类等）。
pub mod types;
/// UCI 引擎管理与异步回包。
pub mod uci;
/// GUI 应用与绘制逻辑。
pub mod ui;

/// 内置 AI 类型导出。
pub use ai::ChessAI;
/// 棋盘类型导出。
pub use board::Board;
/// 公共类型导出。
pub use types::*;
/// 引擎管理器导出。
pub use uci::EngineManager;
/// 应用主结构导出。
pub use ui::ChessApp;
