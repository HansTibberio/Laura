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

use std::mem::transmute;

use super::types::{BitBoard, File, Rank, Square};

/// Represents a sliding piece (like a rook or bishop) on a chessboard, which moves
/// in specific directions defined by a set of delta pairs. Each delta pair defines
/// a direction in which the piece can move.
#[derive(Clone, Copy, Debug)]
pub struct Slider {
    /// Array of `(rank_delta, file_delta)` tuples indicating directions for sliding moves.
    deltas: [(i8, i8); 4],
}

impl Slider {
    /// Computes all possible moves for a sliding piece from a given starting square,
    /// taking into account any blockers that limit movement in each direction.
    ///
    /// This method iterates through each possible direction defined by `deltas`, and
    /// adds each reachable square along that direction to the resulting moves until
    /// a blocker is encountered or the board edge is reached.
    pub fn moves(&self, square: Square, blockers: BitBoard) -> BitBoard {
        let mut moves: BitBoard = BitBoard::EMPTY;
        let rank: i8 = square.rank() as i8;
        let file: i8 = square.file() as i8;

        for &(dr, df) in &self.deltas {
            let mut new_rank: i8 = rank + dr;
            let mut new_file: i8 = file + df;

            while (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
                let new_square: Square = Square::from_file_rank(
                    unsafe { transmute::<u8, File>(new_file as u8) },
                    unsafe { transmute::<u8, Rank>(new_rank as u8) },
                );
                let target_bitboard: BitBoard = new_square.to_bitboard();
                moves |= target_bitboard;

                if target_bitboard & blockers != BitBoard::EMPTY {
                    break;
                }

                new_rank += dr;
                new_file += df;
            }
        }

        moves
    }

    /// Generates a bitboard with all relevant blockers for move generation in each direction
    /// of the slider from the starting square, omitting blockers beyond the edge of the board.
    pub fn relevant_blockers(&self, square: Square) -> BitBoard {
        let mut blockers: BitBoard = BitBoard::EMPTY;
        let rank: i8 = square.rank() as i8;
        let file: i8 = square.file() as i8;

        for &(dr, df) in &self.deltas {
            let mut new_rank: i8 = rank + dr;
            let mut new_file: i8 = file + df;

            while (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
                let new_square: Square = Square::from_file_rank(
                    unsafe { transmute::<u8, File>(new_file as u8) },
                    unsafe { transmute::<u8, Rank>(new_rank as u8) },
                );

                let next_rank = new_rank + dr;
                let next_file = new_file + df;
                if !(0..8).contains(&next_rank) || !(0..8).contains(&next_file) {
                    break;
                }

                blockers |= new_square.to_bitboard();

                new_rank += dr;
                new_file += df;
            }
        }

        blockers
    }
}

/// Constant `Slider` instance representing a rook, which can move vertically or horizontally.
pub const ROOK_SLIDER: Slider = Slider {
    deltas: [(1, 0), (0, -1), (-1, 0), (0, 1)],
};

/// Constant `Slider` instance representing a bishop, which can move diagonally.
pub const BISHOP_SLIDER: Slider = Slider {
    deltas: [(1, 1), (1, -1), (-1, -1), (-1, 1)],
};
