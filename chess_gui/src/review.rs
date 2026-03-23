//! Review state machine.
//!
//! 负责赛后复盘中的主线/分支、局面缓存、分析调度与导航状态。

use crate::board::Board;
use crate::types::*;
use std::collections::HashMap;

/// A review variation branch anchored at a specific ply.
#[derive(Clone)]
pub struct ReviewVariation {
    /// Unique variation id.
    pub id: u32,
    /// User-facing variation name.
    pub name: String,
    /// Parent variation id, or `None` for root branches from mainline.
    pub parent_variation_id: Option<u32>,
    /// Mainline/parent ply where the branch was created.
    pub anchor_index: usize,
    /// Current cursor index when this branch is inactive.
    pub cursor_index: usize,
    /// Full move record sequence for this branch.
    pub records: Vec<MoveRecord>,
}

/// Runtime state for review mode.
#[derive(Clone)]
pub struct ReviewState {
    /// Initial board snapshot before any review moves.
    pub initial_board: Board,
    /// Board snapshots for each ply of the active line.
    pub snapshots: Vec<Board>,
    /// Position keys aligned with `snapshots`.
    pub position_keys: Vec<String>,
    /// Move records of the active line (mainline or variation).
    pub move_records: Vec<MoveRecord>,
    /// Cursor in `snapshots`.
    pub current_index: usize,
    /// Finalized analyses per snapshot.
    pub analyses: Vec<Option<PositionAnalysis>>,
    /// Global cache keyed by position key to avoid repeated analysis.
    pub analysis_cache: HashMap<String, PositionAnalysis>,
    /// Move classification cache for the active line.
    pub classifications: Vec<Option<MoveClassification>>,
    /// Whether review is currently in analysis mode.
    pub analyzing: bool,
    /// Current analysis target index.
    pub analysis_index: usize,
    /// In-flight snapshot index.
    pub pending_analysis_index: Option<usize>,
    /// In-flight depth step index in `analysis_depth_plan`.
    pub pending_analysis_depth_idx: usize,
    /// Multi-pass depth plan (e.g. `[12, 16, 20]`).
    pub analysis_depth_plan: Vec<u32>,
    /// Intermediate multi-pass analysis samples by snapshot index.
    pub analysis_samples: HashMap<usize, Vec<PositionAnalysis>>,
    /// Selected board square in review UI.
    pub selected_square: Option<(usize, usize)>,
    /// Legal moves from `selected_square`.
    pub valid_moves: Vec<Move>,
    /// Immutable original game records.
    pub main_records: Vec<MoveRecord>,
    /// All variation branches.
    pub variations: Vec<ReviewVariation>,
    /// Active variation id, `None` means mainline.
    pub active_variation_id: Option<u32>,
    /// Monotonic id allocator for new variations.
    pub next_variation_id: u32,
    /// Active review tab.
    pub active_tab: ReviewTab,
}

impl ReviewState {
    /// Creates review state from an initial board and game move list.
    pub fn new(initial_board: Board, move_records: Vec<MoveRecord>) -> Self {
        let snapshots = Self::build_snapshots(initial_board.clone(), &move_records);
        let position_keys = Self::build_position_keys(&snapshots);

        ReviewState {
            initial_board,
            snapshots,
            position_keys,
            move_records: move_records.clone(),
            current_index: 0,
            analyses: vec![None; move_records.len() + 1],
            analysis_cache: HashMap::new(),
            classifications: vec![None; move_records.len()],
            analyzing: false,
            analysis_index: 0,
            pending_analysis_index: None,
            pending_analysis_depth_idx: 0,
            analysis_depth_plan: vec![12, 16, 20],
            analysis_samples: HashMap::new(),
            selected_square: None,
            valid_moves: Vec::new(),
            main_records: move_records,
            variations: Vec::new(),
            active_variation_id: None,
            next_variation_id: 1,
            active_tab: ReviewTab::Report,
        }
    }

    /// Sets a depth plan for multi-pass analysis.
    ///
    /// Zero or empty entries are filtered. If all entries are invalid,
    /// a fallback depth `18` is used.
    pub fn set_analysis_depth_plan(&mut self, mut depth_plan: Vec<u32>) {
        depth_plan.retain(|d| *d > 0);
        if depth_plan.is_empty() {
            depth_plan.push(18);
        }
        self.analysis_depth_plan = depth_plan;
        self.pending_analysis_depth_idx = 0;
        self.analysis_samples.clear();
    }

    /// Returns board snapshot at current review cursor.
    pub fn current_board(&self) -> &Board {
        &self.snapshots[self.current_index]
    }

    pub fn is_in_variation(&self) -> bool {
        self.active_variation_id.is_some()
    }

    /// Switches active line back to mainline and keeps current ply if possible.
    pub fn switch_to_main_line(&mut self) {
        self.commit_active_variation_state();
        self.active_variation_id = None;
        let idx = self.current_index.min(self.main_records.len());
        self.rebuild_active_line_from_records(self.main_records.clone(), idx);
    }

    pub fn restore_main_line(&mut self) {
        self.switch_to_main_line();
    }

    /// Switches active line to a specific variation id.
    pub fn switch_to_variation(&mut self, id: u32) {
        if self.active_variation_id == Some(id) {
            return;
        }
        self.commit_active_variation_state();

        if let Some(var) = self.variations.iter().find(|v| v.id == id).cloned() {
            self.active_variation_id = Some(id);
            self.rebuild_active_line_from_records(var.records, var.cursor_index);
        }
    }

    /// Ensures an editable active variation exists and returns its id.
    pub fn ensure_active_variation_for_edit(&mut self, force_new: bool) -> u32 {
        if !force_new && let Some(id) = self.active_variation_id {
            return id;
        }
        self.create_variation_from_current()
    }

    /// Creates a new variation from current cursor of active line.
    pub fn create_variation_from_current(&mut self) -> u32 {
        self.commit_active_variation_state();

        let parent_variation_id = self.active_variation_id;
        let prefix_len = self.current_index.min(self.move_records.len());
        let records = self
            .move_records
            .iter()
            .take(prefix_len)
            .cloned()
            .collect::<Vec<_>>();
        let id = self.next_variation_id;
        self.next_variation_id += 1;

        self.variations.push(ReviewVariation {
            id,
            name: format!("Variation {}", id),
            parent_variation_id,
            anchor_index: prefix_len,
            cursor_index: prefix_len,
            records: records.clone(),
        });

        self.active_variation_id = Some(id);
        self.rebuild_active_line_from_records(records, prefix_len);
        id
    }

