// src/config.rs

//! Config File

// Timer parameters
pub const MOVE_OVERHEAD: u64 = 100;
pub const MINIMUM_TIME: u64 = 30;
pub const OPTIMAL_TIME_BASE: u64 = 65;
pub const INCREMENT_TIME_BASE: u64 = 85;
pub const DEFAULT_MOVESTOGO: u64 = 40;

// Public types
pub type Nodes = u64;
pub type Eval = i32;
pub type Score = i32;

pub const INFINITY: Score = 32_000;
pub const MAX_DEPTH: usize = 127;
