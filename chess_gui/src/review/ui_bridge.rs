//! UI bridge helpers for `ReviewState`.
//!
//! 该模块主要提供面向界面的衍生文本/状态封装，避免 `ui` 层直接拼装
//! 过多领域细节。

use super::*;

impl ReviewState {
    pub fn line_root_branch_ids(&self) -> Vec<u32> {
        self.variations
            .iter()
            .filter(|v| v.parent_variation_id.is_none())
            .map(|v| v.id)
            .collect()
    }

    pub fn line_branch_depth_map(&self) -> HashMap<u32, usize> {
        self.variations
            .iter()
            .map(|v| (v.id, self.line_variation_depth(v.id)))
            .collect()
    }

    pub fn line_branch_labels(&self) -> Vec<String> {
        self.variation_tree()
            .into_iter()
            .map(|(id, _)| self.line_variation_display_label(id))
            .collect()
    }

    pub fn line_branch_count_text(&self) -> String {
        format!("{} branches", self.variations.len())
    }

    pub fn line_main_branch_count_text(&self) -> String {
        format!("{} root branches", self.line_root_branch_ids().len())
    }

    pub fn line_branch_overview_text(&self) -> String {
        format!(
            "{} • {}",
            self.line_branch_count_text(),
            self.line_main_branch_count_text()
        )
    }

    pub fn line_nav_state_text(&self) -> String {
        format!(
            "prev {} / next {}",
            self.line_can_go_prev(),
            self.line_can_go_next()
        )
    }

    pub fn line_analysis_state_text(&self) -> String {
        format!(
            "analyzing {} / pending {:?}",
            self.analyzing, self.pending_analysis_index
        )
    }

    pub fn line_full_state_text(&self) -> String {
        format!(
            "{} | {} | {}",
            self.line_status_hint(),
            self.line_nav_state_text(),
            self.line_analysis_state_text()
        )
    }

    pub fn line_to_debug_json_like(&self) -> String {
        format!(
            r#"{{"line":"{}","kind":"{}","progress":"{}","branches":{},"cache":{}}}"#,
            self.line_name(),
            self.line_kind(),
            self.line_progress_text(),
            self.variations.len(),
            self.analysis_cache.len()
        )
    }

    pub fn line_dump_for_logs(&self) -> String {
        format!(
            "{} | {}",
            self.line_full_state_text(),
            self.line_to_debug_json_like()
        )
    }

    pub fn line_for_ui_subtitle(&self) -> String {
        self.line_subtitle()
    }

    pub fn line_for_ui_title(&self) -> String {
        self.line_title()
    }

    pub fn line_for_ui_branch_hint(&self) -> String {
        self.line_branch_overview_text()
    }

    pub fn line_for_ui_perf_hint(&self) -> String {
        self.line_performance_hint()
    }

    pub fn line_for_ui_info_hint(&self) -> String {
        self.line_info_text()
    }

    pub fn line_for_ui_state_hint(&self) -> String {
        self.line_status_hint()
    }

    pub fn line_for_ui_footer_hint(&self) -> String {
        self.line_footer_hint()
    }

    pub fn line_for_ui_debug_hint(&self) -> String {
        self.line_debug_all()
    }

    pub fn line_for_ui_main_label(&self) -> String {
        self.line_main_label()
    }

    pub fn line_for_ui_variation_label(&self, id: u32) -> String {
        self.line_variation_display_label(id)
    }

    pub fn line_for_ui_variation_hover(&self, id: u32) -> String {
        self.line_variation_hover_text(id)
    }

    pub fn line_for_ui_variation_tree(&self) -> Vec<(u32, usize)> {
        self.variation_tree()
    }

    pub fn line_for_ui_variation_list(&self) -> Vec<(u32, String)> {
        self.line_all_variation_labels()
    }

    pub fn line_for_ui_can_return_main(&self) -> bool {
        self.should_show_mainline_return()
    }

