use std::fmt;

use crate::color::Color;
use crate::square::Square;
use crate::file::File;

/// `CastleRights` represents the castling rights of both players (White and Black)
/// using a bitmask stored in a `u8`. It tracks the availability of kingside and queenside
/// castling rights for both sides.
/// 
/// From Carp: https://github.com/dede1751/carp/blob/main/chess/src/castle.rs
/// 
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub struct CastleRights(u8);

/// Implements the `fmt::Display` trait for the `CastleRights` struct,
/// allowing castling rights to be displayed as a string.
impl fmt::Display for CastleRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s: String = String::from("");

        if self.0 & CASTLE_WK_MASK != 0 {
            s.push('K')
        };
        if self.0 & CASTLE_WQ_MASK != 0 {
            s.push('Q')
        };
        if self.0 & CASTLE_BK_MASK != 0 {
            s.push('k')
        };
        if self.0 & CASTLE_BQ_MASK != 0 {
            s.push('q')
        };
        if s.is_empty() {
            s.push('-')
        };

        write!(f, "{s}")
    }
}


// Constants for castling bitmasks for both White and Black:
/// - `CASTLE_WK_MASK`: White kingside castling (bit 3)
const CASTLE_WK_MASK: u8 = 0b1000;
/// - `CASTLE_WQ_MASK`: White queenside castling (bit 2)
const CASTLE_WQ_MASK: u8 = 0b0100;
/// - `CASTLE_BK_MASK`: Black kingside castling (bit 1)
const CASTLE_BK_MASK: u8 = 0b0010;
/// - `CASTLE_BQ_MASK`: Black queenside castling (bit 0)
const CASTLE_BQ_MASK: u8 = 0b0001;

// Arrays to simplify indexing for kingside and queenside castling rights
// for both White and Black.
const KINGSIDE_CASTLE: [u8; 2] = [CASTLE_WK_MASK, CASTLE_BK_MASK];
const QUEENSIDE_CASTLE: [u8; 2] = [CASTLE_WQ_MASK, CASTLE_BQ_MASK];

// Masks to manipulate and remove specific castling rights.
const ALL_CASTLE: u8 = 0b1111;
const NOT_WK_RIGHTS: u8 = ALL_CASTLE ^ CASTLE_WK_MASK;
const NOT_WQ_RIGHTS: u8 = ALL_CASTLE ^ CASTLE_WQ_MASK;
const NOT_BK_RIGHTS: u8 = ALL_CASTLE ^ CASTLE_BK_MASK;
const NOT_BQ_RIGHTS: u8 = ALL_CASTLE ^ CASTLE_BQ_MASK;
const NOT_WHITE_RIGHTS: u8 = NOT_WK_RIGHTS & NOT_WQ_RIGHTS;
const NOT_BLACK_RIGHTS: u8 = NOT_BK_RIGHTS & NOT_BQ_RIGHTS;

/// Returns the starting and destination squares for a rook during a castling move
/// based on the destination of the king (`dest`).
///
/// - Kingside castling (king moves to G-file): returns (H-file, F-file)
/// - Queenside castling (king moves to C-file): returns (A-file, D-file)
pub const fn get_rook_castling(dest: Square) -> (Square, Square) {
    match dest.file() {
        File::C => (dest.left().left(), dest.right()),
        File::G => (dest.right(), dest.left()),
        _ => unreachable!(),
    }
}

impl CastleRights {

    /// Total number of castling rights for all players.
    pub const NUM_CASTLING_RIGHTS: usize = 16;
    
    /// Creates a new `CastleRights` object with no castling rights (i.e., all rights cleared).
    #[inline(always)]
    pub const fn null() -> Self {
        Self(0)
    }

    /// Converts the castling rights to an index that can be used for array lookups.
    #[inline]
    pub const fn to_index(self) -> usize {
        self.0 as usize
    }

    /// Checks if kingside castling is available for a given color (`Color`).
    #[inline]
    pub const fn has_kingside(self, color: Color) -> bool {
        self.0 & KINGSIDE_CASTLE[color as usize] != 0
    }

    /// Checks if queenside castling is available for a given color (`Color`).
    #[inline]
    pub const fn has_queenside(self, color: Color) -> bool {
        self.0 & QUEENSIDE_CASTLE[color as usize] != 0
    }

    /// Updates the castling rights after a move from `src` to `dest`.
    ///
    /// The castling rights are updated based on the move, potentially clearing the castling
    /// rights if a rook or king has moved from its starting square.
    pub const fn update(self, src: Square, dest: Square) -> CastleRights {
        const CASTLE_RIGHTS_MASK: [u8; Square::NUM_SQUARES] = [
            NOT_WQ_RIGHTS, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, NOT_WHITE_RIGHTS, ALL_CASTLE, ALL_CASTLE, NOT_WK_RIGHTS,
            ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE,
            ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE,
            ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE,
            ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE,
            ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE,
            ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE,
            NOT_BQ_RIGHTS, ALL_CASTLE, ALL_CASTLE, ALL_CASTLE, NOT_BLACK_RIGHTS, ALL_CASTLE, ALL_CASTLE, NOT_BK_RIGHTS
        ];

        let updated: u8 = self.0 & CASTLE_RIGHTS_MASK[src as usize] & CASTLE_RIGHTS_MASK[dest as usize];
        CastleRights(updated)
    }
}

#[test]
fn castling_test(){
    let castle_rights: CastleRights = CastleRights(ALL_CASTLE);
    assert_eq!(castle_rights.has_kingside(Color::White), true);
    assert_eq!(castle_rights.has_queenside(Color::White), true);
    assert_eq!(castle_rights.has_kingside(Color::Black), true);
    assert_eq!(castle_rights.has_queenside(Color::Black), true);
    println!("{}", castle_rights);
    let castle_rights: CastleRights = castle_rights.update(Square::H1, Square::H5);
    let castle_rights: CastleRights = castle_rights.update(Square::E8, Square::E6);
    assert_eq!(castle_rights.has_kingside(Color::White), false);
    assert_eq!(castle_rights.has_queenside(Color::White), true);
    assert_eq!(castle_rights.has_kingside(Color::Black), false);
    assert_eq!(castle_rights.has_queenside(Color::Black), false);
    println!("{}", castle_rights);
}

#[test]
fn rook_castling_test(){
    let mv: (Square, Square) = get_rook_castling(Square::C1);
    println!("{:?}", mv);
}