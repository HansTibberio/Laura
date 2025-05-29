/*
    Laura: A multi-threaded UCI chess engine written in Rust.

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

// src/thread.rs

//! Thread management for parallel search.

use crate::{
    position::Position,
    search::{MainThread, PrincipalVariation, WorkerThread},
    tables::KillerMoves,
    timer::TimeControl,
    transposition::TranspositionTable,
    TimeManager,
};
use laura_core::{legal_moves, Move, MoveList};
use std::{
    collections::HashMap,
    iter::once,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread,
};

#[derive(Debug)]
pub struct Thread {
    pub id: usize,
    pub time_manager: TimeManager,
    pub principal_variation: PrincipalVariation,
    pub killer: KillerMoves,
    pub nodes: u64,
    pub ply: usize,
    pub seldepth: usize,
    pub score: i32,
    pub depth: usize,
    pub completed: usize,
}

impl Thread {
    pub fn new(time_manager: TimeManager, id: usize) -> Self {
        Self {
            id,
            principal_variation: PrincipalVariation::default(),
            nodes: 0,
            ply: 0,
            seldepth: 0,
            score: 0,
            depth: 0,
            time_manager,
            killer: KillerMoves::default(),
            completed: 0,
        }
    }

    pub fn smp(stop: Arc<AtomicBool>, nodes: Arc<AtomicU64>, id: usize) -> Self {
        Self::new(
            TimeManager::new(stop, nodes, TimeControl::Infinite, false),
            id,
        )
    }

    pub fn best_move(&self) -> Move {
        self.principal_variation.moves[0]
    }

    pub fn set_up(&mut self) {
        self.principal_variation = PrincipalVariation::default();
        self.time_manager.reset_buffer();
        self.nodes = 0;
        self.ply = 0;
        self.seldepth = 0;
        self.score = 0;
        self.depth = 0;
        self.completed = 0;
    }
}

#[derive(Debug)]
pub struct ThreadPool {
    main: Thread,
    pool: Vec<Thread>,
    pub threads: usize,
    stop: Arc<AtomicBool>,
    nodes: Arc<AtomicU64>,
}

impl ThreadPool {
    pub fn new(stop: Arc<AtomicBool>) -> Self {
        let nodes: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
        Self {
            main: Thread::smp(stop.clone(), nodes.clone(), 0),
            pool: Vec::new(),
            threads: 1,
            stop,
            nodes,
        }
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    pub fn resize(&mut self, threads: usize) {
        let desired_size: usize = threads.saturating_sub(1);
        let current_size: usize = self.pool.len();

        if desired_size < current_size {
            self.pool.drain(desired_size..);
        }

        let mut id: usize = current_size + 1;
        self.pool.resize_with(desired_size, || {
            let thread_id: usize = id;
            id += 1;
            Thread::smp(self.stop.clone(), self.nodes.clone(), thread_id)
        });

        self.main = Thread::smp(self.stop.clone(), self.nodes.clone(), 0);
        self.threads = threads.max(1);
    }

    pub fn start_search(
        &mut self,
        position: &mut Position,
        ttable: &TranspositionTable,
        time_control: TimeControl,
    ) -> Option<Move> {
        self.main.time_manager = TimeManager::new(
            self.stop.clone(),
            self.nodes.clone(),
            time_control,
            position.white(),
        );

        let moves: MoveList = legal_moves!(&position.board());
        if moves.is_empty() {
            if position.in_check() {
                println!("info depth 0 score mate 0 time 0");
            } else {
                println!("info depth 0 score cp 0 time 0");
            }
            return None;
        }

        if moves.len() == 1 || self.main.time_manager.not_search() {
            return Some(moves[0]);
        }

        self.stop.store(false, Ordering::SeqCst);
        self.nodes.store(0, Ordering::SeqCst);

        let pcopy: Position = position.clone();
        thread::scope(|s| {
            s.spawn(|| {
                self.main.set_up();
                position.iterative_deepening::<MainThread>(&mut self.main, ttable);
                self.stop.store(true, Ordering::SeqCst);
            });
            for thread in self.pool.iter_mut() {
                s.spawn(|| {
                    let mut position: Position = pcopy.clone();
                    thread.set_up();
                    position.iterative_deepening::<WorkerThread>(thread, ttable);
                });
            }
        });

        let threads = once(&self.main).chain(self.pool.iter());

        let max_depth: usize = threads.clone().map(|t| t.completed).max().unwrap_or(0);

        let mut freq: HashMap<Move, usize> = HashMap::new();
        for thread in threads {
            if thread.completed == max_depth {
                if let Some(&mv) = thread.principal_variation.as_slice().first() {
                    *freq.entry(mv).or_insert(0) += 1;
                }
            }
        }

        let best_move: Option<Move> = freq
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(mv, _)| mv);

        best_move
    }
}

#[cfg(test)]
mod test {
    use crate::{timer::TimeControl, transposition::TranspositionTable, Position, ThreadPool};
    use laura_core::Move;
    use std::sync::{atomic::AtomicBool, Arc};

    #[test]
    fn test_best_move() {
        let mut threadpool: ThreadPool = ThreadPool::new(Arc::new(AtomicBool::new(false)));
        let mut position: Position = Position::default();
        let mut ttable: TranspositionTable = TranspositionTable::default();
        ttable.resize(1);
        let best: Option<Move> =
            threadpool.start_search(&mut position, &ttable, TimeControl::Depth(4));
        println!("bestmove {}", best.unwrap());
    }

    #[test]
    fn test_resize() {
        let mut threadpool = ThreadPool::new(Arc::new(AtomicBool::new(false)));
        threadpool.resize(4);
        for t in &threadpool.pool {
            println!("ID: {}", t.id)
        }
        threadpool.resize(6);
        for t in threadpool.pool {
            println!("ID: {}", t.id)
        }
    }
}