    pub fn line_for_ui_is_variation(&self) -> bool {
        self.is_in_variation()
    }

    pub fn line_for_ui_current_is_main(&self) -> bool {
        self.line_is_main()
    }

    pub fn line_for_ui_current_variation_id(&self) -> Option<u32> {
        self.active_variation_id
    }

    pub fn line_for_ui_current_variation_name(&self) -> String {
        self.active_variation_name()
    }

    pub fn line_for_ui_current_progress(&self) -> String {
        self.line_progress_text()
    }

    pub fn line_for_ui_branch_reason(&self) -> String {
        self.line_branch_creation_reason().to_string()
    }

    pub fn line_for_ui_branch_context(&self) -> String {
        self.line_branching_context_hint()
    }

    pub fn line_for_ui_cache_hint(&self) -> String {
        self.line_cache_hit_ratio_hint()
    }

    pub fn line_for_ui_cache_size(&self) -> String {
        self.line_cache_size_hint()
    }

    pub fn line_for_ui_eval_hint(&self) -> Option<String> {
        self.line_eval_summary()
    }

    pub fn line_for_ui_best_hint(&self) -> Option<String> {
        self.line_best_move_text()
    }

    pub fn line_for_ui_current_key(&self) -> Option<String> {
        self.line_current_key()
    }

    pub fn line_for_ui_dump(&self) -> String {
        self.line_dump_for_logs()
    }

    pub fn line_for_ui_tree_dump(&self) -> String {
        self.variation_tree()
            .into_iter()
            .map(|(id, depth)| {
                format!(
                    "{}{}",
                    "  ".repeat(depth),
                    self.line_variation_short_text(id)
                )
            })
            .collect::<Vec<_>>()
            .join("\\n")
    }

    pub fn line_for_ui_summary_block(&self) -> String {
        format!(
            "{}\\n{}\\n{}\\n{}",
            self.line_for_ui_title(),
            self.line_for_ui_subtitle(),
            self.line_for_ui_branch_hint(),
            self.line_for_ui_perf_hint()
        )
    }

    pub fn line_for_ui_help_text(&self) -> String {
        "Main line is preserved. Branches can be switched from the Lines panel.".to_string()
    }

    pub fn line_for_ui_branch_help_text(&self) -> String {
        "Playing a different move from a known position creates/switches to a branch.".to_string()
    }

    pub fn line_for_ui_cache_help_text(&self) -> String {
        "Previously analyzed positions are reused from cache to avoid repeated engine work."
            .to_string()
    }

    pub fn line_for_ui_notes_text(&self) -> String {
        format!(
            "{} {} {}",
            self.line_for_ui_help_text(),
            self.line_for_ui_branch_help_text(),
            self.line_for_ui_cache_help_text()
        )
    }

    pub fn line_for_ui_notes_lines(&self) -> Vec<String> {
        vec![
            self.line_for_ui_help_text(),
            self.line_for_ui_branch_help_text(),
            self.line_for_ui_cache_help_text(),
        ]
    }

    pub fn line_for_ui_is_cache_hot(&self) -> bool {
        self.line_has_cached_current()
    }

    pub fn line_for_ui_cache_state_text(&self) -> String {
        if self.line_for_ui_is_cache_hot() {
            "cache hit".to_string()
        } else {
            "cache miss".to_string()
        }
    }

    pub fn line_for_ui_analysis_state_text(&self) -> String {
        if self.analyzing {
            "analyzing".to_string()
        } else {
            "idle".to_string()
        }
    }

    pub fn line_for_ui_quick_status(&self) -> String {
        format!(
            "{} • {} • {}",
            self.line_for_ui_cache_state_text(),
            self.line_for_ui_analysis_state_text(),
            self.line_for_ui_current_progress()
        )
    }

    pub fn line_for_ui_all_status(&self) -> Vec<String> {
        vec![
            self.line_for_ui_quick_status(),
            self.line_for_ui_branch_hint(),
            self.line_for_ui_perf_hint(),
        ]
    }