    /// Returns variation ids in display order with their tree depth.
    pub fn variation_tree(&self) -> Vec<(u32, usize)> {
        fn collect(
            out: &mut Vec<(u32, usize)>,
            variations: &[ReviewVariation],
            parent: Option<u32>,
            depth: usize,
        ) {
            let mut children = variations
                .iter()
                .filter(|v| v.parent_variation_id == parent)
                .map(|v| v.id)
                .collect::<Vec<_>>();
            children.sort_unstable();
            for id in children {
                out.push((id, depth));
                collect(out, variations, Some(id), depth + 1);
            }
        }

        let mut out = Vec::new();
        collect(&mut out, &self.variations, None, 0);
        out
    }

    /// Applies a move in review mode to the active line.
    ///
    /// This updates snapshots, cache alignment, and triggers analysis restart.
    pub fn apply_review_move(
        &mut self,
        mv: Move,
        san: String,
        color: Color,
        force_new_variation: bool,
    ) {
        let _ = self.ensure_active_variation_for_edit(force_new_variation);

        let current_board = self.current_board().clone();
        let mut next_board = current_board.clone();
        next_board.make_move(mv);
        let fen_after = next_board.to_fen(color.opposite());

        self.move_records.truncate(self.current_index);
        self.snapshots.truncate(self.current_index + 1);
        self.position_keys.truncate(self.current_index + 1);
        self.analyses.truncate(self.current_index + 1);
        self.classifications.truncate(self.current_index);

        self.move_records.push(MoveRecord {
            mv,
            san,
            fen_after,
            color,
        });
        self.position_keys
            .push(next_board.position_key(color.opposite()));
        self.snapshots.push(next_board);
        self.analyses.push(None);
        self.classifications.push(None);

        self.current_index += 1;
        self.selected_square = None;
        self.valid_moves.clear();
        self.start_analysis();

        self.commit_active_variation_state();
    }

    pub fn jump_to_existing_next_move_if_same(&mut self, mv: Move) -> bool {
        if let Some(next) = self.move_records.get(self.current_index)
            && next.mv.from == mv.from
            && next.mv.to == mv.to
            && next.mv.promotion == mv.promotion
        {
            self.current_index += 1;
            self.selected_square = None;
            self.valid_moves.clear();
            return true;
        }
        false
    }

    pub fn is_current_position_a_branching_point(&self) -> bool {
        self.current_index < self.move_records.len()
    }

