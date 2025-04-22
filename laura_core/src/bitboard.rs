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

use core::fmt;
use core::mem::transmute;
use core::ops::Not;

use crate::{
    BitBoardConsts,
    Color::{self, Black, White},
    Square,
};

/// A `BitBoard` represents a 64-bit chessboard where each bit corresponds to a square.
/// It is useful for efficiently representing and manipulating chess positions.
///
/// The bitboard follows the Little-Endian Rank-File (LERF) notation.
/// In this system, the least significant bit (LSB) represents the bottom-left corner of the chessboard,
/// while the most significant bit (MSB) represents the top-right corner.
///
///  ```md,ignore
/// 8 | 56 57 58 59 60 61 62 63
/// 7 | 48 49 50 51 52 53 54 55
/// 6 | 40 41 42 43 44 45 46 47
/// 5 | 32 33 34 35 36 37 38 39
/// 4 | 24 25 26 27 28 29 30 31
/// 3 | 16 17 18 19 20 21 22 23
/// 2 | 8  9  10 11 12 13 14 15
/// 1 | 0  1  2  3  4  5  6  7
///    -------------------------
///      A  B  C  D  E  F  G  H
/// ```
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(pub u64);

/// Implements display formatting for the `BitBoard` struct.
/// This allows for the `BitBoard` to be printed in a human-readable format,
/// where filled squares are shown as '★' and empty squares as '·'.
impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n      Bitboard: {}\n", self.0)?;

        for rank in (0..8).rev() {
            write!(f, "\n{}   ", rank + 1)?;
            for file in 0..8 {
                let square: usize = rank * 8 + file;
                let symbol: &str = if self.get_square(Square::from_index(square)) {
                    "★ "
                } else {
                    "· "
                };
                write!(f, "{}", symbol)?;
            }
        }
        write!(f, "\n\n    A B C D E F G H")?;
        Ok(())
    }
}

/// Implements the `Not` trait for `BitBoard`, allowing the bitwise NOT operation `!`.
/// The bitwise NOT flips all bits in the `BitBoard`, effectively inverting the board state.
impl Not for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

/// Implements `Iterator` for `BitBoard`, allowing iteration over the set squares.
/// Each call to `next` returns the next `Square` that is set (i.e., the next '1' bit)
impl Iterator for BitBoard {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Square> {
        if self.0 != 0 {
            let lsb: u32 = self.0.trailing_zeros();
            self.0 &= self.0 - 1;
            Some(unsafe { transmute::<u8, Square>((lsb as u8) & 63) })
        } else {
            None
        }
    }
}

/// Methods for the `BitBoard` struct, including utilities for manipulating bits and interacting with squares.
impl BitBoard {
    // Predefined `BitBoard` constants for sides, files, and ranks
    BitBoardConsts! {
        WHITE_SIDE = 0x0000_0000_FFFF_FFFF,
        BLACK_SIDE = 0xFFFF_FFFF_0000_0000,
        FILE_A = 0x0101_0101_0101_0101,
        FILE_B = 0x0202_0202_0202_0202,
        FILE_C = 0x0404_0404_0404_0404,
        FILE_D = 0x0808_0808_0808_0808,
        FILE_E = 0x1010_1010_1010_1010,
        FILE_F = 0x2020_2020_2020_2020,
        FILE_G = 0x4040_4040_4040_4040,
        FILE_H = 0x8080_8080_8080_8080,
        RANK_1 = 0x0000_0000_0000_00FF,
        RANK_2 = 0x0000_0000_0000_FF00,
        RANK_3 = 0x0000_0000_00FF_0000,
        RANK_4 = 0x0000_0000_FF00_0000,
        RANK_5 = 0x0000_00FF_0000_0000,
        RANK_6 = 0x0000_FF00_0000_0000,
        RANK_7 = 0x00FF_0000_0000_0000,
        RANK_8 = 0xFF00_0000_0000_0000,
        DARK_SQUARES = 0xAA55_AA55_AA55_AA55,
        LIGHT_SQUARES = 0x55AA_55AA_55AA_55AA,
        EMPTY = 0,
        FULL = 0xFFFF_FFFF_FFFF_FFFF,
    }

