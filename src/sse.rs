/*
    Laura: A multi-threaded UCI chess engine written in Rust.

    Copyright (C) 2024-2026 HansTibberio <hanstiberio@proton.me>

    Laura is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Laura is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Laura. If not, see <https://www.gnu.org/licenses/>.
*/

// src/see.rs

use laura_core::{
    BitBoard, Board, Color, Move, MoveType, Piece, PieceType, Square, get_bishop_attacks,
    get_rook_attacks,
};

/// Static Exchange Evaluation
#[allow(clippy::upper_case_acronyms)]
pub struct SEE;

impl SEE {
    // Static Exchange Evaluation taken from Carp with some adaptations
    // Source: https://github.com/dede1751/carp/blob/main/chess/src/board.rs#L388
    pub fn see(board: &Board, mv: Move, threshold: i32) -> bool {
        let src: Square = mv.get_src();
        let dest: Square = mv.get_dest();
        let move_type: MoveType = mv.get_type();

        // Castling cannot have bad SEE,
        if move_type == MoveType::QueenCastle || move_type == MoveType::KingCastle {
            return true;
        }

        // Piece being swapped off is the promoted piece
        let victim: Piece = if mv.is_promotion() {
            mv.get_prom(board.side())
        } else {
            board.piece_on(src).unwrap()
        };

        let mut move_value: i32 = if mv.is_capture() {
            if move_type == MoveType::EnPassant {
                Self::piece_value(PieceType::Pawn)
            } else {
                Self::piece_value(board.piece_on(dest).unwrap().piece_type())
            }
        } else {
            0
        };

        if mv.is_promotion() {
            move_value +=
                Self::piece_value(victim.piece_type()) - Self::piece_value(PieceType::Pawn);
        }

        // Lose if the balance is already in our opponent's favor and it's their turn
        let mut balance: i32 = move_value - threshold;
        if balance < 0 {
            return false;
        }

        // Win if the balance is still in our favor even if we lose the capturing piece
        balance -= Self::piece_value(victim.piece_type());
        if balance >= 0 {
            return true;
        }

        let diagonals: BitBoard = board.bishops() | board.queens();
        let linnears: BitBoard = board.rooks() | board.queens();

        let mut occupancies: BitBoard = board.combined_bitboard().pop_square(src).set_square(dest);
        if move_type == MoveType::EnPassant {
            let en_passant_square: Square = board.enpassant_square.unwrap().forward(!board.side);
            occupancies = occupancies.pop_square(en_passant_square);
        }

        // Get all pieces covering the exchange square and start exchanging
        let mut attackers: BitBoard = board.attackers(dest, occupancies) & occupancies;
        let mut side_to_move: Color = !board.side();

        loop {
            // SEE terminates when no recapture is possible.
            let own_attackers: BitBoard = attackers & board.sides_bitboard[side_to_move as usize];
            if own_attackers == BitBoard::EMPTY {
                break;
            }

            // Get the least valuable attacker and simulate the recapture
            let (attacker_square, attacker) =
                Self::smallest_attacker(board, own_attackers, side_to_move).unwrap();
            occupancies = occupancies.pop_square(attacker_square);

            let attacker_type: PieceType = attacker.piece_type();

            // Diagonal recaptures uncover bishops/queens
            attackers |= get_bishop_attacks(dest, occupancies) & diagonals;

            // Orthogonal recaptures uncover rooks/queens
            attackers |= get_rook_attacks(dest, occupancies) & linnears;

            // Ignore pieces already "used up"
            attackers &= occupancies;

            // Negamax the balance, cutoff if losing our attacker would still win the exchange
            side_to_move = !side_to_move;
            balance = -balance - 1 - Self::piece_value(attacker_type);

            if balance >= 0 {
                // If the recapturing piece is a king, and the opponent has another attacker,
                // a positive balance should not translate to an exchange win.
                if attacker_type == PieceType::King
                    && attackers & board.sides_bitboard[side_to_move as usize] != BitBoard::EMPTY
                {
                    return board.side() == side_to_move;
                }
                break;
            }
        }

        // We win the exchange if we are not the one who should recapture
        board.side() != side_to_move
    }

    fn smallest_attacker(
        board: &Board,
        attackers: BitBoard,
        side: Color,
    ) -> Option<(Square, Piece)> {
        let side_bitboard: BitBoard = board.sides_bitboard[side as usize];
        const PIECES_BY_COLOR: [[Piece; Piece::COUNT]; 2] = [
            [
                Piece::WP,
                Piece::WN,
                Piece::WB,
                Piece::WR,
                Piece::WQ,
                Piece::WK,
            ],
            [
                Piece::BP,
                Piece::BN,
                Piece::BB,
                Piece::BR,
                Piece::BQ,
                Piece::BK,
            ],
        ];
        for piece in PIECES_BY_COLOR[side as usize] {
            let square: BitBoard =
                attackers & board.pieces_bitboard[piece.piece_index()] & side_bitboard;

            if square != BitBoard::EMPTY {
                return Some((square.to_square().unwrap(), piece));
            }
        }

        None
    }

    pub fn piece_value(piece_type: PieceType) -> i32 {
        match piece_type {
            PieceType::Pawn => 94,
            PieceType::Knight => 281,
            PieceType::Bishop => 297,
            PieceType::Rook => 512,
            PieceType::Queen => 936,
            PieceType::King => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_see_epd() {
        fn parse_epd_line(line: &str) -> Option<(&str, &str, bool)> {
            let mut parts = line.split(';');

            let fen: &str = parts.next()?.trim();
            let mv: &str = parts.next()?.trim();
            let result: &str = parts.next()?.trim();

            let expected: bool = match result {
                "true" => true,
                "false" => false,
                _ => return None,
            };

            Some((fen, mv, expected))
        }

        let content: String =
            std::fs::read_to_string("tests/see.epd").expect("Failed to read EPD file");

        let mut passed: i32 = 0;
        let mut failed: i32 = 0;

        for (i, line) in content.lines().enumerate() {
            let line: &str = line.split("//").next().unwrap().trim();

            if line.is_empty() {
                continue;
            }

            let (fen, mv_str, expected) = match parse_epd_line(line) {
                Some(v) => v,
                None => {
                    println!("Parse error at line {}: {}", i + 1, line);
                    failed += 1;
                    continue;
                }
            };

            let board: Board = match Board::from_str(fen) {
                Ok(b) => b,
                Err(_) => {
                    println!("Invalid FEN at line {}: {}", i + 1, fen);
                    failed += 1;
                    continue;
                }
            };

            let mv: Move = match board.find_move(mv_str) {
                Some(m) => m,
                None => {
                    println!("Illegal move at line {}: {}", i + 1, mv_str);
                    failed += 1;
                    continue;
                }
            };

            let result: bool = SEE::see(&board, mv, 0);

            if result == expected {
                passed += 1;
            } else {
                failed += 1;

                println!(
                    "\nSEE mismatch at line {}\nFEN: {}\nMove: {}\nExpected: {}\nActual: {}\n",
                    i + 1,
                    fen,
                    mv_str,
                    expected,
                    result
                );
            }
        }

        println!(
            "\nSEE test summary\nPassed: {}\nFailed: {}\n",
            passed, failed
        );

        assert_eq!(failed, 0, "SEE test failed with {} incorrect cases", failed);
    }
}
