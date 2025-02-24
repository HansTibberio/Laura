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

use core::fmt::Write;
use core::str::FromStr;

use crate::{BitBoard, CastleRights, Color, File, Piece, Rank, Square, Zobrist};

use super::FenBuffer;

// This implementation is inspired by Carp, particularly its straightforward design for
// managing the board and its data, which simplifies move generation and game logic.
// Source: https://github.com/dede1751/carp/blob/main/chess/src/board.rs

/// Represents a chess board, with bitboards for tracking piece positions,
/// castling rights, en passant squares, the fifty-move rule counter, and
/// Zobrist hashing for fast state comparison.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Board {
    /// Array of bitboards, one for each type of piece. Each bitboard tracks
    /// the positions of that specific piece type on the board.
    pub pieces_bitboard: [BitBoard; Piece::COUNT],

    /// Bitboards for the sides: one for white pieces, one for black pieces.
    pub sides_bitboard: [BitBoard; 2],

    /// Maps squares to the piece occupying them, if any.
    pub piece_map: [Option<Piece>; Square::NUM_SQUARES],

    /// The square available for an en passant capture, if applicable.
    pub enpassant_square: Option<Square>,

    /// The castling rights of the current board.
    pub castling: CastleRights,

    /// Counter for the fifty-move rule, tracking half-moves since the last capture or pawn move.
    pub fifty_move: u8,

    /// The number of the full moves. It starts at 1 and is incremented after Black's move.
    pub full_move: u16,

    /// The Zobrist hash representing the current board state.
    pub zobrist: Zobrist,

    /// The side to move (either White or Black).
    pub side: Color,

    /// Bitboard representing all enemy pieces that are directly checking the allied king.
    pub checkers: BitBoard,
}

/// Displays the current state of the chess board in a readable format, including
/// FEN notation, Zobrist hash, and a grid representation of the board.
impl core::fmt::Display for Board {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "\n FEN: {}\n Zobrist: {}\n\n\t+---+---+---+---+---+---+---+---+",
            self.to_fen(),
            self.zobrist
        )?;

        for rank in (0..Rank::NUM_RANKS).rev() {
            write!(f, "\n     {}  | ", rank + 1)?;

            for file in 0..File::NUM_FILES {
                let square_index: usize = rank * 8 + file;
                let piece = self.piece_map[square_index]
                    .map(|p| p.to_char())
                    .unwrap_or(' ');
                write!(f, "{}", piece)?;
                write!(f, " | ")?;
            }

            if rank != 0 {
                write!(f, "\n\t+---+---+---+---+---+---+---+---+")?;
            }
        }

        write!(
            f,
            "\n\t+---+---+---+---+---+---+---+---+\n\t  A   B   C   D   E   F   G   H\n\n"
        )?;

        write!(f, "\t    Side to move        : ")?;
        match self.side {
            Color::White => writeln!(f, "White")?,
            Color::Black => writeln!(f, "Black")?,
        }
        writeln!(f, "\t    Castling Rights     : {}", self.castling)?;
        write!(f, "\t    En Passante square  : ")?;
        if let Some(square) = self.enpassant_square {
            writeln!(f, "{}", square)?;
        } else {
            writeln!(f, "-")?;
        }
        writeln!(f, "\t    Fifty Rule          : {}", self.fifty_move)?;
        Ok(())
    }
}

/// Parses a FEN string to create a new `Board` instance. The FEN string is split
/// into 6 parts: piece placement, active color, castling rights, en passant target
/// square, halfmove clock, and fullmove number.
impl FromStr for Board {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fen_iter: core::str::SplitWhitespace<'_> = s.split_whitespace();

        let board_str: &str = fen_iter.next().ok_or("FEN string is too short")?;
        let side_str: &str = fen_iter.next().ok_or("Missing side to move")?;
        let castling_str: &str = fen_iter.next().ok_or("Missing castling rights")?;
        let enpassant_str: &str = fen_iter.next().ok_or("Missing en passant square")?;
        let halfmove_str: &str = fen_iter.next().ok_or("Missing halfmove clock")?;
        let fullmove_str: &str = fen_iter.next().ok_or("Missing fullmove number")?;

        let mut board: Board = Self::new();
        let mut count: i32 = 0;

        let (mut file, mut rank) = (File::A, Rank::Eight);
        for token in board_str.chars() {
            match token {
                '/' => {
                    if count != 8 {
                        return Err("FEN row does not contain exactly 8 squares.");
                    };

                    rank = rank.down();
                    count = 0;
                }
                '1'..='8' => {
                    for _ in '1'..=token {
                        file = file.right();
                        count += 1;
                    }
                }
                _ => {
                    board.set_piece(Piece::try_from(token)?, Square::from_file_rank(file, rank));
                    file = file.right();
                    count += 1;
                }
            }
        }

        if count != 8 {
            return Err("The board layout is invalid.");
        }

        board.side = match side_str {
            "w" => {
                board.zobrist.hash_side();
                Color::White
            }
            "b" => Color::Black,
            _ => return Err("Invalid side to move, should be 'w' or 'b'."),
        };

        let castle_rights: CastleRights = castling_str.parse()?;
        board.castling = castle_rights;
        board.zobrist.hash_castle(castle_rights);

        board.enpassant_square = match enpassant_str {
            "-" => None,
            _ => {
                let ep_square: Square = enpassant_str
                    .parse()
                    .map_err(|_| "Invalid en passant square")?;
                if !matches!(ep_square.rank(), Rank::Three | Rank::Six) {
                    return Err("Invalid en passant rank.");
                }
                board.zobrist.hash_enpassant(ep_square);
                Some(ep_square)
            }
        };

