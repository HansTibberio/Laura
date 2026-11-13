/*
    Laura: A multi-threaded UCI chess engine written in Rust.

    Copyright (C) 2024-2026 HansTibberio <hanstiberio@proton.me>

    Laura is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Laura is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Laura. If not, see <https://www.gnu.org/licenses/>.
*/

// src/search.rs

//! Search implementation

use crate::config::{
    SE_BETA_SCALE, SE_DOUBLE_LIMIT, SE_DOUBLE_MARGIN, SE_MIN_DEPTH, SE_TT_DEPTH_MARGIN,
};
use crate::movepicker::{captured_type, moving_piece};
use crate::thread::{ContKeys, TriedMove, TriedMoves};
use crate::{
    config::{
        ASPIRATION_DEPTH_THRESHOLD, ASPIRATION_MARGIN, HIST_CLAMP, INFINITY, LMR_LOWER_DEPTH,
        LMR_TABLE, LMR_THRESHOLD, MATE, MAX_DELTA, MAX_MATE, MAX_PLY,
    },
    movepicker::MovePicker,
    position::Position,
    sse::SEE,
    thread::Thread,
    transposition::{BoundType, EntryHit, TranspositionTable},
};
use laura_core::{Board, Move, MoveType, Piece, PieceType};
use std::fmt;

pub type Score = i32;
pub type Evaluation = i32;

pub trait ThreadType {
    const MAIN: bool;
}
pub struct MainThread;
pub struct WorkerThread;

impl ThreadType for MainThread {
    const MAIN: bool = true;
}
impl ThreadType for WorkerThread {
    const MAIN: bool = false;
}

#[derive(Debug, Clone, Copy)]
pub struct PrincipalVariation {
    pub moves: [Move; MAX_PLY],
    pub len: usize,
}

impl fmt::Display for PrincipalVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.is_empty() {
            write!(f, "pv ")?;
        }

        for &mv in self.as_slice().iter().take(16) {
            write!(f, "{mv} ")?;
        }
        Ok(())
    }
}

impl Default for PrincipalVariation {
    fn default() -> Self {
        Self {
            moves: [Move::null(); MAX_PLY],
            len: 0,
        }
    }
}

impl PrincipalVariation {
    #[inline(always)]
    pub fn push(&mut self, mv: Move) {
        if self.len < MAX_PLY {
            self.moves[self.len] = mv;
            self.len += 1;
        }
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline(always)]
    pub fn push_line(&mut self, mv: Move, old: &PrincipalVariation) {
        self.clear();
        self.push(mv);
        self.len = old.len + 1;
        self.moves[1..=old.len].copy_from_slice(old.as_slice());
    }
}

impl Position {
    pub fn iterative_deepening<T>(&mut self, thread: &mut Thread, ttable: &TranspositionTable)
    where
        T: ThreadType,
    {
        let start_depth: usize = (thread.id & 0b111) + 1;
        let max_depth: usize = thread
            .time_manager
            .time_control()
            .depth()
            .unwrap_or(MAX_PLY);

        // Main Iterative Deepening Loop
        for depth in start_depth..=max_depth {
            if thread.depth > MAX_PLY || thread.time_manager.stop_soft() {
                break;
            }

            let score: Score = self.aspiration_window(thread, ttable, depth);

            if thread.time_manager.stopped() {
                break;
            }

            thread.score = score;
            thread.depth += 1;

            if T::MAIN {
                uci_printer(thread, ttable);
            }
        }

        if T::MAIN && thread.time_manager.stopped() {
            uci_printer(thread, ttable);
        }
    }

