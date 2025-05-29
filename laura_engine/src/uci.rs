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

// src/uci.rs

//! UCI protocol implementation

use crate::{
    config::DEFAULT_SIZE,
    position::Position,
    thread::ThreadPool,
    timer::{TimeControl, TimeParserError},
    transposition::TranspositionTable,
};
use laura_core::{Board, Move};
use std::{
    io::{stdin, BufRead, Stdin},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
        Arc,
    },
    thread,
};

const AUTHOR: &str = "HansTibberio";
const NAME: &str = "Laura";
const VERSION: &str = env!("CARGO_PKG_VERSION");

const THREADS_MIN: usize = 1;
const THREADS_MAX: usize = 512;
const HASH_MIN: usize = 1;
const HASH_MAX: usize = 1048576;

#[derive(Debug)]
pub enum UCICommand {
    Uci,
    IsReady,
    UciNewGame,
    Position(Board),
    Go(TimeControl),
    Stop,
    Quit,
    SetOption { name: String, value: String },
    DividePerft(u8),
    Perft(u8),
    Print,
    Eval,
    License,
    Help,
}

#[derive(Debug)]
pub enum UCIError {
    UnknownCommand(String),
    NoOptionValue,
    InvalidOptionValue,
    InvalidFenPosition,
    InvalidPositionFormat(String),
    InvalidSetOption,
    InvalidGo(TimeParserError),
    IlegalUciMove(String),
}

impl std::fmt::Display for UCIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UCIError::UnknownCommand(s) => write!(f, "[error] unkown command '{s}'."),
            UCIError::NoOptionValue => write!(f, "[error] no option value provided."),
            UCIError::InvalidOptionValue => write!(f, "[error] invalid option value."),
            UCIError::InvalidFenPosition => write!(f, "[error] invalid fen position format."),
            UCIError::InvalidPositionFormat(s) => write!(f, "[error] {s}"),
            UCIError::InvalidSetOption => write!(f, "[error] invalid setoption."),
            UCIError::InvalidGo(err) => write!(f, "[error] '{err:?}'"),
            UCIError::IlegalUciMove(s) => write!(f, "[error] ilegal uci move '{s}'."),
        }
    }
}

impl From<TimeParserError> for UCIError {
    fn from(err: TimeParserError) -> Self {
        Self::InvalidGo(err)
    }
}

impl FromStr for UCICommand {
    type Err = UCIError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: &str = s.trim();

        if s.is_empty() {
            return Err(UCIError::UnknownCommand(s.to_string()));
        }

        let mut tokens = s.split_whitespace();

        match tokens.next() {
            Some("uci") => Ok(Self::Uci),
            Some("isready") => Ok(Self::IsReady),
            Some("ucinewgame") => Ok(Self::UciNewGame),
            Some("position") => {
                let board: Board = match tokens.next() {
                    Some("startpos") => match tokens.next() {
                        Some("moves") => {
                            let mut board: Board = Board::default();
                            for uci_move in tokens {
                                if let Some(mv) = board.find_move(uci_move) {
                                    let board_res: Board = board.make_move(mv);
                                    board = board_res;
                                } else {
                                    return Err(UCIError::IlegalUciMove(uci_move.to_string()));
                                }
                            }
                            return Ok(Self::Position(board));
                        }
                        None => Board::default(),
                        Some(_) => {
                            return Err(UCIError::InvalidPositionFormat(
                                "unexpected token after 'startpos' (expected 'moves' or end of command)".to_string(),
                            ))
                        }
                    },
                    Some("fen") => {
                        let mut fen: String = String::with_capacity(128);
                        for token in tokens.by_ref().take(6) {
                            if !fen.is_empty() {
                                fen.push(' ');
                            }
                            fen.push_str(token);
                        }
                        let mut board: Board = Board::from_str(&fen)
                            .ok()
                            .ok_or(UCIError::InvalidFenPosition)?;

                        if matches!(tokens.next(), Some("moves")) {
                            for uci_move in tokens {
                                if let Some(mv) = board.find_move(uci_move) {
                                    let board_res: Board = board.make_move(mv);
                                    board = board_res;
                                } else {
                                    return Err(UCIError::IlegalUciMove(uci_move.to_string()));
                                }
                            }
                        }

                        board
                    }
                    _ => {
                        return Err(UCIError::InvalidPositionFormat(
                            "expected 'startpos' or 'fen' after 'position'".to_string(),
                        ))
                    }
                };

                Ok(Self::Position(board))
            }
            Some("go") => {
                let mut commands: String = String::with_capacity(64);
                for token in tokens {
                    if !commands.is_empty() {
                        commands.push(' ');
                    }
                    commands.push_str(token);
                }

                let time_control: TimeControl = TimeControl::from_str(&commands)?;
                Ok(Self::Go(time_control))
            }
            Some("stop") => Ok(Self::Stop),
            Some("quit") => Ok(Self::Quit),
            Some("setoption") => {
                let name = match tokens.next() {
                    Some("name") => tokens.next().ok_or(UCIError::NoOptionValue)?.to_string(),
                    _ => return Err(UCIError::InvalidSetOption),
                };
                let value = match tokens.next() {
                    Some("value") => tokens.next().ok_or(UCIError::NoOptionValue)?.to_string(),
                    _ => return Err(UCIError::InvalidSetOption),
                };

                Ok(Self::SetOption { name, value })
            }
            Some("dperft") => match tokens.next().ok_or(UCIError::NoOptionValue)?.parse::<u8>() {
                Ok(depth) if depth > 0 => Ok(Self::DividePerft(depth)),
                _ => Err(UCIError::InvalidOptionValue),
            },
            Some("perft") => match tokens.next().ok_or(UCIError::NoOptionValue)?.parse::<u8>() {
                Ok(depth) if depth > 0 => Ok(Self::Perft(depth)),
                _ => Err(UCIError::InvalidOptionValue),
            },
            Some("print") => Ok(Self::Print),
            Some("eval") => Ok(Self::Eval),
            Some("license") => Ok(Self::License),
            Some("help") => Ok(Self::Help),
            _ => Err(UCIError::UnknownCommand(s.to_string())),
        }
    }
}

