# Development Guide

## 1. 本地开发

```bash
cargo run
```

## 2. 质量门禁

```bash
cargo fmt
cargo test
cargo clippy --all-targets
```

若希望本地按 CI 严格模式执行：

```bash
cargo clippy --all-targets -- -D warnings
```

## 3. 引擎调试

可通过环境变量覆盖引擎路径：

```bash
export CHESS_GUI_STOCKFISH_PATH=/path/to/stockfish
export CHESS_GUI_LC0_PATH=/path/to/lc0
```

辅助脚本：

- `test_uci.py`
- `test_stockfish.exp`

## 4. 模块修改建议

- UI 相关优先改 `src/ui/review_panel.rs`（Review 面板）与 `src/ui.rs`（棋盘和主流程）。
- Review 状态与策略改 `src/review.rs` 与 `src/review/*` 子模块。
- 规则与合法性改 `src/board.rs`，并同步补测试。

## 5. 提交前检查清单

- 新行为是否有文档更新（README 或 docs/*）
- 是否破坏“已分析局面不重复分析”语义
- 是否保持主线/分支切换可逆
- fmt/test/clippy 是否全通过
