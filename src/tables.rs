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

use crate::config::{KILLER_SLOTS, MAX_PLY};
use laura_core::{Color, Move};
use crate::config::{HIST_CLAMP, KILLER_SLOTS, MAX_PLY};

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
        let new_score: i32 = old_score + delta - (old_score * delta.abs()) / 16384;
        let new_score: i32 = old_score + delta - (old_score * delta.abs()) / HIST_CLAMP;

        self.table[color_idx][from][to] = new_score.clamp(-HIST_CLAMP, HIST_CLAMP);
    }
}


        self.table[color_idx][from][to] = new_score.clamp(-16384, 16384);
    }

    fn calculate_bonus(&self, depth: i32) -> i32 {
        let depth: i32 = depth.min(12);
        depth * depth + 2 * depth - 2
    }
#[inline(always)]
fn calculate_bonus(depth: i32) -> i32 {
    let depth: i32 = depth.min(12);
    depth * depth + 2 * depth - 2
}
