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

use crate::{piece::PROM_PIECES, Color, Piece, Square};
use core::fmt;
use core::mem::transmute;

/// Represents a single chess move as a compact 16-bit unsigned integer.
///
/// The move is encoded using the following bit layout:
///
/// ```ignore
/// 0000 0000 0011 1111    source        0x003F
/// 0000 1111 1100 0000    destination   0x0FC0
/// 1111 0000 0000 0000    MoveType      0xF000
/// ```
/// This encoding allows efficient storage and manipulation of moves, which is  
/// especially useful for move generation and search algorithms.
///
/// # Examples
///
/// ```
/// # use laura_core::*;
///
/// let mv = Move::new(Square::E2, Square::E4, MoveType::Quiet);
/// assert_eq!(mv.get_src(), Square::E2);
/// assert_eq!(mv.get_dest(), Square::E4);
/// ```
#[derive(Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Debug, Default, Hash)]
pub struct Move(pub u16);

/// Implements the `Display` trait for pretty-printing moves in algebraic notation.
///
/// If the move is a promotion, the promoted piece is appended at the end, using  
/// lowercase notation by default (e.g., 'q' for queen). Otherwise, only the source  
/// and destination squares are displayed.
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

/// Allows comparing a `Move` against a string slice in algebraic notation.
///
/// This makes it easy to check if a move matches a specific string,  
/// including handling promotion moves (e.g., "e7e8q").
impl PartialEq<&str> for Move {
    fn eq(&self, other: &&str) -> bool {
        let mut move_str: [u8; 6] = [0u8; 6];
        let mut pos: usize = 0;

        let src: &str = self.get_src().to_str();
        let dest: &str = self.get_dest().to_str();

        move_str[pos..pos + src.len()].copy_from_slice(src.as_bytes());
        pos += src.len();

        move_str[pos..pos + dest.len()].copy_from_slice(dest.as_bytes());
        pos += dest.len();

        if self.is_promotion() {
            move_str[pos] = self.get_prom(Color::Black).to_char() as u8;
            pos += 1;
        }

        let move_as_str: &str = core::str::from_utf8(&move_str[..pos]).unwrap_or("");
        move_as_str == *other
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
/// <https://www.chessprogramming.org/Encoding_Moves>
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
#[repr(u8)]
pub enum MoveType {
    /// A standard, non-capturing move (e.g., moving a piece to an empty square).
    Quiet = 0b0000,

    /// A double pawn move, where a pawn advances two squares on its first move.
    DoublePawn = 0b0001,

    /// King-side castling move.
    KingCastle = 0b0010,

    /// Queen-side castling move.
    QueenCastle = 0b0011,

    /// A capture move, where a piece takes an opponent's piece.
    Capture = 0b0100,

    /// En passant capture, a special pawn capture move.
    EnPassant = 0b0101,

    /// Promotion to a Knight after a pawn reaches the last rank.
    PromotionKnight = 0b1000,

    /// Promotion to a Bishop after a pawn reaches the last rank.
    PromotionBishop = 0b1001,

    /// Promotion to a Rook after a pawn reaches the last rank.
    PromotionRook = 0b1010,

    /// Promotion to a Queen after a pawn reaches the last rank.
    PromotionQueen = 0b1011,

    /// A capture move combined with a promotion to a Knight.
    CapPromoKnight = 0b1100,

    /// A capture move combined with a promotion to a Bishop.
    CapPromoBishop = 0b1101,

    /// A capture move combined with a promotion to a Rook.
    CapPromoRook = 0b1110,

    /// A capture move combined with a promotion to a Queen.
    CapPromoQueen = 0b1111,
}

impl Move {
    /// Returns a `Move` instance representing a null move.
    ///
    /// A null move is typically used to signify the absence of a valid move,
    /// or to mark an invalid or uninitialized state.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::null();
    /// assert_eq!(mv.is_null(), true);
    /// ```
    ///
    /// The returned move always has an underlying value of `0`,  
    /// ensuring that `mv.is_null()` will return `true`.
    #[inline(always)]
    pub const fn null() -> Self {
        Self(0)
    }

    /// Checks whether this move is a null move.
    ///
    /// Returns `true` if the underlying value of the move is `0`,  
    /// indicating that it is not a valid, fully initialized move.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::null();
    /// assert!(mv.is_null());
    ///
    /// let valid_move = Move::new(Square::E2, Square::E4, MoveType::Quiet);
    /// assert!(!valid_move.is_null());
    /// ```
    ///
    /// Any move with an internal representation of `0` is considered null.  
    /// All valid moves have a non-zero representation.
    #[inline(always)]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Creates a new move given the source and destination squares, and the move type.
    ///
    /// # Example
    /// ```
    /// # use laura_core::*;
    /// let mv = Move::new(Square::E2, Square::E4, MoveType::DoublePawn);
    /// assert_eq!(mv.get_src(), Square::E2);
    /// assert_eq!(mv.get_dest(), Square::E4);
    /// assert_eq!(mv.get_type(), MoveType::DoublePawn);
    /// ```
    ///
    /// This function does not perform runtime validation of the square or move type;  
    /// invalid inputs may result in unexpected behavior downstream.
    #[inline(always)]
    pub const fn new(src: Square, dest: Square, move_type: MoveType) -> Self {
        Self(((move_type as u16) << 12) | ((dest as u16) << 6) | (src as u16))
    }

    /// Returns the source square of the move.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::new(Square::E2, Square::E4, MoveType::Quiet);
    /// assert_eq!(mv.get_src(), Square::E2);
    /// ```
    ///
    /// # Safety
    ///
    /// This function uses `unsafe` and [`transmute`] to convert a 6-bit value into a [`Square`].  
    /// This is considered safe because the [`Square`] enum is defined as a contiguous range from 0 to 63,
    /// and the move encoding guarantees that only valid 6-bit values are produced.
    #[inline(always)]
    pub const fn get_src(self) -> Square {
        unsafe { transmute(((self.0 & SRC_MASK) as u8) & 63) }
    }

    /// Returns the destination square of the move.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::new(Square::E2, Square::E4, MoveType::Quiet);
    /// assert_eq!(mv.get_dest(), Square::E4);
    /// ```
    ///
    /// # Safety
    ///
    /// This function uses `unsafe` and [`transmute`] to convert a 6-bit value into a [`Square`].  
    /// This is considered safe because the [`Square`] enum is defined as a contiguous range from 0 to 63,
    /// and the move encoding guarantees that only valid 6-bit values are produced.
    #[inline(always)]
    pub const fn get_dest(self) -> Square {
        unsafe { transmute((((self.0 & DEST_MASK) >> 6) as u8) & 63) }
    }

    /// Returns the type of move (e.g., `Quiet`, `Capture`, `EnPassant`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::new(Square::E2, Square::E4, MoveType::Quiet);
    /// assert_eq!(mv.get_type(), MoveType::Quiet);
    /// ```
    ///
    /// # Safety
    ///
    /// This function uses `unsafe` and [`transmute`] to convert a 4-bit value into a [`MoveType`].  
    /// This is considered safe because the [`MoveType`] enum is expected to cover values 0 to 15,  
    /// and the move encoding guarantees that only valid values are used.
    #[inline(always)]
    pub const fn get_type(self) -> MoveType {
        unsafe { transmute((((self.0 & TYPE_MASK) >> 12) as u8) & 15) }
    }

    /// Returns the promotion piece (if any) based on the color.
    ///
    /// If the move is a promotion, this function retrieves the promoted piece from the  
    /// `PROM_PIECES` lookup table, using the given color to differentiate between white and black promotions.  
    /// If the move is not a promotion, the returned piece may be undefined.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::new(Square::E7, Square::E8, MoveType::PromotionQueen);
    /// let promoted_piece = mv.get_prom(Color::White);
    /// assert_eq!(promoted_piece, Piece::WQ);
    /// ```
    ///
    /// # Expected Behavior
    ///
    /// - If the move is a promotion, returns the promoted piece.
    /// - If the move is not a promotion, the returned value is undefined.
    #[inline(always)]
    pub const fn get_prom(self, color: Color) -> Piece {
        PROM_PIECES[color as usize][self.flag() as usize & 0b011]
    }

    /// Returns `true` if the move is a promotion.
    ///
    /// This function checks whether the promotion bit is set in the move’s 16-bit representation.  
    /// If the promotion bit is set, the move is considered a promotion (e.g., to queen, rook, bishop, or knight).
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::new(Square::B7, Square::C8, MoveType::CapPromoQueen);
    /// assert_eq!(mv.get_prom(Color::White), Piece::WQ);
    /// assert_eq!(mv.is_promotion(), true);
    /// assert_eq!(mv.is_underpromotion(), false);
    /// ```
    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        (self.0 & PROM_MASK) != 0
    }

    /// Returns `true` if the move is an underpromotion (promotion to knight, bishop, or rook).
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let queen_promo = Move::new(Square::B7, Square::B8, MoveType::PromotionQueen);
    /// assert_eq!(queen_promo.is_underpromotion(), false);
    ///
    /// let knight_promo = Move::new(Square::B7, Square::B8, MoveType::PromotionKnight);
    /// assert_eq!(knight_promo.is_underpromotion(), true);
    /// ```
    #[inline(always)]
    pub const fn is_underpromotion(self) -> bool {
        self.is_promotion() && self.flag() & 0b1011 != 0b1011
    }

    /// Returns `true` if the move is a capture.
    ///
    /// This function checks whether the move is a capturing move (e.g., a piece is captured on the destination square).  
    /// It examines the capture flag bit in the move’s 16-bit representation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::new(Square::C1, Square::C8, MoveType::Capture);
    ///
    /// assert_eq!(mv.get_type(), MoveType::Capture);
    /// assert_eq!(mv.is_promotion(), false);
    /// assert_eq!(mv.is_capture(), true);
    /// assert_eq!(mv.is_quiet(), false);
    /// ```
    #[inline(always)]
    pub const fn is_capture(self) -> bool {
        ((self.0 & CAP_MASK) >> 14) == 1
    }

    /// Returns `true` if the move is a castle.
    ///
    /// This function checks whether the move is either a king-side or queen-side castling move.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let king_castle = Move::new(Square::E1, Square::G1, MoveType::KingCastle);
    /// assert_eq!(king_castle.is_castle(), true);
    ///
    /// let queen_castle = Move::new(Square::E1, Square::C1, MoveType::QueenCastle);
    /// assert_eq!(queen_castle.is_castle(), true);
    /// ```
    #[inline(always)]
    pub const fn is_castle(self) -> bool {
        self.is_king_castle() || self.is_queen_castle()
    }

    /// Returns `true` if the move is a king-side castle.
    ///
    /// This function checks whether the move type represents a king-side castling move.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::new(Square::E1, Square::G1, MoveType::KingCastle);
    /// assert_eq!(mv.is_king_castle(), true);
    /// ```
    #[inline(always)]
    pub const fn is_king_castle(self) -> bool {
        ((self.0 & TYPE_MASK) >> 12) == MoveType::KingCastle as u16
    }

    /// Returns `true` if the move is a queen-side castle.
    ///
    /// This function checks whether the move type represents a queen-side castling move.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::new(Square::E1, Square::C1, MoveType::QueenCastle);
    /// assert_eq!(mv.is_queen_castle(), true);
    /// ```
    #[inline(always)]
    pub const fn is_queen_castle(self) -> bool {
        ((self.0 & TYPE_MASK) >> 12) == MoveType::QueenCastle as u16
    }

    /// Returns `true` if the move is a double pawn move.
    ///
    /// A double pawn move occurs when a pawn moves forward two squares from its starting rank,  
    /// such as e2 to e4 for White or e7 to e5 for Black. This move can be relevant for en passant captures  
    /// and certain positional evaluations.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::new(Square::E2, Square::E4, MoveType::DoublePawn);
    ///
    /// assert_eq!(mv.is_double_pawn(), true);
    /// ```
    #[inline(always)]
    pub const fn is_double_pawn(self) -> bool {
        ((self.0 & TYPE_MASK) >> 12) == MoveType::DoublePawn as u16
    }

    /// Returns `true` if the move is an en passant capture.
    ///
    /// This function checks whether the move type encoded in the move’s 16-bit representation  
    /// corresponds to an en passant capture. En passant is a special pawn capture that occurs  
    /// when a pawn captures an opposing pawn that has just advanced two squares from its starting rank.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::new(Square::E5, Square::D6, MoveType::EnPassant);
    ///
    /// assert_eq!(mv.is_enpassant(), true);
    /// ```
    #[inline(always)]
    pub const fn is_enpassant(self) -> bool {
        ((self.0 & TYPE_MASK) >> 12) == MoveType::EnPassant as u16
    }

    /// Returns `true` if the move is a quiet move (no capture, promotion, castle or double pawn push).
    ///
    /// A quiet move is a standard, non-special move that simply moves a piece from its source square to the destination  
    /// without capturing an opponent’s piece, promoting a pawn, or involving special move rules like castling or en passant.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv: Move = Move::new(Square::A2, Square::A4, MoveType::Quiet);
    ///
    /// assert_eq!(mv.is_promotion(), false);
    /// assert_eq!(mv.is_capture(), false);
    /// assert_eq!(mv.is_quiet(), true);
    /// ```
    #[inline(always)]
    pub const fn is_quiet(self) -> bool {
        self.flag() == 0
    }

    /// Retrieves the flag bits from the move, which represent the [`MoveType`].
    ///
    /// This function extracts the upper 4 bits (bits 12–15) from the packed 16-bit move representation.  
    /// These bits encode the [`MoveType`], which indicates the kind of move (e.g., `Quiet`, `Capture`, `Promotion`, etc.).
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mv = Move::new(Square::E2, Square::E4, MoveType::Quiet);
    /// let flag = mv.flag();
    /// assert_eq!(flag, MoveType::Quiet as u16);
    /// ```
    #[inline(always)]
    pub const fn flag(self) -> u16 {
        self.0 >> 12
    }
}