    pub fn active_variation_label(&self) -> &'static str {
        if self.active_variation_id.is_some() {
            "Variation"
        } else {
            "Main"
        }
    }

    pub fn active_variation_name(&self) -> String {
        if let Some(id) = self.active_variation_id {
            return self
                .variations
                .iter()
                .find(|v| v.id == id)
                .map(|v| v.name.clone())
                .unwrap_or_else(|| format!("Variation {}", id));
        }
        "Main".to_string()
    }

    pub fn active_variation_cursor(&self) -> usize {
        self.current_index
    }

    pub fn active_variation_total(&self) -> usize {
        self.move_records.len()
    }

    pub fn active_variation_anchor(&self) -> usize {
        self.active_variation_id
            .and_then(|id| self.variations.iter().find(|v| v.id == id))
            .map(|v| v.anchor_index)
            .unwrap_or(0)
    }

    pub fn active_variation_parent_name(&self) -> Option<String> {
        let active_id = self.active_variation_id?;
        let var = self.variations.iter().find(|v| v.id == active_id)?;
        let parent_id = var.parent_variation_id?;
        self.variations
            .iter()
            .find(|v| v.id == parent_id)
            .map(|v| v.name.clone())
    }

    pub fn ensure_main_variation_cache(&mut self) {
        let keys = Self::build_position_keys(&self.snapshots);
        for (idx, key) in keys.iter().enumerate() {
            if let Some(cached) = self.analysis_cache.get(key)
                && idx < self.analyses.len()
            {
                self.analyses[idx] = Some(cached.clone());
            }
        }
    }

    pub fn remove_variation(&mut self, id: u32) {
        let mut to_remove = vec![id];
        let mut i = 0;
        while i < to_remove.len() {
            let parent = to_remove[i];
            for child in self
                .variations
                .iter()
                .filter(|v| v.parent_variation_id == Some(parent))
                .map(|v| v.id)
                .collect::<Vec<_>>()
            {
                to_remove.push(child);
            }
            i += 1;
        }

        self.variations.retain(|v| !to_remove.contains(&v.id));
        if let Some(active) = self.active_variation_id
            && to_remove.contains(&active)
        {
            self.switch_to_main_line();
        }
    }

    pub fn rename_variation(&mut self, id: u32, name: String) {
        if let Some(var) = self.variations.iter_mut().find(|v| v.id == id) {
            var.name = name;
        }
    }

    pub fn get_variation(&self, id: u32) -> Option<&ReviewVariation> {
        self.variations.iter().find(|v| v.id == id)
    }

    pub fn get_variation_mut(&mut self, id: u32) -> Option<&mut ReviewVariation> {
        self.variations.iter_mut().find(|v| v.id == id)
    }

    pub fn ensure_variation_name(&mut self, id: u32, default: &str) {
        if let Some(var) = self.variations.iter_mut().find(|v| v.id == id)
            && var.name.trim().is_empty()
        {
            var.name = default.to_string();
        }
    }

    pub fn current_line_prefix_records(&self) -> Vec<MoveRecord> {
        self.move_records
            .iter()
            .take(self.current_index.min(self.move_records.len()))
            .cloned()
            .collect()
    }

    pub fn has_same_prefix_as_mainline(&self) -> bool {
        let prefix_len = self.current_index.min(self.main_records.len());
        self.move_records
            .iter()
            .take(prefix_len)
            .map(|r| r.mv)
            .eq(self.main_records.iter().take(prefix_len).map(|r| r.mv))
    }

    pub fn needs_new_variation_for_move(&self, mv: Move) -> bool {
        self.is_current_position_a_branching_point()
            && !self
                .move_records
                .get(self.current_index)
                .map(|r| r.mv.from == mv.from && r.mv.to == mv.to && r.mv.promotion == mv.promotion)
                .unwrap_or(false)
    }

    fn switch_to_line_with_index(&mut self, target: Option<u32>, index: usize) {
        match target {
            None => self.switch_to_main_line(),
            Some(id) => self.switch_to_variation(id),
        }
        self.current_index = index.min(self.move_records.len());
        self.selected_square = None;
        self.valid_moves.clear();
        self.commit_active_variation_state();
    }

    fn rebase_to_parent_context_if_shared_node(&mut self) {
        let Some(active_id) = self.active_variation_id else {
            return;
        };
        let Some(active_var) = self.variations.iter().find(|v| v.id == active_id).cloned() else {
            return;
        };

        // Positions at or before anchor still belong to parent context.
        if self.current_index <= active_var.anchor_index {
            let index = self.current_index;
            self.switch_to_line_with_index(active_var.parent_variation_id, index);
        }
    }

    fn find_existing_child_variation_for_move(&self, mv: Move) -> Option<u32> {
        let parent_id = self.active_variation_id;
        let branch_index = self.current_index;
        let prefix = self
            .move_records
            .iter()
            .take(branch_index)
            .map(|r| r.mv)
            .collect::<Vec<_>>();

        self.variations
            .iter()
            .filter(|v| v.parent_variation_id == parent_id)
            .filter(|v| v.anchor_index == branch_index)
            .filter(|v| v.records.len() > branch_index)
            .filter(|v| {
                v.records
                    .iter()
                    .take(branch_index)
                    .map(|r| r.mv)
                    .eq(prefix.iter().copied())
            })
            .filter(|v| {
                v.records
                    .get(branch_index)
                    .map(|r| r.mv == mv)
                    .unwrap_or(false)
            })
            .map(|v| v.id)
            .min()
    }

    fn try_follow_existing_child_variation_move(&mut self, mv: Move) -> bool {
        let branch_index = self.current_index;
        let Some(var_id) = self.find_existing_child_variation_for_move(mv) else {
            return false;
        };

        self.switch_to_line_with_index(Some(var_id), branch_index);
        if self.jump_to_existing_next_move_if_same(mv) {
            self.commit_active_variation_state();
            return true;
        }
        false
    }

    pub fn current_move_count(&self) -> usize {
        self.move_records.len()
    }

    pub fn main_move_count(&self) -> usize {
        self.main_records.len()
    }

    pub fn should_show_mainline_return(&self) -> bool {
        self.is_in_variation() || self.current_move_count() != self.main_move_count()
    }

    pub fn line_title(&self) -> String {
        if let Some(id) = self.active_variation_id {
            if let Some(var) = self.variations.iter().find(|v| v.id == id) {
                return format!(
                    "{} (ply {}..{})",
                    var.name,
                    var.anchor_index,
                    var.records.len()
                );
            }
            format!("Variation {}", id)
        } else {
            "Main Line".to_string()
        }
    }

    pub fn line_subtitle(&self) -> String {
        if self.active_variation_id.is_none() {
            return "Original game".to_string();
        }
        if let Some(parent) = self.active_variation_parent_name() {
            format!("Branched from {}", parent)
        } else {
            "Branched from main line".to_string()
        }
    }

    pub fn line_progress_text(&self) -> String {
        format!(
            "{}/{}",
            self.current_index,
            self.total_positions().saturating_sub(1)
        )
    }

    pub fn line_can_go_prev(&self) -> bool {
        self.current_index > 0
    }

    pub fn line_can_go_next(&self) -> bool {
        self.current_index + 1 < self.total_positions()
    }

    pub fn line_is_main(&self) -> bool {
        self.active_variation_id.is_none()
    }

    pub fn line_id(&self) -> Option<u32> {
        self.active_variation_id
    }

    pub fn line_parent_id(&self) -> Option<u32> {
        let active = self.active_variation_id?;
        self.variations
            .iter()
            .find(|v| v.id == active)
            .and_then(|v| v.parent_variation_id)
    }

    pub fn line_depth(&self) -> usize {
        let mut depth = 0usize;
        let mut current = self.line_parent_id();
        while let Some(parent_id) = current {
            depth += 1;
            current = self
                .variations
                .iter()
                .find(|v| v.id == parent_id)
                .and_then(|v| v.parent_variation_id);
        }
        depth
    }

    pub fn line_branching_ply(&self) -> usize {
        self.active_variation_id
            .and_then(|id| self.variations.iter().find(|v| v.id == id))
            .map(|v| v.anchor_index)
            .unwrap_or(0)
    }

    pub fn line_record_len(&self) -> usize {
        self.move_records.len()
    }

    pub fn line_is_empty(&self) -> bool {
        self.move_records.is_empty()
    }

    pub fn line_is_at_end(&self) -> bool {
        self.current_index == self.move_records.len()
    }

    pub fn line_is_at_start(&self) -> bool {
        self.current_index == 0
    }

    pub fn line_has_mainline_suffix_visible(&self) -> bool {
        self.move_records.len() < self.main_records.len()
    }

    pub fn line_has_diverged_from_mainline(&self) -> bool {
        !self.has_same_prefix_as_mainline()
    }

    pub fn line_contains_move(&self, idx: usize) -> bool {
        idx < self.move_records.len()
    }

    pub fn line_move(&self, idx: usize) -> Option<&MoveRecord> {
        self.move_records.get(idx)
    }

    pub fn main_move(&self, idx: usize) -> Option<&MoveRecord> {
        self.main_records.get(idx)
    }

    pub fn mainline_suffix_from(&self, idx: usize) -> Vec<MoveRecord> {
        self.main_records.iter().skip(idx).cloned().collect()
    }

    pub fn line_name(&self) -> String {
        self.active_variation_name()
    }

    pub fn line_parent_name(&self) -> Option<String> {
        self.active_variation_parent_name()
    }

    pub fn line_anchor(&self) -> usize {
        self.active_variation_anchor()
    }

    pub fn line_cursor(&self) -> usize {
        self.active_variation_cursor()
    }

    pub fn line_total(&self) -> usize {
        self.active_variation_total()
    }

    pub fn line_label(&self) -> &'static str {
        self.active_variation_label()
    }

    pub fn line_summary(&self) -> String {
        format!("{} ({})", self.line_name(), self.line_progress_text())
    }

    pub fn line_branch_summary(&self) -> String {
        if let Some(parent) = self.line_parent_name() {
            format!("from {}", parent)
        } else if self.line_is_main() {
            "main".to_string()
        } else {
            "from main".to_string()
        }
    }

    pub fn line_debug_tag(&self) -> String {
        if let Some(id) = self.line_id() {
            format!("V{}", id)
        } else {
            "Main".to_string()
        }
    }

    pub fn line_kind(&self) -> &'static str {
        if self.line_is_main() {
            "main"
        } else {
            "variation"
        }
    }

    pub fn line_has_parent(&self) -> bool {
        self.line_parent_id().is_some()
    }

    pub fn line_parent_debug_tag(&self) -> Option<String> {
        self.line_parent_id().map(|id| format!("V{}", id))
    }

    pub fn line_anchor_text(&self) -> String {
        format!("ply {}", self.line_anchor())
    }

    pub fn line_depth_text(&self) -> String {
        format!("depth {}", self.line_depth())
    }

    pub fn line_cursor_text(&self) -> String {
        format!("cursor {}", self.line_cursor())
    }

    pub fn line_total_text(&self) -> String {
        format!("total {}", self.line_total())
    }

    pub fn line_meta_text(&self) -> String {
        format!(
            "{} | {} | {}",
            self.line_anchor_text(),
            self.line_depth_text(),
            self.line_cursor_text()
        )
    }

    pub fn line_title_text(&self) -> String {
        format!("{} {}", self.line_label(), self.line_name())
    }

    pub fn line_secondary_text(&self) -> String {
        format!(
            "{} • {}",
            self.line_branch_summary(),
            self.line_total_text()
        )
    }

    pub fn line_has_variations(&self) -> bool {
        self.variations
            .iter()
            .any(|v| v.parent_variation_id == self.active_variation_id)
    }

    pub fn line_children_ids(&self) -> Vec<u32> {
        self.variations
            .iter()
            .filter(|v| v.parent_variation_id == self.active_variation_id)
            .map(|v| v.id)
            .collect()
    }

    pub fn line_root_ids(&self) -> Vec<u32> {
        self.variations
            .iter()
            .filter(|v| v.parent_variation_id.is_none())
            .map(|v| v.id)
            .collect()
    }

    pub fn line_is_known_variation(&self, id: u32) -> bool {
        self.variations.iter().any(|v| v.id == id)
    }

    pub fn line_variation_name(&self, id: u32) -> Option<String> {
        self.variations
            .iter()
            .find(|v| v.id == id)
            .map(|v| v.name.clone())
    }

    pub fn line_variation_depth(&self, id: u32) -> usize {
        let mut depth = 0usize;
        let mut current = self
            .variations
            .iter()
            .find(|v| v.id == id)
            .and_then(|v| v.parent_variation_id);
        while let Some(parent_id) = current {
            depth += 1;
            current = self
                .variations
                .iter()
                .find(|v| v.id == parent_id)
                .and_then(|v| v.parent_variation_id);
        }
        depth
    }

    pub fn line_variation_anchor(&self, id: u32) -> Option<usize> {
        self.variations
            .iter()
            .find(|v| v.id == id)
            .map(|v| v.anchor_index)
    }

    pub fn line_variation_cursor(&self, id: u32) -> Option<usize> {
        self.variations
            .iter()
            .find(|v| v.id == id)
            .map(|v| v.cursor_index)
    }

    pub fn line_variation_record_len(&self, id: u32) -> Option<usize> {
        self.variations
            .iter()
            .find(|v| v.id == id)
            .map(|v| v.records.len())
    }

    pub fn line_variation_summary(&self, id: u32) -> Option<String> {
        let var = self.variations.iter().find(|v| v.id == id)?;
        Some(format!(
            "{} (ply {}..{}, depth {})",
            var.name,
            var.anchor_index,
            var.records.len(),
            self.line_variation_depth(id)
        ))
    }

    pub fn line_variation_children_ids(&self, id: u32) -> Vec<u32> {
        self.variations
            .iter()
            .filter(|v| v.parent_variation_id == Some(id))
            .map(|v| v.id)
            .collect()
    }

    pub fn line_variation_parent_id(&self, id: u32) -> Option<u32> {
        self.variations
            .iter()
            .find(|v| v.id == id)
            .and_then(|v| v.parent_variation_id)
    }

    pub fn line_variation_parent_name(&self, id: u32) -> Option<String> {
        let parent_id = self.line_variation_parent_id(id)?;
        self.line_variation_name(parent_id)
    }

    pub fn line_variation_debug_tag(&self, id: u32) -> String {
        format!("V{}", id)
    }

    pub fn line_variation_kind(&self, _id: u32) -> &'static str {
        "variation"
    }

    pub fn line_variation_anchor_text(&self, id: u32) -> String {
        format!("ply {}", self.line_variation_anchor(id).unwrap_or_default())
    }

    pub fn line_variation_depth_text(&self, id: u32) -> String {
        format!("depth {}", self.line_variation_depth(id))
    }

    pub fn line_variation_cursor_text(&self, id: u32) -> String {
        format!(
            "cursor {}",
            self.line_variation_cursor(id).unwrap_or_default()
        )
    }

    pub fn line_variation_total_text(&self, id: u32) -> String {
        format!(
            "total {}",
            self.line_variation_record_len(id).unwrap_or_default()
        )
    }

    pub fn line_variation_meta_text(&self, id: u32) -> String {
        format!(
            "{} | {} | {}",
            self.line_variation_anchor_text(id),
            self.line_variation_depth_text(id),
            self.line_variation_cursor_text(id)
        )
    }

    pub fn line_variation_title_text(&self, id: u32) -> String {
        format!(
            "{} {}",
            self.line_variation_debug_tag(id),
            self.line_variation_name(id)
                .unwrap_or_else(|| format!("Variation {}", id))
        )
    }

    pub fn line_variation_secondary_text(&self, id: u32) -> String {
        let parent = self
            .line_variation_parent_name(id)
            .unwrap_or_else(|| "main".to_string());
        format!("from {} • {}", parent, self.line_variation_total_text(id))
    }

    pub fn line_variation_has_children(&self, id: u32) -> bool {
        self.variations
            .iter()
            .any(|v| v.parent_variation_id == Some(id))
    }

    pub fn line_variation_exists(&self, id: u32) -> bool {
        self.line_is_known_variation(id)
    }

    pub fn line_variation_path_to_root(&self, id: u32) -> Vec<u32> {
        let mut path = Vec::new();
        let mut current = Some(id);
        while let Some(cid) = current {
            path.push(cid);
            current = self.line_variation_parent_id(cid);
        }
        path.reverse();
        path
    }

    pub fn line_variation_depth_from_root(&self, id: u32) -> usize {
        self.line_variation_path_to_root(id).len().saturating_sub(1)
    }

    pub fn line_variation_node_text(&self, id: u32) -> String {
        format!(
            "{} [{}]",
            self.line_variation_title_text(id),
            self.line_variation_meta_text(id)
        )
    }

    pub fn line_variation_short_text(&self, id: u32) -> String {
        self.line_variation_name(id)
            .unwrap_or_else(|| format!("Variation {}", id))
    }

    pub fn line_variation_hover_text(&self, id: u32) -> String {
        let parent = self
            .line_variation_parent_name(id)
            .unwrap_or_else(|| "main".to_string());
        format!(
            "{}\n{}\nparent: {}",
            self.line_variation_node_text(id),
            self.line_variation_total_text(id),
            parent
        )
    }

    pub fn line_variation_indicator(&self, id: u32) -> &'static str {
        if self.active_variation_id == Some(id) {
            "●"
        } else {
            "○"
        }
    }

    pub fn line_variation_indent_spaces(&self, id: u32) -> String {
        "  ".repeat(self.line_variation_depth(id))
    }

    pub fn line_variation_display_label(&self, id: u32) -> String {
        format!(
            "{}{} {} ({})",
            self.line_variation_indent_spaces(id),
            self.line_variation_indicator(id),
            self.line_variation_short_text(id),
            self.line_variation_record_len(id).unwrap_or_default()
        )
    }

    pub fn line_all_variation_labels(&self) -> Vec<(u32, String)> {
        self.variation_tree()
            .into_iter()
            .map(|(id, _)| (id, self.line_variation_display_label(id)))
            .collect()
    }

    pub fn line_main_label(&self) -> String {
        if self.line_is_main() {
            format!("● Main ({})", self.main_records.len())
        } else {
            format!("○ Main ({})", self.main_records.len())
        }
    }

    pub fn line_can_switch_to_main(&self) -> bool {
        !self.line_is_main()
    }

    pub fn line_can_switch_to_variation(&self, id: u32) -> bool {
        self.line_is_known_variation(id)
    }

    pub fn line_should_create_child_on_branch(&self) -> bool {
        self.is_in_variation() && self.is_current_position_a_branching_point()
    }

    pub fn line_should_create_root_on_branch(&self) -> bool {
        !self.is_in_variation() && self.is_current_position_a_branching_point()
    }

    pub fn line_branch_creation_reason(&self) -> &'static str {
        if self.line_should_create_child_on_branch() {
            "child"
        } else if self.line_should_create_root_on_branch() {
            "root"
        } else if self.is_in_variation() {
            "extend"
        } else {
            "main-edit"
        }
    }

    pub fn line_debug_summary(&self) -> String {
        format!(
            "{} | {} | {} | vars {}",
            self.line_title(),
            self.line_subtitle(),
            self.line_progress_text(),
            self.variations.len()
        )
    }

    pub fn line_variation_count(&self) -> usize {
        self.variations.len()
    }

    pub fn line_has_any_variations(&self) -> bool {
        !self.variations.is_empty()
    }

    pub fn line_active_variation_id_text(&self) -> String {
        self.active_variation_id
            .map(|id| format!("V{}", id))
            .unwrap_or_else(|| "Main".to_string())
    }

    pub fn line_state_text(&self) -> String {
        format!(
            "{} | {}",
            self.line_active_variation_id_text(),
            self.line_progress_text()
        )
    }

    pub fn line_current_move_san(&self) -> Option<String> {
        if self.current_index == 0 {
            return None;
        }
        self.move_records
            .get(self.current_index - 1)
            .map(|r| r.san.clone())
    }

    pub fn line_next_move_san(&self) -> Option<String> {
        self.move_records
            .get(self.current_index)
            .map(|r| r.san.clone())
    }

    pub fn line_current_and_next_text(&self) -> String {
        let curr = self
            .line_current_move_san()
            .unwrap_or_else(|| "-".to_string());
        let next = self.line_next_move_san().unwrap_or_else(|| "-".to_string());
        format!("curr: {}, next: {}", curr, next)
    }

    pub fn line_eval_available(&self) -> bool {
        self.analyses
            .get(self.current_index)
            .and_then(|a| a.as_ref())
            .is_some()
    }

    pub fn line_eval_depth_text(&self) -> Option<String> {
        let analysis = self
            .analyses
            .get(self.current_index)
            .and_then(|a| a.as_ref())?;
        Some(format!("depth {}", analysis.depth))
    }

    pub fn line_eval_cp(&self) -> Option<i32> {
        self.analyses
            .get(self.current_index)
            .and_then(|a| a.as_ref())
            .and_then(|a| a.lines.first().map(|l| l.eval_cp))
    }

    pub fn line_eval_text(&self) -> Option<String> {
        let cp = self.line_eval_cp()?;
        if cp.abs() > 29000 {
            let mate = 30000 - cp.abs();
            if cp > 0 {
                Some(format!("M{}", mate))
            } else {
                Some(format!("-M{}", mate))
            }
        } else {
            Some(format!("{:+.2}", cp as f32 / 100.0))
        }
    }

    pub fn line_eval_summary(&self) -> Option<String> {
        let eval = self.line_eval_text()?;
        let depth = self
            .line_eval_depth_text()
            .unwrap_or_else(|| "depth ?".to_string());
        Some(format!("{} ({})", eval, depth))
    }

    pub fn line_best_move_uci(&self) -> Option<String> {
        self.analyses
            .get(self.current_index)
            .and_then(|a| a.as_ref())
            .and_then(|a| a.lines.first().map(|l| l.best_move_uci.clone()))
    }

    pub fn line_best_move_text(&self) -> Option<String> {
        self.line_best_move_uci().map(|uci| format!("best {}", uci))
    }

    pub fn line_info_text(&self) -> String {
        let eval = self
            .line_eval_summary()
            .unwrap_or_else(|| "eval ?".to_string());
        let best = self
            .line_best_move_text()
            .unwrap_or_else(|| "best ?".to_string());
        format!("{} | {}", eval, best)
    }

    pub fn line_cache_hit_ratio_hint(&self) -> String {
        let analyzed = self.analyses.iter().filter(|a| a.is_some()).count();
        let total = self.analyses.len().max(1);
        format!("analyzed {}/{}", analyzed, total)
    }

    pub fn line_cache_size_hint(&self) -> String {
        format!("cache {}", self.analysis_cache.len())
    }

    pub fn line_performance_hint(&self) -> String {
        format!(
            "{} • {}",
            self.line_cache_hit_ratio_hint(),
            self.line_cache_size_hint()
        )
    }

    pub fn line_tree_hint(&self) -> String {
        format!("{} variations", self.variations.len())
    }

    pub fn line_status_hint(&self) -> String {
        format!("{} • {}", self.line_state_text(), self.line_tree_hint())
    }

    pub fn line_footer_hint(&self) -> String {
        format!(
            "{} • {}",
            self.line_info_text(),
            self.line_performance_hint()
        )
    }

    pub fn line_can_delete_variation(&self, id: u32) -> bool {
        self.line_is_known_variation(id)
    }

    pub fn line_delete_variation(&mut self, id: u32) {
        self.remove_variation(id);
    }

    pub fn line_create_child_variation(&mut self) -> u32 {
        self.create_variation_from_current()
    }

    pub fn line_create_variation_with_name(&mut self, name: String) -> u32 {
        let id = self.create_variation_from_current();
        self.rename_variation(id, name);
        id
    }

    pub fn line_active_or_main_id(&self) -> Option<u32> {
        self.active_variation_id
    }

    pub fn line_is_variation_active(&self, id: u32) -> bool {
        self.active_variation_id == Some(id)
    }

    pub fn line_toggle_to_variation(&mut self, id: u32) {
        if self.active_variation_id == Some(id) {
            self.switch_to_main_line();
        } else {
            self.switch_to_variation(id);
        }
    }

    pub fn line_rebase_active_variation_to_current_main_prefix(&mut self) {
        if self.active_variation_id.is_none() {
            return;
        }
        let prefix = self.main_records[..self.current_index.min(self.main_records.len())].to_vec();
        if let Some(active) = self.active_variation_id
            && let Some(var) = self.variations.iter_mut().find(|v| v.id == active)
        {
            var.records = prefix;
            var.anchor_index = self.current_index.min(self.main_records.len());
            var.cursor_index = var.anchor_index.min(var.records.len());
        }
        self.switch_to_main_line();
    }

    pub fn line_snapshot_key(&self, idx: usize) -> Option<String> {
        self.position_keys.get(idx).cloned()
    }

    pub fn line_current_key(&self) -> Option<String> {
        self.line_snapshot_key(self.current_index)
    }

    pub fn line_has_cached_current(&self) -> bool {
        self.line_current_key()
            .and_then(|k| self.analysis_cache.get(&k).cloned())
            .is_some()
    }

    pub fn line_mark_analysis_cached(&mut self, idx: usize, analysis: PositionAnalysis) {
        if let Some(key) = self.position_keys.get(idx) {
            self.analysis_cache.insert(key.clone(), analysis);
        }
    }

    pub fn line_clear_transient_selection(&mut self) {
        self.selected_square = None;
        self.valid_moves.clear();
    }

    pub fn line_prepare_navigation_change(&mut self) {
        self.line_clear_transient_selection();
        self.commit_active_variation_state();
    }

    pub fn line_after_navigation_change(&mut self) {
        self.start_analysis();
    }

    pub fn line_sync_active_variation_cursor(&mut self) {
        self.commit_active_variation_state();
    }

    pub fn line_sync_and_restart_analysis(&mut self) {
        self.commit_active_variation_state();
        self.start_analysis();
    }

    pub fn line_apply_existing_or_branch_move(&mut self, mv: Move, san: String, color: Color) {
        self.rebase_to_parent_context_if_shared_node();

        if self.jump_to_existing_next_move_if_same(mv) {
            self.line_sync_active_variation_cursor();
            return;
        }

        if self.try_follow_existing_child_variation_move(mv) {
            return;
        }

        let force_new = self.needs_new_variation_for_move(mv);
        self.apply_review_move(mv, san, color, force_new);
    }

    pub fn line_append_move_to_active_variation(&mut self, mv: Move, san: String, color: Color) {
        self.apply_review_move(mv, san, color, false);
    }

    pub fn line_branch_move_from_current(&mut self, mv: Move, san: String, color: Color) {
        self.apply_review_move(mv, san, color, true);
    }

    pub fn line_active_variation_records(&self) -> Vec<MoveRecord> {
        self.move_records.clone()
    }

    pub fn line_main_records(&self) -> Vec<MoveRecord> {
        self.main_records.clone()
    }

    pub fn line_update_main_records_from_current_if_main(&mut self) {
        if self.active_variation_id.is_none() {
            self.main_records = self.move_records.clone();
        }
    }

    pub fn line_is_mainline_position(&self, idx: usize) -> bool {
        if idx == 0 {
            return true;
        }
        let idx0 = idx - 1;
        self.move_records.get(idx0).map(|m| m.mv) == self.main_records.get(idx0).map(|m| m.mv)
    }

    pub fn line_mainline_divergence_index(&self) -> Option<usize> {
        let len = self.move_records.len().min(self.main_records.len());
        for i in 0..len {
            if self.move_records[i].mv != self.main_records[i].mv {
                return Some(i);
            }
        }
        if self.move_records.len() != self.main_records.len() {
            return Some(len);
        }
        None
    }

    pub fn line_is_diverged(&self) -> bool {
        self.line_mainline_divergence_index().is_some()
    }

    pub fn line_divergence_text(&self) -> String {
        if let Some(idx) = self.line_mainline_divergence_index() {
            format!("diverged at ply {}", idx)
        } else {
            "no divergence".to_string()
        }
    }

    pub fn line_mainline_overlay_needed(&self) -> bool {
        self.line_is_diverged() || self.line_has_mainline_suffix_visible()
    }

    pub fn line_mainline_overlay_hint(&self) -> String {
        if self.line_mainline_overlay_needed() {
            "mainline hints shown".to_string()
        } else {
            "on mainline".to_string()
        }
    }

    pub fn line_branching_context_hint(&self) -> String {
        format!(
            "{} • {}",
            self.line_branch_creation_reason(),
            self.line_divergence_text()
        )
    }

    pub fn line_debug_all(&self) -> String {
        format!(
            "{} | {} | {} | {}",
            self.line_debug_summary(),
            self.line_info_text(),
            self.line_performance_hint(),
            self.line_branching_context_hint()
        )
    }

    pub fn line_variation_exists_or_main(&self, id: Option<u32>) -> bool {
        match id {
            None => true,
            Some(i) => self.line_is_known_variation(i),
        }
    }

    pub fn line_switch_to(&mut self, id: Option<u32>) {
        match id {
            None => self.switch_to_main_line(),
            Some(i) => self.switch_to_variation(i),
        }
    }

    pub fn line_select_main(&mut self) {
        self.switch_to_main_line();
    }

    pub fn line_select_variation(&mut self, id: u32) {
        self.switch_to_variation(id);
    }

    pub fn line_new_branch_name_default(&self) -> String {
        format!("Variation {}", self.next_variation_id)
    }

    pub fn line_new_branch_from_current(&mut self) -> u32 {
        self.create_variation_from_current()
    }

    pub fn line_new_named_branch_from_current(&mut self, name: String) -> u32 {
        self.line_create_variation_with_name(name)
    }

    pub fn line_all_branch_ids(&self) -> Vec<u32> {
        self.variations.iter().map(|v| v.id).collect()
    }

    pub fn go_first(&mut self) {
        self.current_index = 0;
    }

    pub fn go_prev(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
        }
    }

    pub fn go_next(&mut self) {
        if self.current_index + 1 < self.snapshots.len() {
            self.current_index += 1;
        }
    }

    pub fn go_last(&mut self) {
        if !self.snapshots.is_empty() {
            self.current_index = self.snapshots.len() - 1;
        }
    }

    pub fn total_positions(&self) -> usize {
        self.snapshots.len()
    }

    fn commit_active_variation_state(&mut self) {
        if let Some(active_id) = self.active_variation_id
            && let Some(var) = self.variations.iter_mut().find(|v| v.id == active_id)
        {
            var.records = self.move_records.clone();
            var.cursor_index = self.current_index;
        }
    }

    fn rebuild_active_line_from_records(
        &mut self,
        records: Vec<MoveRecord>,
        preferred_index: usize,
    ) {
        self.move_records = records;
        self.snapshots = Self::build_snapshots(self.initial_board.clone(), &self.move_records);
        self.position_keys = Self::build_position_keys(&self.snapshots);
        self.analyses = vec![None; self.snapshots.len()];

        for (idx, key) in self.position_keys.iter().enumerate() {
            if let Some(cached) = self.analysis_cache.get(key) {
                self.analyses[idx] = Some(cached.clone());
            }
        }

        self.classifications = vec![None; self.move_records.len()];
        for idx in 1..self.snapshots.len() {
            self.recompute_classification_at(idx);
        }

        self.current_index = preferred_index.min(self.snapshots.len().saturating_sub(1));
        self.selected_square = None;
        self.valid_moves.clear();
        self.start_analysis();
    }

    fn build_snapshots(initial_board: Board, move_records: &[MoveRecord]) -> Vec<Board> {
        let mut snapshots = Vec::with_capacity(move_records.len() + 1);
        snapshots.push(initial_board);

        for record in move_records {
            if let Ok((board, _, _)) = Board::from_fen(&record.fen_after) {
                snapshots.push(board);
            } else {
                let mut board = snapshots.last().unwrap().clone();
                board.make_move(record.mv);
                snapshots.push(board);
            }
        }

        snapshots
    }

    fn build_position_keys(snapshots: &[Board]) -> Vec<String> {
        snapshots
            .iter()
            .enumerate()
            .map(|(idx, board)| board.position_key(Self::color_for_snapshot(idx)))
            .collect()
    }

    fn color_for_snapshot(index: usize) -> Color {
        if index.is_multiple_of(2) {
            Color::White
        } else {
            Color::Black
        }
    }
}

