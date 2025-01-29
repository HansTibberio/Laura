use crate::{
    castle_rights::get_rook_castling, BitBoard, Board, CastleRights, Color, Move, MoveType, Piece,
    PieceType, Square,
};

// This implementation is based on the approach used in Carp,
// which provides a clear and efficient way to apply moves and handling null moves to the board.
// Source: https://github.com/dede1751/carp/blob/main/chess/src/movegen/make_move.rs

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
        let mut board: Board = *self;

        // Ensure the source and destination squares are different.
        assert_ne!(mv.get_src(), mv.get_dest());

        let src: Square = mv.get_src();
        let dest: Square = mv.get_dest();
        let move_type: MoveType = mv.get_type();
        let is_capture: bool = mv.is_capture();

        let piece: Piece = self.piece_on(src).unwrap();
        let piece_type: PieceType = piece.piece_type();

        // Remove the piece from its source square
        board.remove_piece(src);

        // Update fifty-move rule counter
        board.fifty_move = if is_capture || piece_type == PieceType::Pawn {
            0
        } else {
            board.fifty_move + 1
        };

        if board.side == Color::Black {
            board.full_move = board.full_move.saturating_add(1);
        }

        // Handle special move types (En Passant, Castling, Captures)
        match move_type {
            MoveType::EnPassant => {
                board.remove_piece(dest.forward(!self.side));
            }
            MoveType::KingCastle | MoveType::QueenCastle => {
                let rook: Piece = Piece::new(PieceType::Rook, self.side);
                let (rook_src, rook_dest) = get_rook_castling(dest);
                board.remove_piece(rook_src);
                board.set_piece(rook, rook_dest);
            }
            _ if is_capture => {
                board.remove_piece(dest);
            }
            _ => {}
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
            let enpassant_target: Square = src.forward(self.side);
            board.enpassant_square = Some(enpassant_target);
            board.zobrist.hash_enpassant(enpassant_target);
        }

        // Update castling rights and Zobrist hash
        let new_castling_rights: CastleRights = self.castling.update(src, dest);
        board.castling = new_castling_rights;
        board
            .zobrist
            .swap_castle_hash(self.castling, new_castling_rights);

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
        // Ensure there are no checkers on the board.
        assert!(self.checkers.is_empty());

        // Create a copy of the current board, switch the side to move and update the Zobrist hash.
        let mut board: Board = *self;
        board.side = !self.side;
        board.zobrist.hash_side();

        // Reset the en passant square.
        board.enpassant_square = None;

        // If there was an en passant square, update the Zobrist hash for it.
        if let Some(square) = self.enpassant_square {
            board.zobrist.hash_enpassant(square);
        }

        // Clear the checkers state.
        board.checkers = BitBoard::EMPTY;

        // Return the new board state after the null move.
        board
    }
}

#[test]
fn test_move() {
    let board: Board = Board::default();
    assert_eq!(
        board.to_fen(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
    println!("{}", board);
    let mv: Move = Move::new(Square::E2, Square::E4, MoveType::DoublePawn);
    let board: Board = board.make_move(mv);
    assert_eq!(
        board.to_fen(),
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
    );
    println!("{}", board);
    let mv: Move = Move::new(Square::C7, Square::C5, MoveType::DoublePawn);
    let board: Board = board.make_move(mv);
    assert_eq!(
        board.to_fen(),
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2"
    );
    println!("{}", board);
    let mv: Move = Move::new(Square::G1, Square::F3, MoveType::Quiet);
    let board: Board = board.make_move(mv);
    assert_eq!(
        board.to_fen(),
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2"
    );
    println!("{}", board);
}

#[test]
fn test_null() {
    let board: Board = Board::default();
    println!("{}", board);
    let board: Board = board.null_move();
    println!("{}", board);
}