    fn aspiration_window(
        &mut self,
        thread: &mut Thread,
        ttable: &TranspositionTable,
        depth: usize,
    ) -> Score {
        let mut root_pv: PrincipalVariation = PrincipalVariation::default();
        let mut delta: i32 = ASPIRATION_MARGIN;

        let (mut alpha, mut beta) = if depth >= ASPIRATION_DEPTH_THRESHOLD {
            (
                (-INFINITY).max(thread.score - delta),
                INFINITY.min(thread.score + delta),
            )
        } else {
            (-INFINITY, INFINITY)
        };

        loop {
            let score: Score = self.alphabeta::<true>(
                thread,
                ttable,
                depth,
                alpha,
                beta,
                &mut root_pv,
                true,
                false,
            );

            if thread.time_manager.stopped() {
                return -INFINITY;
            }

            match score {
                s if s <= alpha => {
                    // Fail-low, expand window down
                    beta = (alpha + beta) / 2; // Midpoint collapse of beta
                    alpha = (-INFINITY).max(alpha - delta);
                }
                s if s >= beta => {
                    // Fail-high, expand window up
                    beta = (INFINITY).min(beta + delta);
                }
                _ => {
                    // Successful
                    thread.principal_variation = root_pv;
                    thread.completed = depth;
                    return score;
                }
            }

            delta += delta / 2;
            if delta >= MAX_DELTA {
                alpha = -INFINITY;
                beta = INFINITY;
            }
        }
    }

