use laura_engine::{UCICommand, UCIError, UCI};
use std::{
    io::{stdin, BufRead, Stdin},
    str::FromStr,
};

fn main() {
    let mut uci: UCI = UCI::default();
    uci.uci_start();
    let sdtin: Stdin = stdin();
    for line in sdtin.lock().lines() {
        if let Ok(input) = line {
            let command: Result<UCICommand, UCIError> = UCICommand::from_str(&input);
            uci.run(command);
        } else {
            eprintln!("Error: Failed to read input.");
        }
    }
}
