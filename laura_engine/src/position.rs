/*
    Laura: A single-threaded UCI chess engine written in Rust.

    Copyright (C) 2024-2025 HansTibberio <hanstiberio@proton.me>

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

use crate::{evaluation, thread::Thread};
use laura_core::{enumerate_legal_moves, AllMoves, Board, Color, Move};
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

#[derive(Default, Debug, Clone)]
pub struct Position {
    board: Board,
    game: Vec<Board>,
}

impl Position {
    pub fn board(&self) -> Board {
        self.board
    }

    #[inline(always)]
    pub fn key(&self) -> u64 {
        self.board.zobrist.0
    }

    pub fn set_board(&mut self, board: Board) {
        self.board = board
    }

    pub fn perft(&self, depth: u8) -> u64 {
        let total_nodes: u64 = perft::<false>(&self.board, depth);
        total_nodes
    }

    pub fn divided_perft(&self, depth: u8) -> u64 {
        let total_nodes: u64 = perft::<true>(&self.board, depth);
        total_nodes
    }

    pub fn push_move(&mut self, mv: Move, thread: &mut Thread) {
        let new: Board = self.board.make_move(mv);
        let old: Board = replace(&mut self.board, new);
        self.game.push(old);

        thread.ply += 1;
        thread.nodes += 1;
    }

    pub fn push_null(&mut self, thread: &mut Thread) {
        let new: Board = self.board.null_move();
        let old: Board = replace(&mut self.board, new);
        self.game.push(old);

        thread.ply += 1;
        thread.nodes += 1;
    }

    pub fn pop_move(&mut self, thread: &mut Thread) {
        let old: Board = self.game.pop().unwrap();
        self.board = old;

        thread.ply -= 1;
    }

    pub fn evaluate(&self) -> i32 {
        evaluation::evaluate(&self.board)
    }

    pub fn ply(&self) -> usize {
        self.game.len()
    }

    pub fn in_check(&self) -> bool {
        self.board.checkers.count_bits() != 0
    }

    pub fn white(&self) -> bool {
        self.board.side == Color::White
    }
}