    /// Converts the `BitBoard` to a [`Square`] by returning the square corresponding to
    /// the least significant set bit (LSB), or `None` if the bitboard is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // BitBoard with a single bit set at B2
    /// let bitboard = BitBoard(1 << Square::B2 as u64);
    /// assert_eq!(bitboard.to_square(), Some(Square::B2));
    ///
    /// // BitBoard with multiple bits set; returns the least significant one (D1)
    /// let bitboard = BitBoard((1 << Square::D1 as u64) | (1 << Square::E1 as u64));
    /// assert_eq!(bitboard.to_square(), Some(Square::D1));
    ///
    /// // Empty BitBoard returns None
    /// let bitboard = BitBoard::EMPTY;
    /// assert_eq!(bitboard.to_square(), None);
    /// ```
    #[inline(always)]
    pub const fn to_square(self) -> Option<Square> {
        if self.0 != 0 {
            // SAFETY: We just checked that self.0 != 0, so trailing_zeros is in range [0, 63]
            Some(unsafe { transmute::<u8, Square>((self.0.trailing_zeros() as u8) & 63) })
        } else {
            None
        }
    }

    /// Returns a new `BitBoard` with the bit corresponding to the given [`Square`] set to `1`.
    ///
    /// This operation does not mutate the original `BitBoard`, but instead returns a new instance
    /// with the specified square set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // Setting a bit on an empty square
    /// let bitboard = BitBoard::EMPTY;
    /// assert!(bitboard.is_empty());
    /// let square = Square::C2;
    /// let updated = bitboard.set_square(square);
    /// assert_eq!(updated, BitBoard(1 << Square::C2 as u64));
    ///
    /// // Setting a bit on a BitBoard that already has bits set (D1 and E1)
    /// let bitboard = BitBoard((1 << Square::D1 as u64) | (1 << Square::E1 as u64));
    /// let square = Square::C2;
    /// let updated = bitboard.set_square(square);
    /// assert_eq!(updated, BitBoard((1 << Square::D1 as u64) | (1 << Square::E1 as u64) | (1 << Square::C2 as u64)));
    /// ```
    #[inline(always)]
    pub const fn set_square(self, square: Square) -> Self {
        Self(self.0 | (1u64 << square.to_index()))
    }

    /// Returns `true` if the bit corresponding to the given [`Square`] is set in the `BitBoard`, otherwise returns `false`.
    ///
    /// This method performs a bitwise check to determine whether the specified square is active (set to `1`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let bitboard = BitBoard::EMPTY;
    /// assert!(!bitboard.get_square(Square::C3));
    ///
    /// // BitBoard with a single bit set at square C3
    /// let bitboard = BitBoard(1 << Square::C3 as u64);
    /// assert!(bitboard.get_square(Square::C3));
    ///
    /// // BitBoard with multiple bits set (E4 and F6)
    /// let bitboard = BitBoard((1 << Square::E4 as u64) | (1 << Square::F6 as u64));
    /// assert!(bitboard.get_square(Square::F6));
    /// assert!(!bitboard.get_square(Square::G1));
    /// ```
    #[inline(always)]
    pub const fn get_square(self, square: Square) -> bool {
        self.0 & (1u64 << square.to_index()) != 0
    }

