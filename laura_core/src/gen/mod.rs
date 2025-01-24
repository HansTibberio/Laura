#[cfg(not(feature = "bmi2"))]
pub mod black_magics;
pub mod king;
pub mod knight;
pub mod pawn;
#[cfg(feature = "bmi2")]
pub mod pext;
pub mod random;
mod zobrist;
