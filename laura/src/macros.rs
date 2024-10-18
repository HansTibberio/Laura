use std::ops::{BitAnd, BitOr, BitXor};
use std::ops::{BitAndAssign, BitOrAssign, BitXorAssign};
use crate::bitboard::BitBoard;

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
    }
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