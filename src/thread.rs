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

// src/thread.rs

//! Thread management for parallel search.

use crate::config::{
    CORR_BONUS_DIV, CORR_HIST_LIMIT, CORR_SCALE, CORR_WEIGHT_CONT, CORR_WEIGHT_MAJOR,
    CORR_WEIGHT_MINOR, CORR_WEIGHT_NON_PAWN, CORR_WEIGHT_PAWN, MAX_MATE, MAX_PLY, MOVE_OVERHEAD,
};
use crate::search::{Evaluation, Score};
use crate::tables::{
    CaptureHistory, ContKey, ContinuationCorrectionHistory, ContinuationHistory, CorrKeys,
    CorrectionHistory, CountermoveTable, HistoryTable, calculate_bonus,
};
use crate::{
    TimeManager,
    position::Position,
    search::{MainThread, PrincipalVariation, WorkerThread},
    tables::KillerMoves,
    timer::TimeControl,
    transposition::TranspositionTable,
};
use laura_core::{Color, Move, MoveList, Piece, PieceType, Square, legal_moves};
use std::{
    collections::HashMap,
    iter::once,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
};

// Engine options set through UCI that affect the search or its output.
#[derive(Copy, Clone, Debug)]
pub struct SearchOptions {
    // Milliseconds subtracted from the clock for communication lag.
    pub move_overhead: u64,
    // Append "wdl W D L" to the info lines.
    pub show_wdl: bool,
    // Report centipawns normalized so that 100 cp = 50 % win chance.
    pub normalize: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            move_overhead: MOVE_OVERHEAD,
            show_wdl: false,
            normalize: true,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Stack {
    pub score: Score,
    pub static_eval: Evaluation,
    pub move_made: Option<Move>,
    pub piece_moved: Option<Piece>,
    pub excluded: Option<Move>,
    pub double_exts: u8,
}

// Plies back used as continuation history contexts (1 = opponent's last move).
pub const CONT_PLIES: [usize; 3] = [1, 2, 4];

// Plies back used as continuation correction contexts for the last move.
pub const CONT_CORR_PLIES: [usize; 2] = [2, 4];

// Continuation history keys for the current node, one per entry in CONT_PLIES.
pub type ContKeys = [Option<ContKey>; 3];

// A move searched at a node, with the data needed to update histories afterwards.
#[derive(Copy, Clone, Debug)]
pub struct TriedMove {
    pub mv: Move,
    pub piece: Piece,
    // Captured piece type for tactical moves (King = nothing captured).
    pub captured: PieceType,
}

// Fixed-size list of tried moves (no heap allocation per node).
pub struct TriedMoves<const N: usize> {
    moves: [TriedMove; N],
    len: usize,
}

impl<const N: usize> TriedMoves<N> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            moves: [TriedMove {
                mv: Move::null(),
                piece: Piece::WP,
                captured: PieceType::King,
            }; N],
            len: 0,
        }
    }

    // Moves beyond capacity are silently dropped (they would get only a malus).
    #[inline(always)]
    pub fn push(&mut self, tried: TriedMove) {
        if self.len < N {
            self.moves[self.len] = tried;
            self.len += 1;
        }
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[TriedMove] {
        &self.moves[..self.len]
    }
}

#[derive(Debug)]
pub struct Thread {
    pub id: usize,
    pub time_manager: TimeManager,
    pub options: SearchOptions,
    pub principal_variation: PrincipalVariation,

    pub stack: [Stack; MAX_PLY],

    pub killer: KillerMoves,
    pub history: HistoryTable,
    pub capture_history: CaptureHistory,
    pub continuation_history: ContinuationHistory,
    pub countermoves: CountermoveTable,

    pub pawn_corr: CorrectionHistory,
    pub non_pawn_corr: [CorrectionHistory; 2],
    pub minor_corr: CorrectionHistory,
    pub major_corr: CorrectionHistory,
    pub cont_corr: ContinuationCorrectionHistory,

    pub nodes: u64,
    pub ply: usize,
    pub seldepth: usize,
    pub score: Score,
    pub depth: usize,
    pub completed: usize,

    // Nodes spent under each root move, [from][to] (time management).
    pub root_nodes: Box<[[u64; 64]; 64]>,
}

