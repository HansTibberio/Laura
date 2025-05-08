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

// src/tables.rs

//! Search tables for move ordering.

use crate::config::{KILLER_SLOTS, MAX_PLY};
use laura_core::Move;

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
    pub fn store(&mut self, ply: usize, mv: Move) {
        if self.table[ply][0] != Some(mv) {
            self.table[ply][1] = self.table[ply][0];
            self.table[ply][0] = Some(mv);
        }
    }

    pub fn get(&self, ply: usize) -> [Option<Move>; KILLER_SLOTS] {
        self.table[ply]
    }
}