pub fn uci_start() {
    println!("{NAME} {VERSION} by {AUTHOR}");
}

pub fn uci_listener() {
    let (sender, receiver) = mpsc::channel();
    let stop: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let stop_clone: Arc<AtomicBool> = Arc::clone(&stop);
    thread::spawn(move || {
        uci_loop(receiver, stop_clone);
    });

    let stdin: Stdin = stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(cmd) => {
                let command: Result<UCICommand, UCIError> = UCICommand::from_str(&cmd);
                match command {
                    Ok(UCICommand::Stop) => {
                        stop.store(true, Ordering::SeqCst);
                    }
                    Ok(UCICommand::Quit) => {
                        stop.store(true, Ordering::SeqCst);
                        std::process::exit(0);
                    }
                    _ => {
                        if sender.send(command).is_err() {
                            eprintln!("info string [error] failed to send command.");
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("info string [error] reading stdin: {}.", e);
            }
        }
    }
}

pub fn uci_loop(receiver: Receiver<Result<UCICommand, UCIError>>, stop: Arc<AtomicBool>) {
    let mut position: Position = Position::default();
    let mut threadpool: ThreadPool = ThreadPool::new(stop);
    let mut ttable: TranspositionTable = TranspositionTable::default();
    ttable.resize(DEFAULT_SIZE);

    while let Ok(command) = receiver.recv() {
        match command {
            Ok(UCICommand::Uci) => {
                println!("id name {} {}", NAME, VERSION);
                println!("id author {}", AUTHOR);

                // UCI options
                println!("option name Hash type spin default 16 min 1 max 1048576");
                println!("option name Threads type spin default 1 min 1 max 512");

                println!("uciok");
            }
            Ok(UCICommand::IsReady) => {
                println!("readyok");
            }
            Ok(UCICommand::UciNewGame) => {
                position.set_board(Board::default());
                ttable.clear(threadpool.threads);
            }
            Ok(UCICommand::Position(pos)) => {
                position.set_board(pos);
            }
            Ok(UCICommand::Go(time_control)) => {
                ttable.age();
                let best: Option<Move> =
                    threadpool.start_search(&mut position, &ttable, time_control);
                if let Some(mv) = best {
                    println!("bestmove {}", mv);
                }
            }
            Ok(UCICommand::Stop) | Ok(UCICommand::Quit) => {
                eprintln!("info string [warning] unexpected stop/quit.");
                continue;
            }
            Ok(UCICommand::SetOption { name, value }) => match name.to_lowercase().as_str() {
                "hash" => match value.parse::<usize>() {
                    Ok(mb) if (HASH_MIN..=HASH_MAX).contains(&mb) => {
                        ttable.resize(mb);
                        println!("info string Hash size set to {} MB", mb);
                    }
                    _ => eprintln!(
                        "info string [error] Invalid value for Hash: '{}'. Must be between {} and {}.",
                        value, HASH_MIN, HASH_MAX
                    ),
                },
                "threads" => match value.parse::<usize>() {
                    Ok(n) if (THREADS_MIN..=THREADS_MAX).contains(&n) => {
                        threadpool.resize(n);
                        println!("info string Threads set to {}", n);
                    }
                    _ => eprintln!(
                        "info string [error] Invalid value for Threads: '{}'. Must be between {} and {}.",
                        value, THREADS_MIN, THREADS_MAX
                    ),
                },
                _ => eprintln!("info string [error] unrecognized option '{}'", name),
            },
            Ok(UCICommand::DividePerft(depth)) => {
                position.divided_perft(depth);
            }
            Ok(UCICommand::Perft(depth)) => {
                position.perft(depth);
            }
            Ok(UCICommand::Print) => {
                println!("{}", position.board());
            }
            Ok(UCICommand::Eval) => {
                if position.in_check() {
                    println!("none: king in check.");
                } else {
                    println!("{}", position.evaluate());
                }
            }
            Ok(UCICommand::License) => {
                println!("Laura is licensed under the GNU GPL v3.0.");
                println!("See https://www.gnu.org/licenses/gpl-3.0.html for details.");
            }
            Ok(UCICommand::Help) => {
                println!("Laura: A multi-threaded UCI chess engine written in Rust.");
                println!("For more information, visit: https://github.com/HansTibberio/Laura");
            }
            Err(UCIError::UnknownCommand(s)) if s.is_empty() => {}
            Err(e) => eprintln!("info string {e}"),
        }
    }
}