    /// Returns a new `BitBoard` with the bit corresponding to the given [`Square`] cleared (set to `0`).
    ///
    /// This operation does not mutate the original `BitBoard`, but returns a new instance
    /// with the specified square cleared. If the square was already cleared, the result is unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // Clear a square that is set (C4)
    /// let bitboard = BitBoard(1 << Square::C4 as u64);
    /// let updated = bitboard.pop_square(Square::C4);
    /// assert_eq!(updated, BitBoard::EMPTY);
    ///
    /// // Clear a square in a multi-bit board (D5 and E6)
    /// let bitboard = BitBoard((1 << Square::D5 as u64) | (1 << Square::E6 as u64));
    /// let updated = bitboard.pop_square(Square::D5);
    /// let expected = BitBoard(1 << Square::E6 as u64);
    /// assert_eq!(updated, expected);
    ///
    /// // Clearing a square that is already empty does nothing
    /// let bitboard = BitBoard(1 << Square::H1 as u64);
    /// let updated = bitboard.pop_square(Square::A2);
    /// assert_eq!(updated, bitboard);
    /// ```
    #[inline(always)]
    pub const fn pop_square(self, square: Square) -> Self {
        Self(self.0 & !(1u64 << square.to_index()))
    }

    /// Returns the number of set bits in the `BitBoard`, representing how many squares are currently occupied.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // Empty BitBoard has 0 bits set
    /// let bitboard = BitBoard::EMPTY;
    /// assert_eq!(bitboard.count_bits(), 0);
    ///
    /// // BitBoard with a single square set (E4)
    /// let bitboard = BitBoard(1 << Square::E4 as u64);
    /// assert_eq!(bitboard.count_bits(), 1);
    ///
    /// // BitBoard with multiple squares set (A1, H8 and D4)
    /// let bitboard = BitBoard((1 << Square::A1 as u64)
    ///                       | (1 << Square::H8 as u64)
    ///                       | (1 << Square::D4 as u64));
    /// assert_eq!(bitboard.count_bits(), 3);
    /// ```
    #[inline(always)]
    pub const fn count_bits(self) -> u32 {
        self.0.count_ones()
    }

    /// Flips the `BitBoard` vertically by mirroring its bits across the horizontal axis (rank 4).
    ///
    /// This operation swaps the ranks of the board so that rank 1 becomes rank 8, rank 2 becomes rank 7, and so on.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A piece on A1 is flipped to A8
    /// let bitboard = BitBoard(1 << Square::A1 as u64);
    /// let flipped = bitboard.flip();
    /// assert_eq!(flipped, BitBoard(1 << Square::A8 as u64));
    ///
    /// // Multiple pieces on the first and second ranks flipped to eighth and seventh
    /// let bitboard = BitBoard((1 << Square::B1 as u64) | (1 << Square::C2 as u64));
    /// let flipped = bitboard.flip();
    /// let expected = BitBoard((1 << Square::B8 as u64) | (1 << Square::C7 as u64));
    /// assert_eq!(flipped, expected);
    ///
    /// // Flipping twice returns the original position
    /// let original = bitboard;
    /// let flipped_twice = bitboard.flip().flip();
    /// assert_eq!(flipped_twice, original);
    /// ```
    #[inline(always)]
    pub const fn flip(self) -> Self {
        Self(self.0.swap_bytes())
    }

    /// Shifts the `BitBoard` one rank forward relative to the side to move.
    ///
    /// For [`White`], this shifts all bits one rank up (towards rank 8).  
    /// For [`Black`], this shifts all bits one rank down (towards rank 1).
    ///
    /// This function is commonly used to generate forward moves, such as pawn pushes,
    /// and respects the perspective of the side to move.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White pawn on D2 moves forward to D3
    /// let white_pawn = BitBoard(1 << Square::D2 as u64);
    /// let advanced = white_pawn.forward(Color::White);
    /// assert_eq!(advanced, BitBoard(1 << Square::D3 as u64));
    ///
    /// // A Black pawn on E7 moves forward to E6 (i.e., shifted down the board)
    /// let black_pawn = BitBoard(1 << Square::E7 as u64);
    /// let advanced = black_pawn.forward(Color::Black);
    /// assert_eq!(advanced, BitBoard(1 << Square::E6 as u64));
    /// ```
    ///
    /// # Expected behavior
    ///
    /// A square on rank 8 remains off-board after shifting forward,
    /// and cannot be shifted back by the same operation
    /// ```
    /// # use laura_core::*;
    ///
    /// let offboard = BitBoard(1 << Square::H8 as u64).forward(Color::White);
    /// assert_eq!(offboard, BitBoard(0));
    /// ```
    #[inline(always)]
    pub const fn forward(self, side: Color) -> Self {
        match side {
            White => Self(self.0 << 8),
            Black => Self(self.0 >> 8),
        }
    }

