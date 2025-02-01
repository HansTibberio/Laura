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

use std::fmt;

use crate::Color;

/// Enum representing the different types of chess pieces.
#[derive(PartialEq, Eq, Ord, PartialOrd, Copy, Clone, Debug, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
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
    pub const PAWN: usize = 0;
    pub const KNIGHT: usize = 1;
    pub const BISHOP: usize = 2;
    pub const ROOK: usize = 3;
    pub const QUEEN: usize = 4;
    pub const KING: usize = 5;

    /// Returns a `PieceType` from a given index without bounds checking.
    /// 
    /// ### Safety
    /// This is an unsafe operation as it directly converts the index to `PieceType`.
    #[inline]
    pub const unsafe fn from_index_unchecked(index: u8) -> Self {
        std::mem::transmute(index)
    }
}

/// Enum representing all possible chess pieces, combining both color and piece type.
/// The first six are White pieces, and the last six are Black pieces.
#[rustfmt::skip]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[repr(u8)]
pub enum Piece {
    WP, WN, WB, WR, WQ, WK,
    BP, BN, BB, BR, BQ, BK,
}

/// Implementing `Display` for `Piece` to print the piece as a single character.
impl fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            _ => Err("Invalid character for chess piece"),
        }
    }
}

/// A 2D array representing the pieces available for promotion in chess.
pub const PROM_PIECES: [[Piece; 4]; 2] = [
    [Piece::WN, Piece::WB, Piece::WR, Piece::WQ],
    [Piece::BN, Piece::BB, Piece::BR, Piece::BQ],
];

impl Piece {
    /// The number of unique pieces on chess.
    pub const COUNT: usize = 6;

    /// Total number of pieces on chess (6x2 = 12).
    pub const NUM_PIECES: usize = 12;

    /// Creates a new `Piece` given a `PieceType` and a `Color`.
    /// The piece is determined by the combination of the piece type and the color.
    #[inline(always)]
    pub const fn new(piece_type: PieceType, color: Color) -> Self {
        let index: u8 = color as u8 * 6 + piece_type as u8;
        unsafe { std::mem::transmute(index) }
    }

    /// Returns the `PieceType` index of the `Piece` as a usize.
    /// This index is used to identify the piece type within the range of 0-5.
    #[inline(always)]
    pub const fn piece_index(self) -> usize {
        let index: u8 = self as u8 % 6;
        index as usize
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
            Some(unsafe { std::mem::transmute::<u8, Piece>(index as u8 & 15) })
        } else {
            None
        }
    }

    /// Returns the `Color` of the `Piece` (either `White` or `Black`).
    #[inline(always)]
    pub const fn color(self) -> Color {
        if (self as u8) < 6 {
            Color::White
        } else {
            Color::Black
        }
    }

    /// Returns the `PieceType` of the `Piece` (e.g., Pawn, Knight, etc.).
    #[inline(always)]
    pub const fn piece_type(self) -> PieceType {
        let index: u8 = self as u8 % 6;
        unsafe { PieceType::from_index_unchecked(index) }
    }

    /// Returns the corresponding character for the `Piece`.
    /// Uppercase for white pieces, lowercase for black pieces.
    #[inline]
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

#[test]
fn test() {
    let piece: Piece = Piece::new(PieceType::King, Color::White);
    println!(
        "Char: '{}' Color: {}, Type: {}",
        piece,
        piece.color(),
        piece.piece_type()
    );

    let piece: Option<Piece> = Piece::from_index(12);
    println!("{:?}", piece);
}

#[test]
fn test_from() {
    let c: char = 'N';
    let piece: Piece = Piece::try_from(c).unwrap();
    println!(
        "Char: '{}' Color: {}, Type: {}",
        piece,
        piece.color(),
        piece.piece_type()
    );
}

#[test]
fn test_piece() {
    let piece: usize = Piece::piece_index(Piece::WN);
    println!("{}", piece)
}
