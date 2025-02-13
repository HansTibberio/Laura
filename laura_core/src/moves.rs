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

use core::fmt;
use core::mem::transmute;

use crate::{piece::PROM_PIECES, Color, Piece, Square};

/// Represents a chess move using a 16-bit integer.
/// The move encodes the source square, destination square, move type, and any promotion.
///
///     0000 0000 0011 1111    source        0x003F
///     0000 1111 1100 0000    destination   0x0FC0
///     1111 0000 0000 0000    MoveType      0xF000
///
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct Move(pub u16);

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_promotion() {
            write!(
                f,
                "{}{}{}",
                self.get_src(),
                self.get_dest(),
                self.get_prom(Color::Black).to_char()
            )
        } else {
            write!(f, "{}{}", self.get_src(), self.get_dest())
        }
    }
}

// Bit masks to extract parts of the move from the 16-bit representation.
const SRC_MASK: u16 = 0b00000000_00111111;
const DEST_MASK: u16 = 0b00001111_11000000;
const TYPE_MASK: u16 = 0b11110000_00000000;
const PROM_MASK: u16 = 0b10000000_00000000;
const CAP_MASK: u16 = 0b01000000_00000000;

/// Enum representing the different types of moves in chess, including promotions and special moves.
///
/// https://www.chessprogramming.org/Encoding_Moves
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
#[repr(u8)]
pub enum MoveType {
    Quiet = 0b0000,
    DoublePawn = 0b0001,
    KingCastle = 0b0010,
    QueenCastle = 0b0011,
    Capture = 0b0100,
    EnPassant = 0b0101,
    PromotionKnight = 0b1000,
    PromotionBishop = 0b1001,
    PromotionRook = 0b1010,
    PromotionQueen = 0b1011,
    CapPromoKnight = 0b1100,
    CapPromoBishop = 0b1101,
    CapPromoRook = 0b1110,
    CapPromoQueen = 0b1111,
}

impl Move {
    /// Represents a null move (an invalid or empty move).
    #[inline(always)]
    pub const fn null() -> Self {
        Self(0)
    }

    /// Returns `true` if the move is a null move.
    /// # Example
    /// ```
    /// # use laura_core::*;
    /// let mv: Move = Move::null();
    /// assert_eq!(mv.is_null(), true);
    /// ```
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Creates a new move given the source and destination squares, and the move type.
    /// # Example
    /// ```
    /// # use laura_core::*;
    /// let mv = Move::new(Square::E2, Square::E4, MoveType::DoublePawn);
    /// assert_eq!(mv.get_src(), Square::E2);
    /// assert_eq!(mv.get_dest(), Square::E4);
    /// assert_eq!(mv.get_type(), MoveType::DoublePawn);
    /// ```
    #[inline]
    pub const fn new(src: Square, dest: Square, move_type: MoveType) -> Self {
        Self((move_type as u16) << 12 | (dest as u16) << 6 | (src as u16))
    }

    /// Returns the destination square of the move.
    #[inline(always)]
    pub const fn get_dest(self) -> Square {
        unsafe { transmute((((self.0 & DEST_MASK) >> 6) as u8) & 63) }
    }

    /// Returns the source square of the move.
    #[inline(always)]
    pub const fn get_src(self) -> Square {
        unsafe { transmute(((self.0 & SRC_MASK) as u8) & 63) }
    }

    /// Returns the type of move (e.g., `Quiet`, `Capture`, `EnPassant`).
    #[inline(always)]
    pub const fn get_type(self) -> MoveType {
        unsafe { transmute((((self.0 & TYPE_MASK) >> 12) as u8) & 15) }
    }

    /// Returns the promotion piece (if any) based on the color.
    /// This function retrieves the promoted piece for the corresponding color.
    #[inline(always)]
    pub const fn get_prom(self, color: Color) -> Piece {
        PROM_PIECES[color as usize][self.flag() as usize & 0b011]
    }

    /// Returns `true` if the move is a promotion.
    /// # Example
    /// ```
    /// # use laura_core::*;
    /// let mv: Move = Move::new(Square::B7, Square::C8, MoveType::CapPromoQueen);
    /// assert_eq!(mv.get_prom(Color::White), Piece::WQ);
    /// assert_eq!(mv.is_promotion(), true);
    /// assert_eq!(mv.is_underpromotion(), false);
    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        (self.0 & PROM_MASK) != 0
    }

    /// Returns `true` if the move is an underpromotion (promotion to knight, bishop, or rook).
    #[inline(always)]
    pub const fn is_underpromotion(self) -> bool {
        self.is_promotion() && self.flag() & 0b1011 != 0b1011
    }

    /// Returns `true` if the move is a capture.
    /// # Example
    /// ```
    /// # use laura_core::*;
    /// let mv: Move = Move::new(Square::C1, Square::C8, MoveType::Capture);
    /// assert_eq!(mv.get_type(), MoveType::Capture);
    /// assert_eq!(mv.is_promotion(), false);
    /// assert_eq!(mv.is_capture(), true);
    /// assert_eq!(mv.is_quiet(), false);
    /// ```
    #[inline(always)]
    pub const fn is_capture(self) -> bool {
        ((self.0 & CAP_MASK) >> 14) == 1
    }

    /// Returns `true` if the move is a quiet move (no capture, promotion, castle or double pawn push).
    /// # Example
    /// ```
    /// # use laura_core::*;
    /// let mv: Move = Move::new(Square::A2, Square::A4, MoveType::Quiet);
    /// assert_eq!(mv.is_promotion(), false);
    /// assert_eq!(mv.is_capture(), false);
    /// assert_eq!(mv.is_quiet(), true);
    /// ```
    #[inline(always)]
    pub const fn is_quiet(self) -> bool {
        self.flag() == 0
    }

    /// Retrieves the flag bits from the move, which represent the `MoveType`.
    #[inline(always)]
    pub const fn flag(self) -> u16 {
        self.0 >> 12
    }
}