    /// Returns a new `BitBoard` representing the squares to the `"left"` of the current positions,
    /// from the perspective of the given [`Color`].
    ///
    /// From [`White`]'s perspective, `"left"` means west (toward file A), and bits are shifted right.
    ///
    /// From [`Black`]'s perspective, `"left"` means east (toward file H), and bits are shifted left.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White piece on D4 moves left to C4
    /// let bitboard = BitBoard(1 << Square::D4 as u64);
    /// let result = bitboard.left(Color::White);
    /// assert_eq!(result, BitBoard(1 << Square::C4 as u64));
    ///
    /// // A Black piece on D4 moves left to E4 (inverted perspective)
    /// let bitboard = BitBoard(1 << Square::D4 as u64);
    /// let result = bitboard.left(Color::Black);
    /// assert_eq!(result, BitBoard(1 << Square::E4 as u64));
    /// ```
    ///
    /// # Note
    ///
    /// This operation does **not** wrap around the board. Any square on the edge (file A for White,
    /// file H for Black) will be cleared in the result.
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White piece on file A cannot move left
    /// let bitboard = BitBoard(1 << Square::A2 as u64);
    /// assert_eq!(bitboard.left(Color::White), BitBoard::EMPTY);
    ///
    /// // A Black piece on file H cannot move left
    /// let bitboard = BitBoard(1 << Square::H7 as u64);
    /// assert_eq!(bitboard.left(Color::Black), BitBoard::EMPTY);
    /// ```
    #[inline(always)]
    pub const fn left(self, side: Color) -> Self {
        match side {
            White => Self((self.0 & !BitBoard::FILE_A.0) >> 1),
            Black => Self((self.0 & !BitBoard::FILE_H.0) << 1),
        }
    }

    /// Returns a new `BitBoard` representing the squares to the `"left"` of the current positions,
    /// from the perspective of a specific color known at compile time.
    ///
    /// For [`White`], `"left"` refers to west (toward file A), and bits are shifted right.  
    /// For [`Black`], `"left"` refers to east (toward file H), and bits are shifted left.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White piece on D4 moves left to C4
    /// let bitboard = BitBoard(1 << Square::D4 as u64);
    /// let result = bitboard.left_for::<{ Color::White as usize }>();
    /// assert_eq!(result, BitBoard(1 << Square::C4 as u64));
    ///
    /// // A Black piece on D4 moves left to E4 (inverted perspective)
    /// let bitboard = BitBoard(1 << Square::D4 as u64);
    /// let result = bitboard.left_for::<{ Color::Black as usize }>();
    /// assert_eq!(result, BitBoard(1 << Square::E4 as u64));
    /// ```
    ///
    /// # Note
    ///
    /// This operation does **not** wrap around the board. Any square on the edge (file A for White,
    /// file H for Black) will be cleared in the result.
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White piece on file A cannot move left
    /// let bitboard = BitBoard(1 << Square::A2 as u64);
    /// let result = bitboard.left_for::<{ Color::White as usize }>();
    /// assert_eq!(result, BitBoard::EMPTY);
    ///
    /// // A Black piece on file H cannot move left
    /// let bitboard = BitBoard(1 << Square::H7 as u64);
    /// let result = bitboard.left_for::<{ Color::Black as usize }>();
    /// assert_eq!(result, BitBoard::EMPTY);
    /// ```
    #[inline(always)]
    pub const fn left_for<const COLOR: usize>(self) -> Self {
        if COLOR == White as usize {
            Self((self.0 & !BitBoard::FILE_A.0) >> 1)
        } else {
            Self((self.0 & !BitBoard::FILE_H.0) << 1)
        }
    }

