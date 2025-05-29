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

// src/config.rs

//! Engine configuration constants.

// Timer parameters
pub const MOVE_OVERHEAD: u64 = 50;
pub const MINIMUM_TIME: u64 = 30;
pub const OPTIMAL_TIME_BASE: u64 = 65;
pub const INCREMENT_TIME_BASE: u64 = 85;
pub const DEFAULT_MOVESTOGO: u64 = 40;

// Search parameters
pub const INFINITY: i32 = 32_001;
pub const MATE: i32 = 32_000;
pub const MAX_MATE: i32 = MATE - MAX_PLY as i32;
pub const MAX_PLY: usize = 128;
pub const ASPIRATION_MARGIN: i32 = 25;
pub const ASPIRATION_DEPTH_THRESHOLD: usize = 5;
pub const MAX_DELTA: i32 = 1_025;

// Tables parameters
pub const KILLER_SLOTS: usize = 2;

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
pub const DEFAULT_SIZE: usize = 16;
pub const DATA_MASK: u128 = 0xFFFF_FFFF_FFFF_FFFF;
pub const KEY_MASK: u128 = 0xFFFF;
pub const KEY_WRAPPER_MASK: u64 = 0xFFFF;
