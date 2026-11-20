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
// Score representing positive or negative infinity.
pub const INFINITY: i32 = 32_001;
// Base score used to represent a checkmate.
pub const MATE: i32 = 32_000;
// Maximum mate score adjusted for the maximum search ply.
pub const MAX_MATE: i32 = MATE - MAX_PLY as i32;
// Maximum supported search ply.
pub const MAX_PLY: usize = 128;
// Initial aspiration window half-width.
pub const ASPIRATION_MARGIN: i32 = 25;
// Minimum search depth required to enable aspiration windows.
pub const ASPIRATION_DEPTH_THRESHOLD: usize = 5;
// Maximum evaluation change considered for delta pruning.
pub const MAX_DELTA: i32 = 1_025;

// Singular Extensions
// Minimum depth required to consider a singular extension.
pub const SE_MIN_DEPTH: usize = 8;
// Minimum TT depth advantage required to consider a singular extension.
pub const SE_TT_DEPTH_MARGIN: usize = 3;
// Scaling factor applied to the singular extension beta threshold.
pub const SE_BETA_SCALE: i32 = 2;
// Margin used to trigger a double singular extension.
pub const SE_DOUBLE_MARGIN: i32 = 60;
// Maximum number of consecutive double singular extensions.
pub const SE_DOUBLE_LIMIT: u8 = 6;

// LMR values
// Minimum move count required to apply late move reductions.
pub const LMR_THRESHOLD: usize = 3;
// Minimum search depth required to apply late move reductions.
pub const LMR_LOWER_DEPTH: usize = 3;
// Precomputed logarithmic reduction table indexed by search depth and move number.
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
// Number of killer moves stored per ply.
pub const KILLER_SLOTS: usize = 2;
// Maximum absolute value allowed for history scores.
pub const HIST_CLAMP: i32 = 16384;

// Correction History parameters
// Number of entries per side to move in each hashed correction table (power of two).
pub const CORR_HIST_SIZE: usize = 16_384;
// Maximum absolute value of a correction history entry (gravity limit).
pub const CORR_HIST_LIMIT: i32 = 1_024;
// Divisor applied to (search score - static eval) * depth to obtain the update bonus.
pub const CORR_BONUS_DIV: i32 = 8;
// Divisor applied to the weighted sum of entries to obtain the correction in centipawns.
pub const CORR_SCALE: i32 = 131_072;
// Weight of the pawn structure correction table.
pub const CORR_WEIGHT_PAWN: i32 = 6_400;
// Weight of each side's non-pawn material correction table.
pub const CORR_WEIGHT_NON_PAWN: i32 = 6_700;
// Weight of the minor piece (knights and bishops) correction table.
pub const CORR_WEIGHT_MINOR: i32 = 6_500;
// Weight of the major piece (rooks and queens) correction table.
pub const CORR_WEIGHT_MAJOR: i32 = 3_600;
// Weight of the continuation correction table (the 4-ply context is read at half weight).
pub const CORR_WEIGHT_CONT: i32 = 5_900;

// Transposition table parameters
// Score threshold used to distinguish mate scores in the TT.
pub const TTMATE: i32 = 30_000;
// Bit offset of the generation age field in the TT entry.
pub const AGE_OFFSET: u8 = 3;
// Bit offset of the bound type field in the TT entry.
pub const BOUND_OFFSET: u8 = 1;
// Bit mask used to extract the bound type field.
pub const BOUNDTYPE_MASK: u8 = 0x6;
// Bit mask used to identify principal variation nodes.
pub const PV_NODE_MASK: u8 = 0x1;
// Number of bytes in one megabyte.
pub const MEGABYTE: usize = 1_024 * 1_024;
// Number of entries stored per transposition table cluster.
pub const ENTRIES_PER_CELL: usize = 3;
// Number of distinct transposition table generations.
pub const MAX_AGE: u8 = 1 << 5;
// Bit mask used to wrap the transposition table generation counter.
pub const AGE_MASK: u8 = MAX_AGE - 1;
// Weight assigned to the age component during TT replacement evaluation.
pub const AGE_WEIGHT: i32 = 8;
// Default transposition table size in megabytes.
pub const DEFAULT_SIZE: usize = 16;
// Mask used to extract the 64-bit data field from the packed entry.
pub const DATA_MASK: u128 = 0xFFFF_FFFF_FFFF_FFFF;
// Mask used to extract the 16-bit partial Zobrist key from the packed 128-bit entry
pub const KEY_MASK: u128 = 0xFFFF;
// Mask used to extract the lower 16 bits of the full Zobrist key.
pub const KEY_WRAPPER_MASK: u64 = 0xFFFF;

// OpenBench
// Default depth used by `bench` when no depth is given.
pub const BENCH_DEPTH: usize = 13;
