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

// src/movepicker.rs

//! Staged move picker for search.
//!
//! Order:
//!   1. TT move
//!   2. Good captures   (MVV + capture history, SEE >= 0, lazy SEE at pick time)
//!   3. Killer 1, Killer 2
//!   4. Countermove
//!   5. Quiets          (butterfly + continuation history 1/2/4 plies)
//!   6. Bad captures    (in the same MVV + capture history order)
//!
//! All moves live in a single buffer: captures at [0, cap_end), quiets after them.
//! Captures failing SEE are moved down to [0, bad_end), which never overlaps
//! with the captures still pending selection.

use crate::sse::SEE;
use crate::thread::{ContKeys, Thread};
use laura_core::{Board, Move, MoveType, Piece, PieceType, quiet_moves, tactical_moves};

const MAX_MOVES: usize = 256;

// Weight of the victim value against capture history in capture ordering.
const MVV_WEIGHT: i32 = 16;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Stage {
    TTMove,
    GenCaptures,
    GoodCaptures,
    GenQuiets,
    Killer1,
    Killer2,
    Counter,
    Quiets,
    BadCaptures,
    Done,
}

pub struct MovePicker {
    stage: Stage,

    tt_move: Move,
    killers: [Move; 2],
    counter: Move,
    cont_keys: ContKeys,

    moves: [Move; MAX_MOVES],
    scores: [i32; MAX_MOVES],

    cur: usize,
    cap_end: usize,
    bad_end: usize,
    quiet_end: usize,

    pub skip_quiets: bool,
}

// Moving piece of a move.
#[inline(always)]
pub fn moving_piece(board: &Board, mv: Move) -> Piece {
    unsafe { board.piece_on(mv.get_src()).unwrap_unchecked() }
}

// Captured piece type for capture history. King stands for "nothing captured"
#[inline(always)]
pub fn captured_type(board: &Board, mv: Move) -> PieceType {
    if mv.get_type() == MoveType::EnPassant {
        PieceType::Pawn
    } else if mv.is_capture() {
        unsafe {
            board
                .piece_on(mv.get_dest())
                .unwrap_unchecked()
                .piece_type()
        }
    } else {
        PieceType::King
    }
}

impl MovePicker {
    // Picker for the main search: uses killers, countermove and continuation history.
    pub fn new(tt_move: Option<Move>, thread: &Thread, cont_keys: ContKeys) -> Self {
        let killers: [Option<Move>; 2] = thread.killer.get(thread.ply);
        Self::build(
            tt_move,
            [
                killers[0].unwrap_or(Move::null()),
                killers[1].unwrap_or(Move::null()),
            ],
            thread.countermove().unwrap_or(Move::null()),
            cont_keys,
        )
    }

    // Picker for quiescence: no refutation moves. Quiets are only generated
    // when skip_quiets stays false (check evasions).
    pub fn new_qsearch(tt_move: Option<Move>, cont_keys: ContKeys) -> Self {
        Self::build(tt_move, [Move::null(); 2], Move::null(), cont_keys)
    }

    fn build(
        tt_move: Option<Move>,
        killers: [Move; 2],
        counter: Move,
        cont_keys: ContKeys,
    ) -> Self {
        let tt_move: Move = tt_move.unwrap_or(Move::null());

        // Refutations must be distinct from each other and from the TT move.
        let killer1: Move = if killers[0] != tt_move {
            killers[0]
        } else {
            Move::null()
        };
        let killer2: Move = if killers[1] != tt_move && killers[1] != killer1 {
            killers[1]
        } else {
            Move::null()
        };
        let counter: Move = if counter != tt_move && counter != killer1 && counter != killer2 {
            counter
        } else {
            Move::null()
        };

        Self {
            stage: Stage::TTMove,
            tt_move,
            killers: [killer1, killer2],
            counter,
            cont_keys,
            moves: [Move::null(); MAX_MOVES],
            scores: [0; MAX_MOVES],
            cur: 0,
            cap_end: 0,
            bad_end: 0,
            quiet_end: 0,
            skip_quiets: false,
        }
    }

