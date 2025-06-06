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

use crate::Color;
use core::fmt;

/// Enum representing the different types of chess pieces.
#[derive(PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub enum PieceType {
    /// The pawn piece. Typically moves forward one square at a time, but captures diagonally.
    Pawn,

    /// The knight piece. Moves in an "L" shape: two squares in one direction and one square
    /// perpendicular, or vice versa. The knight can jump over other pieces.
    Knight,

    /// The bishop piece. Moves diagonally any number of squares, but only on one color of square.
    Bishop,

    /// The rook piece. Moves horizontally or vertically any number of squares.
    Rook,

    /// The queen piece. Combines the moves of both the rook and bishop, moving horizontally,
    /// vertically, or diagonally any number of squares.
    Queen,

    /// The king piece. Moves one square in any direction: horizontally, vertically, or diagonally.
    /// It is the most important piece, and its capture (checkmate) ends the game.
    King,
}

/// Implementing `Display` for `PieceType` to allow converting the enum into a human-readable string.
impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Pawn => write!(f, "Pawn"),
            Self::Knight => write!(f, "Knight"),
            Self::Bishop => write!(f, "Bishop"),
            Self::Rook => write!(f, "Rook"),
            Self::Queen => write!(f, "Queen"),
            Self::King => write!(f, "King"),
        }
    }
}

impl PieceType {
    /// Represents the pawn piece index (0).
    pub const PAWN: usize = 0;

    /// Represents the knight piece index (1).
    pub const KNIGHT: usize = 1;

    /// Represents the bishop piece index (2).
    pub const BISHOP: usize = 2;

    /// Represents the rook piece index (3).
    pub const ROOK: usize = 3;

    /// Represents the queen piece index (4).
    pub const QUEEN: usize = 4;

    /// Represents the king piece index (5).
    pub const KING: usize = 5;

    /// Returns a `PieceType` from a given index without bounds checking.
    ///
    /// # Safety
    /// This is an unsafe operation as it directly converts the index to `PieceType`.
    #[inline(always)]
    pub const unsafe fn from_index_unchecked(index: u8) -> Self {
        core::mem::transmute(index)
    }

    /// Returns the corresponding character for the `PieceType`.
    #[inline(always)]
    pub const fn to_char(&self) -> char {
        match self {
            Self::Pawn => 'P',
            Self::Knight => 'N',
            Self::Bishop => 'B',
            Self::Rook => 'R',
            Self::Queen => 'Q',
            Self::King => 'K',
        }
    }
}

/// Enum representing all possible chess pieces, combining both color and piece type.
/// The first six are White pieces, and the last six are Black pieces.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum Piece {
    /// `WP` represents a White Pawn.
    WP,

    /// `WN` represents a White Knight.
    WN,

    /// `WB` represents a White Bishop.
    WB,

    /// `WR` represents a White Rook.
    WR,

    /// `WQ` represents a White Queen.
    WQ,

    /// `WK` represents a White King.
    WK,

    /// `BP` represents a Black Pawn.
    BP,

    /// `BN` represents a Black Knight.
    BN,

    /// `BB` represents a Black Bishop.
    BB,

    /// `BR` represents a Black Rook.
    BR,

    /// `BQ` represents a Black Queen.
    BQ,

    /// `BK` represents a Black King.
    BK,
}

/// Implementing `Display` for `Piece` to print the piece as a single character.
impl fmt::Display for Piece {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

/// Attempt to convert a character into a `Piece`.
/// Returns an error if the character does not correspond to a valid chess piece.
impl TryFrom<char> for Piece {
    type Error = &'static str;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'P' => Ok(Piece::WP),
            'N' => Ok(Piece::WN),
            'B' => Ok(Piece::WB),
            'R' => Ok(Piece::WR),
            'Q' => Ok(Piece::WQ),
            'K' => Ok(Piece::WK),
            'p' => Ok(Piece::BP),
            'n' => Ok(Piece::BN),
            'b' => Ok(Piece::BB),
            'r' => Ok(Piece::BR),
            'q' => Ok(Piece::BQ),
            'k' => Ok(Piece::BK),
            _ => Err("Invalid piece character"),
        }
    }
}

/// A 2D array representing the pieces available for promotion in chess.
pub(crate) const PROM_PIECES: [[Piece; 4]; 2] = [
    [Piece::WN, Piece::WB, Piece::WR, Piece::WQ],
    [Piece::BN, Piece::BB, Piece::BR, Piece::BQ],
];

impl Piece {
    /// The number of unique pieces on chess.
    pub const COUNT: usize = 6;

    /// Total number of pieces on chess (6x2 = 12).
    pub const NUM_PIECES: usize = 12;

    /// Creates a new `Piece` given a [`PieceType`] and a [`Color`].
    /// The piece is determined by the combination of the piece type and the color.
    #[inline(always)]
    pub const fn new(piece_type: PieceType, color: Color) -> Self {
        let index: u8 = color as u8 * 6 + piece_type as u8;
        unsafe { core::mem::transmute(index) }
    }

    /// Returns the [`PieceType`] index of the `Piece` as a usize.
    /// This index is used to identify the piece type within the range of 0-5.
    #[inline(always)]
    pub const fn piece_index(self) -> usize {
        (self as u8 % 6) as usize
    }

    /// Returns the index of the `Piece` as a usize.
    #[inline(always)]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    /// Converts a usize index to a `Piece`, if the index is valid (less than 12).
    #[inline(always)]
    pub const fn from_index(index: usize) -> Option<Self> {
        if index < 12 {
            Some(unsafe { core::mem::transmute::<u8, Piece>(index as u8 & 15) })
        } else {
            None
        }
    }

    /// Returns the [`Color`] of the `Piece` (either `White` or `Black`).
    #[inline(always)]
    pub const fn color(self) -> Color {
        if (self as u8) < 6 {
            Color::White
        } else {
            Color::Black
        }
    }

    /// Returns the [`PieceType`] of the `Piece` (e.g., Pawn, Knight, etc.).
    #[inline(always)]
    pub const fn piece_type(self) -> PieceType {
        let index: u8 = self as u8 % 6;
        unsafe { PieceType::from_index_unchecked(index) }
    }

    /// Returns the corresponding character for the `Piece`.
    /// Uppercase for white pieces, lowercase for black pieces.
    #[inline(always)]
    pub const fn to_char(&self) -> char {
        match self {
            Self::WP => 'P',
            Self::WN => 'N',
            Self::WB => 'B',
            Self::WR => 'R',
            Self::WQ => 'Q',
            Self::WK => 'K',
            Self::BP => 'p',
            Self::BN => 'n',
            Self::BB => 'b',
            Self::BR => 'r',
            Self::BQ => 'q',
            Self::BK => 'k',
        }
    }
}
