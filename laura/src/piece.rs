use crate::color::Color;
use std::fmt;

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
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
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
    
    /// Returns a `PieceType` from a given index without bounds checking.
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

impl Piece {

    /// Creates a new `Piece` given a `PieceType` and a `Color`.
    /// The piece is determined by the combination of the piece type and the color.
    #[inline]
    pub const fn new(piece_type: PieceType, color: Color) -> Self {
        let index: u8 = color as u8 * 6 + piece_type as u8;
        unsafe { std::mem::transmute(index) }
    }

     /// Converts a usize index to a `Piece`, if the index is valid (less than 12).
    #[inline]
    pub const fn from_index(index: usize) -> Option<Self> {
        if index < 12 {
        Some( unsafe { std::mem::transmute(index as u8 & 15) } )
        } else {
            None
        }
    }

    /// Returns the `Color` of the `Piece` (either `White` or `Black`).
    #[inline]
    pub const fn color(self) -> Color {
        if (self as u8) < 6 {
            Color::White
        } else {
            Color::Black
        }
    }

    /// Returns the `PieceType` of the `Piece` (e.g., Pawn, Knight, etc.).
    #[inline]
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
    println!("Char: '{}' Color: {}, Type: {}", piece, piece.color(), piece.piece_type());

    let piece: Option<Piece> = Piece::from_index(12);
    println!("{:?}", piece);
}