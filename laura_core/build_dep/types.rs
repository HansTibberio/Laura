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

//  Types definition for Magic BitBoards generation.

/// A `BitBoard` represents a 64-bit chessboard where each bit corresponds to a square.
/// It is useful for efficiently representing and manipulating chess positions.
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(pub u64);

/// Macro to implement bitwise operators for a type.
///
/// This macro generates implementations for bitwise operations such as `&`, `|`, and `^` for the `BitBoard` type.
/// The generated code allows `BitBoard` objects to be used in bitwise operations with another `BitBoard`.
macro_rules! impl_bitwise_op {
    ($trait:ident, $func:ident) => {
        impl std::ops::$trait for BitBoard {
            type Output = Self;

            fn $func(self, other: Self) -> BitBoard {
                Self(std::ops::$trait::$func(self.0, other.0))
            }
        }
    };
}

/// Macro to implement bitwise assignment operators for a type.
///
/// This macro generates implementations for bitwise assignment operations such as `&=`, `|=`, and `^=` for the `BitBoard` type.
/// The generated code allows `BitBoard` objects to perform bitwise assignment operations with another `BitBoard`.
macro_rules! impl_bitwise_assign_op {
    ($trait:ident, $func:ident) => {
        impl std::ops::$trait for BitBoard {
            fn $func(&mut self, other: Self) {
                std::ops::$trait::$func(&mut self.0, other.0)
            }
        }
    };
}

// Implementing bitwise operators for BitBoard
impl_bitwise_op!(BitAnd, bitand);
impl_bitwise_op!(BitOr, bitor);
impl_bitwise_op!(BitXor, bitxor);

// Implementing bitwise assignment operators for BitBoard
impl_bitwise_assign_op!(BitAndAssign, bitand_assign);
impl_bitwise_assign_op!(BitOrAssign, bitor_assign);
impl_bitwise_assign_op!(BitXorAssign, bitxor_assign);

/// Implements the `Not` trait for `BitBoard`, allowing the bitwise NOT operation `!`.
/// The bitwise NOT flips all bits in the `BitBoard`, effectively inverting the board state.
impl std::ops::Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

/// Implements `Iterator` for `BitBoard`, allowing iteration over the set squares.
/// Each call to `next` returns the next `Square` that is set (i.e., the next '1' bit)
impl Iterator for BitBoard {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            let square: Square = self.to_square();
            self.0 &= self.0 - 1;

            Some(square)
        }
    }
}

/// Methods for the `BitBoard` struct, including utilities for manipulating bits and interacting with squares.
impl BitBoard {
    pub const EMPTY: BitBoard = BitBoard(0);
    pub const FULL: BitBoard = BitBoard(0xFFFF_FFFF_FFFF_FFFF);

    pub const fn set_square(self, square: Square) -> Self {
        Self(self.0 | 1u64 << square.to_index())
    }

    pub const fn get_square(self, square: Square) -> bool {
        self.0 & (1u64 << square.to_index()) != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn count_bits(self) -> u32 {
        self.0.count_ones()
    }

    pub const fn to_square(self) -> Square {
        unsafe { std::mem::transmute((self.0.trailing_zeros() as u8) & 63) }
    }

    pub fn set_blockers(self, index: usize) -> BitBoard {
        self.into_iter()
            .enumerate()
            .filter(|(count, _)| index & (1 << count) != 0)
            .fold(BitBoard::EMPTY, |bitboard: BitBoard, (_, square)| {
                bitboard | square.to_bitboard()
            })
    }
}

/// Enum representing each square on a chessboard, from A1 to H8.
/// The squares are ordered by rank (rows) and file (columns), with A1 as the bottom-left and H8 as the top-right.
#[derive(PartialEq, Ord, Eq, PartialOrd, Copy, Clone, Debug, Hash)]
#[repr(u8)]
pub enum Square {
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
}

impl Square {
    pub const NUM_SQUARES: usize = 64;

    pub const fn from_file_rank(file: File, rank: Rank) -> Self {
        let index: u8 = (rank as u8) << 3 ^ (file as u8);
        unsafe { std::mem::transmute(index & 63) }
    }

    pub const fn from_index(index: usize) -> Self {
        unsafe { std::mem::transmute(index as u8 & 63) }
    }

    pub const fn to_index(self) -> usize {
        self as usize
    }

    pub const fn to_bitboard(self) -> BitBoard {
        BitBoard(1u64 << self.to_index())
    }

    pub const fn rank(self) -> Rank {
        unsafe { std::mem::transmute((self as u8 >> 3) & 7) }
    }

    pub const fn file(self) -> File {
        unsafe { std::mem::transmute(self as u8 & 7) }
    }
}

/// Enum representing the files (columns) on a chessboard.
/// Files are labeled from 'A' to 'H'.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
#[repr(u8)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

/// Enum representing the ranks (rows) on a chessboard.
/// Ranks are numbered from 'One' (1) to 'Eight' (8).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
#[repr(u8)]
pub enum Rank {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}
