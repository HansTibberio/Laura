// src/config.rs

//! Config File

// Timer parameters
pub const MOVE_OVERHEAD: u64 = 50;
pub const MINIMUM_TIME: u64 = 30;
pub const OPTIMAL_TIME_BASE: u64 = 65;
pub const INCREMENT_TIME_BASE: u64 = 85;
pub const DEFAULT_MOVESTOGO: u64 = 40;

// Search parameters

pub const INFINITY: i32 = 32_001;
pub const MATE: i32 = 32_000;
pub const MAX_MATE: i32 = MATE - MAX_PLY as i32;
pub const MAX_PLY: usize = 128;
pub const ASPIRATION_MARGIN: i32 = 25;
pub const ASPIRATION_DEPTH_THRESHOLD: usize = 5;
pub const MAX_DELTA: i32 = 1025;
