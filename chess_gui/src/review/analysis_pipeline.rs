//! Analysis scheduling and multi-pass aggregation for review mode.

use super::*;

impl ReviewState {
    /// Resets analysis pipeline state and starts a new review pass.
    pub fn start_analysis(&mut self) {
        self.analyzing = true;
        self.analysis_index = 0;
        self.pending_analysis_index = None;
        self.pending_analysis_depth_idx = 0;
        self.analysis_samples.clear();
    }

    /// Returns next analysis request as `(snapshot_index, fen, depth)`.
    ///
    /// Cached positions are consumed directly without external requests.
    pub fn next_analysis_request(&mut self) -> Option<(usize, String, u32)> {
        if !self.analyzing || self.pending_analysis_index.is_some() {
            return None;
        }

        let depth = self.analysis_depth_plan.last().copied().unwrap_or(18);
        while self.analysis_index < self.snapshots.len() {
            let idx = self.analysis_index;
            if self.analyses.get(idx).and_then(|a| a.as_ref()).is_some() {
                self.analysis_index += 1;
                continue;
            }

            if let Some(cached) = self
                .position_keys
                .get(idx)
                .and_then(|key| self.analysis_cache.get(key))
                .cloned()
            {
                self.set_analysis(idx, cached);
                continue;
            }

            let board = &self.snapshots[idx];
            let color = Self::color_for_snapshot(idx);
            self.pending_analysis_index = Some(idx);
            self.pending_analysis_depth_idx = 0;
            return Some((
                idx,
                board.to_fen(color),
                self.analysis_depth_plan.first().copied().unwrap_or(depth),
            ));
        }

        self.analyzing = false;
        None
    }

    /// Backward-compatible helper that returns `(index, fen)` only.
    pub fn next_analysis_fen(&mut self) -> Option<(usize, String)> {
        self.next_analysis_request().map(|(idx, fen, _)| (idx, fen))
    }

    /// Ingests one analysis sample and returns an optional follow-up request.
    pub fn ingest_analysis_result(&mut self, analysis: PositionAnalysis) -> Option<(String, u32)> {
        let idx = self.pending_analysis_index?;
        let samples = self.analysis_samples.entry(idx).or_default();
        samples.push(analysis);

        let next_depth_idx = self.pending_analysis_depth_idx + 1;
        if next_depth_idx < self.analysis_depth_plan.len() {
            self.pending_analysis_depth_idx = next_depth_idx;
            let board = &self.snapshots[idx];
            let color = Self::color_for_snapshot(idx);
            return Some((
                board.to_fen(color),
                self.analysis_depth_plan[next_depth_idx],
            ));
        }

        let samples = self.analysis_samples.remove(&idx).unwrap_or_default();
        let verified = Self::aggregate_verified_analysis(samples);
        self.set_analysis(idx, verified);
        None
    }

    /// Commits finalized analysis to index and updates cache/classification.
    pub fn set_analysis(&mut self, index: usize, analysis: PositionAnalysis) {
        if let Some(key) = self.position_keys.get(index) {
            self.analysis_cache.insert(key.clone(), analysis.clone());
        }

        if index < self.analyses.len() {
            self.analyses[index] = Some(analysis.clone());
        }

        self.recompute_classification_at(index);
        if index + 1 < self.analyses.len() {
            self.recompute_classification_at(index + 1);
        }

        self.analysis_index = index + 1;
        self.pending_analysis_index = None;
        self.pending_analysis_depth_idx = 0;
        self.analysis_samples.remove(&index);
        if self.analysis_index >= self.snapshots.len() {
            self.analyzing = false;
        }
    }

    /// Returns normalized progress in `[0.0, 1.0]`.
    pub fn analysis_progress(&self) -> f32 {
        if self.snapshots.is_empty() {
            return 1.0;
        }
        let position_progress = self.analysis_index as f32;
        let depth_progress = if self.pending_analysis_index.is_some() {
            let denom = self.analysis_depth_plan.len().max(1) as f32;
            self.pending_analysis_depth_idx as f32 / denom
        } else {
            0.0
        };
        (position_progress + depth_progress) / self.snapshots.len() as f32
    }

    /// Aggregates multi-depth samples into one stable analysis result.
    fn aggregate_verified_analysis(samples: Vec<PositionAnalysis>) -> PositionAnalysis {
        if samples.is_empty() {
            return PositionAnalysis {
                lines: Vec::new(),
                depth: 0,
                verification_passes: 0,
                stability_pct: 0,
            };
        }

        let max_depth = samples.iter().map(|s| s.depth).max().unwrap_or(0);
        let mut vote_points: HashMap<String, f32> = HashMap::new();
        let mut eval_weighted_sum: HashMap<String, f32> = HashMap::new();
        let mut eval_weight_sum: HashMap<String, f32> = HashMap::new();
        let mut best_hits: HashMap<String, u32> = HashMap::new();

        for sample in &samples {
            let depth_weight = 1.0 + sample.depth as f32 / 12.0;
            for (rank, line) in sample.lines.iter().take(3).enumerate() {
                let rank_points = match rank {
                    0 => 3.0,
                    1 => 2.0,
                    _ => 1.0,
                };
                let weight = depth_weight * rank_points;
                *vote_points.entry(line.best_move_uci.clone()).or_insert(0.0) += weight;
                *eval_weighted_sum
                    .entry(line.best_move_uci.clone())
                    .or_insert(0.0) += line.eval_cp as f32 * weight;
                *eval_weight_sum
                    .entry(line.best_move_uci.clone())
                    .or_insert(0.0) += weight;

                if rank == 0 {
                    *best_hits.entry(line.best_move_uci.clone()).or_insert(0) += 1;
                }
            }
        }

        let mut ranked = vote_points
            .iter()
            .map(|(mv, points)| {
                let avg_eval = if let Some(sum_w) = eval_weight_sum.get(mv) {
                    let sum = eval_weighted_sum.get(mv).copied().unwrap_or(0.0);
                    if *sum_w > 0.0 {
                        (sum / *sum_w).round() as i32
                    } else {
                        0
                    }
                } else {
                    0
                };
                let hits = *best_hits.get(mv).unwrap_or(&0);
                (mv.clone(), *points, hits, avg_eval)
            })
            .collect::<Vec<_>>();

        ranked.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.2.cmp(&a.2))
                .then_with(|| b.3.cmp(&a.3))
                .then_with(|| a.0.cmp(&b.0))
        });

        let lines = ranked
            .iter()
            .take(3)
            .map(|(mv, _, _, eval)| AnalysisLine {
                eval_cp: *eval,
                best_move_uci: mv.clone(),
            })
            .collect::<Vec<_>>();

        let top_move = ranked.first().map(|v| v.0.clone()).unwrap_or_default();
        let best_move_consensus = samples
            .iter()
            .filter(|s| {
                s.lines
                    .first()
                    .map(|l| l.best_move_uci == top_move)
                    .unwrap_or(false)
            })
            .count();
        let stability_pct = ((best_move_consensus * 100) / samples.len()) as u8;

        PositionAnalysis {
            lines,
            depth: max_depth,
            verification_passes: samples.len().min(u8::MAX as usize) as u8,
            stability_pct,
        }
    }
}
