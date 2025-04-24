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
    pub fn iterative_deepening<T>(&mut self, thread: &mut Thread)
    where
        T: ThreadType,
    {
        let mut pv_table: PrincipalVariation = PrincipalVariation::default();

        let alpha: i32 = -INFINITY;
        let beta: i32 = INFINITY;
        while thread.depth < MAX_DEPTH && thread.time_manager.go_search(thread.depth + 1) {
            let score: i32 = self.negamax(thread, thread.depth + 1, alpha, beta, &mut pv_table);

            if thread.time_manager.should_stop() {
                break;
            }

            thread.principal_variation = pv_table;
            thread.score = score;
            thread.depth += 1;

            if T::MAIN {
                println!("{}", thread);
            }
        }
    }

    #[allow(unused_assignments)]
    pub fn negamax(
        &mut self,
        thread: &mut Thread,
        depth: usize,
        mut alpha: i32,
        beta: i32,
        pv_table: &mut PrincipalVariation,
    ) -> Score {
        if thread.time_manager.should_stop() {
            return 0;
        }

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
            let score = -self.negamax(thread, depth - 1, -beta, -alpha, &mut old_pv);
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

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use laura_core::Board;

    use crate::{search::MainThread, thread::Thread, Position, TimeManager};

    #[test]
    fn test_negamax() {
        let mut position: Position = Position::default();
        position.set_board(
            Board::from_str("r1bqkb1r/pppppppp/5n2/8/1nB5/2N1P3/PPPP1PPP/R1BQK1NR w KQkq - 0 1")
                .unwrap(),
        );
        let mut thread: Thread = Thread::new(TimeManager::fixed_depth(5));
        let _ = position.iterative_deepening::<MainThread>(&mut thread);
        println!("Score: {}", thread.score);
        println!("Best move: {}", thread.best_move());
    }
}
