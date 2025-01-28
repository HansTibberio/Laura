use std::ops::{BitAnd, BitOr, BitXor};
use std::ops::{BitAndAssign, BitOrAssign, BitXorAssign};

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

/// A macro to create a `BlackMagicEntry` instance with the provided parameters.
///
/// This macro simplifies the initialization of `BlackMagicEntry` structs by directly
/// mapping the values to the struct's fields, which represent various components
/// of the magic bitboard setup for chess engines or similar applications.
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
                #[inline(always)]
                pub const fn $allied_fn(&self) -> BitBoard {
                    BitBoard(self.pieces_bitboard[$piece_index].0 & self.sides_bitboard[self.side as usize].0)
                }

                #[inline(always)]
                pub const fn $enemy_fn(&self) -> BitBoard {
                    BitBoard(self.pieces_bitboard[$piece_index].0 & self.sides_bitboard[self.side as usize ^ 1].0)
                }

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
