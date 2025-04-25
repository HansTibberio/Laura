use std::{
    fmt::Display,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread,
};

use laura_core::{legal_moves, Move, MoveList};

use crate::{
    config::{INFINITY, MATE, MAX_DEPTH, MAX_MATE},
    position::Position,
    search::{MainThread, PrincipalVariation, WorkerThread},
    timer::TimeControl,
    TimeManager,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchStack {
    pub score: i32,
    pub best_move: Option<Move>,
}

#[derive(Debug)]
pub struct Thread {
    pub search_stack: [SearchStack; MAX_DEPTH],
    pub time_manager: TimeManager,
    pub principal_variation: PrincipalVariation,
    pub nodes: u64,
    pub ply: usize,
    pub score: i32,
    pub depth: usize,
}

impl Display for Thread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let score: String = if self.score.abs() >= MAX_MATE {
            let mate_in: i32 = (MATE - self.score.abs() + 1) / 2;

            if self.score > 0 {
                format!("mate {}", mate_in)
            } else {
                format!("mate -{}", mate_in)
            }
        } else {
            format!("cp {}", self.score)
        };

        let time: u128 = self.time_manager.elapsed().as_millis().max(1);
        let nodes: u64 = self.time_manager.nodes();
        let nps: u128 = (nodes as u128 * 1000) / time;
        write!(
            f,
            "info time {} score {} depth {} nodes {} nps {} {}",
            time, score, self.depth, nodes, nps, self.principal_variation
        )
    }
}

impl Thread {
    pub fn new(time_manager: TimeManager) -> Self {
        Self {
            search_stack: [SearchStack::default(); MAX_DEPTH],
            principal_variation: PrincipalVariation::default(),
            nodes: 0,
            ply: 0,
            score: -INFINITY,
            depth: 0,
            time_manager,
        }
    }

    pub fn spinner(stop: Arc<AtomicBool>, nodes: Arc<AtomicU64>) -> Self {
        Self::new(TimeManager::spinner(stop, nodes))
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
    }
}

#[derive(Debug)]
pub struct ThreadPool {
    main: Thread,
    pool: Vec<Thread>,
    stop: Arc<AtomicBool>,
    nodes: Arc<AtomicU64>,
}

impl ThreadPool {
    pub fn new(stop: Arc<AtomicBool>) -> Self {
        let nodes: Arc<AtomicU64> = Arc::new(AtomicU64::new(0));
        Self {
            main: Thread::spinner(stop.clone(), nodes.clone()),
            pool: Vec::new(),
            stop,
            nodes,
        }
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }

    pub fn set_up(&mut self) {
        self.main.set_up();
        self.pool.iter_mut().for_each(|thread| thread.set_up());
    }

    pub fn resize(&mut self, threads: usize) {
        self.main = Thread::spinner(self.stop.clone(), self.nodes.clone());
        self.pool.resize_with(threads, || {
            Thread::spinner(self.stop.clone(), self.nodes.clone())
        });
    }

    pub fn start_search(
        &mut self,
        position: &mut Position,
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
                println!("info time 0 score mate 0 depth 0");
            } else {
                println!("info time 0 score cp 0 depth 0");
            }
            return None;
        }

        if moves.len() == 1 || self.main.time_manager.not_search() {
            return Some(moves.moves()[0]);
        }

        self.set_up();
        self.stop.store(false, Ordering::SeqCst);
        self.nodes.store(0, Ordering::SeqCst);

        let pcopy: Position = position.clone();
        thread::scope(|s| {
            s.spawn(|| {
                self.main.set_up();
                position.iterative_deepening::<MainThread>(&mut self.main);
                self.stop.store(true, Ordering::SeqCst);
            });
            for thread in self.pool.iter_mut() {
                s.spawn(|| {
                    let mut position: Position = pcopy.clone();
                    thread.set_up();
                    position.iterative_deepening::<WorkerThread>(thread);
                });
            }
        });
        let best: Move = self.main.best_move();
        Some(best)
    }
}

#[test]
fn test_best_move() {
    let mut threadpool = ThreadPool::new(Arc::new(AtomicBool::new(false)));
    let mut position = Position::default();
    let best = threadpool.start_search(&mut position, TimeControl::Depth(4));
    println!("Best: {}", best.unwrap());
}
