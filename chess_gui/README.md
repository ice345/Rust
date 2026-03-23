# Chess GUI

一个基于 Rust + `egui/eframe` 的桌面国际象棋项目，包含对弈、PGN、赛后 Review、走法分类与引擎分析。

## 项目状态（2026-03-21）

- 棋规完整：王车易位、过路兵、升变、将杀、逼和、三次重复、50 步规则、子力不足。
- 对弈可用：人类执白 vs AI 执黑，支持 `Stockfish` / `LC0` / `Built-in AI`。
- Review 可用：Top3 推荐可视化、分支树、分类统计、评估走势图、稳定性投票。
- 性能优化：已分析局面缓存复用；回到已分析主线/分支不重复分析。
- 工程质量：`cargo fmt`、`cargo test`、`cargo clippy --all-targets` 已通过（当前 clippy clean）。

## 快速开始

### 1. 环境要求

- Rust stable（edition 2024）
- Linux 图形环境（X11/Wayland）
- 可选外部引擎：
  - Stockfish（默认尝试 `/usr/bin/stockfish`、`stockfish`）
  - LC0（默认尝试 `/usr/bin/lc0`、`lc0`）

可用环境变量覆盖引擎路径：

```bash
export CHESS_GUI_STOCKFISH_PATH=/path/to/stockfish
export CHESS_GUI_LC0_PATH=/path/to/lc0
```

### 2. 运行

```bash
cargo run
```

### 3. 常用检查

```bash
cargo fmt
cargo test
cargo clippy --all-targets
```

## 主要功能

- 人机对弈与难度选择：`Easy / Medium / Hard / Expert`
- 执方选择：支持 `You play White/Black`（AI 不再固定执黑）
- 实时走法历史（SAN）
- 历史回看：点击走法可查看当时布局；点击棋盘可直接回到 live 对局
- 快速结束：`Resign` 一键认输
- 一键 `Copy PGN`
- 赛后 Review：
  - Top3 候选线（面板 + 棋盘箭头）
  - 分支树（主线/子分支切换）
  - 走法分类：`Brilliant/Critical/Best/Excellent/Okay/Inaccuracy/Mistake/Blunder`
  - 评估图 + 准确率 + 分类统计
  - 多深度复核 + 稳定性指标（Vote/Stable%）

## 核心架构

```text
src/
├── main.rs
├── lib.rs
├── types.rs
├── board.rs
├── ai.rs
├── uci.rs
├── pgn.rs
├── ui.rs
├── ui/
│   └── review_panel.rs
├── review.rs
└── review/
    ├── analysis_pipeline.rs
    ├── classification.rs
    └── ui_bridge.rs
```

- `board.rs`：棋盘状态、合法走法生成、规则判定、FEN/UCI 辅助。
- `ai.rs`：内置 AI 搜索（minimax + alpha-beta + 置换表）。
- `uci.rs`：UCI 引擎进程、异步请求/回包。
- `review.rs`：Review 状态机（主线/分支、缓存、导航）。
- `review/analysis_pipeline.rs`：分析请求调度与结果回填。
- `review/classification.rs`：分类重算与上下文判定。
- `ui.rs` + `ui/review_panel.rs`：主界面与 Review 面板绘制。

## Review 行为说明（关键）

- 进入 Review 后按引擎策略执行分析：
  - Stockfish：多深度复核（默认 `12/16/20`）
  - LC0 / Built-in：单轮策略（响应优先）
- 局面分析结果按 position key 缓存。
- 已分析局面再次访问时直接复用，不重复请求引擎。
- 在分支探索后回到对局原始主线，不会丢失主线记录。

## 文档索引

- `docs/ARCHITECTURE.md`：模块职责、状态流、分析流水线
- `docs/REVIEW_WORKFLOW.md`：Review 交互语义、分支/缓存行为
- `docs/DEVELOPMENT.md`：开发规范、调试与提交流程
- `CHANGELOG.md`：当日改动汇总
- `CHANGELOG_2026-03-21.md`：完整改动流水
- `CHANGELOG_lite.md`：简版续接摘要

## 已知限制

- 当前默认模式是“人类执白、AI 执黑”（未提供完整双人本地 UI 流程）。
- `Brilliant/Critical` 仍是工程化启发式，不是完整战术语义引擎。
- 当前无 CI 配置（建议后续补 `fmt/test/clippy -D warnings`）。