    /// Returns a new `BitBoard` representing the squares to the `"right"` of the current positions,
    /// from the perspective of the given [`Color`].
    ///
    /// From the perspective of [`White`], `"right"` means east (toward file H), and bits are shifted left.  
    /// From the perspective of [`Black`], `"right"` means west (toward file A), and bits are shifted right.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White piece on D4 moves right to E4
    /// let bitboard = BitBoard(1 << Square::D4 as u64);
    /// let result = bitboard.right(Color::White);
    /// assert_eq!(result, BitBoard(1 << Square::E4 as u64));
    ///
    /// // A Black piece on D4 moves right to C4 (inverted perspective)
    /// let bitboard = BitBoard(1 << Square::D4 as u64);
    /// let result = bitboard.right(Color::Black);
    /// assert_eq!(result, BitBoard(1 << Square::C4 as u64));
    /// ```
    ///
    /// # Expected behavior
    ///
    /// Squares on the edge files are cleared to avoid horizontal wrap-around:
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White piece on file H cannot move right
    /// let bitboard = BitBoard(1 << Square::H2 as u64);
    /// assert_eq!(bitboard.right(Color::White), BitBoard::EMPTY);
    ///
    /// // A Black piece on file A cannot move right
    /// let bitboard = BitBoard(1 << Square::A7 as u64);
    /// assert_eq!(bitboard.right(Color::Black), BitBoard::EMPTY);
    /// ```
    #[inline(always)]
    pub const fn right(self, side: Color) -> Self {
        match side {
            White => Self((self.0 & !BitBoard::FILE_H.0) << 1),
            Black => Self((self.0 & !BitBoard::FILE_A.0) >> 1),
        }
    }

    /// Returns a new `BitBoard` representing the squares to the `"right"` of the current positions,
    /// from the perspective of a specific color known at compile time.
    ///
    /// For [`White`], `"right"` means east (toward file H), and bits are shifted left.  
    /// For [`Black`], `"right"` means west (toward file A), and bits are shifted right.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White piece on D4 moves right to E4
    /// let bitboard = BitBoard(1 << Square::D4 as u64);
    /// let result = bitboard.right_for::<{ Color::White as usize }>();
    /// assert_eq!(result, BitBoard(1 << Square::E4 as u64));
    ///
    /// // A Black piece on D4 moves right to C4 (inverted perspective)
    /// let bitboard = BitBoard(1 << Square::D4 as u64);
    /// let result = bitboard.right_for::<{ Color::Black as usize }>();
    /// assert_eq!(result, BitBoard(1 << Square::C4 as u64));
    /// ```
    ///
    /// # Expected behavior
    ///
    /// Squares on the edge files are cleared to avoid horizontal wrap-around:
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White piece on file H cannot move right
    /// let bitboard = BitBoard(1 << Square::H2 as u64);
    /// let result = bitboard.right_for::<{ Color::White as usize }>();
    /// assert_eq!(result, BitBoard::EMPTY);
    ///
    /// // A Black piece on file A cannot move right
    /// let bitboard = BitBoard(1 << Square::A7 as u64);
    /// let result = bitboard.right_for::<{ Color::Black as usize }>();
    /// assert_eq!(result, BitBoard::EMPTY);
    /// ```
    #[inline(always)]
    pub const fn right_for<const COLOR: usize>(self) -> Self {
        if COLOR == White as usize {
            Self((self.0 & !BitBoard::FILE_H.0) << 1)
        } else {
            Self((self.0 & !BitBoard::FILE_A.0) >> 1)
        }
    }

