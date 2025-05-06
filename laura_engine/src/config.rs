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
pub const MAX_DELTA: i32 = 1025;