        board.fifty_move = halfmove_str
            .parse::<u8>()
            .map_err(|_| "Invalid halfmove clock")?;
        if board.fifty_move > 100 {
            return Err("Halfmove Clock exceeds the maximum allowed value.");
        }

        board.full_move = fullmove_str
            .parse::<u16>()
            .map_err(|_| "Invalid fullmove number")?;
        if board.full_move == 0 {
            return Err("Fullmove number must be positive.");
        }

        board.checkers = board.checkers();

        Ok(board)
    }
}

impl Default for Board {
    /// Constructs a default chess board, representing the standard starting position
    /// for a chess game, using FEN notation. The default position is the classic setup
    /// with castling rights and no en passant.
    #[inline]
    fn default() -> Self {
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
            .parse()
            .unwrap()
    }
}

impl Board {
    /// Creates a new empty board with no pieces. The bitboards are initialized as empty,
    /// and castling rights, en passant square, and other attributes are set to their
    /// default (empty or zero) values.
    const fn new() -> Self {
        Self {
            pieces_bitboard: [BitBoard::EMPTY; Piece::COUNT],
            sides_bitboard: [BitBoard::EMPTY; 2],
            piece_map: [None; Square::NUM_SQUARES],
            enpassant_square: None,
            castling: CastleRights::null(),
            fifty_move: 0,
            full_move: 1,
            zobrist: Zobrist::null(),
            side: Color::White,
            checkers: BitBoard::EMPTY,
        }
    }

    /// Converts the current board state into a FEN (Forsyth-Edwards Notation) string.
    ///
    /// FEN is a standard notation for describing a particular board position of a chess game.
    /// It includes information about the placement of pieces, which side is to move, castling rights,
    /// en passant target squares, the half-move clock (for the fifty-move rule), and the full-move number.
    pub fn to_fen(&self) -> FenBuffer {
        let mut fen: FenBuffer = FenBuffer::new();
        for rank in (0..Rank::NUM_RANKS).rev() {
            let mut empty_squares: i32 = 0;

            for file in 0..File::NUM_FILES {
                let square_index: usize = rank * 8 + file;

                if let Some(piece) = self.piece_map[square_index] {
                    if empty_squares > 0 {
                        let _ = write!(fen, "{}", empty_squares);
                        empty_squares = 0;
                    }
                    let _ = write!(fen, "{}", piece.to_char());
                } else {
                    empty_squares += 1;
                }
            }

            if empty_squares > 0 {
                let _ = write!(fen, "{}", empty_squares);
            }

            if rank != Rank::One.to_index() {
                let _ = write!(fen, "/");
            }
        }

        let _ = write!(fen, " {} ", self.side);

        let _ = write!(fen, "{} ", self.castling);

        if let Some(enpassant_square) = self.enpassant_square {
            let _ = write!(fen, "{}", enpassant_square);
        } else {
            let _ = write!(fen, "-");
        }

        let _ = write!(fen, " {} {}", self.fifty_move, self.full_move);

        fen
    }

    /// Sets a piece on the board at a given square and updates the corresponding bitboards
    /// and Zobrist hash. This method modifies both the specific piece bitboard and the
    /// side's bitboard (either White or Black).
    pub fn set_piece(&mut self, piece: Piece, square: Square) {
        let index: usize = piece.piece_index();
        let color: usize = piece.color() as usize;

        self.pieces_bitboard[index] = self.pieces_bitboard[index].set_square(square);
        self.sides_bitboard[color] = self.sides_bitboard[color].set_square(square);
        self.piece_map[square.to_index()] = Some(piece);
        self.zobrist.hash_piece(piece, square);
    }

    /// Removes a piece from a square and updates the corresponding bitboards and
    /// Zobrist hash.
    /// 
    /// # Panics
    /// This function will panic if no piece is present on the specified square,
    /// as it calls `unwrap()` on an `Option`.
    pub fn remove_piece(&mut self, square: Square) {
        let piece: Piece = self.piece_on(square).unwrap();
        let index: usize = piece.piece_index();
        let color: usize = piece.color() as usize;

        self.pieces_bitboard[index] = self.pieces_bitboard[index].pop_square(square);
        self.sides_bitboard[color] = self.sides_bitboard[color].pop_square(square);
        self.piece_map[square.to_index()] = None;
        self.zobrist.hash_piece(piece, square);
    }

    /// Returns the piece located on the specified square.
    #[inline]
    pub fn piece_on(&self, square: Square) -> Option<Piece> {
        self.piece_map[square.to_index()]
    }

    /// Returns the side to move (white or black).
    #[inline(always)]
    pub const fn side(&self) -> Color {
        self.side
    }

    /// Returns the castling rights of the current board.
    #[inline(always)]
    pub const fn castling_rights(&self) -> CastleRights {
        self.castling
    }

    /// Returns the Zobrist hash of the current board position.
    ///
    /// The Zobrist hash is a unique value representing the current state of the board.
    /// It is used for hashing positions in transposition tables.
    #[inline(always)]
    pub const fn zobrist(&self) -> Zobrist {
        self.zobrist
    }

    /// Returns the current value of the fifty-move counter.
    ///
    /// The fifty-move rule in chess allows a draw to be claimed if no capture or pawn movement
    /// has occurred in the last fifty moves.
    #[inline(always)]
    pub const fn fifty_move(&self) -> u8 {
        self.fifty_move
    }

    /// Returns the full number of moves made since the start of the game.
    #[inline(always)]
    pub const fn full_move(&self) -> u16 {
        self.full_move
    }

    /// Constructs the classic "Kiwipete" position (Peter McKenzie),
    /// a well-known test position for perft.
    #[inline]
    pub fn kiwipete() -> Self {
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
            .parse()
            .unwrap()
    }
}