    #[inline(always)]
    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn next(&mut self, board: &Board, thread: &Thread) -> Option<Move> {
        loop {
            match self.stage {
                Stage::TTMove => {
                    self.stage = Stage::GenCaptures;
                    if !self.tt_move.is_null() {
                        return Some(self.tt_move);
                    }
                }

                Stage::GenCaptures => {
                    self.generate_captures(board, thread);
                    self.cur = 0;
                    self.stage = Stage::GoodCaptures;
                }

                Stage::GoodCaptures => {
                    while self.cur < self.cap_end {
                        let mv: Move = self.select_best(self.cur, self.cap_end);
                        self.cur += 1;

                        if mv == self.tt_move {
                            continue;
                        }

                        // Losing captures are deferred until after the quiets.
                        if !SEE::see(board, mv, 0) {
                            self.moves[self.bad_end] = mv;
                            self.bad_end += 1;
                            continue;
                        }

                        return Some(mv);
                    }

                    if self.skip_quiets {
                        self.cur = 0;
                        self.stage = Stage::BadCaptures;
                    } else {
                        self.stage = Stage::GenQuiets;
                    }
                }

                Stage::GenQuiets => {
                    self.generate_quiets(board, thread);
                    self.cur = self.cap_end;
                    self.stage = Stage::Killer1;
                }

                Stage::Killer1 => {
                    self.stage = Stage::Killer2;
                    if self.is_generated_quiet(self.killers[0]) {
                        return Some(self.killers[0]);
                    }
                }

                Stage::Killer2 => {
                    self.stage = Stage::Counter;
                    if self.is_generated_quiet(self.killers[1]) {
                        return Some(self.killers[1]);
                    }
                }

                Stage::Counter => {
                    self.stage = Stage::Quiets;
                    if self.is_generated_quiet(self.counter) {
                        return Some(self.counter);
                    }
                }

                Stage::Quiets => {
                    if !self.skip_quiets {
                        while self.cur < self.quiet_end {
                            let mv: Move = self.select_best(self.cur, self.quiet_end);
                            self.cur += 1;

                            if mv == self.tt_move
                                || mv == self.killers[0]
                                || mv == self.killers[1]
                                || mv == self.counter
                            {
                                continue;
                            }

                            return Some(mv);
                        }
                    }

                    self.cur = 0;
                    self.stage = Stage::BadCaptures;
                }

                Stage::BadCaptures => {
                    if self.cur < self.bad_end {
                        let mv: Move = self.moves[self.cur];
                        self.cur += 1;
                        return Some(mv);
                    }
                    self.stage = Stage::Done;
                }

                Stage::Done => return None,
            }
        }
    }

    // Captures and queen promotions, scored by MVV + capture history.
    #[allow(semicolon_in_expressions_from_non_local_macros)]
    fn generate_captures(&mut self, board: &Board, thread: &Thread) {
        let captures = tactical_moves!(board);
        let mut len: usize = 0;

        for &mv in captures.iter() {
            let piece: Piece = moving_piece(board, mv);
            let captured: PieceType = captured_type(board, mv);

            let mut victim: i32 = if captured == PieceType::King {
                0
            } else {
                SEE::piece_value(captured)
            };
            if mv.is_promotion() {
                victim += SEE::piece_value(PieceType::Queen) - SEE::piece_value(PieceType::Pawn);
            }

            self.moves[len] = mv;
            self.scores[len] =
                MVV_WEIGHT * victim + thread.capture_history.get_score(piece, mv, captured);
            len += 1;
        }

        self.cap_end = len;
        self.quiet_end = len;
    }

    // Quiet moves (incl. double pushes, castling and underpromotions),
    // scored by butterfly + continuation history.
    #[allow(semicolon_in_expressions_from_non_local_macros)]
    fn generate_quiets(&mut self, board: &Board, thread: &Thread) {
        let quiets = quiet_moves!(board);
        let side = board.side();
        let mut len: usize = self.cap_end;

        for &mv in quiets.iter() {
            let piece: Piece = moving_piece(board, mv);
            self.moves[len] = mv;
            self.scores[len] = thread.quiet_score(side, piece, mv, &self.cont_keys);
            len += 1;
        }

        self.quiet_end = len;
    }

    // Selection sort step: brings the best move of [start, end) to start.
    #[inline(always)]
    fn select_best(&mut self, start: usize, end: usize) -> Move {
        let mut best: usize = start;
        for i in (start + 1)..end {
            if self.scores[i] > self.scores[best] {
                best = i;
            }
        }
        self.moves.swap(start, best);
        self.scores.swap(start, best);
        self.moves[start]
    }

