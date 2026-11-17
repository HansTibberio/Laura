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

// src/config.rs

//! Engine configuration constants.

use std::sync::LazyLock;

// Timer parameters
// Time reserved for communication and move execution overhead.
pub const MOVE_OVERHEAD: u64 = 50;
// Minimum time allocation for a single move.
pub const MINIMUM_TIME: u64 = 30;
// Default number of moves remaining until the next time control.
pub const DEFAULT_MOVESTOGO: u64 = 20;
// Percentage of the increment included in the ideal move time.
pub const INCREMENT_TIME_BASE: u64 = 75;
// Base soft time limit as a percentage of the ideal move time.
pub const SOFT_TIME_BASE: u64 = 60;
// Hard time limit as a percentage of the ideal move time.
pub const HARD_TIME_BASE: u64 = 300;
// Maximum hard time limit as a percentage of the remaining clock.
pub const HARD_TIME_MAX_PCT: u64 = 50;
// Soft time scaling: base * stability * node fraction.
// Multipliers indexed by the number of consecutive iterations retaining the same best move.
pub const TM_STABILITY: [f64; 7] = [2.2, 1.6, 1.4, 1.1, 1.0, 0.95, 0.9];
// Base coefficient used to calculate the node-based time scaling factor.
pub const TM_NODE_BASE: f64 = 1.5;
// Scaling coefficient applied to the node-based time factor.
pub const TM_NODE_SCALE: f64 = 1.35;
// Minimum allowed combined time scaling factor.
pub const TM_SCALE_MIN: f64 = 0.35;
// Maximum allowed combined time scaling factor.
pub const TM_SCALE_MAX: f64 = 3.0;
// Minimum search depth required before applying node-based time scaling.
pub const TM_MIN_DEPTH: usize = 5;

// Search parameters
pub const INFINITY: i32 = 32_001;
pub const MATE: i32 = 32_000;
pub const MAX_MATE: i32 = MATE - MAX_PLY as i32;
pub const MAX_PLY: usize = 128;
pub const ASPIRATION_MARGIN: i32 = 25;
pub const ASPIRATION_DEPTH_THRESHOLD: usize = 5;
pub const MAX_DELTA: i32 = 1_025;

// Singular Extensions
pub const SE_MIN_DEPTH: usize = 8;
pub const SE_TT_DEPTH_MARGIN: usize = 3;
pub const SE_BETA_SCALE: i32 = 2;
pub const SE_DOUBLE_MARGIN: i32 = 60;
pub const SE_DOUBLE_LIMIT: u8 = 6;

// LMR values
pub const LMR_THRESHOLD: usize = 3;
pub const LMR_LOWER_DEPTH: usize = 3;
// (0.6897 + (depth).ln() * (moves).ln() / 2.5467)
pub static LMR_TABLE: LazyLock<[[i32; 64]; 64]> = LazyLock::new(|| {
    let mut table: [[i32; 64]; 64] = [[0i32; 64]; 64];
    for (depth, row) in table.iter_mut().enumerate().skip(1) {
        for (moves, value) in row.iter_mut().enumerate().skip(1) {
            *value = (0.6897 + (depth as f64).ln() * (moves as f64).ln() / 2.5467) as i32;
        }
    }
    table
});

// Tables parameters
pub const KILLER_SLOTS: usize = 2;
pub const HIST_CLAMP: i32 = 16384;

// Transposition table parameters
pub const TTMATE: i32 = 30_000;
pub const AGE_OFFSET: u8 = 3;
pub const BOUND_OFFSET: u8 = 1;
pub const BOUNDTYPE_MASK: u8 = 0x6;
pub const PV_NODE_MASK: u8 = 0x1;
pub const MEGABYTE: usize = 1_024 * 1_024;
pub const ENTRIES_PER_CELL: usize = 3;
pub const MAX_AGE: u8 = 1 << 5;
pub const AGE_MASK: u8 = MAX_AGE - 1;
pub const AGE_WEIGHT: i32 = 8;
pub const DEFAULT_SIZE: usize = 16;
pub const DATA_MASK: u128 = 0xFFFF_FFFF_FFFF_FFFF;
pub const KEY_MASK: u128 = 0xFFFF;
pub const KEY_WRAPPER_MASK: u64 = 0xFFFF;
