/*
    Laura: A single-threaded UCI chess engine written in Rust.

    Copyright (C) 2024-2025 HansTibberio <hanstiberio@proton.me>

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

use crate::{
    config::{
        ASPIRATION_DEPTH_THRESHOLD, ASPIRATION_MARGIN, INFINITY, MATE, MAX_DELTA, MAX_MATE, MAX_PLY,
    },
    movepicker::MovePicker,
    position::Position,
    thread::Thread,
    transposition::{BoundType, EntryHit, TranspositionTable},
};
use laura_core::Move;
use std::fmt;

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

trait NodeType {
    const PV_NODE: bool;
    const ROOT_NODE: bool;
}

struct RootNode;
struct PvNode;
struct NonPv;

impl NodeType for RootNode {
    const PV_NODE: bool = true;
    const ROOT_NODE: bool = true;
}

impl NodeType for PvNode {
    const PV_NODE: bool = true;
    const ROOT_NODE: bool = false;
}

impl NodeType for NonPv {
    const PV_NODE: bool = false;
    const ROOT_NODE: bool = false;
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

        for &mv in self.as_slice() {
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

            let score: i32 = self.aspiration_window(thread, ttable, depth);

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
    ) -> i32 {
        let mut root_pv: PrincipalVariation = PrincipalVariation::default();
        let mut delta: i32 = ASPIRATION_MARGIN;

        let (mut alpha, mut beta) = if depth >= ASPIRATION_DEPTH_THRESHOLD {
            (
                (-INFINITY).max(thread.score - delta),
                (INFINITY).min(thread.score + delta),
            )
        } else {
            (-INFINITY, INFINITY)
        };

        loop {
            let score: i32 =
                self.alphabeta::<RootNode>(thread, ttable, depth, alpha, beta, &mut root_pv);

            if thread.time_manager.stopped() {
                return -INFINITY;
            }

            match score {
                s if s <= alpha => {
                    // Fail-low, expand window down
                    alpha -= delta;
                }
                s if s >= beta => {
                    // Fail-high, expand window up
                    beta += delta;
                }
                _ => {
                    // Succesful
                    thread.principal_variation = root_pv;
                    thread.completed = depth;
                    return score;
                }
            }

            delta *= 2;
            if delta >= MAX_DELTA {
                alpha = -INFINITY;
                beta = INFINITY;
            }
        }
    }

    // Alpha-Beta with Fail-Soft
    #[allow(unused_assignments)]
    fn alphabeta<Node: NodeType>(
        &mut self,
        thread: &mut Thread,
        ttable: &TranspositionTable,
        mut depth: usize,
        mut alpha: i32,
        mut beta: i32,
        node_pv: &mut PrincipalVariation,
    ) -> i32 {
        let mut temp_pv: PrincipalVariation = PrincipalVariation::default();
        let child_pv: &mut PrincipalVariation = &mut temp_pv;

        // Hard Limit Time Control
        if thread.time_manager.stop_hard(thread.nodes) {
            return 0;
        }

        // Update thread selective depth
        thread.seldepth = if Node::ROOT_NODE {
            0
        } else {
            thread.seldepth.max(thread.ply)
        };

        // 1. Check Extension
        let in_check: bool = self.in_check();
        if in_check && depth < MAX_PLY {
            depth += 1;
        }

        // 2. Quiescence Search
        if depth == 0 || thread.ply >= MAX_PLY {
            if Node::PV_NODE {
                return self.quiescence::<PvNode>(thread, ttable, alpha, beta, node_pv);
            } else {
                return self.quiescence::<NonPv>(thread, ttable, alpha, beta, node_pv);
            }
        }

        node_pv.len = 0;

        // Limit the depth
        depth = depth.min(MAX_PLY - 1);

        if !Node::ROOT_NODE {
            // 3. Mate Distance Pruning.
            alpha = alpha.max(mated_in(thread.ply));
            beta = beta.min(mate_in(thread.ply + 1));

            if alpha >= beta {
                return alpha;
            }
        }

        // 4. Probe the Transposition Table
        let tt_entry: Option<EntryHit> = ttable.probe(self.key(), thread.ply);
        let mut tt_move: Option<Move> = None;

        if let Some(entry) = tt_entry {
            if entry.depth >= depth && !Node::PV_NODE {
                let entry_score: i32 = entry.score;

                if entry.bound == BoundType::Exact
                    || (entry.bound == BoundType::LowerBound && entry_score >= beta)
                    || (entry.bound == BoundType::UpperBound && entry_score <= alpha)
                {
                    return entry_score;
                }
            }

            if let Some(mv) = entry.legal_move(&self.board()) {
                tt_move = Some(mv);
            }
        }

        // 5. Iternal Iterative Reduction
        if Node::PV_NODE && depth >= 4 && tt_entry.is_none() {
            depth -= 1;
        }

        let alpha_orig: i32 = alpha;
        let mut best_move: Move = Move::default();
        let mut best_score: i32 = -INFINITY;
        let mut move_count: usize = 0;

        let killers: [Option<Move>; 2] = thread.killer.get(thread.ply);
        let mut picker: MovePicker = MovePicker::new(tt_move, killers);

        // Main Alpha-Beta Loop
        while let Some(mv) = picker.next(&self.board()) {
            self.push_move(mv, thread);
            ttable.prefetch(self.key());
            move_count += 1;
            let mut score: i32;

            // Principal Variation Search
            if move_count == 1 {
                // First move: Full Window Search
                score = if Node::PV_NODE {
                    -self.alphabeta::<PvNode>(thread, ttable, depth - 1, -beta, -alpha, child_pv)
                } else {
                    -self.alphabeta::<NonPv>(thread, ttable, depth - 1, -beta, -alpha, child_pv)
                };
            } else {
                // Later moves: Null window search
                score = -self.alphabeta::<NonPv>(
                    thread,
                    ttable,
                    depth - 1,
                    -alpha - 1,
                    -alpha,
                    child_pv,
                );

                // If it fails high and we are in a PV node, re-search with full window
                if score > alpha && Node::PV_NODE {
                    score = -self.alphabeta::<PvNode>(
                        thread,
                        ttable,
                        depth - 1,
                        -beta,
                        -alpha,
                        child_pv,
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
                    if Node::PV_NODE {
                        node_pv.push_line(best_move, child_pv);
                    }
                }

                // Beta Pruning
                if score >= beta {
                    if mv.is_quiet() {
                        thread.killer.store(thread.ply, mv);
                    }
                    break;
                }
            }
        }

        if move_count == 0 {
            return if in_check {
                // We are being mated
                mated_in(thread.ply)
            } else {
                // Stalemate
                0
            };
        }

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
            0,
            depth,
            bound,
            Node::PV_NODE,
            thread.ply,
        );

        best_score
    }

    #[allow(unused_assignments, unused_variables)]
    fn quiescence<Node: NodeType>(
        &mut self,
        thread: &mut Thread,
        ttable: &TranspositionTable,
        mut alpha: i32,
        beta: i32,
        node_pv: &mut PrincipalVariation,
    ) -> i32 {
        let mut temp_pv: PrincipalVariation = PrincipalVariation::default();
        let child_pv: &mut PrincipalVariation = &mut temp_pv;
        node_pv.len = 0;

        // Hard Limit Time Control
        if thread.time_manager.stop_hard(thread.nodes) {
            return 0;
        }

        let in_check: bool = self.in_check();

        // Update thread selective depth
        if thread.ply > thread.seldepth {
            thread.seldepth = thread.ply;
        }

        // Probe the Transposition Table
        let tt_entry: Option<EntryHit> = ttable.probe(self.key(), thread.ply);
        let mut tt_move: Option<Move> = None;

        if let Some(entry) = tt_entry {
            if !Node::PV_NODE {
                let entry_score: i32 = entry.score;

                if entry.bound == BoundType::Exact
                    || (entry.bound == BoundType::LowerBound && entry_score >= beta)
                    || (entry.bound == BoundType::UpperBound && entry_score <= alpha)
                {
                    return entry_score;
                }
            }

            if let Some(mv) = entry.legal_move(&self.board()) {
                tt_move = Some(mv);
            }
        }

        let alpha_orig: i32 = alpha;

        let stand_pat: i32 = self.evaluate();

        // Standing Pat Prunning
        // Fail-soft beta cuttof
        if stand_pat >= beta {
            return stand_pat;
        }

        // Improve alpha
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        let mut best_score: i32 = stand_pat;
        let mut best_move: Move = Move::default();
        let mut move_count: usize = 0;

        let mut picker: MovePicker = MovePicker::new(tt_move, [None, None]);
        picker.skip_quiets = true;

        // Main Quiescence Loop
        while let Some(mv) = picker.next(&self.board()) {
            self.push_move(mv, thread);
            ttable.prefetch(self.key());
            move_count += 1;
            let score: i32 = -self.quiescence::<Node>(thread, ttable, -beta, -alpha, child_pv);
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
                    if Node::PV_NODE {
                        node_pv.push_line(mv, child_pv);
                    }
                }

                // Beta Pruning
                if score >= beta {
                    break;
                }
            }
        }

        if move_count == 0 && in_check {
            // We are being mated
            return mated_in(thread.ply);
        }

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
            0,
            0,
            bound,
            Node::PV_NODE,
            thread.ply,
        );

        best_score
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
