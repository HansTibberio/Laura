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
use core::str::FromStr;

use crate::{BitBoard, Color, File, Rank, SquareDocs};

SquareDocs! {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8
}

/// Parse a square from its algebraic notation, e.g., "e4" or "g5".
impl FromStr for Square {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err("Invalid Square!");
        };

        let index: usize = Self::SQUARE_NAMES
            .iter()
            .position(|&tgt| tgt == s)
            .ok_or("Invalid Square Name!")?;

        Ok(Square::from_index(index))
    }
}

/// Implement `Display` to allow squares to be printed in their standard format (e.g., "e4").
impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::SQUARE_NAMES[*self as usize])?;
        Ok(())
    }
}

impl Square {
    /// Total number of squares on a chessboard (8x8 = 64).
    pub const NUM_SQUARES: usize = 64;

    /// Create a [`Square`] from a [`File`] (column) and [`Rank`] (row).
    /// The index is calculated by shifting the rank and XORing with the file.
    #[inline(always)]
    pub const fn from_file_rank(file: File, rank: Rank) -> Self {
        let index: u8 = ((rank as u8) << 3) ^ (file as u8);
        unsafe { transmute(index & 63) }
    }

    /// Convert an index (0-63) to a [`Square`].
    /// # Example
    /// ```
    /// # use laura_core::*;
    /// let square = Square::from_index(42);
    /// assert_eq!(square, Square::C6);
    /// ```
    #[inline(always)]
    pub const fn from_index(index: usize) -> Self {
        unsafe { transmute(index as u8 & 63) }
    }

    /// Convert a [`Square`] to its index (0 for A1, 63 for H8).
    #[inline(always)]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    /// Convert a [`Square`] to a [`BitBoard`]
    /// # Example
    /// ```
    /// # use laura_core::*;
    /// let square = Square::F3;
    /// assert_eq!(square.to_bitboard(), BitBoard(2097152));
    /// ```
    #[inline(always)]
    pub const fn to_bitboard(self) -> BitBoard {
        BitBoard(1u64 << self.to_index())
    }

    /// Get the rank (row) of the square.
    #[inline(always)]
    pub const fn rank(self) -> Rank {
        unsafe { transmute((self as u8 >> 3) & 7) }
    }

    /// Get the file (column) of the square.
    #[inline(always)]
    pub const fn file(self) -> File {
        unsafe { transmute(self as u8 & 7) }
    }

    /// Get the square one rank down from original (towards rank 1).
    /// Wrap linear over the Square enum (H1.down() = H8)
    #[inline(always)]
    pub const fn down(self) -> Self {
        unsafe { transmute((self as u8).wrapping_sub(8) & 63) }
    }

    /// Get the square one rank up from original (towards rank 8).
    /// Wrap linear over the Square enum (A8.up() = A1)
    #[inline(always)]
    pub const fn up(self) -> Self {
        unsafe { transmute((self as u8 + 8) & 63) }
    }

    /// Get the square one file to the left from original (towards file A).
    /// Wrap linear over the Square enum (A4.left() = H3)
    #[inline(always)]
    pub const fn left(self) -> Self {
        unsafe { transmute((self as u8).wrapping_sub(1) & 63) }
    }

    /// Get the square one file to the right from original (towards file H).
    /// Wrap linear over the Square enum (H4.right() = A5)
    #[inline(always)]
    pub const fn right(self) -> Self {
        unsafe { transmute((self as u8 + 1) & 63) }
    }

    /// Get the square forwards depending on the color (White moves up, Black moves down).
    #[inline(always)]
    pub const fn forward(self, color: Color) -> Self {
        match color {
            Color::White => self.up(),
            Color::Black => self.down(),
        }
    }

    /// Get the square backwards depending on the color (White moves down, Black moves up).
    #[inline(always)]
    pub const fn backward(self, color: Color) -> Self {
        match color {
            Color::White => self.down(),
            Color::Black => self.up(),
        }
    }

    /// Get the square one file to the right from original.
    /// Considering the given side's perspective.
    #[inline(always)]
    pub const fn right_color(self, color: Color) -> Self {
        match color {
            Color::White => self.right(),
            Color::Black => self.left(),
        }
    }

    /// Get the square one file to the left from original.
    /// Considering the given side's perspective.
    #[inline(always)]
    pub const fn left_color(self, color: Color) -> Self {
        match color {
            Color::White => self.left(),
            Color::Black => self.right(),
        }
    }

    /// Returns the algebraic notation of the square.
    ///
    /// # Example
    /// ```
    /// # use laura_core::*;
    /// let square = Square::A1;
    /// assert_eq!(square.to_str(), "a1");
    /// ```
    #[inline]
    pub fn to_str(&self) -> &'static str {
        Self::SQUARE_NAMES[*self as usize]
    }

    const SQUARE_NAMES: [&'static str; Self::NUM_SQUARES] = [
        "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2", "f2", "g2",
        "h2", "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4", "c4", "d4", "e4", "f4",
        "g4", "h4", "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "a6", "b6", "c6", "d6", "e6",
        "f6", "g6", "h6", "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8",
        "e8", "f8", "g8", "h8",
    ];
}