impl Thread {
    pub fn new(time_manager: TimeManager, id: usize) -> Self {
        Self {
            id,
            principal_variation: PrincipalVariation::default(),
            nodes: 0,
            ply: 0,
            seldepth: 0,
            score: 0,
            depth: 0,
            time_manager,
            options: SearchOptions::default(),
            killer: KillerMoves::default(),
            history: HistoryTable::default(),
            capture_history: CaptureHistory::default(),
            completed: 0,
            continuation_history: ContinuationHistory::default(),
            countermoves: CountermoveTable::default(),
            pawn_corr: CorrectionHistory::default(),
            non_pawn_corr: [CorrectionHistory::default(), CorrectionHistory::default()],
            minor_corr: CorrectionHistory::default(),
            major_corr: CorrectionHistory::default(),
            cont_corr: ContinuationCorrectionHistory::default(),
            stack: [Stack::default(); MAX_PLY],
            root_nodes: Box::new([[0; 64]; 64]),
        }
    }

    pub fn smp(stop: Arc<AtomicBool>, nodes: Arc<AtomicU64>, id: usize) -> Self {
        Self::new(
            TimeManager::new(stop, nodes, TimeControl::Infinite, false, MOVE_OVERHEAD),
            id,
        )
    }

    #[inline(always)]
    pub fn set_up(&mut self) {
        self.principal_variation = PrincipalVariation::default();
        self.time_manager.reset_buffer();
        self.stack = [Stack::default(); MAX_PLY];
        self.nodes = 0;
        self.ply = 0;
        self.seldepth = 0;
        self.score = 0;
        self.depth = 0;
        self.completed = 0;
        self.root_nodes.iter_mut().for_each(|row| row.fill(0));
    }

    #[inline(always)]
    pub fn push_move(&mut self, piece: Piece, mv: Move) {
        self.stack[self.ply].piece_moved = Some(piece);
        self.stack[self.ply].move_made = Some(mv);
        self.ply += 1;
        self.nodes += 1;
    }

    #[inline(always)]
    pub fn push_null(&mut self) {
        self.stack[self.ply].piece_moved = None;
        self.stack[self.ply].move_made = None;
        self.ply += 1;
        self.nodes += 1;
    }

    #[inline(always)]
    pub fn pop_move(&mut self) {
        self.ply -= 1;
    }

    // The move played plies_back plies before the current node, if it was a real move.
    #[inline(always)]
    fn previous(&self, plies_back: usize) -> Option<(Piece, Move)> {
        if self.ply < plies_back {
            return None;
        }
        let entry: &Stack = &self.stack[self.ply - plies_back];
        match (entry.piece_moved, entry.move_made) {
            (Some(piece), Some(mv)) => Some((piece, mv)),
            _ => None,
        }
    }

    // Continuation history keys for the current node (1, 2 and 4 plies back).
    // A null move or the root boundary leaves the corresponding slot empty.
    #[inline(always)]
    pub fn cont_keys(&self) -> ContKeys {
        CONT_PLIES.map(|back| {
            self.previous(back)
                .map(|(piece, mv)| ContinuationHistory::key(piece, mv.get_dest()))
        })
    }

    // Countermove stored for the opponent's last move.
    #[inline(always)]
    pub fn countermove(&self) -> Option<Move> {
        self.previous(1)
            .and_then(|(piece, mv)| self.countermoves.get(piece, mv.get_dest()))
    }

    // Combined quiet-move score used for ordering, pruning and LMR.
    // The 4-ply context is weighted at half, as it is a weaker predictor (or not).
    #[inline(always)]
    pub fn quiet_score(&self, side: Color, piece: Piece, mv: Move, keys: &ContKeys) -> i32 {
        let mut score: i32 = self.history.get_score(mv, side);
        if let Some(key) = keys[0] {
            score += self.continuation_history.get_score(key, piece, mv);
        }
        if let Some(key) = keys[1] {
            score += self.continuation_history.get_score(key, piece, mv);
        }
        if let Some(key) = keys[2] {
            score += self.continuation_history.get_score(key, piece, mv) / 2;
        }
        score
    }

    // Continuation correction contexts (2 and 4 plies back) and the last move (piece, to).
    // A null move as last move disables the table; a null context only empties its slot.
    #[inline(always)]
    fn cont_corr_contexts(&self) -> Option<(Piece, Square, [Option<ContKey>; 2])> {
        let (piece, mv) = self.previous(1)?;
        let contexts = CONT_CORR_PLIES.map(|back| {
            self.previous(back)
                .map(|(ctx_piece, ctx_mv)| ContinuationHistory::key(ctx_piece, ctx_mv.get_dest()))
        });
        Some((piece, mv.get_dest(), contexts))
    }

