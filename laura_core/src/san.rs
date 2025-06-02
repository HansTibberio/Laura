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

use crate::{
    enumerate_legal_moves, gen_moves, AllMoves, Board, Color, Move, Piece, PieceType, Square,
};
use core::fmt;

/// Converts a given move to its Standard Algebraic Notation (SAN) representation.
///
/// This function creates a [`SanBuffered`] instance that formats the move according to  
/// the rules of SAN notation. SAN notation is commonly used in chess notation to describe moves.
///
/// # Examples
///
/// ```
/// # use laura_core::*;
///
/// let board = Board::default();
/// let mv = Move::new(Square::A2, Square::A4, MoveType::Quiet);
///
/// assert_eq!(board.to_san(mv), "a4");
/// ```
pub fn to_san(mv: Move, board: &Board) -> SanBuffered {
    SanBuffered { mv, board: *board }
}

/// A wrapper that holds a move and the corresponding board state for SAN rendering.
///
/// The `SanBuffered` struct provides an efficient way to render a move in Standard Algebraic Notation (SAN),
/// using a precomputed board state to handle disambiguation, captures, promotions, and checks.
///
/// It also implements `Display` and `PartialEq<&str>` to easily print or compare the SAN representation.
///
/// # Examples
///
/// ```
/// # use laura_core::*;
///
/// let board = Board::default();
/// let mv = Move::new(Square::A2, Square::A4, MoveType::Quiet);
/// let san = to_san(mv, &board);
///
/// println!("{}", san); // Outputs: "a4"
/// assert_eq!(san, "a4");
/// ```
#[derive(Debug)]
pub struct SanBuffered {
    mv: Move,
    board: Board,
}

impl PartialEq<&str> for SanBuffered {
    fn eq(&self, other: &&str) -> bool {
        let mut buffer: [u8; 16] = [0u8; 16];
        let san_str: &str = self.render_san(&mut buffer);
        san_str == *other
    }
}

impl fmt::Display for SanBuffered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer: [u8; 16] = [0u8; 16];
        let san_str: &str = self.render_san(&mut buffer);
        write!(f, "{}", san_str)
    }
}

impl SanBuffered {
    /// Renders the move in Standard Algebraic Notation (SAN) and writes it into the provided buffer.
    fn render_san<'a>(&self, buffer: &'a mut [u8; 16]) -> &'a str {
        let mut idx: usize = 0;

        let new_board: Board = self.board.make_move(self.mv);
        let src: Square = self.mv.get_src();
        let dest: Square = self.mv.get_dest();
        let piece: Piece = self.board.piece_on(src).unwrap();
        let piece_type: PieceType = piece.piece_type();
        let is_promotion: bool = self.mv.is_promotion();
        let promotion: Option<Piece> = if is_promotion {
            Some(self.mv.get_prom(Color::White))
        } else {
            None
        };

        if piece_type == PieceType::King && self.mv.is_castle() {
            if self.mv.is_king_castle() {
                buffer[idx..idx + 3].copy_from_slice(b"O-O");
                idx += 3;
            } else {
                buffer[idx..idx + 5].copy_from_slice(b"O-O-O");
                idx += 5;
            }
        } else {
            if piece_type != PieceType::Pawn {
                buffer[idx] = piece_type.to_char() as u8;
                idx += 1;
            }

            if piece_type == PieceType::Pawn {
                if self.mv.is_capture() {
                    buffer[idx] = src.file().to_char() as u8;
                    idx += 1;
                }
            } else {
                let mut ambiguous: bool = false;
                let mut file_disambiguates: bool = true;
                let mut rank_disambiguates: bool = true;

                enumerate_legal_moves::<AllMoves, _>(&self.board, |candidate_mv| {
                    if candidate_mv == self.mv {
                        return true;
                    }
                    if candidate_mv.get_dest() != dest {
                        return true;
                    }
                    let candidate_src: Square = candidate_mv.get_src();
                    if let Some(candidate_piece) = self.board.piece_on(candidate_src) {
                        if candidate_piece.piece_type() == piece_type {
                            ambiguous = true;
                            if candidate_src.file() == src.file() {
                                file_disambiguates = false;
                            }
                            if candidate_src.rank() == src.rank() {
                                rank_disambiguates = false;
                            }
                        }
                    }
                    true
                });

                if ambiguous {
                    if file_disambiguates {
                        buffer[idx] = src.file().to_char() as u8;
                        idx += 1;
                    }
                    if rank_disambiguates {
                        buffer[idx] = src.rank().to_char() as u8;
                        idx += 1;
                    }
                    if !file_disambiguates && !rank_disambiguates {
                        buffer[idx] = src.file().to_char() as u8;
                        idx += 1;
                        buffer[idx] = src.rank().to_char() as u8;
                        idx += 1;
                    }
                }
            }

            if self.mv.is_capture() {
                buffer[idx] = b'x';
                idx += 1;
            }

            buffer[idx] = dest.file().to_char() as u8;
            idx += 1;
            buffer[idx] = dest.rank().to_char() as u8;
            idx += 1;

            if let Some(p) = promotion {
                buffer[idx] = b'=';
                idx += 1;
                buffer[idx] = p.piece_type().to_char() as u8;
                idx += 1;
            }

            let check: bool = !new_board.checkers.is_empty();
            let mate: bool = check && gen_moves::<AllMoves>(&new_board).is_empty();

            if mate {
                buffer[idx] = b'#';
                idx += 1;
            } else if check {
                buffer[idx] = b'+';
                idx += 1;
            }
        }

        unsafe { core::str::from_utf8_unchecked(&buffer[..idx]) }
    }
}