mod analysis_pipeline;
mod classification;
mod ui_bridge;
#[cfg(test)]
mod tests {
    use super::*;

    fn build_mainline_review() -> ReviewState {
        let mut board = Board::new();

        let e4 = Move {
            from: (6, 4),
            to: (4, 4),
            promotion: None,
        };
        board.make_move(e4);
        let rec1 = MoveRecord {
            mv: e4,
            san: "e4".to_string(),
            fen_after: board.to_fen(Color::Black),
            color: Color::White,
        };

        let e5 = Move {
            from: (1, 4),
            to: (3, 4),
            promotion: None,
        };
        board.make_move(e5);
        let rec2 = MoveRecord {
            mv: e5,
            san: "e5".to_string(),
            fen_after: board.to_fen(Color::White),
            color: Color::Black,
        };

        ReviewState::new(Board::new(), vec![rec1, rec2])
    }

    #[test]
    fn test_analysis_cache_avoids_re_request() {
        let board = Board::new();
        let mut review = ReviewState::new(board.clone(), vec![]);
        let key = board.position_key(Color::White);
        let cached = PositionAnalysis {
            lines: vec![AnalysisLine {
                eval_cp: 12,
                best_move_uci: "e2e4".to_string(),
            }],
            depth: 18,
            verification_passes: 1,
            stability_pct: 100,
        };
        review.analysis_cache.insert(key, cached.clone());

        review.start_analysis();
        let req = review.next_analysis_fen();

        assert!(req.is_none(), "cached position should not request engine");
        assert!(review.analyses[0].is_some());
        assert_eq!(review.analyses[0].as_ref().unwrap().depth, cached.depth);
    }

