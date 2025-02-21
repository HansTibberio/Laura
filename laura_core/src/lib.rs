/*
    Laura-Core: a fast and efficient move generator for chess engines.

    Copyright (C) 2024-2025 HansTibberio <hanstiberio@proton.me>

    Laura-Core is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Laura-Core is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Laura-Core. If not, see <https://www.gnu.org/licenses/>.
*/
#![doc = include_str!("../README.md")]
#![no_std]

mod bitboard;
mod board;
mod castle_rights;
mod color;
mod file;
mod gen;
mod macros;
mod move_list;
mod moves;
mod piece;
mod rank;
mod square;
mod zobrist;

pub use bitboard::*;
pub use board::board::*;
pub use board::movegen::*;
pub use castle_rights::*;
pub use color::*;
pub use file::*;
#[cfg(not(feature = "bmi2"))]
pub use gen::black_magics::*;
#[cfg(feature = "bmi2")]
pub use gen::pext::*;
pub use gen::{king::*, knight::*, pawn::*, rays::*};
pub use move_list::*;
pub use moves::*;
pub use piece::*;
pub use rank::*;
pub use square::*;
pub use zobrist::*;
