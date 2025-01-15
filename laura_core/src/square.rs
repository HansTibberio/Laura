use std::fmt;
use std::mem::transmute;
use std::str::FromStr;

use crate::{BitBoard, Color, File, Rank};

/// Enum representing each square on a chessboard, from A1 to H8.
/// The squares are ordered by rank (rows) and file (columns), with A1 as the bottom-left and H8 as the top-right.
#[derive(PartialEq, Ord, Eq, PartialOrd, Copy, Clone, Debug, Hash)]
#[repr(u8)]
#[rustfmt::skip]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
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
        let square: String = String::from(Self::SQUARE_NAMES[*self as usize]);
        write!(f, "{square}")
    }
}

impl Square {
    /// Total number of squares on a chessboard (8x8 = 64).
    pub const NUM_SQUARES: usize = 64;

    /// Create a `Square` from a `File` (column) and `Rank` (row).
    /// The index is calculated by shifting the rank and XORing with the file.
    #[inline]
    pub const fn from_file_rank(file: File, rank: Rank) -> Self {
        let index: u8 = (rank as u8) << 3 ^ (file as u8);
        unsafe { transmute(index & 63) }
    }

    /// Convert an index (0-63) to a `Square`.
    #[inline]
    pub const fn from_index(index: usize) -> Self {
        unsafe { transmute(index as u8 & 63) }
    }

    /// Convert a `Square` to its index (0 for A1, 63 for H8).
    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    /// Convert a `Square` to `Bitboard`
    #[inline]
    pub const fn to_bitboard(self) -> BitBoard {
        BitBoard(1u64 << self.to_index())
    }

    /// Get the rank (row) of the square.
    #[inline]
    pub const fn rank(self) -> Rank {
        unsafe { transmute((self as u8 >> 3) & 7) }
    }

    /// Get the file (column) of the square.
    #[inline]
    pub const fn file(self) -> File {
        unsafe { transmute(self as u8 & 7) }
    }

    /// Get the square one rank down from original (towards rank 1).
    /// Wrap linear over the Square enum (H1.down() = H8)
    #[inline]
    pub const fn down(self) -> Self {
        unsafe { transmute((self as u8).wrapping_sub(8) & 63) }
    }

    /// Get the square one rank up from original (towards rank 8).
    /// Wrap linear over the Square enum (A8.up() = A1)
    #[inline]
    pub const fn up(self) -> Self {
        unsafe { transmute((self as u8 + 8) & 63) }
    }

    /// Get the square one file to the left from original (towards file A).
    /// Wrap linear over the Square enum (A4.left() = H3)
    #[inline]
    pub const fn left(self) -> Self {
        unsafe { transmute((self as u8).wrapping_sub(1) & 63) }
    }

    /// Get the square one file to the right from original (towards file H).
    /// Wrap linear over the Square enum (H4.right() = A5)
    #[inline]
    pub const fn right(self) -> Self {
        unsafe { transmute((self as u8 + 1) & 63) }
    }

    /// Get the square forwards depending on the color (White moves up, Black moves down).
    #[inline]
    pub const fn forward(self, color: Color) -> Self {
        match color {
            Color::White => self.up(),
            Color::Black => self.down(),
        }
    }

    /// Get the square backwards depending on the color (White moves down, Black moves up).
    #[inline]
    pub const fn backward(self, color: Color) -> Self {
        match color {
            Color::White => self.down(),
            Color::Black => self.up(),
        }
    }

    const SQUARE_NAMES: [&'static str; Self::NUM_SQUARES] = [
        "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "a2", "b2", "c2", "d2", "e2", "f2", "g2",
        "h2", "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "a4", "b4", "c4", "d4", "e4", "f4",
        "g4", "h4", "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "a6", "b6", "c6", "d6", "e6",
        "f6", "g6", "h6", "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a8", "b8", "c8", "d8",
        "e8", "f8", "g8", "h8",
    ];
}

#[test]
fn test() {
    let square: Square = Square::from_str("f5").unwrap();
    println!(
        "Square: {}, File: {}, Rank: {}, Index: {}",
        square,
        square.file(),
        square.rank(),
        square.to_index()
    );
    println!(
        "Up: {}, Down: {}, Left: {}, Right: {}",
        square.up(),
        square.down(),
        square.left(),
        square.right()
    );
    println!(
        "Forward: {}, Backward: {}",
        square.forward(Color::White),
        square.backward(Color::White)
    );
}

#[test]
fn c6_square_to_bitboard() {
    let square = Square::from_index(42);
    assert_eq!(square, Square::C6);
    println!("{}", square.to_bitboard())
}
