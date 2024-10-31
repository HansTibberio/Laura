use super::board::Board;

use crate::bitboard::BitBoard;
use crate::moves::{Move, MoveType};
use crate::piece::{Piece, PieceType};
use crate::square::Square;
use crate::castle_rights::{get_rook_castling, CastleRights};


impl Board {

    /// Executes a move on the chessboard, updating the board state, castling rights, 
    /// en passant square, fifty-move rule counter, and Zobrist hash accordingly.
    ///
    /// This function clones the current board state, applies the given move, 
    /// and returns the resulting board. The move can include special cases such as captures, 
    /// pawn promotions, castling, and en passant captures.
    /// 
    /// ### Panics
    /// The function will panic if the source and destination squares of the move are the same.
    pub fn make_move(&self, mv: Move) -> Board {
        let mut board: Board = self.clone();
        
        // Ensure the source and destination squares are different.
        assert_ne!(mv.get_src(), mv.get_dest());

        let src: Square = mv.get_src();
        let dest: Square = mv.get_dest();
        let piece: Piece = self.piece_on(src);
        let piece_type: PieceType = piece.piece_type();
        let move_type: MoveType = mv.get_type();
        let is_capture: bool = mv.is_capture();

        // Remove the piece from its source square
        board.remove_piece(src);

        // Update fifty-move rule counter
        board.fifty_move = if is_capture || piece_type == PieceType::Pawn { 0 } else { board.fifty_move + 1 };

        // Handle special move types (En Passant, Castling, Captures)
        match move_type {
            MoveType::EnPassant => {
                board.remove_piece(dest.forward(!self.side));
            },
            MoveType::KingCastle | MoveType::QueenCastle => {
                let rook: Piece = Piece::new(PieceType::Rook, self.side);
                let (rook_src, rook_dest) = get_rook_castling(dest);
                board.remove_piece(rook_src);
                board.set_piece(rook, rook_dest);
            },
            _ if is_capture => {
                board.remove_piece(dest);
            },
            _ => {},
        }

        // Handle promotions or move the piece to its destination
        if mv.is_promotion() {
            board.set_piece(mv.get_prom(self.side), dest);
        } else {
            board.set_piece(piece, dest);
        }

        // Update en passant square and Zobrist hash
        if let Some(square) = self.enpassant_square {
            board.enpassant_square = None;
            board.zobrist.hash_enpassant(square);
        }

        if move_type == MoveType::DoublePawn {
            let enpassant_target = src.forward(self.side);
            board.enpassant_square = Some(enpassant_target);
            board.zobrist.hash_enpassant(enpassant_target);
        }

        // Update castling rights and Zobrist hash
        let new_castling_rights: CastleRights = self.castling.update(src, dest);
        board.castling = new_castling_rights;
        board.zobrist.swap_castle_hash(self.castling, new_castling_rights);

        // Toggle side to move and update Zobrist hash
        board.side = !self.side;
        board.zobrist.hash_side();

        // Recalculate checkers for the new board state
        board.checkers = board.checkers();

        // Return the updated board
        board
    }

    /// Executes a null move, switching the turn to the opponent without making any actual moves.
    /// 
    /// This function is useful for certain algorithms where you want to evaluate a position
    /// as if the current player passed their turn. It asserts that the current player is not in check
    /// before performing the null move. The function will reset the en passant square and clear any checkers
    /// on the board.
    /// 
    /// ### Panics
    /// This function will panic if the current player's checkers are not empty, indicating that the
    /// game state is invalid for performing a null move.
    pub fn null_move(&self) -> Board {
        assert!(self.checkers.is_empty());

        let mut board: Board = self.clone();
        board.side = !self.side;
        board.zobrist.hash_side();

        board.enpassant_square = None;
        if let Some(square) = self.enpassant_square {
            board.zobrist.hash_enpassant(square);
        }

        board.checkers = BitBoard::EMPTY;

        board
    }

}

#[test]
fn test_move(){
    let board: Board = Board::default();
    let mv: Move = Move::new(Square::G1, Square::F3, MoveType::Quiet);
    let board: Board = board.make_move(mv);
    println!("{}", board);
    let mv: Move = Move::new(Square::E7, Square::E6, MoveType::Quiet);
    let board: Board = board.make_move(mv);
    println!("{}", board);
    let mv: Move = Move::new(Square::B1, Square::C3, MoveType::Quiet);
    let board: Board = board.make_move(mv);
    println!("{}", board);
}
