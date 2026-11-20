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

// src/position.rs

//! Position management.

use crate::{evaluation, tables::CorrKeys, thread::Thread};
use laura_core::{AllMoves, BitBoard, Board, Color, Move, Piece, PieceType, enumerate_legal_moves};
use std::{
    mem::replace,
    time::{Duration, Instant},
};

fn perft<const DIV: bool>(board: &Board, depth: u8) -> u64 {
    let start: Instant = Instant::now();
    let total_nodes: u64 = inner_perft::<DIV>(board, depth);
    let duration: Duration = start.elapsed();

    let nps: f64 = total_nodes as f64 / duration.as_secs_f64();
    println!("{total_nodes} nodes in {duration:?} -> {nps:.0} nodes/s");

    total_nodes
}

#[allow(unused_assignments)]
fn inner_perft<const DIV: bool>(board: &Board, depth: u8) -> u64 {
    let mut total: u64 = 0;

    if !DIV && depth <= 1 {
        enumerate_legal_moves::<AllMoves, _>(board, |_| -> bool {
            total += 1;
            true
        });
        return total;
    }

    enumerate_legal_moves::<AllMoves, _>(board, |mv| -> bool {
        let mut nodes: u64 = 0;
        if DIV && depth == 1 {
            nodes = 1;
        } else {
            let board_res: Board = board.make_move(mv);
            nodes = if depth == 1 {
                1
            } else {
                inner_perft::<false>(&board_res, depth - 1)
            };
        }

        total += nodes;

        if DIV && nodes > 0 {
            println!("{} -> {}", mv, nodes);
        }

        true
    });

    total
}