    // Alpha-Beta with Fail-Soft
    #[allow(non_upper_case_globals, clippy::too_many_arguments)]
    fn alphabeta<const RootNode: bool>(
        &mut self,
        thread: &mut Thread,
        ttable: &TranspositionTable,
        mut depth: usize,
        mut alpha: i32,
        mut beta: i32,
        node_pv: &mut PrincipalVariation,
        do_null: bool,
        cut_node: bool,
    ) -> Score {
        let mut temp_pv: PrincipalVariation = PrincipalVariation::default();
        let child_pv: &mut PrincipalVariation = &mut temp_pv;
        node_pv.clear();

        let is_pv: bool = beta - alpha > 1;

        // Hard Limit Time Control
        if thread.time_manager.stop_hard(thread.nodes) {
            return 0;
        }

        // Update thread selective depth
        thread.seldepth = if RootNode {
            0
        } else {
            thread.seldepth.max(thread.ply)
        };

        // 1. Check Extension
        // Extends the search depth when the side to move is in check
        let in_check: bool = self.in_check();
        if in_check && depth < MAX_PLY {
            depth += 1;
        }

        // 2. Quiescence Search
        // Switches to quiescence search at leaf nodes to avoid evaluating
        // tactically unstable positions caused by captures or checks.
        if depth == 0 || thread.ply >= MAX_PLY {
            return self.quiescence(thread, ttable, alpha, beta, child_pv);
        }

        // Limit the depth
        depth = depth.min(MAX_PLY - 1);

        // Singular verification search: same node, with the TT move excluded.
        let excluded: Option<Move> = thread.stack[thread.ply].excluded;

        // Double extensions accumulated along the current path.
        let double_exts: u8 = if RootNode {
            0
        } else {
            thread.stack[thread.ply - 1].double_exts
        };
        thread.stack[thread.ply].double_exts = double_exts;

        if !RootNode {
            // 3. Mate Distance Pruning.
            // Narrows the search window based on the maximum achievable mate
            // distance from the current ply, avoiding redundant mate searches.
            alpha = alpha.max(mated_in(thread.ply));
            beta = beta.min(mate_in(thread.ply + 1));

            if alpha >= beta {
                return alpha;
            }

            // Stop searching if the position is a rule-based draw
            if self.is_draw() {
                return 0;
            }
        }

        // 4. Probe the Transposition Table
        // The entry (score, bound, static eval) is usable even when it carries no move:
        // fail-low (ALL) nodes are stored with a null move. Move legality only decides
        // whether the move may be used for ordering, never whether the entry is valid.
        let tt_entry: Option<EntryHit> = if excluded.is_some() {
            None
        } else {
            ttable
                .probe(self.key(), thread.ply)
                .filter(|entry| entry.bound != BoundType::None)
        };
        let mut tt_move: Option<Move> = None;

        if let Some(entry) = tt_entry {
            if entry.is_legal(&self.board()) {
                tt_move = Some(entry.mv);
            }

            if !is_pv && entry.depth >= depth && tt_cutoff(&entry, alpha, beta) {
                return entry.score;
            }
        }

        let static_eval: Evaluation = if in_check {
            thread.stack[thread.ply].static_eval = -INFINITY;
            -INFINITY
        } else if let Some(entry) = tt_entry {
            let tt_score: Score = entry.score;
            let tt_evaluation: Evaluation = entry.evaluation;

            thread.stack[thread.ply].static_eval = if tt_evaluation == -INFINITY {
                self.evaluate()
            } else {
                tt_evaluation
            };

            match entry.bound {
                BoundType::Exact => tt_score,
                BoundType::LowerBound if tt_score > thread.stack[thread.ply].static_eval => {
                    tt_score
                }
                BoundType::UpperBound if tt_score < thread.stack[thread.ply].static_eval => {
                    tt_score
                }
                _ => thread.stack[thread.ply].static_eval,
            }
        } else {
            thread.stack[thread.ply].static_eval = self.evaluate();
            thread.stack[thread.ply].static_eval
        };

        // 5. Improving
        // Compares the current raw evaluation with the evaluation two or four
        // plies earlier to determine whether the position is improving.
        let raw_eval: Evaluation = thread.stack[thread.ply].static_eval;
        let improving: bool = if in_check {
            false
        } else if thread.ply >= 2 && thread.stack[thread.ply - 2].static_eval != -INFINITY {
            raw_eval > thread.stack[thread.ply - 2].static_eval
        } else if thread.ply >= 4 && thread.stack[thread.ply - 4].static_eval != -INFINITY {
            raw_eval > thread.stack[thread.ply - 4].static_eval
        } else {
            true
        };

        // 6. Forward static pruning techniques
        if !in_check && !is_pv && excluded.is_none() {
            // 6.1. Reverse Futility Pruning
            // Skips shallow nodes when the static evaluation already exceeds beta
            // by a safe margin, assuming the position is unlikely to fail low.
            let rfp_margin: i32 = 80 * (depth as i32) - 55 * (improving as i32);
            if depth <= 8 && static_eval >= beta + rfp_margin {
                return static_eval;
            }

            // 6.2. Razoring
            // Uses quiescence search to verify whether a shallow position
            // is too far below alpha to justify a full-depth search.
            let razor_margin: i32 = 300 * depth as i32;
            if depth <= 2 && (static_eval + razor_margin) <= alpha {
                let razor_score: Score = self.quiescence(thread, ttable, alpha, beta, child_pv);
                if razor_score <= alpha {
                    return razor_score;
                }
            }

            // 6.3. Null Move Pruning
            // Performs a reduced-depth null move search to detect positions
            // where the side to move can still exceed beta after passing.
            if depth > 3
                && !self.possible_zugzwang()
                && do_null
                && static_eval + 70 * (improving as i32) >= beta
            {
                let r: usize = (4 + depth / 4).min(depth);
                self.push_null(thread);
                let null_score: Score = -self.alphabeta::<false>(
                    thread,
                    ttable,
                    depth - r,
                    -beta,
                    -beta + 1,
                    child_pv,
                    false,
                    false,
                );
                self.pop_move(thread);

                if null_score >= beta {
                    return null_score;
                }
            }
        }

        // 7. Internal Iterative Reduction
        // Reduces the search depth in positions without a transposition table hit,
        // trading depth for faster move ordering and lower search overhead.
        if !RootNode && depth >= 4 && tt_move.is_none() && !in_check && excluded.is_none() {
            depth -= 1;
        }

        let alpha_orig: i32 = alpha;
        let mut best_move: Move = Move::default();
        let mut best_score: i32 = -INFINITY;
        let mut move_count: usize = 0;

        let cont_keys: ContKeys = thread.cont_keys();
        let mut picker: MovePicker = MovePicker::new(tt_move, thread, cont_keys);

        let mut quiets_tried: TriedMoves<64> = TriedMoves::new();
        let mut caps_tried: TriedMoves<32> = TriedMoves::new();

        // 8. Main Alpha-Beta Loop
        // Iterates over ordered legal moves, searches each candidate and updates
        // the best score and alpha until a beta cutoff or move exhaustion.
        while let Some(mv) = picker.next(&self.board(), thread) {
            if excluded == Some(mv) {
                continue;
            }

            move_count += 1;
            let mut score: Score;
            let board: Board = self.board();

            let is_quiet: bool = !mv.is_capture() && !mv.is_promotion();
            let is_tactical: bool = !is_quiet && !mv.is_underpromotion();
            let piece: Piece = moving_piece(&board, mv);
            let quiet_score: i32 = if is_quiet {
                thread.quiet_score(board.side(), piece, mv, &cont_keys)
            } else {
                0
            };
            let tried: TriedMove = TriedMove {
                mv,
                piece,
                captured: if is_tactical {
                    captured_type(&board, mv)
                } else {
                    PieceType::King
                },
            };

            // 9. Quiet moves pruning
            // Applies selective pruning heuristics to late or low-value quiet moves
            // in order to reduce the search tree in non-tactical positions.
            if !RootNode && !is_pv && !in_check && is_quiet {
                // 9.1. History Leaf Pruning
                // Skips quiet moves with consistently poor history scores at shallow depth,
                // assuming they are unlikely to improve alpha.
                if depth < 6 && quiet_score < -5000 {
                    picker.skip_quiets = true;
                }

                // 9.2. Futility Pruning
                // Prunes quiet moves when the static evaluation plus a depth-based margin
                // is still insufficient to raise alpha.
                let futility_margin: i32 = 80 * depth as i32;
                if static_eval + futility_margin < alpha && depth <= 6 {
                    picker.skip_quiets = true;
                }

                // 9.3. Late Move Pruning
                // Skips late quiet moves at shallow depth after enough moves
                // have already been searched.
                if depth < 4 && move_count >= 9 {
                    picker.skip_quiets = true;
                }

                if picker.skip_quiets && is_quiet && best_score > -MAX_MATE {
                    continue;
                }
            }

            // SEE pruning techniques
            if !RootNode && !is_pv && !in_check && move_count > 1 {
                let see_margin: i32 = if is_quiet {
                    -65 * depth as i32
                } else {
                    -20 * (depth * depth) as i32
                };
                if best_score > -MAX_MATE && depth <= 9 && !SEE::see(&self.board(), mv, see_margin)
                {
                    continue;
                }
            }

            // 10. Singular Extensions
            // If the TT move is the only one that holds the score (every other move
            // fails low against s_beta in a reduced search), it is extended.
            let mut extension: i32 = 0;
            if !RootNode
                && excluded.is_none()
                && depth >= SE_MIN_DEPTH
                && tt_move == Some(mv)
                && let Some(entry) = tt_entry
                && entry.depth + SE_TT_DEPTH_MARGIN >= depth
                && entry.bound != BoundType::UpperBound
                && entry.score.abs() < MAX_MATE
            {
                let s_beta: i32 = entry.score - SE_BETA_SCALE * depth as i32;
                let s_depth: usize = (depth - 1) / 2;
                let mut s_pv: PrincipalVariation = PrincipalVariation::default();

                thread.stack[thread.ply].excluded = Some(mv);
                let s_score: Score = self.alphabeta::<false>(
                    thread,
                    ttable,
                    s_depth,
                    s_beta - 1,
                    s_beta,
                    &mut s_pv,
                    true,
                    cut_node,
                );
                thread.stack[thread.ply].excluded = None;

                if thread.time_manager.stopped() {
                    return 0;
                }

                if s_score < s_beta {
                    // 10.1. Singular (double extension if clearly singular)
                    extension = if !is_pv
                        && s_score < s_beta - SE_DOUBLE_MARGIN
                        && double_exts < SE_DOUBLE_LIMIT
                    {
                        2
                    } else {
                        1
                    };
                } else if !is_pv && s_beta >= beta {
                    // 10.2. Multi-cut: another move also beats beta
                    return s_beta;
                } else if entry.score >= beta || cut_node {
                    // 10.3. Negative extension: the TT move is likely not the only good move
                    extension = -1;
                }
            }

            let new_depth: usize = (depth as i32 - 1 + extension) as usize;
            thread.stack[thread.ply].double_exts = double_exts + (extension == 2) as u8;

            self.push_move(mv, thread);
            ttable.prefetch(self.key());

            // 11. Principal Variation Search
            if move_count == 1 {
                // 11.1. First move: Full Window Search
                score = -self.alphabeta::<false>(
                    thread, ttable, new_depth, -beta, -alpha, child_pv, true, !cut_node,
                )
            } else {
                // 11.2. Later moves: Null Window Search + Late Moves Reduction
                let reduction: usize =
                    if move_count >= LMR_THRESHOLD && depth >= LMR_LOWER_DEPTH && !in_check {
                        if is_quiet {
                            let mut r: i32 = lmr_reduction(depth, move_count) as i32;
                            let gives_check: bool = self.in_check();

                            r += !is_pv as i32;
                            r += cut_node as i32;
                            r += !improving as i32;
                            r -= in_check as i32;
                            r -= gives_check as i32;

                            if quiet_score > HIST_CLAMP / 2 {
                                r -= 1;
                            } else if quiet_score < -HIST_CLAMP / 2 {
                                r += 1;
                            }

                            r.clamp(1, (depth - 1) as i32) as usize
                        } else {
                            1
                        }
                    } else {
                        0
                    };

                // 11.3. Reduced depth + Null Window Search
                score = -self.alphabeta::<false>(
                    thread,
                    ttable,
                    new_depth.saturating_sub(reduction),
                    -alpha - 1,
                    -alpha,
                    child_pv,
                    true,
                    true,
                );

                // 11.4. If it fails high, and it has been reduced, let's re-search with Full depth & Null Window
                if score > alpha && reduction > 0 {
                    score = -self.alphabeta::<false>(
                        thread,
                        ttable,
                        new_depth,
                        -alpha - 1,
                        -alpha,
                        child_pv,
                        true,
                        true,
                    );
                }

                // 11.5. If it fails high, and we are in a PV node, re-search with Full Window
                if score > alpha && is_pv {
                    score = -self.alphabeta::<false>(
                        thread, ttable, new_depth, -beta, -alpha, child_pv, true, false,
                    );
                }
            }
            self.pop_move(thread);

            if thread.time_manager.stopped() {
                return 0;
            }

            // Best move and alpha update
            if score > best_score {
                best_score = score;

                if score > alpha {
                    alpha = score;
                    best_move = mv;
                    if is_pv {
                        node_pv.push_line(best_move, child_pv);
                    }
                }

                // Beta Pruning
                if score >= beta {
                    // Underpromotions are left out of every history table.
                    if is_quiet || is_tactical {
                        thread.update_histories(
                            board.side(),
                            tried,
                            is_quiet,
                            quiets_tried.as_slice(),
                            caps_tried.as_slice(),
                            depth,
                            &cont_keys,
                        );
                    }
                    break;
                }
            }

            // Tracks searched moves that did not produce a cutoff
            if is_quiet {
                quiets_tried.push(tried);
            } else if is_tactical {
                caps_tried.push(tried);
            }
        }

        if move_count == 0 {
            // Only the excluded move was legal: it is trivially singular.
            if excluded.is_some() {
                return alpha;
            }
            if in_check {
                // We are being mated
                return mated_in(thread.ply);
            }
            // Stalemate
            return 0;
        }

        // A singular search result is only valid without the excluded move:
        // it must never overwrite the real entry of this position.
        if !thread.time_manager.stopped() && excluded.is_none() {
            let bound: BoundType = if best_score >= beta {
                BoundType::LowerBound
            } else if best_score > alpha_orig {
                BoundType::Exact
            } else {
                BoundType::UpperBound
            };

            ttable.insert(
                self.key(),
                best_move,
                best_score,
                thread.stack[thread.ply].static_eval,
                depth,
                bound,
                is_pv,
                thread.ply,
            );
        }

        best_score
    }

