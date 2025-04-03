// src/timer.rs

//! Timer implementation

use std::{
    str::{FromStr, SplitWhitespace},
    time::{Duration, Instant},
};

use crate::config::{
    DEFAULT_MOVESTOGO, INCREMENT_TIME_BASE, MINIMUM_TIME, MOVE_OVERHEAD, OPTIMAL_TIME_BASE,
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
                "movestogo" => movestogo = Some(parse_value(&mut tokens, "movestogo")?),
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

#[derive(Debug, Clone, Copy)]
pub struct TimeManager {
    // Time Control
    control: TimeControl,
    // Starting Time
    start_time: Instant,
    //  Soft Limit Time
    soft_limit: Duration,
    // Hard Limit Time
    hard_limit: Duration,
}

impl Default for TimeManager {
    fn default() -> Self {
        Self {
            control: TimeControl::Infinite,
            start_time: Instant::now(),
            soft_limit: Duration::default(),
            hard_limit: Duration::default(),
        }
    }
}

impl TimeManager {
    pub fn start(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn set_control(&mut self, time_control: TimeControl) {
        self.control = time_control
    }
}

pub fn calculate_time(remaining: u64, increment: u64, movestogo: Option<u64>) -> (u64, u64) {
    let max_time: u64 = remaining.saturating_sub(MOVE_OVERHEAD);

    let limit_time: u64 = if let Some(movestogo) = movestogo {
        max_time / movestogo
    } else {
        (max_time / DEFAULT_MOVESTOGO) + (increment * INCREMENT_TIME_BASE / 100)
    };

    let hard_time: u64 = limit_time.max(MINIMUM_TIME);
    let soft_time: u64 = (hard_time.min(max_time) * OPTIMAL_TIME_BASE / 100).max(MINIMUM_TIME);

    (soft_time, hard_time)
}

#[test]
fn test() {
    let (soft, hard) = calculate_time(100, 0, None);
    println!("Soft: {}, Hard: {}", soft, hard);
}