    // Continuation correction value; the 4-ply context is read at half weight.
    #[inline(always)]
    fn cont_correction(&self) -> i32 {
        let Some((piece, to, [ctx2, ctx4])) = self.cont_corr_contexts() else {
            return 0;
        };
        let mut value: i32 = 0;
        if let Some(ctx) = ctx2 {
            value += self.cont_corr.get(ctx, piece, to);
        }
        if let Some(ctx) = ctx4 {
            value += self.cont_corr.get(ctx, piece, to) / 2;
        }
        value
    }

    // Static evaluation adjusted by the correction histories.
    // The result is kept out of the mate range so it is never mistaken for a mate score.
    #[inline(always)]
    pub fn corrected_eval(&self, raw: Evaluation, side: Color, keys: &CorrKeys) -> Evaluation {
        let weighted: i32 = CORR_WEIGHT_PAWN * self.pawn_corr.get(side, keys.pawn)
            + CORR_WEIGHT_NON_PAWN
                * (self.non_pawn_corr[0].get(side, keys.non_pawn[0])
                    + self.non_pawn_corr[1].get(side, keys.non_pawn[1]))
            + CORR_WEIGHT_MINOR * self.minor_corr.get(side, keys.minor)
            + CORR_WEIGHT_MAJOR * self.major_corr.get(side, keys.major)
            + CORR_WEIGHT_CONT * self.cont_correction();

        (raw + weighted / CORR_SCALE).clamp(-MAX_MATE + 1, MAX_MATE - 1)
    }

    // Moves every correction table towards the error observed at this node
    // (search score - corrected static eval), weighted by depth.
    #[inline(always)]
    pub fn update_correction(
        &mut self,
        side: Color,
        keys: &CorrKeys,
        depth: usize,
        best_score: Score,
        static_eval: Evaluation,
    ) {
        let bonus: i32 = ((best_score - static_eval) * depth as i32 / CORR_BONUS_DIV)
            .clamp(-CORR_HIST_LIMIT / 4, CORR_HIST_LIMIT / 4);

        self.pawn_corr.update(side, keys.pawn, bonus);
        self.non_pawn_corr[0].update(side, keys.non_pawn[0], bonus);
        self.non_pawn_corr[1].update(side, keys.non_pawn[1], bonus);
        self.minor_corr.update(side, keys.minor, bonus);
        self.major_corr.update(side, keys.major, bonus);

        if let Some((piece, to, contexts)) = self.cont_corr_contexts() {
            for ctx in contexts.into_iter().flatten() {
                self.cont_corr.update(ctx, piece, to, bonus);
            }
        }
    }

    // Updates every history table after a beta cutoff.
    #[allow(clippy::too_many_arguments)]
    pub fn update_histories(
        &mut self,
        side: Color,
        best: TriedMove,
        best_is_quiet: bool,
        quiets: &[TriedMove],
        captures: &[TriedMove],
        depth: usize,
        keys: &ContKeys,
    ) {
        let bonus: i32 = calculate_bonus(depth as i32);

        if best_is_quiet {
            // Butterfly + continuation: bonus to the cutoff move, malus to the rest.
            self.history.add_score(best.mv, side, bonus);
            for key in keys.iter().flatten() {
                self.continuation_history
                    .add_score(*key, best.piece, best.mv, bonus);
            }
            for tried in quiets {
                self.history.add_score(tried.mv, side, -bonus);
                for key in keys.iter().flatten() {
                    self.continuation_history
                        .add_score(*key, tried.piece, tried.mv, -bonus);
                }
            }

            // Refutation moves
            self.killer.store(self.ply, best.mv);
            if let Some((prev_piece, prev_mv)) = self.previous(1) {
                self.countermoves
                    .store(prev_piece, prev_mv.get_dest(), best.mv);
            }
        } else {
            self.capture_history
                .add_score(best.piece, best.mv, best.captured, bonus);
        }

        // Captures that were searched and failed to cut are always penalised.
        for tried in captures {
            self.capture_history
                .add_score(tried.piece, tried.mv, tried.captured, -bonus);
        }
    }
}

#[derive(Debug)]
pub struct ThreadPool {
    main: Thread,
    pool: Vec<Thread>,
    pub threads: usize,
    pub options: SearchOptions,
    stop: Arc<AtomicBool>,
    nodes: Arc<AtomicU64>,
}

impl ThreadPool {
    pub fn new(stop: Arc<AtomicBool>) -> Self {
        let nodes: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
        Self {
            main: Thread::smp(stop.clone(), nodes.clone(), 0),
            pool: Vec::new(),
            threads: 1,
            options: SearchOptions::default(),
            stop,
            nodes,
        }
    }