    /// Returns a new `BitBoard` representing the squares diagonally `"up-left"` from the current positions,
    /// from the perspective of the given [`Color`].
    ///
    /// For [`White`], this corresponds to a shift one rank forward and one file to the left.  
    /// For [`Black`], this corresponds to a shift one rank backward and one file to the right.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White pawn on E4 attacks diagonally up-left to D5
    /// let bitboard = BitBoard(1 << Square::E4 as u64);
    /// let result = bitboard.up_left(Color::White);
    /// assert_eq!(result, BitBoard(1 << Square::D5 as u64));
    ///
    /// // A Black pawn on D5 attacks diagonally up-left to E4 (from Black's perspective)
    /// let bitboard = BitBoard(1 << Square::D5 as u64);
    /// let result = bitboard.up_left(Color::Black);
    /// assert_eq!(result, BitBoard(1 << Square::E4 as u64));
    /// ```
    ///
    /// # Expected behavior
    ///
    /// Movement from edge files is masked out to prevent wrap-around:
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White piece on file A cannot move diagonally up-left
    /// let bitboard = BitBoard(1 << Square::A2 as u64);
    /// let result = bitboard.up_left(Color::White);
    /// assert_eq!(result, BitBoard::EMPTY);
    ///
    /// // A Black piece on file H cannot move diagonally up-left
    /// let bitboard = BitBoard(1 << Square::H7 as u64);
    /// let result = bitboard.up_left(Color::Black);
    /// assert_eq!(result, BitBoard::EMPTY);
    /// ```
    #[inline(always)]
    pub const fn up_left(self, side: Color) -> Self {
        match side {
            White => Self((self.0 & !BitBoard::FILE_A.0) << 7),
            Black => Self((self.0 & !BitBoard::FILE_H.0) >> 7),
        }
    }

    /// Returns a new `BitBoard` representing the squares diagonally `"up-left"` from the current positions,
    /// from the perspective of a specific color known at compile time.
    ///
    /// For [`White`], this corresponds to a shift one rank forward and one file to the left.  
    /// For [`Black`], this corresponds to a shift one rank backward and one file to the right.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White pawn on E4 attacks diagonally up-left to D5
    /// let bitboard = BitBoard(1 << Square::E4 as u64);
    /// let result = bitboard.up_left_for::<{ Color::White as usize }>();
    /// assert_eq!(result, BitBoard(1 << Square::D5 as u64));
    ///
    /// // A Black pawn on D5 attacks diagonally up-left to E4 (from Black's perspective)
    /// let bitboard = BitBoard(1 << Square::D5 as u64);
    /// let result = bitboard.up_left_for::<{ Color::Black as usize }>();
    /// assert_eq!(result, BitBoard(1 << Square::E4 as u64));
    /// ```
    ///
    /// # Expected behavior
    ///
    /// Movement from edge files is masked out to prevent wrap-around:
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White piece on file A cannot move diagonally up-left
    /// let bitboard = BitBoard(1 << Square::A2 as u64);
    /// let result = bitboard.up_left_for::<{ Color::White as usize }>();
    /// assert_eq!(result, BitBoard::EMPTY);
    ///
    /// // A Black piece on file H cannot move diagonally up-left
    /// let bitboard = BitBoard(1 << Square::H7 as u64);
    /// let result = bitboard.up_left_for::<{ Color::Black as usize }>();
    /// assert_eq!(result, BitBoard::EMPTY);
    /// ```
    #[inline(always)]
    pub const fn up_left_for<const COLOR: usize>(self) -> Self {
        if COLOR == White as usize {
            Self((self.0 & !BitBoard::FILE_A.0) << 7)
        } else {
            Self((self.0 & !BitBoard::FILE_H.0) >> 7)
        }
    }

