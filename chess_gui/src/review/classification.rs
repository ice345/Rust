//! Move classification recomputation for review timeline.

use super::*;

impl ReviewState {
    fn material_score(board: &Board, color: Color) -> i32 {
        let mut score = 0;
        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = board.squares[row][col]
                    && piece.color == color
                {
                    score += piece.piece_type.material_value() * 100;
                }
            }
        }
        score
    }

    fn total_non_king_material_cp(board: &Board) -> i32 {
        let mut total = 0;
        for row in 0..8 {
            for col in 0..8 {
                if let Some(piece) = board.squares[row][col]
                    && piece.piece_type != PieceType::King
                {
                    total += piece.piece_type.material_value() * 100;
                }
            }
        }
        total
    }

    /// Recomputes move classification for the move that leads to `index`.
    pub(super) fn recompute_classification_at(&mut self, index: usize) {
        if index == 0 {
            return;
        }
        let Some(prev) = self.analyses.get(index - 1).and_then(|a| a.as_ref()) else {
            return;
        };
        let Some(curr) = self.analyses.get(index).and_then(|a| a.as_ref()) else {
            return;
        };
        if prev.lines.is_empty() || curr.lines.is_empty() {
            return;
        }

        let move_idx = index - 1;
        let prev_eval = prev.lines[0].eval_cp;
        let curr_eval = curr.lines[0].eval_cp;

        let prev_is_white_to_move = (index - 1).is_multiple_of(2);
        let curr_is_white_to_move = index.is_multiple_of(2);

        let prev_white_pov = if prev_is_white_to_move {
            prev_eval
        } else {
            -prev_eval
        };
        let curr_white_pov = if curr_is_white_to_move {
            curr_eval
        } else {
            -curr_eval
        };

        let is_white_move = self
            .move_records
            .get(move_idx)
            .map(|r| r.color == Color::White)
            .unwrap_or(false);

        let mover_before = if is_white_move {
            prev_white_pov
        } else {
            -prev_white_pov
        };
        let mover_after = if is_white_move {
            curr_white_pov
        } else {
            -curr_white_pov
        };
        let top2_gap = if prev.lines.len() >= 2 {
            Some((prev.lines[0].eval_cp - prev.lines[1].eval_cp).max(0))
        } else {
            None
        };
        let played_uci = self
            .move_records
            .get(move_idx)
            .map(|r| Board::move_to_uci(&r.mv))
            .unwrap_or_default();
        let move_rank = prev
            .lines
            .iter()
            .position(|line| line.best_move_uci == played_uci);
        let played_eval_from_prev =
            move_rank.and_then(|rank| prev.lines.get(rank).map(|l| l.eval_cp));

        // Prefer same-node eval for the played move when available (top lines),
        // because it is directly comparable with the best line and avoids
        // cross-position depth drift causing false "blunder" labels.
        let effective_after = played_eval_from_prev.unwrap_or(mover_after);
        let effective_loss = (mover_before - effective_after).max(0);
        let effective_gain = effective_after - mover_before;

        let material_delta = if index < self.snapshots.len() {
            let before = &self.snapshots[index - 1];
            let after = &self.snapshots[index];
            let mover = if is_white_move {
                Color::White
            } else {
                Color::Black
            };
            Self::material_score(after, mover) - Self::material_score(before, mover)
        } else {
            0
        };
        let remaining_material_cp = if index < self.snapshots.len() {
            Self::total_non_king_material_cp(&self.snapshots[index - 1])
        } else {
            7800
        };

        if move_idx < self.classifications.len() {
            self.classifications[move_idx] = Some(MoveClassification::from_engine_context(
                effective_loss,
                effective_gain,
                move_rank,
                top2_gap,
                material_delta,
                mover_before,
                effective_after,
                remaining_material_cp,
            ));
        }
    }
}
