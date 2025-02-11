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

use std::fmt;
use std::str::FromStr;

use crate::{BitBoard, CastleRights, Color, File, Piece, Rank, Square, Zobrist};

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
///
/// The board is displayed using Unicode characters for better visual clarity.
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_str: String = format!(
            "\n FEN: {}\n Zobrist: {}\n\n\t┏━━━┳━━━┳━━━┳━━━┳━━━┳━━━┳━━━┳━━━┓",
            self.to_fen(),
            self.zobrist
        );

        for rank in (0..Rank::NUM_RANKS).rev() {
            board_str.push_str(format!("\n      {} ┃ ", rank + 1).as_str());

            for file in 0..File::NUM_FILES {
                let square_index: usize = rank * 8 + file;
                let piece_str: String =
                    self.piece_map[square_index].map_or(String::from(" "), |p| p.to_string());
                board_str.push_str(&piece_str);
                board_str.push_str(" ┃ ");
            }

            if rank != Rank::One.to_index() {
                board_str.push_str("\n\t┣━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━┫");
            }
        }

        board_str
            .push_str("\n\t┗━━━┻━━━┻━━━┻━━━┻━━━┻━━━┻━━━┻━━━┛\n\t  A   B   C   D   E   F   G   H\n");

        let enpassant_str = match self.enpassant_square {
            Some(square) => format!("{square}"),
            None => String::from("-"),
        };

        write!(
            f,
            "{}
            Side to move        : {}
            Castling Rights     : {}
            En Passante square  : {}
            Fifty Rule          : {}
            ",
            board_str, self.side, self.castling, enpassant_str, self.fifty_move,
        )
    }
}

/// Parses a FEN string to create a new `Board` instance. The FEN string is split
/// into 6 parts: piece placement, active color, castling rights, en passant target
/// square, halfmove clock, and fullmove number.
impl FromStr for Board {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fen: Vec<&str> = s.split_whitespace().collect();

        if fen.len() != 6 {
            return Err("FEN string has an invalid length.");
        }

        let mut board: Board = Self::new();
        let board_str: &str = fen[0];
        let mut count: i32 = 0;

        let (mut file, mut rank) = (File::A, Rank::Eight);
        for token in board_str.chars() {
            match token {
                '/' => {
                    if count != 8 {
                        return Err("FEN string contains invalid delimiters.");
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

        match fen[1] {
            "w" => {
                board.side = Color::White;
                board.zobrist.hash_side();
            }
            "b" => board.side = Color::Black,
            _ => return Err("Invalid side to move, should be 'w' or 'b'."),
        }

        let castle_rights: CastleRights = fen[2].parse()?;
        board.castling = castle_rights;
        board.zobrist.hash_castle(castle_rights);

        match fen[3] {
            "-" => board.enpassant_square = None,
            _ => {
                let ep_square: Square = fen[3].parse()?;
                if ep_square.rank() != Rank::Three && ep_square.rank() != Rank::Six {
                    return Err("En passant square is invalid.");
                }

                board.enpassant_square = Some(ep_square);
                board.zobrist.hash_enpassant(ep_square);
            }
        }

        match fen[4].parse::<u8>() {
            Ok(half_move) if half_move <= 100 => board.fifty_move = half_move,
            Ok(_) => return Err("Halfmove Clock exceeds the maximum allowed value."),
            Err(_) => return Err("Invalid Halfmove Clock"),
        }

        match fen[5].parse::<u16>() {
            Ok(full_move) if full_move > 0 => board.full_move = full_move,
            Ok(_) => return Err("Fullmove number must be positive"),
            Err(_) => return Err("Invalid Fullmove Number"),
        }

        board.checkers = board.checkers();

        Ok(board)
    }
}

/// Constructs a default chess board, representing the standard starting position
/// for a chess game, using FEN notation. The default position is the classic setup
/// with castling rights and no en passant.
impl Default for Board {
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
    pub const fn new() -> Self {
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
    pub fn to_fen(&self) -> String {
        let mut fen: String = String::new();

        for rank in (0..Rank::NUM_RANKS).rev() {
            let mut empty_squares: i32 = 0;

            for file in 0..File::NUM_FILES {
                let square_index: usize = rank * 8 + file;

                if let Some(piece) = self.piece_map[square_index] {
                    if empty_squares > 0 {
                        fen.push_str(&empty_squares.to_string());
                        empty_squares = 0;
                    }
                    fen.push(piece.to_char());
                } else {
                    empty_squares += 1;
                }
            }

            if empty_squares > 0 {
                fen.push_str(&empty_squares.to_string());
            }

            if rank != Rank::One.to_index() {
                fen.push('/');
            }
        }

        fen.push(' ');
        fen.push(match self.side {
            Color::White => 'w',
            Color::Black => 'b',
        });
        fen.push(' ');

        fen.push_str(&self.castling.to_string());
        fen.push(' ');

        if let Some(enpassant_square) = self.enpassant_square {
            fen.push_str(&enpassant_square.to_string());
        } else {
            fen.push('-');
        }

        fen.push_str(&format!(" {} {}", self.fifty_move, self.full_move));

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
}

#[test]
fn test_board() {
    let board: Board = Board::new();
    println!("{}", board);
}

#[test]
fn test_from_string() {
    let board: Board =
        Board::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    println!("{}", board);
    println!("{}", board.to_fen());
}

#[test]
fn test_default() {
    let board: Board =
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let board_default: Board = Board::default();
    println!("{}", board);
    assert_eq!(board, board_default);
}