    #[test]
    fn test_variation_keeps_mainline() {
        let board = Board::new();
        let mut review = ReviewState::new(board, vec![]);

        // Create a variation move from the initial position.
        let mv = Move {
            from: (6, 4),
            to: (4, 4),
            promotion: None,
        };
        let san = "e4".to_string();
        review.apply_review_move(mv, san, Color::White, true);

        assert!(review.is_in_variation());
        assert_eq!(review.main_records.len(), 0);
        assert_eq!(review.move_records.len(), 1);

        review.switch_to_main_line();
        assert!(!review.is_in_variation());
        assert_eq!(review.move_records.len(), 0);
    }

    #[test]
    fn test_multi_depth_verification_vote() {
        let board = Board::new();
        let mut review = ReviewState::new(board, vec![]);
        review.start_analysis();

        let req1 = review
            .next_analysis_request()
            .expect("first request should be created");
        assert_eq!(req1.2, 12);

        let next = review.ingest_analysis_result(PositionAnalysis {
            lines: vec![
                AnalysisLine {
                    eval_cp: 30,
                    best_move_uci: "e2e4".to_string(),
                },
                AnalysisLine {
                    eval_cp: 20,
                    best_move_uci: "d2d4".to_string(),
                },
            ],
            depth: 12,
            verification_passes: 1,
            stability_pct: 100,
        });
        assert_eq!(next.as_ref().map(|(_, d)| *d), Some(16));

        let next = review.ingest_analysis_result(PositionAnalysis {
            lines: vec![
                AnalysisLine {
                    eval_cp: 28,
                    best_move_uci: "d2d4".to_string(),
                },
                AnalysisLine {
                    eval_cp: 27,
                    best_move_uci: "e2e4".to_string(),
                },
            ],
            depth: 16,
            verification_passes: 1,
            stability_pct: 100,
        });
        assert_eq!(next.as_ref().map(|(_, d)| *d), Some(20));

        let done = review.ingest_analysis_result(PositionAnalysis {
            lines: vec![
                AnalysisLine {
                    eval_cp: 35,
                    best_move_uci: "e2e4".to_string(),
                },
                AnalysisLine {
                    eval_cp: 24,
                    best_move_uci: "d2d4".to_string(),
                },
            ],
            depth: 20,
            verification_passes: 1,
            stability_pct: 100,
        });

        assert!(done.is_none());
        let final_analysis = review.analyses[0]
            .as_ref()
            .expect("final analysis should exist");
        assert_eq!(final_analysis.depth, 20);
        assert_eq!(final_analysis.verification_passes, 3);
        assert_eq!(final_analysis.lines[0].best_move_uci, "e2e4");
        assert!(final_analysis.stability_pct >= 60);
    }

