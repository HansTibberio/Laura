/*
    Laura: A multi-threaded UCI chess engine written in Rust.

    Copyright (C) 2024-2026 HansTibberio <hanstiberio@proton.me>

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

// src/timer.rs

//! Timer implementation

use crate::config::{
    DEFAULT_MOVESTOGO, HARD_TIME_BASE, HARD_TIME_MAX_PCT, INCREMENT_TIME_BASE, MINIMUM_TIME,
    SOFT_TIME_BASE, TM_NODE_BASE, TM_NODE_SCALE, TM_SCALE_MAX, TM_SCALE_MIN, TM_STABILITY,
};
use std::{
    str::{FromStr, SplitWhitespace},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy)]
pub enum TimeControl {
    Depth(u32),
    MoveTime(u64),
    DynamicTime {
        wtime: u64,
        btime: u64,
        winc: Option<u64>,
        binc: Option<u64>,
        movestogo: Option<u64>,
    },
    Nodes(u64),
    Infinite,
}

#[derive(Debug)]
pub enum TimeParserError {
    MissingValue(String),
    InvalidValue,
    UnknownParameter(String),
}

impl FromStr for TimeControl {
    type Err = TimeParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens: SplitWhitespace<'_> = s.split_whitespace();

        let mut depth: Option<u32> = None;
        let mut movetime: Option<u64> = None;
        let mut wtime: Option<u64> = None;
        let mut btime: Option<u64> = None;
        let mut winc: Option<u64> = None;
        let mut binc: Option<u64> = None;
        let mut movestogo: Option<u64> = None;
        let mut nodes: Option<u64> = None;

        while let Some(token) = tokens.next() {
            match token {
                "depth" => depth = Some(parse_value::<u32>(&mut tokens, "depth")?),
                "movetime" => movetime = Some(parse_value::<u64>(&mut tokens, "movetime")?),
                "wtime" => wtime = Some(parse_value::<u64>(&mut tokens, "wtime")?),
                "btime" => btime = Some(parse_value::<u64>(&mut tokens, "btime")?),
                "winc" => winc = Some(parse_value::<u64>(&mut tokens, "winc")?),
                "binc" => binc = Some(parse_value::<u64>(&mut tokens, "binc")?),
                "movestogo" => movestogo = Some(parse_value::<u64>(&mut tokens, "movestogo")?),
                "nodes" => nodes = Some(parse_value::<u64>(&mut tokens, "nodes")?),
                "infinite" => return Ok(Self::Infinite),
                _ => return Err(TimeParserError::UnknownParameter(token.to_string())),
            };
        }

        if let Some(depth) = depth {
            return Ok(Self::Depth(depth));
        }

        if let Some(movetime) = movetime {
            return Ok(Self::MoveTime(movetime));
        }

        if let (Some(wtime), Some(btime)) = (wtime, btime) {
            if (winc.is_some() && binc.is_some()) || (winc.is_none() && binc.is_none()) {
                return Ok(Self::DynamicTime {
                    wtime,
                    btime,
                    winc,
                    binc,
                    movestogo,
                });
            } else {
                return Err(TimeParserError::MissingValue(
                    "Missing some (wtime, btime, winc, binc, or movestogo)".to_string(),
                ));
            }
        }

        if let Some(nodes) = nodes {
            return Ok(Self::Nodes(nodes));
        }

        Err(TimeParserError::InvalidValue)
    }
}

impl TimeControl {
    pub fn depth(&self) -> Option<usize> {
        match self {
            TimeControl::Depth(depth) => Some(*depth as usize),
            _ => None,
        }
    }
}

fn parse_value<T: FromStr>(
    tokens: &mut SplitWhitespace<'_>,
    key: &str,
) -> Result<T, TimeParserError> {
    tokens
        .next()
        .ok_or_else(|| TimeParserError::MissingValue(key.to_string()))?
        .parse::<T>()
        .map_err(|_| TimeParserError::InvalidValue)
}

#[derive(Debug, Clone)]
pub struct TimeManager {
    // Time Control
    time_control: TimeControl,
    // Starting Time
    start_time: Instant,
    // Soft Limit Time (rescaled during the search)
    soft_limit: Duration,
    // Soft Limit Time before any scaling
    base_soft: Duration,
    // Hard Limit Time
    hard_limit: Duration,
    // Stop Timer
    stop: Arc<AtomicBool>,
    // Node Count
    nodes: Arc<AtomicU64>,
    // Node Count Buffer
    buffer: u64,
}

impl TimeManager {
    pub fn new(
        stop: Arc<AtomicBool>,
        nodes: Arc<AtomicU64>,
        time_control: TimeControl,
        white: bool,
        move_overhead: u64,
    ) -> Self {
        let (soft_limit, hard_limit) = match time_control {
            TimeControl::Depth(_) => (Duration::ZERO, Duration::ZERO),
            TimeControl::MoveTime(time) => (
                Duration::from_millis(time - move_overhead.min(time)),
                Duration::from_millis(time - move_overhead.min(time)),
            ),
            TimeControl::DynamicTime {
                wtime,
                btime,
                winc,
                binc,
                movestogo,
            } => {
                let (remaining, increment) = if white {
                    match winc {
                        Some(winc) => (wtime, winc),
                        None => (wtime, 0),
                    }
                } else {
                    match binc {
                        Some(binc) => (btime, binc),
                        None => (btime, 0),
                    }
                };

                let (soft, hard) = calculate_time(remaining, increment, movestogo, move_overhead);

                (Duration::from_millis(soft), Duration::from_millis(hard))
            }
            TimeControl::Nodes(_) => (Duration::ZERO, Duration::ZERO),
            TimeControl::Infinite => (Duration::ZERO, Duration::ZERO),
        };

        Self {
            time_control,
            start_time: Instant::now(),
            soft_limit,
            base_soft: soft_limit,
            hard_limit,
            stop,
            nodes,
            buffer: 0,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn stopped(&self) -> bool {
        self.stop.load(Ordering::SeqCst)
    }

    pub fn time_control(&self) -> TimeControl {
        self.time_control
    }

    pub fn set_control(&mut self, time_control: TimeControl) {
        self.time_control = time_control
    }

    pub fn nodes(&self) -> u64 {
        self.nodes.load(Ordering::SeqCst)
    }

    pub fn stop_soft(&mut self) -> bool {
        if self.stop.load(Ordering::SeqCst) {
            return true;
        }

        let stop: bool = match self.time_control {
            TimeControl::MoveTime(_) | TimeControl::DynamicTime { .. } => {
                self.elapsed() >= self.soft_limit
            }
            TimeControl::Nodes(control_nodes) => self.nodes() >= control_nodes,
            _ => false,
        };

        if stop {
            self.stop.store(true, Ordering::SeqCst);
        }

        stop
    }

    pub fn stop_hard(&mut self, nodes: u64) -> bool {
        if self.stop.load(Ordering::SeqCst) {
            return true;
        }
        let searched: u64 = nodes - self.buffer;

        if searched < 1024 {
            return false;
        }
        self.nodes.fetch_add(searched, Ordering::Relaxed);
        self.buffer = nodes;

        let stop: bool = match self.time_control {
            TimeControl::Depth(_) | TimeControl::Infinite => self.stop.load(Ordering::SeqCst),
            TimeControl::MoveTime(time) => {
                let past_time: bool = self.elapsed().as_millis() as u64 >= time;
                past_time
            }
            TimeControl::DynamicTime { .. } => self.elapsed() >= self.hard_limit,
            TimeControl::Nodes(control_nodes) => self.nodes() >= control_nodes,
        };

        if stop {
            self.stop.store(true, Ordering::SeqCst);
        }

        stop
    }

    pub fn not_search(&self) -> bool {
        match self.time_control {
            TimeControl::MoveTime(_) | TimeControl::DynamicTime { .. } => {
                self.hard_limit == Duration::ZERO
            }
            _ => false,
        }
    }

    pub fn reset_buffer(&mut self) {
        self.buffer = 0;
    }

    // Rescales the soft limit from what the search sees. Called by the main
    // thread after each completed iteration; only affects clock-based searches.
    pub fn update_soft(&mut self, stability: usize, best_fraction: f64) {
        if !matches!(self.time_control, TimeControl::DynamicTime { .. }) {
            return;
        }

        let stability_factor: f64 = TM_STABILITY[stability.min(TM_STABILITY.len() - 1)];
        let node_factor: f64 = (TM_NODE_BASE - best_fraction.clamp(0.0, 1.0)) * TM_NODE_SCALE;
        let scale: f64 = (stability_factor * node_factor).clamp(TM_SCALE_MIN, TM_SCALE_MAX);

        self.soft_limit = self.base_soft.mul_f64(scale).min(self.hard_limit);
    }

    #[cfg(test)]
    pub fn limits(&self) -> (Duration, Duration) {
        (self.soft_limit, self.hard_limit)
    }
}

pub fn calculate_time(
    remaining: u64,
    increment: u64,
    movestogo: Option<u64>,
    move_overhead: u64,
) -> (u64, u64) {
    let max_time: u64 = remaining.saturating_sub(move_overhead);
    let movestogo: u64 = movestogo.unwrap_or(DEFAULT_MOVESTOGO).max(1);

    // Ideal time for this move: an even share of the clock plus most of the increment.
    let ideal_time: u64 = max_time / movestogo + increment * INCREMENT_TIME_BASE / 100;

    // The hard limit leaves room for the soft limit to stretch when the search is unstable.
    let hard_time: u64 = (ideal_time * HARD_TIME_BASE / 100)
        .min(max_time * HARD_TIME_MAX_PCT / 100)
        .max(MINIMUM_TIME);
    let soft_time: u64 = (ideal_time * SOFT_TIME_BASE / 100)
        .max(MINIMUM_TIME)
        .min(hard_time);

    (soft_time, hard_time)
}

#[cfg(test)]
mod test {
    use crate::config::MOVE_OVERHEAD;
    use crate::timer::{TimeControl, TimeManager, calculate_time};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64},
    };

    #[test]
    fn test() {
        for (time, inc) in [
            (20, 0),
            (1_000, 100),
            (10_000, 100),
            (60_000, 600),
            (300_000, 0),
        ] {
            let (soft, hard) = calculate_time(time, inc, None, MOVE_OVERHEAD);
            println!("{time}+{inc}: soft {soft} ms, hard {hard} ms");
            assert!(soft <= hard);
        }
    }

    #[test]
    fn soft_scaling() {
        let control: TimeControl = "wtime 10000 btime 10000 winc 100 binc 100".parse().unwrap();
        let mut tm = TimeManager::new(
            Arc::new(AtomicBool::new(false)),
            Arc::new(AtomicU64::new(0)),
            control,
            true,
            MOVE_OVERHEAD,
        );
        let (base, hard) = tm.limits();

        // Best move just changed and shares the nodes: think longer.
        tm.update_soft(0, 0.3);
        let (unstable, _) = tm.limits();
        // Best move held for many iterations and dominates: stop earlier.
        tm.update_soft(10, 0.95);
        let (stable, _) = tm.limits();

        println!("base {base:?} unstable {unstable:?} stable {stable:?} hard {hard:?}");
        assert!(unstable > base && stable < base && unstable <= hard);
    }
}
