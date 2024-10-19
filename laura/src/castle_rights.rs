use std::fmt;

use crate::color::Color;
use crate::square::Square;
use crate::file::File;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub struct CastleRights(u8);

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

const CASTLE_WK_MASK: u8 = 0b1000;
const CASTLE_WQ_MASK: u8 = 0b0100;
const CASTLE_BK_MASK: u8 = 0b0010;
const CASTLE_BQ_MASK: u8 = 0b0001;

const KINGSIDE_CASTLE: [u8; 2] = [CASTLE_WK_MASK, CASTLE_BK_MASK];
const QUEENSIDE_CASTLE: [u8; 2] = [CASTLE_WQ_MASK, CASTLE_BQ_MASK];

const ALL_CASTLE: u8 = 0b1111;
const NOT_WK_RIGHTS: u8 = ALL_CASTLE ^ CASTLE_WK_MASK;
const NOT_WQ_RIGHTS: u8 = ALL_CASTLE ^ CASTLE_WQ_MASK;
const NOT_BK_RIGHTS: u8 = ALL_CASTLE ^ CASTLE_BK_MASK;
const NOT_BQ_RIGHTS: u8 = ALL_CASTLE ^ CASTLE_BQ_MASK;
const NOT_WHITE_RIGHTS: u8 = NOT_WK_RIGHTS & NOT_WQ_RIGHTS;
const NOT_BLACK_RIGHTS: u8 = NOT_BK_RIGHTS & NOT_BQ_RIGHTS;

pub const fn get_rook_castling(dest: Square) -> (Square, Square) {
    match dest.file() {
        File::C => (dest.left().left(), dest.right()),
        File::G => (dest.right(), dest.left()),
        _ => unreachable!(),
    }
}

impl CastleRights {
    
    #[inline]
    pub const fn null() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn to_index(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub const fn has_kingside(self, color: Color) -> bool {
        self.0 & KINGSIDE_CASTLE[color as usize] != 0
    }

    #[inline]
    pub const fn has_queenside(self, color: Color) -> bool {
        self.0 & QUEENSIDE_CASTLE[color as usize] != 0
    }

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