    /// Returns a new `BitBoard` representing the squares diagonally `"up-right"` from the current positions,
    /// from the perspective of the given [`Color`].
    ///
    /// For [`White`], this corresponds to a shift one rank forward and one file to the right.  
    /// For [`Black`], this corresponds to a shift one rank backward and one file to the left.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White pawn on E4 attacks diagonally up-right to F5
    /// let bitboard = BitBoard(1 << Square::E4 as u64);
    /// let result = bitboard.up_right(Color::White);
    /// assert_eq!(result, BitBoard(1 << Square::F5 as u64));
    ///
    /// // A Black pawn on D5 attacks diagonally up-right to C4 (from Black's perspective)
    /// let bitboard = BitBoard(1 << Square::D5 as u64);
    /// let result = bitboard.up_right(Color::Black);
    /// assert_eq!(result, BitBoard(1 << Square::C4 as u64));
    /// ```
    ///
    /// # Expected behavior
    ///
    /// Movement from edge files is masked out to prevent wrap-around:
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White piece on file H cannot move diagonally up-right
    /// let bitboard = BitBoard(1 << Square::H2 as u64);
    /// let result = bitboard.up_right(Color::White);
    /// assert_eq!(result, BitBoard::EMPTY);
    ///
    /// // A Black piece on file A cannot move diagonally up-right
    /// let bitboard = BitBoard(1 << Square::A7 as u64);
    /// let result = bitboard.up_right(Color::Black);
    /// assert_eq!(result, BitBoard::EMPTY);
    /// ```
    #[inline(always)]
    pub const fn up_right(self, side: Color) -> Self {
        match side {
            White => Self((self.0 & !BitBoard::FILE_H.0) << 9),
            Black => Self((self.0 & !BitBoard::FILE_A.0) >> 9),
        }
    }

    /// Returns a new `BitBoard` representing the squares diagonally `"up-right"` from the current positions,
    /// from the perspective of a specific color known at compile time.
    ///
    /// For [`White`], this corresponds to a shift one rank forward and one file to the right.  
    /// For [`Black`], this corresponds to a shift one rank backward and one file to the left.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White pawn on E4 attacks diagonally up-right to F5
    /// let bitboard = BitBoard(1 << Square::E4 as u64);
    /// let result = bitboard.up_right_for::<{ Color::White as usize }>();
    /// assert_eq!(result, BitBoard(1 << Square::F5 as u64));
    ///
    /// // A Black pawn on D5 attacks diagonally up-right to C4 (from Black's perspective)
    /// let bitboard = BitBoard(1 << Square::D5 as u64);
    /// let result = bitboard.up_right_for::<{ Color::Black as usize }>();
    /// assert_eq!(result, BitBoard(1 << Square::C4 as u64));
    /// ```
    ///
    /// # Expected behavior
    ///
    /// Squares on the edge files are cleared to avoid horizontal wrap-around:
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // A White piece on file H cannot move diagonally up-right
    /// let bitboard = BitBoard(1 << Square::H4 as u64);
    /// let result = bitboard.up_right_for::<{ Color::White as usize }>();
    /// assert_eq!(result, BitBoard::EMPTY);
    ///
    /// // A Black piece on file A cannot move diagonally up-right
    /// let bitboard = BitBoard(1 << Square::A6 as u64);
    /// let result = bitboard.up_right_for::<{ Color::Black as usize }>();
    /// assert_eq!(result, BitBoard::EMPTY);
    /// ```
    #[inline(always)]
    pub const fn up_right_for<const COLOR: usize>(self) -> Self {
        if COLOR == White as usize {
            Self((self.0 & !BitBoard::FILE_H.0) << 9)
        } else {
            Self((self.0 & !BitBoard::FILE_A.0) >> 9)
        }
    }

    /// Returns `true` if the `BitBoard` is empty (i.e., no bits are set), otherwise returns `false`.
    ///
    /// An empty `BitBoard` means that no squares are occupied — all 64 bits are zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// // An explicitly empty BitBoard
    /// let empty = BitBoard::EMPTY;
    /// assert!(empty.is_empty());
    ///
    /// // A BitBoard with a single square set is not empty
    /// let bitboard = BitBoard(1 << Square::E4 as u64);
    /// assert!(!bitboard.is_empty());
    ///
    /// // BitBoard cleared by an operation becomes empty
    /// let occupied = BitBoard(1 << Square::D2 as u64);
    /// let cleared = occupied.pop_square(Square::D2);
    /// assert!(cleared.is_empty());
    /// ```
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
