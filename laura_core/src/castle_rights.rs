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
use std::str::FromStr;

use crate::{BitBoard, Color, File, MoveType, Square};

// This implementation is based on the approach used in Carp, which licensed under the GPLv3. 
// Source: https://github.com/dede1751/carp/blob/main/chess/src/castle.rs 

/// `CastleRights` represents the castling rights of both players (White and Black)
/// using a bitmask stored in a `u8`. It tracks the availability of kingside and queenside
/// castling rights for both sides.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub struct CastleRights(u8);

/// Implement the `FromStr` trait for `CastleRights`.
/// This allows parsing a string into a `CastleRights` object.
impl FromStr for CastleRights {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rights: u8 = 0;

        for ch in s.chars() {
            match ch {
                'K' => rights |= CASTLE_WK_MASK,
                'Q' => rights |= CASTLE_WQ_MASK,
                'k' => rights |= CASTLE_BK_MASK,
                'q' => rights |= CASTLE_BQ_MASK,
                '-' => {
                    if s.len() != 1 {
                        return Err("Invalid format for castling rights");
                    }
                    rights = 0;
                }
                _ => return Err("Invalid character in castling rights"),
            }
        }

        Ok(CastleRights(rights))
    }
}

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

/// Index representing the king-side castle in arrays.
pub const KING_SIDE: usize = 0;
/// Index representing the queen-side castle in arrays.
pub const QUEEN_SIDE: usize = 1;

/// Array mapping castle types to their corresponding move types.
pub const CASTLE_TYPE: [MoveType; 2] = [MoveType::KingCastle, MoveType::QueenCastle];

/// Array defining the starting squares for castling moves for white and black.
pub const SOURCE: [Square; 2] = [Square::E1, Square::E8];

/// Array defining the intermediate squares crossed during castling.
/// The first dimension represents the king-side (0) or queen-side (1).
/// The second dimension represents white (0) or black (1).
pub const MEDIUM: [[Square; 2]; 2] = [[Square::F1, Square::F8], [Square::D1, Square::D8]];

/// Array defining the destination squares for castling moves.
/// The structure mirrors that of `MEDIUM`.
pub const DESTINATION: [[Square; 2]; 2] = [[Square::G1, Square::G8], [Square::C1, Square::C8]];

/// Array defining bitboard representations of squares that must be empty
/// for castling to be valid. The first dimension represents the king-side (0)
/// or queen-side (1), and the second represents white (0) or black (1).
pub const PRESENCE: [[BitBoard; 2]; 2] = [
    [BitBoard(0x0000000000000060), BitBoard(0x6000000000000000)],
    [BitBoard(0x000000000000000E), BitBoard(0x0E00000000000000)],
];

/// Array defining the castle rights mask for each square on the board.
/// Each value specifies the valid castling rights for the corresponding square.
const CASTLE_RIGHTS_MASK: [u8; Square::NUM_SQUARES] = [
    NOT_WQ_RIGHTS,    //  A1
    ALL_CASTLE,       //  B1
    ALL_CASTLE,       //  C1
    ALL_CASTLE,       //  D1
    NOT_WHITE_RIGHTS, //  E1
    ALL_CASTLE,       //  F1
    ALL_CASTLE,       //  G1
    NOT_WK_RIGHTS,    //  H1
    ALL_CASTLE,       //  A2
    ALL_CASTLE,       //  B2
    ALL_CASTLE,       //  C2
    ALL_CASTLE,       //  D2
    ALL_CASTLE,       //  E2
    ALL_CASTLE,       //  F2
    ALL_CASTLE,       //  G2
    ALL_CASTLE,       //  H2
    ALL_CASTLE,       //  A3
    ALL_CASTLE,       //  B3
    ALL_CASTLE,       //  C3
    ALL_CASTLE,       //  D3
    ALL_CASTLE,       //  E3
    ALL_CASTLE,       //  F3
    ALL_CASTLE,       //  G3
    ALL_CASTLE,       //  H3
    ALL_CASTLE,       //  A4
    ALL_CASTLE,       //  B4
    ALL_CASTLE,       //  C4
    ALL_CASTLE,       //  D4
    ALL_CASTLE,       //  E4
    ALL_CASTLE,       //  F4
    ALL_CASTLE,       //  G4
    ALL_CASTLE,       //  H4
    ALL_CASTLE,       //  A5
    ALL_CASTLE,       //  B5
    ALL_CASTLE,       //  C5
    ALL_CASTLE,       //  D5
    ALL_CASTLE,       //  E5
    ALL_CASTLE,       //  F5
    ALL_CASTLE,       //  G5
    ALL_CASTLE,       //  H5
    ALL_CASTLE,       //  A6
    ALL_CASTLE,       //  B6
    ALL_CASTLE,       //  C6
    ALL_CASTLE,       //  D6
    ALL_CASTLE,       //  E6
    ALL_CASTLE,       //  F6
    ALL_CASTLE,       //  G6
    ALL_CASTLE,       //  H6
    ALL_CASTLE,       //  A7
    ALL_CASTLE,       //  B7
    ALL_CASTLE,       //  C7
    ALL_CASTLE,       //  D7
    ALL_CASTLE,       //  E7
    ALL_CASTLE,       //  F7
    ALL_CASTLE,       //  G7
    ALL_CASTLE,       //  H7
    NOT_BQ_RIGHTS,    //  A8
    ALL_CASTLE,       //  B8
    ALL_CASTLE,       //  C8
    ALL_CASTLE,       //  D8
    NOT_BLACK_RIGHTS, //  E8
    ALL_CASTLE,       //  F8
    ALL_CASTLE,       //  G8
    NOT_BK_RIGHTS,    //  H8
];

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
    #[inline]
    pub const fn update(self, src: Square, dest: Square) -> CastleRights {
        CastleRights(
            self.0 & CASTLE_RIGHTS_MASK[src.to_index()] & CASTLE_RIGHTS_MASK[dest.to_index()],
        )
    }
}

#[test]
fn castling_test() {
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
fn rook_castling_test() {
    let mv: (Square, Square) = get_rook_castling(Square::C1);
    println!("{:?}", mv);
}

#[test]
fn test_from_string() {
    let castle_rights: CastleRights = CastleRights::from_str("Kk").unwrap();
    assert_eq!(castle_rights.has_kingside(Color::White), true);
    assert_eq!(castle_rights.has_queenside(Color::White), false);
    assert_eq!(castle_rights.has_kingside(Color::Black), true);
    assert_eq!(castle_rights.has_queenside(Color::Black), false);
    println!("{}", castle_rights);
}