    // A killer/countermove is only played if the generator produced it in
    // this position.
    #[inline(always)]
    fn is_generated_quiet(&self, mv: Move) -> bool {
        !mv.is_null() && self.moves[self.cap_end..self.quiet_end].contains(&mv)
    }
}

#[cfg(test)]
mod test {
    use super::{MovePicker, Stage};
    use crate::sse::SEE;
    use crate::thread::Thread;
    use crate::timer::{TimeControl, TimeManager};
    use laura_core::{AllMoves, Board, Move, gen_moves};
    use std::str::FromStr;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64},
    };

    fn thread() -> Thread {
        Thread::new(
            TimeManager::new(
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicU64::new(0)),
                TimeControl::Infinite,
                true,
            ),
            0,
        )
    }

    fn all_picked(board: &Board, tt: Option<Move>, thread: &Thread) -> Vec<(Move, Stage)> {
        let mut picker: MovePicker = MovePicker::new(tt, thread, thread.cont_keys());
        let mut out = Vec::new();
        while let Some(mv) = picker.next(board, thread) {
            out.push((mv, picker.stage()));
        }
        out
    }

    #[test]
    fn picks_every_legal_move_exactly_once() {
        let fens = [
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "4k3/4p3/8/1p1p4/r3Q3/8/8/4K3 w - - 0 1",
            "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/7P/P1P1P3/RNBQKBNR w KQkq e6 0 1",
            "8/2P5/8/8/8/8/k6K/8 w - - 0 1",
        ];
        let thread: Thread = thread();
        for fen in fens {
            let board: Board = Board::from_str(fen).unwrap();
            let legal = gen_moves::<AllMoves>(&board);
            let tt: Option<Move> = legal.last().copied();
            let picked = all_picked(&board, tt, &thread);
            assert_eq!(picked.len(), legal.len(), "{fen}");
            for mv in legal.iter() {
                assert_eq!(
                    picked.iter().filter(|(m, _)| m == mv).count(),
                    1,
                    "{fen} {mv}"
                );
            }
            assert_eq!(picked[0].0, tt.unwrap());
        }
    }

    #[test]
    fn invalid_refutations_are_ignored() {
        let mut thread: Thread = thread();
        let board: Board = Board::default();
        // A killer that is not legal here must never be returned.
        let other: Board =
            Board::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .unwrap();
        let foreign: Move = other.find_move("e1g1").unwrap();
        thread.killer.store(0, foreign);
        let picked = all_picked(&board, None, &thread);
        assert_eq!(picked.len(), 20);
        assert!(picked.iter().all(|(m, _)| *m != foreign));
    }

    #[test]
    fn good_captures_before_quiets_bad_captures_last() {
        let board: Board = Board::from_str("4k3/4p3/8/1p1p4/r3Q3/8/8/4K3 w - - 0 1").unwrap();
        let thread: Thread = thread();
        let picked = all_picked(&board, None, &thread);
        let mut seen_quiet: bool = false;
        let mut seen_bad: bool = false;
        for (mv, _) in &picked {
            let good: bool = SEE::see(&board, *mv, 0);
            if mv.is_capture() {
                if good {
                    assert!(!seen_quiet && !seen_bad, "good capture {mv} after quiets");
                } else {
                    seen_bad = true;
                }
            } else {
                assert!(!seen_bad, "quiet {mv} after bad captures");
                seen_quiet = true;
            }
        }
        // Bad captures come last, still in MVV order: Qxa4 (rook, recaptured
        // by b5) before Qxe7 (pawn, recaptured by the king).
        assert!(picked[0].0.is_capture() && SEE::see(&board, picked[0].0, 0));
        let tail: Vec<String> = picked[picked.len() - 2..]
            .iter()
            .map(|(m, _)| m.to_string())
            .collect();
        assert_eq!(tail, ["e4a4", "e4e7"]);
    }

    #[test]
    fn qsearch_skips_quiets() {
        let board: Board = Board::from_str("4k3/4p3/8/1p1p4/r3Q3/8/8/4K3 w - - 0 1").unwrap();
        let thread: Thread = thread();
        let mut picker: MovePicker = MovePicker::new_qsearch(None, thread.cont_keys());
        picker.skip_quiets = true;
        while let Some(mv) = picker.next(&board, &thread) {
            assert!(mv.is_capture() || mv.is_promotion());
        }
    }
}
