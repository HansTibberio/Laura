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

// src/lib.rs

#![allow(dead_code)]

mod config;
mod evaluation;
mod movepicker;
mod position;
mod search;
mod sse;
mod tables;
mod thread;
mod timer;
mod transposition;
mod uci;

pub use position::Position;
pub use thread::ThreadPool;
pub use timer::TimeManager;
pub use uci::*;
