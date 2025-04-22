// src/uci.rs

//! UCI protocol implementation

use std::str::FromStr;

use laura_core::Board;

use crate::{
    position::Position,
    timer::{TimeControl, TimeManager, TimeParserError},
};

const AUTHOR: &str = "HansTibberio";
const NAME: &str = "Laura";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub enum UCICommand {
    Uci,
    IsReady,
    UciNewGame,
    Position(Board),
    Go(TimeControl),
    Stop,
    Quit,

    DividePerft(u8),
    Perft(u8),
    Print,
    Eval,
}

#[derive(Debug)]
pub enum UCIError {
    UnknownCommand(String),
    NoOptionValue,
    InvalidOptionValue,
    InvalidPosition,
    InvalidGo(TimeParserError),
    IlegalUciMove,
}

impl std::fmt::Display for UCIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UCIError::UnknownCommand(s) => write!(f, "Error: Unkown command: '{s}'."),
            UCIError::NoOptionValue => write!(f, "Error: No option value provided."),
            UCIError::InvalidOptionValue => write!(f, "Error: Invalid option value."),
            UCIError::InvalidPosition => write!(f, "Error: Invalid position format."),
            UCIError::InvalidGo(err) => write!(f, "Error: '{err:?}'"),
            UCIError::IlegalUciMove => write!(f, "Error: Ilegal uci move"),
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
                let mut board: Board = match tokens.next() {
                    Some("startpos") => Board::default(),
                    Some("fen") => {
                        let mut fen: String = String::with_capacity(128);
                        for token in tokens.by_ref().take(6) {
                            if !fen.is_empty() {
                                fen.push(' ');
                            }
                            fen.push_str(token);
                        }
                        Board::from_str(&fen)
                            .ok()
                            .ok_or(UCIError::InvalidPosition)?
                    }
                    _ => return Err(UCIError::InvalidPosition),
                };

                if matches!(tokens.next(), Some("moves")) {
                    for uci_move in tokens {
                        if let Some(mv) = board.find_move(uci_move) {
                            let board_res: Board = board.make_move(mv);
                            board = board_res;
                        } else {
                            return Err(UCIError::IlegalUciMove);
                        }
                    }
                }

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
            _ => Err(UCIError::UnknownCommand(s.to_string())),
        }
    }
}

#[derive(Default)]
pub struct UCI {
    position: Position,
    time_manager: TimeManager,
}

impl UCI {
    pub fn uci_start() {
        println!("{NAME} {VERSION} by {AUTHOR}");
    }

    pub fn run(&mut self, command: Result<UCICommand, UCIError>) {
        match command {
            Ok(UCICommand::Uci) => {
                println!("id name {NAME} {VERSION}");
                println!("id author {AUTHOR}");
                println!("uciok");
            }
            Ok(UCICommand::IsReady) => {
                println!("readyok");
            }
            Ok(UCICommand::UciNewGame) => {
                self.position.set_board(Board::default());
            }
            Ok(UCICommand::Position(pos)) => {
                self.position.set_board(pos);
            }
            Ok(UCICommand::Go(time_control)) => {
                // TODO!
                self.time_manager.start();
                self.time_manager.set_control(time_control);
                println!("TimeControl: {time_control:?}")
            }
            Ok(UCICommand::Stop) => todo!(),
            Ok(UCICommand::Quit) => {
                std::process::exit(0);
            }

            Ok(UCICommand::DividePerft(depth)) => {
                self.position.divided_perft(depth);
            }
            Ok(UCICommand::Perft(depth)) => {
                self.position.perft(depth);
            }
            Ok(UCICommand::Print) => {
                println!("{}", self.position.board());
            }
            Ok(UCICommand::Eval) => {
                if self.position.in_check() {
                    println!("None: King in check");
                } else {
                    println!("{}", self.position.evaluate());
                }
            }
            Err(UCIError::UnknownCommand(s)) if s.is_empty() => {}
            Err(e) => eprintln!("info string {e}"),
        }
    }
}