    #[test]
    fn test_best_move_from_engine_line_not_misclassified_by_eval_drift() {
        let mut board = Board::new();
        let e4 = Move {
            from: (6, 4),
            to: (4, 4),
            promotion: None,
        };
        board.make_move(e4);
        let rec = MoveRecord {
            mv: e4,
            san: "e4".to_string(),
            fen_after: board.to_fen(Color::Black),
            color: Color::White,
        };
        let mut review = ReviewState::new(Board::new(), vec![rec]);

        // Before move: e2e4 is top-1.
        review.analyses[0] = Some(PositionAnalysis {
            lines: vec![
                AnalysisLine {
                    eval_cp: 36,
                    best_move_uci: "e2e4".to_string(),
                },
                AnalysisLine {
                    eval_cp: 14,
                    best_move_uci: "d2d4".to_string(),
                },
            ],
            depth: 20,
            verification_passes: 1,
            stability_pct: 100,
        });

        // After move: simulate noisy drift in a separate search result.
        review.analyses[1] = Some(PositionAnalysis {
            lines: vec![AnalysisLine {
                eval_cp: 320,
                best_move_uci: "c7c5".to_string(),
            }],
            depth: 20,
            verification_passes: 1,
            stability_pct: 100,
        });

        review.recompute_classification_at(1);
        assert_eq!(review.classifications[0], Some(MoveClassification::Best));
    }

