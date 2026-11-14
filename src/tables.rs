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

// src/tables.rs

//! Search tables for move ordering.

use crate::config::{HIST_CLAMP, KILLER_SLOTS, MAX_PLY};
use laura_core::{Color, Move, Piece, PieceType, Square};

// Killer Moves
#[derive(Debug)]
pub struct KillerMoves {
    table: [[Option<Move>; KILLER_SLOTS]; MAX_PLY],
}

impl Default for KillerMoves {
    fn default() -> Self {
        Self {
            table: [[None; KILLER_SLOTS]; MAX_PLY],
        }
    }
}

impl KillerMoves {
    #[inline(always)]
    pub fn store(&mut self, ply: usize, mv: Move) {
        if self.table[ply][0] != Some(mv) {
            self.table[ply][1] = self.table[ply][0];
            self.table[ply][0] = Some(mv);
        }
    }

    #[inline(always)]
    pub fn get(&self, ply: usize) -> [Option<Move>; KILLER_SLOTS] {
        self.table[ply]
    }
}

// History Table
#[derive(Debug, Copy, Clone)]
pub struct HistoryTable {
    // [side_to_move][from_square][to_square]
    table: [[[i32; 64]; 64]; 2],
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self {
            table: [[[0; 64]; 64]; 2],
        }
    }
}

impl HistoryTable {
    // Updates de move that causes the beta cutoff
    #[inline(always)]
    pub fn update_cutoff(&mut self, mv: Move, depth: usize, color: Color) {
        let bonus: i32 = calculate_bonus(depth as i32);
        self.add_score(mv, color, bonus);
    }

    // Penalizes the moves that were searched but didn't cause a beta cutoff
    #[inline(always)]
    pub fn update_non_cutoffs(&mut self, quiets: &[Move], depth: usize, color: Color) {
        let penalty: i32 = -calculate_bonus(depth as i32);
        for &mv in quiets {
            self.add_score(mv, color, penalty);
        }
    }

    // Gets the score of a Move
    #[inline(always)]
    pub fn get_score(&self, mv: Move, color: Color) -> i32 {
        let color_idx: usize = color as usize;
        let from: usize = mv.get_src() as usize;
        let to: usize = mv.get_dest() as usize;

        self.table[color_idx][from][to]
    }

    // Adds a score (bonus or penalties) to a move
    #[inline(always)]
    pub fn add_score(&mut self, mv: Move, color: Color, delta: i32) {
        let color_idx: usize = color as usize;
        let from: usize = mv.get_src() as usize;
        let to: usize = mv.get_dest() as usize;

        let entry: &mut i32 = &mut self.table[color_idx][from][to];
        *entry = gravity(*entry, delta);
    }
}

// Countermove Table
// Indexed by the previous move: [moved piece (with color)][destination square].
// Using the colored piece avoids mixing white and black refutations in the same slot.
#[derive(Debug, Copy, Clone)]
pub struct CountermoveTable {
    table: [[Option<Move>; 64]; Piece::NUM_PIECES],
}

impl Default for CountermoveTable {
    fn default() -> Self {
        Self {
            table: [[None; 64]; Piece::NUM_PIECES],
        }
    }
}

impl CountermoveTable {
    #[inline(always)]
    pub fn store(&mut self, prev_piece: Piece, prev_to: Square, response: Move) {
        self.table[prev_piece.to_index()][prev_to.to_index()] = Some(response);
    }

    #[inline(always)]
    pub fn get(&self, prev_piece: Piece, prev_to: Square) -> Option<Move> {
        self.table[prev_piece.to_index()][prev_to.to_index()]
    }
}

// Key into the continuation history: identifies the (piece, to) of a previous move.
// It is piece_index * 64 + to_square.
pub type ContKey = usize;

// Sub-table addressed by the current move: [piece][to].
type PieceToHistory = [[i32; 64]; Piece::NUM_PIECES];

// Continuation History
// [prev_piece][prev_to][curr_piece][curr_to], flattened on the first two indices.
// Pieces carry color: 12 * 64 * 12 * 64 * 4 bytes = 2.25 MB per thread.
#[derive(Debug)]
pub struct ContinuationHistory {
    table: Box<[PieceToHistory]>,
}

impl Default for ContinuationHistory {
    fn default() -> Self {
        // Allocated through a Vec so the 2.25 MB array is never built on the stack.
        Self {
            table: vec![[[0; 64]; Piece::NUM_PIECES]; Piece::NUM_PIECES * 64].into_boxed_slice(),
        }
    }
}

impl ContinuationHistory {
    #[inline(always)]
    pub fn key(prev_piece: Piece, prev_to: Square) -> ContKey {
        prev_piece.to_index() * 64 + prev_to.to_index()
    }

    #[inline(always)]
    pub fn get_score(&self, key: ContKey, piece: Piece, mv: Move) -> i32 {
        unsafe {
            *self
                .table
                .get_unchecked(key)
                .get_unchecked(piece.to_index())
                .get_unchecked(mv.get_dest().to_index())
        }
    }

    #[inline(always)]
    pub fn add_score(&mut self, key: ContKey, piece: Piece, mv: Move, delta: i32) {
        let entry: &mut i32 = &mut self.table[key][piece.to_index()][mv.get_dest().to_index()];
        *entry = gravity(*entry, delta);
    }
}

// Capture History
// [moving piece (with color)][to][captured piece type].
// Quiet queen promotions use the King slot as "nothing captured", since a king is never captured.
#[derive(Debug, Clone)]
pub struct CaptureHistory {
    table: Box<[[[i32; 6]; 64]; Piece::NUM_PIECES]>,
}

impl Default for CaptureHistory {
    fn default() -> Self {
        Self {
            table: Box::new([[[0; 6]; 64]; Piece::NUM_PIECES]),
        }
    }
}

impl CaptureHistory {
    #[inline(always)]
    pub fn get_score(&self, piece: Piece, mv: Move, captured: PieceType) -> i32 {
        self.table[piece.to_index()][mv.get_dest().to_index()][captured as usize]
    }

    #[inline(always)]
    pub fn add_score(&mut self, piece: Piece, mv: Move, captured: PieceType, delta: i32) {
        let entry: &mut i32 =
            &mut self.table[piece.to_index()][mv.get_dest().to_index()][captured as usize];
        *entry = gravity(*entry, delta);
    }
}

// History gravity: bonuses shrink as the entry approaches the clamp.
#[inline(always)]
fn gravity(old: i32, delta: i32) -> i32 {
    (old + delta - (old * delta.abs()) / HIST_CLAMP).clamp(-HIST_CLAMP, HIST_CLAMP)
}

#[inline(always)]
pub fn calculate_bonus(depth: i32) -> i32 {
    (300 * depth - 250).clamp(1, 2500)
}
