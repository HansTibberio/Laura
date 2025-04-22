use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    thread,
};

use laura_core::{legal_moves, Move, MoveList};

use crate::{
    config::{INFINITY, MAX_DEPTH},
    position::Position,
    search::{MainThread, PrincipalVariation, WorkerThread},
    timer::TimeManager,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchStack {
    pub score: i32,
    pub best_move: Option<Move>,
}

#[derive(Debug)]
pub struct Thread {
    pub search_stack: [SearchStack; MAX_DEPTH],

    pub principal_variation: PrincipalVariation,
    pub nodes: u64,
    pub ply: usize,

    pub score: i32,
    pub depth: usize,
    pub stop: bool,
}

impl Thread {
    pub fn new() -> Self {
        Self {
            search_stack: [SearchStack::default(); MAX_DEPTH],
            principal_variation: PrincipalVariation::default(),
            nodes: 0,
            ply: 0,
            score: -INFINITY,
            depth: 0,
            stop: false,
        }
    }

    pub fn best_move(&self) -> Move {
        self.principal_variation.best_move()
    }

    pub fn set_up(&mut self) {
        self.principal_variation = PrincipalVariation::default();
        self.nodes = 0;
        self.ply = 0;
        self.score = -INFINITY;
        self.depth = 0;
        self.stop = false;
    }
}

#[derive(Debug)]
pub struct ThreadPool {
    pub main: Thread,
    pub pool: Vec<Thread>,
    pub time_manager: TimeManager,
    stop: AtomicBool,
    nodes: AtomicU64,
}

impl ThreadPool {
    pub fn new(stop: AtomicBool) -> Self {
        Self {
            main: Thread::new(),
            pool: Vec::new(),
            time_manager: TimeManager::default(),
            stop,
            nodes: AtomicU64::new(0),
        }
    }

    pub fn set_up(&mut self) {
        self.main.set_up();
        self.pool.iter_mut().for_each(|thread| thread.set_up());
    }

    pub fn resize(&mut self, threads: usize) {
        self.main = Thread::new();
        self.pool.resize_with(threads - 1, Thread::new);
    }

    pub fn start_search(
        &mut self,
        position: &mut Position,
        time_manager: TimeManager,
    ) -> Option<Move> {
        let moves: MoveList = legal_moves!(&position.board());
        if moves.is_empty() {
            if position.in_check() {
                println!("info time 0 score mate 0 depth 0");
            } else {
                println!("info time 0 score cp 0 depth 0");
            }
            return None;
        }

        if moves.len() == 1 {
            return Some(moves.moves()[0]);
        }

        self.time_manager = time_manager;

        self.set_up();
        self.stop.store(false, Ordering::Release);
        self.nodes = AtomicU64::new(0);

        let pcopy: Position = position.clone();
        thread::scope(|s| {
            s.spawn(|| {
                self.main.set_up();
                // TODO! delete the depth
                position.iterative_deepening::<MainThread>(&mut self.main, 5);
                self.stop.store(true, Ordering::Release);
            });
            for thread in self.pool.iter_mut() {
                let mut position: Position = pcopy.clone();
                thread.set_up();
                // TODO! delete the depth
                position.iterative_deepening::<WorkerThread>(thread, 5);
            }
        });

        let best: Move = self.main.best_move();
        Some(best)
    }
}

#[test]
fn test_resize() {
    let mut threadpool = ThreadPool::new(AtomicBool::new(false));
    let mut position = Position::default();
    let time_manager = TimeManager::default();
    let best = threadpool.start_search(&mut position, time_manager);
    println!("Best: {}", best.unwrap());
}
