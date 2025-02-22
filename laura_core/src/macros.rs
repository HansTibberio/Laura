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
use core::ops::{BitAnd, BitOr, BitXor};
use core::ops::{BitAndAssign, BitOrAssign, BitXorAssign};

use crate::{BitBoard, Board};

/// Macro to implement bitwise operators for a type.
///
/// This macro generates implementations for bitwise operations such as `&`, `|`, and `^` for the `BitBoard` type.
/// The generated code allows `BitBoard` objects to be used in bitwise operations with another `BitBoard`.
macro_rules! impl_bitwise_op {
    ($trait:ident, $func:ident) => {
        impl $trait for BitBoard {
            type Output = Self;

            #[inline]
            fn $func(self, other: Self) -> BitBoard {
                Self($trait::$func(self.0, other.0))
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
        impl $trait for BitBoard {
            #[inline]
            fn $func(&mut self, other: Self) {
                $trait::$func(&mut self.0, other.0)
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

/// Macro to generate the docs for the BitBoard constants
#[macro_export]
macro_rules! BitBoardConsts {
    ($($name:ident > $value:expr),* $(,)?) => {
        $(
            #[doc = concat!("BitBoard representing `", stringify!($name), "`.")]
            pub const $name: BitBoard = BitBoard($value);
        )*
    };
}


/// Macro to generate the Square enum and the documentation for each square.
#[macro_export]
macro_rules! SquareDocs {
    ($($square:ident),*) => {
        /// Enum representing each square on a chessboard, from A1 to H8.
        /// The squares are ordered by rank (rows) and file (columns),
        /// with A1 as the bottom-left and H8 as the top-right.
        #[derive(PartialEq, Ord, Eq, PartialOrd, Copy, Clone, Debug, Hash)]
        #[repr(u8)]
        pub enum Square {
            $(
                #[doc = concat!("The square `", stringify!($square), "`.")]
                $square,
            )*
        }
    };
}

/// A macro to create a `BlackMagicEntry` instance with the provided parameters.
///
/// This macro simplifies the initialization of `BlackMagicEntry` structs by directly
/// mapping the values to the struct's fields, which represent various components
/// of the magic bitboard setup for chess engines or similar applications.
#[doc(hidden)]
#[macro_export]
macro_rules! BlackMagic {
    ($mg: expr, $nm: expr, $o: expr) => {
        BlackMagicEntry {
            magic: $mg,
            not_mask: $nm,
            offset: $o,
        }
    };
}

/// Macro to implement board lookup functions for specific piece types.
///
/// This macro generates functions within the `Board` struct to retrieve bitboards
/// representing the positions of allied, enemy, and all pieces of a specified type.
/// Each piece type has three corresponding functions:
/// - An `allied_fn` function to get positions of allied pieces of this type.
/// - An `enemy_fn` function to get positions of enemy pieces of this type.
/// - A `total_fn` function to get all positions of this piece type, regardless of side.
macro_rules! impl_piece_lookups {
    ($($piece_index:expr, $allied_fn:ident, $enemy_fn:ident, $total_fn:ident),*) => {
        impl Board {
            $(
                /// Returns the positions of the current player's (allied)
                #[doc = stringify!($total_fn)]
                /// pieces.
                #[inline(always)]
                pub const fn $allied_fn(&self) -> BitBoard {
                    BitBoard(self.pieces_bitboard[$piece_index].0 & self.sides_bitboard[self.side as usize].0)
                }

                /// Returns the positions of the opponent's (enemy)
                #[doc = stringify!($total_fn)]
                /// pieces.
                #[inline(always)]
                pub const fn $enemy_fn(&self) -> BitBoard {
                    BitBoard(self.pieces_bitboard[$piece_index].0 & self.sides_bitboard[self.side as usize ^ 1].0)
                }

                /// Returns the positions of all
                #[doc = stringify!($total_fn)]
                /// pieces, regardless of side.
                #[inline(always)]
                pub const fn $total_fn(&self) -> BitBoard {
                    self.pieces_bitboard[$piece_index]
                }
            )*
        }
    };
}

// Implementing piece lookups
impl_piece_lookups! {
    0, allied_pawns, enemy_pawns, pawns,
    1, allied_knights, enemy_knights, knights,
    2, allied_bishops, enemy_bishops, bishops,
    3, allied_rooks, enemy_rooks, rooks,
    4, allied_queens, enemy_queens, queens,
    5, allied_king, enemy_king, kings
}

/// Calls the provided move handler function with a newly created move.
/// This macro simplifies move generation by constructing a `Move`
/// with the given source, destination, and move type, then passing it to the handler.
#[doc(hidden)]
#[macro_export]
macro_rules! Call_Handler {
    ($handler:expr, $src:expr, $dest:expr, $move_type:ident) => {
        $handler(Move::new($src, $dest, MoveType::$move_type))
    };
}

/// Enumerates all possible moves for different piece types.
/// This macro calls specific move generation functions for pawns, knights, bishops, rooks, and queens.
/// Considering check conditions, pinned pieces, and the provided move handler.
#[doc(hidden)]
#[macro_export]
macro_rules! Enumerate_Moves {
    ($check:expr, $board:expr, $diagonal_pins:expr, $linear_pins:expr, $handler:expr) => {
        enumerate_pawn_moves::<$check, ALL_MOVES, F>(
            $board,
            $board.allied_pawns(),
            $diagonal_pins,
            $linear_pins,
            &mut $handler,
        );
        enumerate_knight_moves::<$check, ALL_MOVES, F>(
            $board,
            $board.allied_knights(),
            $diagonal_pins,
            $linear_pins,
            &mut $handler,
        );
        enumerate_bishop_moves::<$check, ALL_MOVES, F>(
            $board,
            $board.allied_bishops() | $board.allied_queens(),
            $diagonal_pins,
            $linear_pins,
            &mut $handler,
        );
        enumerate_rook_moves::<$check, ALL_MOVES, F>(
            $board,
            $board.allied_rooks() | $board.allied_queens(),
            $diagonal_pins,
            $linear_pins,
            &mut $handler,
        );
    };
}
