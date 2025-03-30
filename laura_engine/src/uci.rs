// src/uci.rs

//! UCI protocol implementation

use std::{str::FromStr, sync::atomic::AtomicBool};

use laura_core::Board;

use crate::engine::Engine;

const NAME: &str = "Laura";
const AUTHOR: &str = "HansTibberio";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub enum UCICommand {
    Uci,
    IsReady,
    UciNewGame,
    Position(Board),
    Go(String),
    Stop,
    Quit,

    DividePerft(usize),
    Perft(usize),
    Print,
    Eval,
}

#[derive(Debug)]
pub enum UciError {
    UnknownCommand(String),
    NoOptionValueProvided,
    InvalidOptionValue,
    InvalidPosition,
    InvalidGo,
}

impl FromStr for UCICommand {
    type Err = UciError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split_whitespace();

        match tokens.next() {
            Some("uci") => Ok(Self::Uci),
            Some("isready") => Ok(Self::IsReady),
            Some("ucinewgame") => Ok(Self::UciNewGame),
            Some("position") => {
                let board: Board = match tokens.next() {
                    Some("startpos") => Board::default(),
                    Some("fen") => {
                        let fen: String = tokens.take(6).collect::<Vec<&str>>().join(" ");
                        Board::from_str(&fen)
                            .ok()
                            .ok_or(UciError::InvalidPosition)?
                    }
                    _ => return Err(UciError::InvalidPosition),
                };
                Ok(Self::Position(board))
            }
            Some("go") => todo!(),
            Some("stop") => Ok(Self::Stop),
            Some("quit") => Ok(Self::Quit),

            Some("dperft") => {
                match tokens
                    .next()
                    .ok_or(UciError::NoOptionValueProvided)?
                    .parse::<usize>()
                {
                    Ok(depth) if depth > 0 => Ok(Self::DividePerft(depth)),
                    _ => Err(UciError::InvalidOptionValue),
                }
            }
            Some("perft") => {
                match tokens
                    .next()
                    .ok_or(UciError::NoOptionValueProvided)?
                    .parse::<usize>()
                {
                    Ok(depth) if depth > 0 => Ok(Self::Perft(depth)),
                    _ => Err(UciError::InvalidOptionValue),
                }
            }
            Some("print") => Ok(Self::Print),
            Some("eval") => Ok(Self::Eval),
            _ => Err(UciError::UnknownCommand(s.to_string())),
        }
    }
}

#[derive(Default)]
pub struct UCI {
    engine: Engine,
    _search: AtomicBool,
}

impl UCI {
    pub fn start(&self) {
        println!("id name {NAME} {VERSION}");
        println!("id author {AUTHOR}");
        println!("uciok");
    }

    pub fn run(&mut self, command: Result<UCICommand, UciError>) {
        match command {
            Ok(UCICommand::Uci) => {
                self.start();
            }
            Ok(UCICommand::IsReady) => {
                println!("readyok");
            }
            Ok(UCICommand::UciNewGame) => todo!(),
            Ok(UCICommand::Position(pos)) => {
                self.engine.set_board(pos);
            }
            Ok(UCICommand::Go(_params)) => todo!(),
            Ok(UCICommand::Stop) => todo!(),
            Ok(UCICommand::Quit) => {
                std::process::exit(0);
            }

            Ok(UCICommand::DividePerft(depth)) => todo!(),
            Ok(UCICommand::Perft(depth)) => todo!(),
            Ok(UCICommand::Print) => {
                println!("{}", self.engine.board);
            }
            Ok(UCICommand::Eval) => todo!(),

            Err(UciError::UnknownCommand(cmd)) => {
                eprintln!("Error: Unknown command: '{}'", cmd)
            }
            Err(UciError::NoOptionValueProvided) => {
                eprintln!("Error: No option value provided.")
            }
            Err(UciError::InvalidOptionValue) => {
                eprintln!("Error: Invalid option value.")
            }
            Err(UciError::InvalidPosition) => {
                eprintln!("Error: Invalid position format.")
            }
            Err(UciError::InvalidGo) => {
                eprintln!("Error: Invalid parameters for 'go'")
            }
        }
    }
}