    #[test]
    fn test_replay_mainline_move_from_variation_restores_main() {
        let mut review = build_mainline_review();
        review.go_next(); // after 1.e4, black to move

        let c5 = Move {
            from: (1, 2),
            to: (3, 2),
            promotion: None,
        };
        review.line_apply_existing_or_branch_move(c5, "c5".to_string(), Color::Black);
        assert!(review.is_in_variation());
        assert_eq!(review.variations.len(), 1);

        review.go_prev(); // back to branch point
        let e5 = Move {
            from: (1, 4),
            to: (3, 4),
            promotion: None,
        };
        review.line_apply_existing_or_branch_move(e5, "e5".to_string(), Color::Black);

        assert!(!review.is_in_variation());
        assert_eq!(review.current_index, 2);
        assert_eq!(review.variations.len(), 1);
    }

    #[test]
    fn test_reuse_existing_branch_and_create_sibling_under_main() {
        let mut review = build_mainline_review();
        review.go_next(); // after 1.e4

        let c5 = Move {
            from: (1, 2),
            to: (3, 2),
            promotion: None,
        };
        review.line_apply_existing_or_branch_move(c5, "c5".to_string(), Color::Black);
        let first_var = review
            .active_variation_id
            .expect("variation should be created");
        assert_eq!(review.variations.len(), 1);

        review.switch_to_main_line();
        review.go_prev(); // back to branch point on main
        review.line_apply_existing_or_branch_move(c5, "c5".to_string(), Color::Black);

        assert_eq!(review.active_variation_id, Some(first_var));
        assert_eq!(
            review.variations.len(),
            1,
            "same move should reuse existing branch"
        );

        review.go_prev(); // back to branch point in reused variation
        let d5 = Move {
            from: (1, 3),
            to: (3, 3),
            promotion: None,
        };
        review.line_apply_existing_or_branch_move(d5, "d5".to_string(), Color::Black);
        let second_var = review
            .active_variation_id
            .expect("new sibling variation should be active");

        assert_ne!(second_var, first_var);
        assert_eq!(review.variations.len(), 2);
        let second = review
            .variations
            .iter()
            .find(|v| v.id == second_var)
            .expect("second variation exists");
        assert_eq!(
            second.parent_variation_id, None,
            "branch at anchor should be sibling under main line"
        );
    }
}
