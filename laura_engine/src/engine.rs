// src/engine.rs

//! Engine implementation

use std::sync::atomic::{AtomicBool, Ordering};

use laura_core::{enumerate_legal_moves, Board, ALL_MOVES};

use crate::timer::TimeManager;

fn perft<const DIV: bool>(board: &Board, depth: usize) -> usize {
    let start: std::time::Instant = std::time::Instant::now();
    let total_nodes: usize = inner_perft::<DIV>(board, depth);
    let duration: std::time::Duration = start.elapsed();

    let nps: f64 = total_nodes as f64 / duration.as_secs_f64();
    println!("{total_nodes} nodes in {duration:?} -> {nps:.0} nodes/s");

    total_nodes
}

#[allow(unused_assignments)]
fn inner_perft<const DIV: bool>(board: &Board, depth: usize) -> usize {
    let mut total: usize = 0;

    if !DIV && depth <= 1 {
        enumerate_legal_moves::<ALL_MOVES, _>(board, |_| -> bool {
            total += 1;
            true
        });
        return total;
    }

    enumerate_legal_moves::<ALL_MOVES, _>(board, |mv| -> bool {
        let mut nodes: usize = 0;
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

#[derive(Default, Debug)]
pub struct Engine {
    board: Board,
    pub timer: TimeManager,
    pub stop: AtomicBool,
}

impl Engine {
    pub fn board(&self) -> Board {
        self.board
    }

    pub fn set_board(&mut self, board: Board) {
        self.board = board
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Release);
    }

    pub fn is_stopped(&self) -> bool {
        self.stop.load(Ordering::Acquire)
    }

    pub fn perft(&self, depth: usize) -> usize {
        let total_nodes: usize = perft::<false>(&self.board, depth);
        total_nodes
    }

    pub fn divided_perft(&self, depth: usize) -> usize {
        let total_nodes: usize = perft::<true>(&self.board, depth);
        total_nodes
    }
}
