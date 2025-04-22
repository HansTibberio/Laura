// src/search.rs

//! Search implementation

use std::fmt;

use laura_core::{legal_moves, Move, MoveList};

use crate::{
    config::{Score, INFINITY, MAX_DEPTH},
    position::Position,
    thread::Thread,
};

pub trait ThreadType {
    const MAIN: bool;
}
pub struct MainThread;
pub struct WorkerThread;

impl ThreadType for MainThread {
    const MAIN: bool = true;
}
impl ThreadType for WorkerThread {
    const MAIN: bool = false;
}

#[derive(Debug, Clone, Copy)]
pub struct PrincipalVariation {
    moves: [Move; MAX_DEPTH],
    len: usize,
}

impl fmt::Display for PrincipalVariation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pv")?;

        for mv in self.moves() {
            write!(f, " {mv}")?;
        }

        Ok(())
    }
}

impl Default for PrincipalVariation {
    fn default() -> Self {
        Self {
            moves: [Move::null(); MAX_DEPTH],
            len: 0,
        }
    }
}

impl PrincipalVariation {
    pub fn push_line(&mut self, mv: Move, old: &Self) {
        self.len = old.len + 1;
        self.moves[0] = mv;
        self.moves[1..=old.len].copy_from_slice(&old.moves[..old.len]);
    }

    pub fn set_len(&mut self, len: usize) {
        self.len = len
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn moves(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    pub fn best_move(&self) -> Move {
        self.moves[0]
    }
}

impl Position {
    #[allow(clippy::extra_unused_type_parameters)]
    pub fn iterative_deepening<ThreadType>(&mut self, thread: &mut Thread, depth: u8) {
        let mut pv_table: PrincipalVariation = PrincipalVariation::default();

        let alpha: i32 = -INFINITY;
        let beta: i32 = INFINITY;
        let score: i32 = self.negamax(depth, alpha, beta, &mut pv_table);
        thread.principal_variation = pv_table;
        thread.score = score;
    }

    #[allow(unused_assignments)]
    pub fn negamax(
        &mut self,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        pv_table: &mut PrincipalVariation,
    ) -> Score {
        if depth == 0 {
            return self.evaluate();
        }
        let mut old_pv: PrincipalVariation = PrincipalVariation::default();
        pv_table.set_len(0);

        let moves: MoveList = legal_moves!(&self.board());
        let mut best_move: Move = Move::default();
        let mut best_score: Score = -INFINITY;

        for mv in moves {
            self.push_move(mv);
            let score = -self.negamax(depth - 1, -beta, -alpha, &mut old_pv);
            self.pop_move();

            if score > best_score {
                best_score = score;

                if score > alpha {
                    best_move = mv;
                    pv_table.push_line(best_move, &old_pv);
                    alpha = score;
                }

                if score >= beta {
                    alpha = beta;

                    break;
                }
            }
        }

        alpha
    }
}

#[test]
fn test_negamax() {
    let mut position: Position = Position::default();
    let mut thread: Thread = Thread::new();
    let _ = position.iterative_deepening::<MainThread>(&mut thread, 6);
    println!("Score: {}", thread.score);
    println!("{}", thread.principal_variation);
    println!("Best move: {}", thread.best_move());
}