// SplitMix64 finalizer: spreads every input bit over the whole output.
#[inline(always)]
const fn mix(mut z: u64) -> u64 {
    z ^= z >> 30;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 27;
    z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

// Order-dependent hash of a set of bitboards (each one keeps its own role).
#[inline(always)]
fn hash_bitboards(bitboards: &[u64]) -> u64 {
    bitboards.iter().fold(
        0x9E37_79B9_7F4A_7C15,
        |hash, &bitboard| mix(hash ^ bitboard),
    )
}

#[derive(Default, Debug, Clone)]
pub struct Position {
    board: Board,
    game: Vec<Board>,
}

impl Position {
    #[inline(always)]
    pub fn board(&self) -> Board {
        self.board
    }

    #[inline(always)]
    pub fn key(&self) -> u64 {
        self.board.zobrist.0
    }

    #[inline(always)]
    pub fn set_board(&mut self, board: Board) {
        self.board = board
    }

    #[inline(always)]
    pub fn set_game(&mut self, history: Vec<Board>) {
        self.game = history;
    }

    pub fn perft(&self, depth: u8) -> u64 {
        let total_nodes: u64 = perft::<false>(&self.board, depth);
        total_nodes
    }

    pub fn divided_perft(&self, depth: u8) -> u64 {
        let total_nodes: u64 = perft::<true>(&self.board, depth);
        total_nodes
    }

    #[inline(always)]
    pub fn push_move(&mut self, mv: Move, thread: &mut Thread) {
        let new: Board = self.board.make_move(mv);
        let old: Board = replace(&mut self.board, new);
        let piece: Piece = unsafe { old.piece_on(mv.get_src()).unwrap_unchecked() };
        self.game.push(old);

        thread.push_move(piece, mv)
    }

    #[inline(always)]
    pub fn push_null(&mut self, thread: &mut Thread) {
        let new: Board = self.board.null_move();
        let old: Board = replace(&mut self.board, new);
        self.game.push(old);

        thread.push_null()
    }

    #[inline(always)]
    pub fn pop_move(&mut self, thread: &mut Thread) {
        let old: Board = self.game.pop().unwrap();
        self.board = old;

        thread.pop_move();
    }

    #[inline(always)]
    pub fn evaluate(&self) -> i32 {
        evaluation::evaluate(&self.board)
    }

    // Keys of the position components indexed by the correction histories.
    #[inline(always)]
    pub fn corr_keys(&self) -> CorrKeys {
        let board: &Board = &self.board;
        let white: u64 = board.sides_bitboard[Color::White as usize].0;
        let black: u64 = board.sides_bitboard[Color::Black as usize].0;
        let pawns: u64 = board.pieces_bitboard[PieceType::PAWN].0;
        let knights: u64 = board.pieces_bitboard[PieceType::KNIGHT].0;
        let bishops: u64 = board.pieces_bitboard[PieceType::BISHOP].0;
        let rooks: u64 = board.pieces_bitboard[PieceType::ROOK].0;
        let queens: u64 = board.pieces_bitboard[PieceType::QUEEN].0;
        let kings: u64 = board.pieces_bitboard[PieceType::KING].0;

        let non_pawn = |side: u64| -> u64 {
            hash_bitboards(&[
                knights & side,
                bishops & side,
                rooks & side,
                queens & side,
                kings & side,
            ])
        };

        CorrKeys {
            pawn: hash_bitboards(&[pawns & white, pawns & black]),
            non_pawn: [non_pawn(white), non_pawn(black)],
            minor: hash_bitboards(&[
                knights & white,
                bishops & white,
                knights & black,
                bishops & black,
            ]),
            major: hash_bitboards(&[rooks & white, queens & white, rooks & black, queens & black]),
        }
    }

    #[inline(always)]
    pub fn ply(&self) -> usize {
        self.game.len()
    }

    #[inline(always)]
    pub fn in_check(&self) -> bool {
        self.board.checkers.count_bits() != 0
    }

    #[inline(always)]
    pub fn white(&self) -> bool {
        self.board.side == Color::White
    }

    #[inline(always)]
    pub fn possible_zugzwang(&self) -> bool {
        (self.board.allied_presence() ^ self.board.allied_king() ^ self.board.allied_pawns())
            .is_empty()
    }

    #[inline(always)]
    pub fn is_draw(&self) -> bool {
        // Fifty-move rule
        if self.board.fifty_move >= 100 {
            return true;
        }

        // Threefold repetition
        let key: u64 = self.key();
        let max_back: usize = self.board.fifty_move as usize;
        if self
            .game
            .iter()
            .rev()
            .take(max_back)
            .skip(1)
            .step_by(2)
            .any(|board| board.zobrist.0 == key)
        {
            return true;
        }

        // Insufficient material
        if self.is_insufficient_material() {
            return true;
        }

        false
    }

    #[inline(always)]
    fn is_insufficient_material(&self) -> bool {
        let board: &Board = &self.board;
        let pawns: BitBoard = board.pawns();
        let rooks: BitBoard = board.rooks();
        let queens: BitBoard = board.queens();

        if !pawns.is_empty() || !rooks.is_empty() || !queens.is_empty() {
            return false;
        }

        let white: BitBoard = board.white_bitboard();
        let black: BitBoard = board.black_bitboard();
        let bishops: BitBoard = board.bishops();
        let knights: BitBoard = board.knights();

        let white_bishops: u32 = (white & bishops).count_bits();
        let black_bishops: u32 = (black & bishops).count_bits();
        let white_knights: u32 = (white & knights).count_bits();
        let black_knights: u32 = (black & knights).count_bits();

        let white_minors: u32 = white_knights + white_bishops;
        let black_minors: u32 = black_knights + black_bishops;

        match (white_minors, black_minors) {
            // KK
            (0, 0) => true,

            // KNK or KBK
            (1, 0) | (0, 1) => true,

            // KNNK
            (2, 0) if white_bishops == 0 => true,
            (0, 2) if black_bishops == 0 => true,

            // KBKB
            (1, 1) if white_knights == 0 && black_knights == 0 => {
                let white_bishop_bb: BitBoard = white & bishops;
                let black_bishop_bb: BitBoard = black & bishops;

                let wb_on_light: bool = !(white_bishop_bb & BitBoard::LIGHT_SQUARES).is_empty();
                let bb_on_light: bool = !(black_bishop_bb & BitBoard::LIGHT_SQUARES).is_empty();

                wb_on_light == bb_on_light
            }

            _ => false,
        }
    }
}
