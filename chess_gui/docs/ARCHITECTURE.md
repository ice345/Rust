# Architecture

本文档描述当前 `chess_gui` 的核心模块职责与关键数据流。

## 1. 顶层模块

- `src/main.rs`：程序入口，创建并运行 `ChessApp`。
- `src/lib.rs`：模块导出层。
- `src/types.rs`：公共类型（棋子/走法/状态/分类/分析结果）。

## 2. 对弈层

- `src/board.rs`
  - 棋盘状态、合法走法生成、规则判定。
  - 支持 FEN、UCI 相关转换。
  - 提供三次重复所需 `position_key`。
- `src/ai.rs`
  - 内置 AI 搜索（minimax + alpha-beta + transposition table）。
- `src/uci.rs`
  - UCI 引擎进程管理与异步通信。
  - 输出统一 `EngineResult` 给 UI 消费。

## 3. Review 层

- `src/review.rs`
  - ReviewState 主状态机。
  - 管理主线/分支、游标、缓存、选中状态。
- `src/review/analysis_pipeline.rs`
  - 分析调度：`start_analysis -> next_analysis_request -> ingest_analysis_result`。
- `src/review/classification.rs`
  - 基于分析上下文重算走法分类。
- `src/review/ui_bridge.rs`
  - Review 面向 UI 的桥接接口（`line_for_ui_*`）。

## 4. UI 层

- `src/ui.rs`
  - 应用模式切换（Playing/Review）。
  - 棋盘渲染与主交互入口。
- `src/ui/review_panel.rs`
  - Review 右侧面板绘制（Report/Analysis/导航）。

## 5. 关键数据流

### 5.1 对弈模式

1. 用户走子 -> `Board::generate_moves` 校验。
2. 记录 `MoveRecord` + 更新 `position_history`。
3. AI 由 Built-in 或 UCI 返回走子。
4. 更新 `GameState`（含和棋/将杀判定）。

### 5.2 Review 模式

1. 进入 Review 时构建 `ReviewState`。
2. 按引擎策略设置深度计划并启动分析。
3. 每个局面通过 position key 查缓存；命中则直接复用。
4. 未命中局面请求引擎；回包后写入缓存并触发分类重算。
5. UI 显示 Top Lines、分类、评估图、分支树。

## 6. 分支模型

- 主线数据：`main_records`
- 活动线数据：`move_records + current_index`
- 分支节点：`ReviewVariation { id, parent_variation_id, anchor_index, cursor_index, records }`
- 行为原则：
  - 同节点同走法优先复用已有分支。
  - 回到主线后可继续定位原对局手数。
  - 已分析局面不重复分析。
