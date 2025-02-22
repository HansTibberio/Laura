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

use crate::get_king_attacks;
use crate::get_knight_attacks;
use crate::get_pawn_attacks;

#[cfg(not(feature = "bmi2"))]
use crate::{get_bishop_attacks, get_rook_attacks};
#[cfg(feature = "bmi2")]
use crate::{get_bishop_attacks, get_rook_attacks};

use crate::{BitBoard, Board, Color, Move, Piece, Square};

use super::movegen::*;

impl Board {
    /// Returns the bitboard representing all pieces for the white side.
    #[inline(always)]
    pub const fn white_bitboard(&self) -> BitBoard {
        self.sides_bitboard[Color::White as usize]
    }

    /// Returns the bitboard representing all pieces for the black side.
    #[inline(always)]
    pub const fn black_bitboard(&self) -> BitBoard {
        self.sides_bitboard[Color::Black as usize]
    }

    /// Returns a bitboard representing all pieces currently on the board for both sides.
    ///
    /// This function combines the bitboards for both white and black pieces by performing
    /// a bitwise OR operation.
    #[inline(always)]
    pub const fn combined_bitboard(&self) -> BitBoard {
        BitBoard(self.white_bitboard().0 | self.black_bitboard().0)
    }

    /// Returns a `BitBoard` representing the presence of a specified piece type and color on the board.
    /// Combines the bitboard for the specified piece with the bitboard for the side it belongs to.
    #[inline(always)]
    pub const fn piece_presence(&self, piece: Piece) -> BitBoard {
        BitBoard(
            self.pieces_bitboard[piece.piece_index()].0
                & self.sides_bitboard[piece.color() as usize].0,
        )
    }

    /// Returns a `BitBoard` representing the presence of all allied pieces for the current side on the board.
    #[inline(always)]
    pub const fn allied_presence(&self) -> BitBoard {
        self.sides_bitboard[self.side as usize]
    }

    /// Returns a `BitBoard` representing the presence of all enemy pieces for the opposing side on the board.
    #[inline(always)]
    pub const fn enemy_presence(&self) -> BitBoard {
        self.sides_bitboard[self.side as usize ^ 1]
    }

    /// Returns a `BitBoard` representing the presence of enemy queens and bishops on the board.
    /// This combines the bitboards for enemy queens and bishops into a single bitboard.
    #[inline(always)]
    pub fn enemy_queen_bishops(&self) -> BitBoard {
        self.enemy_queens() | self.enemy_bishops()
    }

    /// Returns a `BitBoard` representing the presence of enemy queens and rooks on the board.
    /// This combines the bitboards for enemy queens and rooks into a single bitboard.
    #[inline(always)]
    pub fn enemy_queen_rooks(&self) -> BitBoard {
        self.enemy_queens() | self.enemy_rooks()
    }

    /// Returns a `BitBoard` representing all enemy pieces that are attacking a specified square,
    /// based on the given blockers on the board. Evaluates potential attacks from enemy knights,
    /// kings, pawns, queens, bishops, and rooks against the square.
    #[inline]
    pub fn attackers(&self, square: Square, blockers: BitBoard) -> BitBoard {
        self.enemy_presence()
            & (self.knights() & get_knight_attacks(square)
                | self.kings() & get_king_attacks(square)
                | self.pawns() & get_pawn_attacks(self.side, square)
                | (self.queens() | self.bishops()) & get_bishop_attacks(square, blockers)
                | (self.queens() | self.rooks()) & get_rook_attacks(square, blockers))
    }

    /// Checks if a specified square is currently under attack by any enemy piece.
    #[inline(always)]
    pub fn attacked_square(&self, square: Square, blockers: BitBoard) -> bool {
        self.attackers(square, blockers) != BitBoard::EMPTY
    }

    /// Returns a `BitBoard` representing all enemy pieces that are directly checking the allied king.
    /// Uses the current combined board state to evaluate potential checks.
    #[inline(always)]
    pub fn checkers(&self) -> BitBoard {
        self.attackers(self.allied_king().to_square(), self.combined_bitboard())
    }

    /// Finds legal move in board from the uci-formatted move string
    #[inline]
    pub fn find_move(&self, move_str: &str) -> Option<Move> {
        gen_moves::<ALL_MOVES>(self)
            .moves()
            .iter()
            .find(|&mv| *mv == move_str).copied()
    }
}