    fn quiescence(
        &mut self,
        thread: &mut Thread,
        ttable: &TranspositionTable,
        mut alpha: i32,
        beta: i32,
        node_pv: &mut PrincipalVariation,
    ) -> Score {
        let mut temp_pv: PrincipalVariation = PrincipalVariation::default();
        let child_pv: &mut PrincipalVariation = &mut temp_pv;
        node_pv.clear();

        let is_pv: bool = beta - alpha > 1;

        // Hard Limit Time Control
        if thread.time_manager.stop_hard(thread.nodes) {
            return 0;
        }

        let in_check: bool = self.in_check();
        if thread.ply >= MAX_PLY {
            return if in_check { 0 } else { self.evaluate() };
        }

        // Stop searching if the position is a rule-based draw
        if self.is_draw() {
            return 0;
        }

        // Update thread selective depth
        if thread.ply > thread.seldepth {
            thread.seldepth = thread.ply;
        }

        // Probe the Transposition Table
        let tt_entry: Option<EntryHit> = ttable
            .probe(self.key(), thread.ply)
            .filter(|entry| entry.bound != BoundType::None);
        let mut tt_move: Option<Move> = None;

        if let Some(entry) = tt_entry {
            // Outside of check, quiescence only searches tactical moves:
            // a quiet TT move coming from the main search must not be played here.
            if entry.is_legal(&self.board())
                && (in_check || entry.mv.is_capture() || entry.mv.is_promotion())
            {
                tt_move = Some(entry.mv);
            }

            // Any stored depth (>= 0) is enough for quiescence
            if !is_pv && tt_cutoff(&entry, alpha, beta) {
                return entry.score;
            }
        }

        let alpha_orig: i32 = alpha;
        let mut picker: MovePicker = MovePicker::new_qsearch(tt_move, thread.cont_keys());
        let stand_pat: Evaluation;

        if in_check {
            thread.stack[thread.ply].static_eval = -INFINITY;
            stand_pat = thread.stack[thread.ply].static_eval
        } else {
            if let Some(entry) = tt_entry {
                let tt_score: Score = entry.score;
                let tt_evaluation: Evaluation = entry.evaluation;

                thread.stack[thread.ply].static_eval = if tt_evaluation == -INFINITY {
                    self.evaluate()
                } else {
                    tt_evaluation
                };

                match entry.bound {
                    BoundType::Exact => {
                        stand_pat = tt_score;
                    }
                    BoundType::LowerBound if tt_score > thread.stack[thread.ply].static_eval => {
                        stand_pat = tt_score
                    }
                    BoundType::UpperBound if tt_score < thread.stack[thread.ply].static_eval => {
                        stand_pat = tt_score
                    }
                    _ => stand_pat = thread.stack[thread.ply].static_eval,
                }
            } else {
                thread.stack[thread.ply].static_eval = self.evaluate();
                stand_pat = thread.stack[thread.ply].static_eval
            }

            // Standing Pat Pruning
            // Fail-soft beta cutoff
            if stand_pat >= beta {
                return stand_pat;
            }

            // Improve alpha
            if stand_pat > alpha {
                alpha = stand_pat;
            }
            picker.skip_quiets = true;
        }

        let mut best_score: Score = stand_pat;
        let mut best_move: Move = Move::default();
        let mut move_count: usize = 0;

        // Main Quiescence Loop
        while let Some(mv) = picker.next(&self.board(), thread) {
            move_count += 1;

            if !in_check && mv.is_capture() && !mv.is_promotion() {
                let captured = SEE::piece_value(if mv.get_type() == MoveType::EnPassant {
                    PieceType::Pawn
                } else {
                    unsafe {
                        self.board()
                            .piece_on(mv.get_dest())
                            .unwrap_unchecked()
                            .piece_type()
                    }
                });

                // Quiescence Delta Pruning
                if !is_pv && (best_score + captured + 200) < alpha {
                    continue;
                }

                // Quiescence SEE Pruning
                if move_count > 1 && !SEE::see(&self.board(), mv, 0) {
                    continue;
                }
            }

            self.push_move(mv, thread);
            ttable.prefetch(self.key());

            let score: Score = -self.quiescence(thread, ttable, -beta, -alpha, child_pv);
            self.pop_move(thread);

            if thread.time_manager.stopped() {
                return 0;
            }

            // Best move and alpha update
            if score > best_score {
                best_score = score;

                if score > alpha {
                    alpha = score;
                    best_move = mv;
                    if is_pv {
                        node_pv.push_line(best_move, child_pv);
                    }
                }

                // Beta Pruning
                if alpha >= beta {
                    break;
                }
            }
        }

        if move_count == 0 && in_check {
            // We are being mated
            return mated_in(thread.ply);
        }

        if !thread.time_manager.stopped() {
            let bound: BoundType = if best_score >= beta {
                BoundType::LowerBound
            } else if best_score > alpha_orig {
                BoundType::Exact
            } else {
                BoundType::UpperBound
            };

            ttable.insert(
                self.key(),
                best_move,
                best_score,
                thread.stack[thread.ply].static_eval,
                0,
                bound,
                is_pv,
                thread.ply,
            );
        }

        best_score
    }
}

