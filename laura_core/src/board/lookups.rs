use super::board::Board;
use super::movegen::get_bishop_attacks; 
use super::movegen::get_king_attacks;
use super::movegen::get_knight_attacks; 
use super::movegen::get_pawn_attacks; 
use super::movegen::get_rook_attacks;

use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::piece::Piece;
use crate::square::Square;


impl Board {
    
    /// Returns the bitboard representing all pieces for the white side.
    #[inline(always)]
    pub const fn white_bitboard(&self) -> BitBoard {
        self.sides_bitboard[Color::White as usize]
    }

    /// Returns the bitboard representing all pieces for the black side.
    #[inline(always)]
    pub const fn black_bitboard(&self) -> BitBoard {
        self.sides_bitboard[Color::Black as usize]
    }

    /// Returns a bitboard representing all pieces currently on the board for both sides.
    /// 
    /// This function combines the bitboards for both white and black pieces by performing
    /// a bitwise OR operation.
    #[inline(always)]
    pub const fn combined_bitboard(&self) -> BitBoard {
        BitBoard(self.white_bitboard().0 | self.black_bitboard().0)
    }

    /// Returns a `BitBoard` representing the presence of a specified piece type and color on the board.
    /// Combines the bitboard for the specified piece with the bitboard for the side it belongs to.
    #[inline(always)]
    pub const fn piece_presence(&self, piece: Piece) -> BitBoard {
        BitBoard(self.pieces_bitboard[piece.piece_index()].0 & self.sides_bitboard[piece.color() as usize].0)
    }

    /// Returns a `BitBoard` representing the presence of all allied pieces for the current side on the board.
    #[inline(always)]
    pub const fn allied_presence(&self) -> BitBoard {
        self.sides_bitboard[self.side as usize]
    }

    /// Returns a `BitBoard` representing the presence of all enemy pieces for the opposing side on the board.
    #[inline(always)]
    pub const fn enemy_presence(&self) -> BitBoard {
        self.sides_bitboard[self.side as usize ^ 1]
    }

    /// Returns a `BitBoard` representing the presence of enemy queens and bishops on the board.
    /// This combines the bitboards for enemy queens and bishops into a single bitboard.
    #[inline(always)]
    pub const fn enemy_queen_bishops(&self) -> BitBoard {
        BitBoard(self.enemy_queens().0 | self.enemy_bishops().0)
    }

    /// Returns a `BitBoard` representing the presence of enemy queens and rooks on the board.
    /// This combines the bitboards for enemy queens and rooks into a single bitboard.
    #[inline(always)]
    pub const fn enemy_queen_rooks(&self) -> BitBoard {
        BitBoard(self.enemy_queens().0 | self.enemy_rooks().0)
    }

    /// Returns a `BitBoard` representing all enemy pieces that are attacking a specified square,
    /// based on the given blockers on the board. Evaluates potential attacks from enemy knights,
    /// kings, pawns, queens, bishops, and rooks against the square.
    pub fn attackers(&self, square: Square, blockers: BitBoard) -> BitBoard {
        self.enemy_presence() & (
            self.knights() & get_knight_attacks(square)
            | self.kings() & get_king_attacks(square)
            | self.pawns() & get_pawn_attacks(self.side, square)
            | (self.queens() | self.bishops()) & get_bishop_attacks(square, blockers)
            | (self.queens() | self.rooks()) & get_rook_attacks(square, blockers)
        )
    }

    /// Checks if a specified square is currently under attack by any enemy piece.
    #[inline(always)]
    pub fn attacked_square(&self, square: Square, blockers: BitBoard) -> bool {
        self.attackers(square, blockers) != BitBoard::EMPTY
    }

    /// Returns a `BitBoard` representing all enemy pieces that are directly checking the allied king.
    /// Uses the current combined board state to evaluate potential checks.
    pub fn checkers(&self) -> BitBoard {
        self.attackers(self.allied_king().to_square(), self.combined_bitboard())
    }
}