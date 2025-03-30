// src/engine.rs

//! Engine

use laura_core::Board;

#[derive(Default, Clone, Debug)]
pub struct Engine {
    pub board: Board,
}

impl Engine {
    pub fn set_board(&mut self, board: Board) {
        self.board = board
    }
}
