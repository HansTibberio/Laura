use laura_engine::{UCICommand, UciError, UCI};
use std::{
    io::{stdin, BufRead, Stdin},
    str::FromStr,
};

fn main() {
    let mut uci: UCI = UCI::default();
    uci.start();
    let sdtin: Stdin = stdin();
    for line in sdtin.lock().lines() {
        if let Ok(input) = line {
            let command: Result<UCICommand, UciError> = UCICommand::from_str(&input);
            uci.run(command);
        } else {
            eprintln!("Error: Failed to read input.");
        }
    }
}
