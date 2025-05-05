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

use crate::BitBoard;
use core::fmt;
use core::mem::transmute;

const RANK_BITBOARDS: [BitBoard; 8] = [
    BitBoard::RANK_1,
    BitBoard::RANK_2,
    BitBoard::RANK_3,
    BitBoard::RANK_4,
    BitBoard::RANK_5,
    BitBoard::RANK_6,
    BitBoard::RANK_7,
    BitBoard::RANK_8,
];

/// Enum representing the ranks (rows) on a chessboard.
/// Ranks are numbered from 'One' (1) to 'Eight' (8).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
#[repr(u8)]
pub enum Rank {
    /// Rank One
    One,
    /// Rank Two
    Two,
    /// Rank Three
    Three,
    /// Rank Four
    Four,
    /// Rank Five
    Five,
    /// Rank Six
    Six,
    /// Rank Seven
    Seven,
    /// Rank Eight
    Eight,
}

/// Implementing `Display` for `Rank` to convert the enum to a string representation (One-Eight).
impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::One => write!(f, "One"),
            Self::Two => write!(f, "Two"),
            Self::Three => write!(f, "Three"),
            Self::Four => write!(f, "Four"),
            Self::Five => write!(f, "Five"),
            Self::Six => write!(f, "Six"),
            Self::Seven => write!(f, "Seven"),
            Self::Eight => write!(f, "Eight"),
        }
    }
}

impl Rank {
    /// Total number of ranks (8 in standard chess).
    pub const NUM_RANKS: usize = 8;
    /// Array containing all possible ranks (One to Eight).
    pub const ALL: [Self; 8] = [
        Self::One,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
    ];

    /// Converts an index (0-7) to the corresponding `Rank`.
    #[inline(always)]
    pub const fn from_index(index: usize) -> Rank {
        unsafe { transmute(index as u8 & 7) }
    }

    /// Converts a `Rank` into its corresponding index (0 for One, 7 for Eight).
    #[inline(always)]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    /// Gets rank above, wraps Eight->One
    #[inline(always)]
    pub const fn up(self) -> Self {
        unsafe { transmute((self as u8 + 1) & 7) }
    }

    /// Gets rank below, wraps One->Eight
    #[inline(always)]
    pub const fn down(self) -> Self {
        unsafe { transmute((self as u8).wrapping_sub(1) & 7) }
    }

    /// Gets the Bitboard of the rank.
    #[inline(always)]
    pub const fn to_bitboard(self) -> BitBoard {
        RANK_BITBOARDS[self.to_index()]
    }
}
