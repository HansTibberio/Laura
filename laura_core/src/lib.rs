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

#![no_std]

pub mod bitboard;
pub mod board;
pub mod castle_rights;
pub mod color;
pub mod file;
pub mod gen;
pub mod macros;
pub mod move_list;
pub mod moves;
pub mod piece;
pub mod rank;
pub mod square;
pub mod zobrist;

pub use bitboard::BitBoard;
pub use board::board::Board;
pub use board::movegen;
pub use castle_rights::CastleRights;
pub use color::Color;
pub use file::File;
pub use move_list::MoveList;
pub use moves::{Move, MoveType};
pub use piece::{Piece, PieceType};
pub use rank::Rank;
pub use square::Square;
pub use zobrist::Zobrist;
