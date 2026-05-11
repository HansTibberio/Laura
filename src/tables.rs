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
use laura_core::{Color, Move, PieceType};

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
    /// Updates de move that causes the beta cutoff
    #[inline(always)]
    pub fn update_cutoff(&mut self, mv: Move, depth: usize, color: Color) {
        let bonus: i32 = calculate_bonus(depth as i32);
        self.add_score(mv, color, bonus);
    }

    /// Penalizes the moves that were searched but didn't cause a beta cutoff
    #[inline(always)]
    pub fn update_non_cutoffs(&mut self, quiets: &[Move], depth: usize, color: Color) {
        let penalty: i32 = -calculate_bonus(depth as i32);
        for &mv in quiets {
            self.add_score(mv, color, penalty);
        }
    }

    /// Gets the score of a Move
    #[inline(always)]
    pub fn get_score(&self, mv: Move, color: Color) -> i32 {
        let color_idx: usize = color as usize;
        let from: usize = mv.get_src() as usize;
        let to: usize = mv.get_dest() as usize;

        self.table[color_idx][from][to]
    }

    /// Adds a score (bonus or penalties) to a move
    #[inline(always)]
    fn add_score(&mut self, mv: Move, color: Color, delta: i32) {
        let color_idx: usize = color as usize;
        let from: usize = mv.get_src() as usize;
        let to: usize = mv.get_dest() as usize;

        let old_score: i32 = self.table[color_idx][from][to];
        let new_score: i32 = old_score + delta - (old_score * delta.abs()) / HIST_CLAMP;

        self.table[color_idx][from][to] = new_score.clamp(-HIST_CLAMP, HIST_CLAMP);
    }
}

// Countermove History
#[derive(Debug, Copy, Clone)]
pub struct CountermoveTable {
    // [piece_type][to_square]
    table: [[Option<Move>; 64]; 6],
}

impl Default for CountermoveTable {
    fn default() -> Self {
        Self {
            table: [[None; 64]; 6],
        }
    }
}

impl CountermoveTable {
    #[inline(always)]
    pub fn store(&mut self, prev_mv: Move, prev_piece: PieceType, response: Move) {
        let piece_idx: usize = prev_piece as usize;
        let to_idx: usize = prev_mv.get_dest().to_index();
        self.table[piece_idx][to_idx] = Some(response);
    }

    #[inline(always)]
    pub fn get(&self, prev_mv: Move, prev_piece: PieceType) -> Option<Move> {
        self.table[prev_piece as usize][prev_mv.get_dest().to_index()]
    }
}

// Continuation History
pub struct ContinuationHistory {
    // [prev_piece][prev_to][curr_piece][curr_to]
    table: Box<[[[[i32; 64]; 6]; 64]; 6]>,
}

impl Default for ContinuationHistory {
    fn default() -> Self {
        Self {
            table: Box::new([[[[0; 64]; 6]; 64]; 6]),
        }
    }
}

impl ContinuationHistory {
    #[inline(always)]
    pub fn update_cutoff(
        &mut self,
        prev_piece: PieceType,
        prev_to: usize,
        curr_mv: Move,
        curr_piece: PieceType,
        depth: i32,
    ) {
        let bonus: i32 = calculate_bonus(depth);
        self.add_score(prev_piece, prev_to, curr_mv, curr_piece, bonus);
    }

    #[inline(always)]
    pub fn update_non_cutoffs(
        &mut self,
        prev_piece: PieceType,
        prev_to: usize,
        quiets: &[(Move, PieceType)],
        depth: i32,
    ) {
        let penalty: i32 = -calculate_bonus(depth);
        for &(mv, piece) in quiets {
            self.add_score(prev_piece, prev_to, mv, piece, penalty);
        }
    }

    #[inline(always)]
    pub fn get_score(
        &self,
        prev_piece: PieceType,
        prev_to: usize,
        curr_mv: Move,
        curr_piece: PieceType,
    ) -> i32 {
        self.table[prev_piece as usize][prev_to][curr_piece as usize][curr_mv.get_dest() as usize]
    }

    #[inline(always)]
    fn add_score(
        &mut self,
        prev_piece: PieceType,
        prev_to: usize,
        curr_mv: Move,
        curr_piece: PieceType,
        delta: i32,
    ) {
        let entry = &mut self.table[prev_piece as usize][prev_to][curr_piece as usize]
            [curr_mv.get_dest() as usize];
        *entry =
            (*entry + delta - (*entry * delta.abs()) / HIST_CLAMP).clamp(-HIST_CLAMP, HIST_CLAMP);
    }
}

// Capture History
#[derive(Debug, Clone)]
pub struct CaptureHistory {
    table: Box<[[[i32; 6]; 64]; 6]>,
}

impl Default for CaptureHistory {
    fn default() -> Self {
        Self {
            table: Box::new([[[0; 6]; 64]; 6]),
        }
    }
}

impl CaptureHistory {
    #[inline(always)]
    pub fn update_cutoff(
        &mut self,
        mv: Move,
        moving_piece: PieceType,
        captured: PieceType,
        depth: i32,
    ) {
        let bonus = calculate_bonus(depth);
        self.add_score(mv, moving_piece, captured, bonus);
    }

    #[inline(always)]
    pub fn update_non_cutoffs(
        &mut self,
        captures: &[(Move, PieceType, PieceType)], // (mv, moving_piece, captured_piece)
        depth: i32,
    ) {
        let penalty = -calculate_bonus(depth);
        for &(mv, moving, captured) in captures {
            self.add_score(mv, moving, captured, penalty);
        }
    }

    #[inline(always)]
    pub fn get_score(&self, mv: Move, moving_piece: PieceType, captured: PieceType) -> i32 {
        self.table[moving_piece as usize][mv.get_dest() as usize][captured as usize]
    }

    #[inline(always)]
    fn add_score(&mut self, mv: Move, moving_piece: PieceType, captured: PieceType, delta: i32) {
        let entry =
            &mut self.table[moving_piece as usize][mv.get_dest() as usize][captured as usize];
        *entry =
            (*entry + delta - (*entry * delta.abs()) / HIST_CLAMP).clamp(-HIST_CLAMP, HIST_CLAMP);
    }
}

#[inline(always)]
fn calculate_bonus(depth: i32) -> i32 {
    let depth: i32 = depth.min(12);
    depth * depth + 2 * depth - 2
}