    // Resets every per-thread table (histories, killers, countermoves and
    // correction histories) so the next search starts from a clean state.
    pub fn clear(&mut self) {
        self.main = Thread::smp(self.stop.clone(), self.nodes.clone(), 0);
        for thread in self.pool.iter_mut() {
            *thread = Thread::smp(self.stop.clone(), self.nodes.clone(), thread.id);
        }
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    pub fn resize(&mut self, threads: usize) {
        let desired_size: usize = threads.saturating_sub(1);
        let current_size: usize = self.pool.len();

        if desired_size < current_size {
            self.pool.drain(desired_size..);
        }

        let mut id: usize = self.pool.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        self.pool.resize_with(desired_size, || {
            let thread_id: usize = id;
            id += 1;
            Thread::smp(self.stop.clone(), self.nodes.clone(), thread_id)
        });

        self.main = Thread::smp(self.stop.clone(), self.nodes.clone(), 0);
        self.threads = threads.max(1);
    }

    #[allow(semicolon_in_expressions_from_non_local_macros)]
    pub fn start_search(
        &mut self,
        position: &mut Position,
        ttable: &TranspositionTable,
        time_control: TimeControl,
    ) -> Option<Move> {
        self.main.options = self.options;
        self.main.time_manager = TimeManager::new(
            self.stop.clone(),
            self.nodes.clone(),
            time_control,
            position.white(),
            self.options.move_overhead,
        );

        self.stop.store(false, Ordering::SeqCst);
        self.nodes.store(0, Ordering::SeqCst);

        let moves: MoveList = legal_moves!(&position.board());
        if moves.is_empty() {
            let show_wdl: bool = self.options.show_wdl;
            if position.in_check() {
                let wdl: &str = if show_wdl { " wdl 0 0 1000" } else { "" };
                println!("info depth 0 score mate 0{wdl} time 0");
            } else {
                let wdl: &str = if show_wdl { " wdl 0 1000 0" } else { "" };
                println!("info depth 0 score cp 0{wdl} time 0");
            }
            return None;
        }

        if moves.len() == 1 || self.main.time_manager.not_search() {
            self.main.time_manager = TimeManager::new(
                self.stop.clone(),
                self.nodes.clone(),
                TimeControl::Depth(1),
                position.white(),
                self.options.move_overhead,
            );
            self.main.set_up();
            position.iterative_deepening::<MainThread>(&mut self.main, ttable);

            return self
                .main
                .principal_variation
                .as_slice()
                .first()
                .copied()
                .or(Some(moves[0]));
        }

        let pcopy: Position = position.clone();
        thread::scope(|s| {
            s.spawn(|| {
                self.main.set_up();
                position.iterative_deepening::<MainThread>(&mut self.main, ttable);
                self.stop.store(true, Ordering::SeqCst);
            });
            for thread in self.pool.iter_mut() {
                s.spawn(|| {
                    let mut position: Position = pcopy.clone();
                    thread.set_up();
                    position.iterative_deepening::<WorkerThread>(thread, ttable);
                });
            }
        });

        let threads = once(&self.main).chain(self.pool.iter());

        let max_depth: usize = threads.clone().map(|t| t.completed).max().unwrap_or(0);

        let mut freq: HashMap<Move, usize> = HashMap::new();
        for thread in threads {
            if thread.completed == max_depth
                && let Some(&mv) = thread.principal_variation.as_slice().first()
            {
                *freq.entry(mv).or_insert(0) += 1;
            }
        }

        let best_move: Option<Move> = freq
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(mv, _)| mv);

        best_move
    }
}

#[cfg(test)]
mod test {
    use crate::{Position, ThreadPool, timer::TimeControl, transposition::TranspositionTable};
    use laura_core::Move;
    use std::sync::{Arc, atomic::AtomicBool};

    #[test]
    fn test_best_move() {
        let mut threadpool: ThreadPool = ThreadPool::new(Arc::new(AtomicBool::new(false)));
        let mut position: Position = Position::default();
        let mut ttable: TranspositionTable = TranspositionTable::default();
        ttable.resize(1);
        let best: Option<Move> =
            threadpool.start_search(&mut position, &ttable, TimeControl::Depth(4));
        println!("bestmove {}", best.unwrap());
    }

    #[test]
    fn test_resize() {
        let mut threadpool = ThreadPool::new(Arc::new(AtomicBool::new(false)));
        threadpool.resize(4);
        for t in &threadpool.pool {
            println!("ID: {}", t.id)
        }
        threadpool.resize(6);
        for t in threadpool.pool {
            println!("ID: {}", t.id)
        }
    }
}