// Returns true if a TT entry's bound proves the score is outside (or equal to) the window.
#[inline(always)]
fn tt_cutoff(entry: &EntryHit, alpha: i32, beta: i32) -> bool {
    match entry.bound {
        BoundType::Exact => true,
        BoundType::LowerBound => entry.score >= beta,
        BoundType::UpperBound => entry.score <= alpha,
        BoundType::None => false,
    }
}

#[inline(always)]
fn mate_in(ply: usize) -> i32 {
    MATE - ply as i32
}

#[inline(always)]
fn mated_in(ply: usize) -> i32 {
    -MATE + ply as i32
}

#[inline(always)]
fn uci_printer(thread: &mut Thread, ttable: &TranspositionTable) {
    let score: String = if thread.score.abs() >= MAX_MATE {
        let mate_in: i32 = (MATE - thread.score.abs() + 1) / 2;

        if thread.score > 0 {
            format!("mate {}", mate_in)
        } else {
            format!("mate -{}", mate_in)
        }
    } else {
        format!("cp {}", thread.score)
    };

    let time: u128 = thread.time_manager.elapsed().as_millis().max(1);
    let nodes: u64 = thread.time_manager.nodes();
    let nps: u128 = (nodes as u128 * 1000) / time;
    println!(
        "info depth {} seldepth {} score {} time {} nodes {} nps {} hashfull {} {}",
        thread.depth,
        thread.seldepth,
        score,
        time,
        nodes,
        nps,
        ttable.hash_full(),
        thread.principal_variation
    );
}

#[inline(always)]
pub fn lmr_reduction(depth: usize, move_count: usize) -> usize {
    LMR_TABLE[depth.min(63)][move_count.min(63)] as usize
}