    pub fn line_for_ui_badge_text(&self) -> String {
        if self.line_is_main() {
            "MAIN".to_string()
        } else {
            "VAR".to_string()
        }
    }

    pub fn line_for_ui_badge_color_key(&self) -> &'static str {
        if self.line_is_main() {
            "main"
        } else {
            "variation"
        }
    }

    pub fn line_for_ui_branch_badge_text(&self, id: u32) -> String {
        if self.line_is_variation_active(id) {
            "ACTIVE".to_string()
        } else {
            "VAR".to_string()
        }
    }

    pub fn line_for_ui_branch_badge_color_key(&self, id: u32) -> &'static str {
        if self.line_is_variation_active(id) {
            "active"
        } else {
            "inactive"
        }
    }

    pub fn line_for_ui_branch_rows(&self) -> Vec<(u32, usize, String)> {
        self.variation_tree()
            .into_iter()
            .map(|(id, depth)| (id, depth, self.line_variation_display_label(id)))
            .collect()
    }

    pub fn line_for_ui_main_row(&self) -> (Option<u32>, usize, String) {
        (None, 0, self.line_main_label())
    }

    pub fn line_for_ui_rows(&self) -> Vec<(Option<u32>, usize, String)> {
        let mut rows = vec![self.line_for_ui_main_row()];
        rows.extend(
            self.line_for_ui_branch_rows()
                .into_iter()
                .map(|(id, depth, label)| (Some(id), depth + 1, label)),
        );
        rows
    }

    pub fn line_for_ui_switch(&mut self, row_id: Option<u32>) {
        self.line_switch_to(row_id);
    }

    pub fn line_for_ui_create_branch_here(&mut self) -> u32 {
        self.line_new_branch_from_current()
    }

    pub fn line_for_ui_remove_branch(&mut self, id: u32) {
        self.line_delete_variation(id);
    }

    pub fn line_for_ui_rename_branch(&mut self, id: u32, name: String) {
        self.rename_variation(id, name);
    }

    pub fn line_for_ui_branch_exists(&self, id: u32) -> bool {
        self.line_variation_exists(id)
    }

    pub fn line_for_ui_branch_name(&self, id: u32) -> Option<String> {
        self.line_variation_name(id)
    }

    pub fn line_for_ui_branch_depth(&self, id: u32) -> usize {
        self.line_variation_depth(id)
    }

    pub fn line_for_ui_branch_meta(&self, id: u32) -> Option<String> {
        self.line_variation_summary(id)
    }

    pub fn line_for_ui_branch_hover(&self, id: u32) -> String {
        self.line_variation_hover_text(id)
    }

    pub fn line_for_ui_branch_selected(&self, id: Option<u32>) -> bool {
        self.active_variation_id == id
    }

    pub fn line_for_ui_branch_click(&mut self, id: Option<u32>) {
        self.line_switch_to(id);
    }

    pub fn line_for_ui_branch_count(&self) -> usize {
        self.variations.len()
    }

    pub fn line_for_ui_branch_tree_count(&self) -> usize {
        self.variation_tree().len()
    }

    pub fn line_for_ui_branch_root_count(&self) -> usize {
        self.line_root_branch_ids().len()
    }

    pub fn line_for_ui_branch_counts_text(&self) -> String {
        format!(
            "{} total, {} roots",
            self.line_for_ui_branch_count(),
            self.line_for_ui_branch_root_count()
        )
    }

    pub fn line_for_ui_branch_tree_text(&self) -> String {
        self.line_for_ui_tree_dump()
    }

    pub fn line_for_ui_branch_notes_text(&self) -> String {
        "Tip: click a line to switch instantly; cache keeps analysis hot.".to_string()
    }

    pub fn line_for_ui_branch_panel_header(&self) -> String {
        format!("Lines ({})", self.line_for_ui_branch_count())
    }

    pub fn line_for_ui_branch_panel_subheader(&self) -> String {
        self.line_for_ui_branch_counts_text()
    }

    pub fn line_for_ui_branch_panel_footer(&self) -> String {
        self.line_for_ui_branch_notes_text()
    }

    pub fn line_for_ui_branch_panel_state(&self) -> String {
        format!(
            "{} • {}",
            self.line_for_ui_branch_panel_subheader(),
            self.line_for_ui_quick_status()
        )
    }

    pub fn line_for_ui_branch_panel_debug(&self) -> String {
        format!(
            "{}\\n{}",
            self.line_for_ui_branch_tree_text(),
            self.line_for_ui_dump()
        )
    }

    pub fn line_for_ui_branch_panel_can_show(&self) -> bool {
        true
    }

    pub fn line_for_ui_branch_panel_rows(&self) -> Vec<(Option<u32>, usize, String)> {
        self.line_for_ui_rows()
    }

    pub fn line_for_ui_branch_panel_select(&mut self, id: Option<u32>) {
        self.line_for_ui_switch(id);
    }

    pub fn line_for_ui_branch_panel_create(&mut self) -> u32 {
        self.line_for_ui_create_branch_here()
    }

    pub fn line_for_ui_branch_panel_remove(&mut self, id: u32) {
        self.line_for_ui_remove_branch(id);
    }

    pub fn line_for_ui_branch_panel_rename(&mut self, id: u32, name: String) {
        self.line_for_ui_rename_branch(id, name);
    }

    pub fn line_for_ui_branch_panel_refresh(&mut self) {
        self.commit_active_variation_state();
    }

    pub fn line_for_ui_branch_panel_sync(&mut self) {
        self.line_for_ui_branch_panel_refresh();
    }

    pub fn line_for_ui_branch_panel_hint(&self) -> String {
        "Main line remains intact; variations form a tree.".to_string()
    }

    pub fn line_for_ui_branch_panel_perf_hint(&self) -> String {
        self.line_for_ui_perf_hint()
    }

    pub fn line_for_ui_branch_panel_eval_hint(&self) -> String {
        self.line_for_ui_eval_hint()
            .unwrap_or_else(|| "eval pending".to_string())
    }

    pub fn line_for_ui_branch_panel_all_hints(&self) -> Vec<String> {
        vec![
            self.line_for_ui_branch_panel_hint(),
            self.line_for_ui_branch_panel_perf_hint(),
            self.line_for_ui_branch_panel_eval_hint(),
        ]
    }

    pub fn line_for_ui_branch_panel_summary(&self) -> String {
        format!(
            "{} • {}",
            self.line_for_ui_branch_panel_header(),
            self.line_for_ui_branch_panel_subheader()
        )
    }

    pub fn line_for_ui_branch_panel_status(&self) -> String {
        format!(
            "{} • {}",
            self.line_for_ui_branch_panel_summary(),
            self.line_for_ui_quick_status()
        )
    }

    pub fn line_for_ui_branch_panel_help(&self) -> String {
        format!(
            "{} {}",
            self.line_for_ui_branch_panel_hint(),
            self.line_for_ui_branch_panel_footer()
        )
    }

    pub fn line_for_ui_branch_panel_dump(&self) -> String {
        format!(
            "{}\\n{}\\n{}",
            self.line_for_ui_branch_panel_status(),
            self.line_for_ui_branch_tree_text(),
            self.line_for_ui_dump()
        )
    }

    pub fn line_for_ui_branch_panel_short(&self) -> String {
        self.line_for_ui_branch_panel_status()
    }

    pub fn line_for_ui_branch_panel_long(&self) -> String {
        self.line_for_ui_branch_panel_dump()
    }

    pub fn line_for_ui_branch_panel_text(&self) -> String {
        self.line_for_ui_branch_panel_short()
    }

    pub fn line_for_ui_branch_panel_debug_text(&self) -> String {
        self.line_for_ui_branch_panel_long()
    }

    pub fn line_for_ui_branch_panel_current_text(&self) -> String {
        self.line_for_ui_current_progress()
    }

    pub fn line_for_ui_branch_panel_title(&self) -> String {
        self.line_for_ui_branch_panel_header()
    }

    pub fn line_for_ui_branch_panel_subtitle(&self) -> String {
        self.line_for_ui_branch_panel_subheader()
    }

    pub fn line_for_ui_branch_panel_state_text(&self) -> String {
        self.line_for_ui_branch_panel_state()
    }

    pub fn line_for_ui_branch_panel_footer_text(&self) -> String {
        self.line_for_ui_branch_panel_footer()
    }

    pub fn line_for_ui_branch_panel_hint_text(&self) -> String {
        self.line_for_ui_branch_panel_hint()
    }

    pub fn line_for_ui_branch_panel_eval_text(&self) -> String {
        self.line_for_ui_branch_panel_eval_hint()
    }

    pub fn line_for_ui_branch_panel_perf_text(&self) -> String {
        self.line_for_ui_branch_panel_perf_hint()
    }

    pub fn line_for_ui_branch_panel_help_text(&self) -> String {
        self.line_for_ui_branch_panel_help()
    }

    pub fn line_for_ui_branch_panel_status_text(&self) -> String {
        self.line_for_ui_branch_panel_status()
    }

    pub fn line_for_ui_branch_panel_dump_text(&self) -> String {
        self.line_for_ui_branch_panel_dump()
    }

    pub fn line_for_ui_branch_panel_rows_count(&self) -> usize {
        self.line_for_ui_branch_panel_rows().len()
    }

    pub fn line_for_ui_branch_panel_has_rows(&self) -> bool {
        self.line_for_ui_branch_panel_rows_count() > 0
    }

    pub fn line_for_ui_branch_panel_current_id(&self) -> Option<u32> {
        self.line_for_ui_current_variation_id()
    }

    pub fn line_for_ui_branch_panel_current_name(&self) -> String {
        self.line_for_ui_current_variation_name()
    }

    pub fn line_for_ui_branch_panel_current_kind(&self) -> &'static str {
        if self.line_for_ui_current_is_main() {
            "main"
        } else {
            "variation"
        }
    }

    pub fn line_for_ui_branch_panel_current_label(&self) -> String {
        format!(
            "{} {}",
            self.line_for_ui_branch_panel_current_kind(),
            self.line_for_ui_branch_panel_current_name()
        )
    }

    pub fn line_for_ui_branch_panel_current_summary(&self) -> String {
        format!(
            "{} • {}",
            self.line_for_ui_branch_panel_current_label(),
            self.line_for_ui_current_progress()
        )
    }

    pub fn line_for_ui_branch_panel_current_debug(&self) -> String {
        format!(
            "{} • {}",
            self.line_for_ui_branch_panel_current_summary(),
            self.line_for_ui_branch_context()
        )
    }

    pub fn line_for_ui_branch_panel_main_selectable(&self) -> bool {
        true
    }

    pub fn line_for_ui_branch_panel_variation_selectable(&self, id: u32) -> bool {
        self.line_for_ui_branch_exists(id)
    }

    pub fn line_for_ui_branch_panel_toggle(&mut self, id: Option<u32>) {
        self.line_for_ui_branch_panel_select(id);
    }

    pub fn line_for_ui_branch_panel_create_named(&mut self, name: String) -> u32 {
        self.line_for_ui_new_named_branch(name)
    }

    pub fn line_for_ui_new_named_branch(&mut self, name: String) -> u32 {
        self.line_new_named_branch_from_current(name)
    }

    pub fn line_for_ui_new_branch_default(&mut self) -> u32 {
        self.line_for_ui_branch_panel_create()
    }

    pub fn line_for_ui_branch_panel_remove_if_exists(&mut self, id: u32) {
        if self.line_for_ui_branch_exists(id) {
            self.line_for_ui_branch_panel_remove(id);
        }
    }

    pub fn line_for_ui_branch_panel_rename_if_exists(&mut self, id: u32, name: String) {
        if self.line_for_ui_branch_exists(id) {
            self.line_for_ui_branch_panel_rename(id, name);
        }
    }

    pub fn line_for_ui_branch_panel_current_is_main(&self) -> bool {
        self.line_for_ui_current_is_main()
    }

    pub fn line_for_ui_branch_panel_current_is_variation(&self) -> bool {
        !self.line_for_ui_current_is_main()
    }

    pub fn line_for_ui_branch_panel_current_branch_id(&self) -> Option<u32> {
        self.line_for_ui_current_variation_id()
    }

    pub fn line_for_ui_branch_panel_current_branch_name(&self) -> String {
        self.line_for_ui_current_variation_name()
    }

    pub fn line_for_ui_branch_panel_current_branch_depth(&self) -> usize {
        self.line_depth()
    }

    pub fn line_for_ui_branch_panel_current_branch_anchor(&self) -> usize {
        self.line_anchor()
    }

    pub fn line_for_ui_branch_panel_current_branch_meta(&self) -> String {
        self.line_meta_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_text(&self) -> String {
        self.line_title_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_subtext(&self) -> String {
        self.line_secondary_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_tooltip(&self) -> String {
        self.line_footer_hint()
    }

    pub fn line_for_ui_branch_panel_current_branch_progress(&self) -> String {
        self.line_progress_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_nav(&self) -> String {
        self.line_nav_state_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_analysis(&self) -> String {
        self.line_analysis_state_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_cache(&self) -> String {
        self.line_performance_hint()
    }

    pub fn line_for_ui_branch_panel_current_branch_eval(&self) -> String {
        self.line_info_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_status(&self) -> String {
        self.line_state_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_debug_text(&self) -> String {
        self.line_debug_all()
    }

    pub fn line_for_ui_branch_panel_current_branch_all(&self) -> Vec<String> {
        vec![
            self.line_for_ui_branch_panel_current_branch_text(),
            self.line_for_ui_branch_panel_current_branch_subtext(),
            self.line_for_ui_branch_panel_current_branch_meta(),
            self.line_for_ui_branch_panel_current_branch_progress(),
        ]
    }

    pub fn line_for_ui_branch_panel_current_branch_summary_block(&self) -> String {
        self.line_for_ui_branch_panel_current_branch_all()
            .join(" | ")
    }

    pub fn line_for_ui_branch_panel_current_branch_notes(&self) -> String {
        self.line_for_ui_notes_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_hints(&self) -> Vec<String> {
        self.line_for_ui_notes_lines()
    }

    pub fn line_for_ui_branch_panel_current_branch_hint_text(&self) -> String {
        self.line_for_ui_branch_panel_current_branch_hints()
            .join(" ")
    }

    pub fn line_for_ui_branch_panel_current_branch_full_text(&self) -> String {
        format!(
            "{}\\n{}\\n{}",
            self.line_for_ui_branch_panel_current_branch_summary_block(),
            self.line_for_ui_branch_panel_current_branch_status(),
            self.line_for_ui_branch_panel_current_branch_hint_text()
        )
    }

    pub fn line_for_ui_branch_panel_current_branch_short_text(&self) -> String {
        self.line_for_ui_branch_panel_current_branch_summary_block()
    }

    pub fn line_for_ui_branch_panel_current_branch_debug_dump(&self) -> String {
        format!(
            "{}\\n{}",
            self.line_for_ui_branch_panel_current_branch_full_text(),
            self.line_for_ui_dump()
        )
    }

    pub fn line_for_ui_branch_panel_current_branch_show_hint(&self) -> bool {
        true
    }

    pub fn line_for_ui_branch_panel_current_branch_show_debug(&self) -> bool {
        false
    }

    pub fn line_for_ui_branch_panel_current_branch_show_eval(&self) -> bool {
        self.line_eval_available()
    }

    pub fn line_for_ui_branch_panel_current_branch_show_cache(&self) -> bool {
        true
    }

    pub fn line_for_ui_branch_panel_current_branch_show_tree(&self) -> bool {
        self.line_has_any_variations()
    }

    pub fn line_for_ui_branch_panel_current_branch_show_mainline_suffix(&self) -> bool {
        self.line_has_mainline_suffix_visible()
    }

    pub fn line_for_ui_branch_panel_current_branch_show_divergence(&self) -> bool {
        self.line_is_diverged()
    }

    pub fn line_for_ui_branch_panel_current_branch_divergence_text(&self) -> String {
        self.line_divergence_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_suffix_text(&self) -> String {
        self.line_mainline_overlay_hint()
    }

    pub fn line_for_ui_branch_panel_current_branch_cache_text(&self) -> String {
        self.line_for_ui_cache_state_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_analysis_text(&self) -> String {
        self.line_for_ui_analysis_state_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_quick_text(&self) -> String {
        self.line_for_ui_quick_status()
    }

    pub fn line_for_ui_branch_panel_current_branch_perf_text(&self) -> String {
        self.line_for_ui_perf_hint()
    }

    pub fn line_for_ui_branch_panel_current_branch_eval_text2(&self) -> String {
        self.line_for_ui_eval_hint()
            .unwrap_or_else(|| "eval pending".to_string())
    }

    pub fn line_for_ui_branch_panel_current_branch_best_text(&self) -> String {
        self.line_for_ui_best_hint()
            .unwrap_or_else(|| "best pending".to_string())
    }

    pub fn line_for_ui_branch_panel_current_branch_key_text(&self) -> String {
        self.line_for_ui_current_key()
            .unwrap_or_else(|| "key ?".to_string())
    }

    pub fn line_for_ui_branch_panel_current_branch_perf_block(&self) -> String {
        format!(
            "{} • {} • {}",
            self.line_for_ui_branch_panel_current_branch_cache_text(),
            self.line_for_ui_branch_panel_current_branch_analysis_text(),
            self.line_for_ui_branch_panel_current_branch_perf_text()
        )
    }

    pub fn line_for_ui_branch_panel_current_branch_eval_block(&self) -> String {
        format!(
            "{} • {}",
            self.line_for_ui_branch_panel_current_branch_eval_text2(),
            self.line_for_ui_branch_panel_current_branch_best_text()
        )
    }

    pub fn line_for_ui_branch_panel_current_branch_key_block(&self) -> String {
        self.line_for_ui_branch_panel_current_branch_key_text()
    }

    pub fn line_for_ui_branch_panel_current_branch_info_block(&self) -> String {
        format!(
            "{}\\n{}\\n{}",
            self.line_for_ui_branch_panel_current_branch_perf_block(),
            self.line_for_ui_branch_panel_current_branch_eval_block(),
            self.line_for_ui_branch_panel_current_branch_key_block()
        )
    }

    pub fn line_for_ui_branch_panel_current_branch_overlay_block(&self) -> String {
        format!(
            "{} • {}",
            self.line_for_ui_branch_panel_current_branch_divergence_text(),
            self.line_for_ui_branch_panel_current_branch_suffix_text()
        )
    }

    pub fn line_for_ui_branch_panel_current_branch_complete_block(&self) -> String {
        format!(
            "{}\\n{}\\n{}",
            self.line_for_ui_branch_panel_current_branch_summary_block(),
            self.line_for_ui_branch_panel_current_branch_info_block(),
            self.line_for_ui_branch_panel_current_branch_overlay_block()
        )
    }

    pub fn line_for_ui_branch_panel_current_branch_final_text(&self) -> String {
        self.line_for_ui_branch_panel_current_branch_complete_block()
    }
}
