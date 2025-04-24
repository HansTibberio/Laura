use laura_engine::{uci_run, uci_start, Position, ThreadPool, UCICommand, UCIError};
use std::{
    io::{stdin, BufRead, Stdin},
    str::FromStr,
    sync::{atomic::AtomicBool, Arc},
};

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

fn main() {
    let (sender, receiver): (Sender<String>, Receiver<String>) = mpsc::channel();
    thread::spawn(move || {
        let stdin: Stdin = stdin();
        for line in stdin.lock().lines() {
            if let Ok(cmd) = line {
                if sender.send(cmd).is_err() {
                    break;
                }
            }
        }
    });

    uci_loop(receiver);
}

fn uci_loop(receiver: Receiver<String>) {
    let mut position: Position = Position::default();
    let mut threadpool: ThreadPool = ThreadPool::new(Arc::new(AtomicBool::new(false)));

    uci_start();

    while let Ok(line) = receiver.recv() {
        let command: Result<UCICommand, UCIError> = UCICommand::from_str(&line);
        uci_run(&mut position, &mut threadpool, command);
    }
}
