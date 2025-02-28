/*
    Laura-Core: a fast and efficient move generator for chess engines.

    Copyright (C) 2024-2025 HansTibberio <hanstiberio@proton.me>

    Laura-Core is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Laura-Core is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Laura-Core. If not, see <https://www.gnu.org/licenses/>.
*/

use std::str::FromStr;

use laura_core::*;

/// A collection of standardized perft test positions for the Laura-Core move generation.
/// These positions are used to validate move generation correctness and measure performance.
#[rustfmt::skip]
pub const PERFT_TEST: [(&str, u64, usize); 34] = [
    // Initial Position
    ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 119060324, 6),

    // Kiwipete
    ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 193690690, 5),

    // Positions useful for debugging from CPW
    // (https://www.chessprogramming.org/Perft_Results)
    ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 11030083, 6),
    ("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 15833292, 5),
    ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 89941194, 5),
    ("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 164075551, 5),


    // Below are positions corresponding to Martin Sedlak's test.
    // (http://www.talkchess.com/forum/viewtopic.php?t=47318)

    // Ilegal en passant moves:
    ("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 1015133, 6),
    ("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 1134888, 6),

    // Avoid ilegal en passant capture:
    ("8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1", 824064, 6),
    ("8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1", 824064, 6),

    // En passant capture checks opponent:
    ("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 1440467, 6),
    ("8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1", 1440467, 6),

    // Short castling gives check:
    ("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 661072, 6),
    ("4k2r/8/8/8/8/8/8/5K2 b k - 0 1", 661072, 6),

    // Long castling gives check:
    ("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 803711, 6),
    ("r3k3/8/8/8/8/8/8/3K4 b q - 0 1", 803711, 6),

    // Castling
    ("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 1274206, 4),
    ("r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1", 1274206, 4),

    // Castling Prevented
    ("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 1720476, 4),
    ("r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1", 1720476, 4),

    // Promote out of check
    ("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 3821001, 6),
    ("3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1", 3821001, 6),

    // Discovered check
    ("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 1004658, 5),
    ("5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1", 1004658, 5),

    // Promote to give check
    ("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 217342, 6),
    ("8/k7/8/8/8/8/1p6/4K3 b - - 0 1", 217342, 6),

    // Underpromote to check
    ("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 92683, 6),
    ("8/8/8/8/8/k7/p1K5/8 b - - 0 1", 92683, 6),

    // Self stalemate
    ("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 2217, 6),
    ("8/8/8/8/8/p7/8/k1K5 b - - 0 1", 2217, 6),

    // Stalemate / checkmate
    ("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 567584, 7),
    ("8/8/8/8/1k6/8/K1p5/8 b - - 0 1", 567584, 7),

    // Double check
    ("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 23527, 4),
    ("8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1", 23527, 4),
];

/// Performs a Perft (performance test) for the given board at the specified depth.
/// This function generates all possible moves, counts the total number of nodes, and measures the time taken.
///
/// It prints the total number of nodes, the duration, and the performance in nodes per second.
/// If `DIV` is true, it divides the search into individual moves and counts the nodes for each move.
pub fn perft<const DIV: bool>(board: &Board, depth: usize) -> usize {
    let start: std::time::Instant = std::time::Instant::now();
    let total_nodes: usize = inner_perft::<DIV>(board, depth);
    let duration: std::time::Duration = start.elapsed();

    let nps: f64 = total_nodes as f64 / duration.as_secs_f64();
    println!("{total_nodes} nodes in {duration:?} -> {nps:.0} nodes/s");

    total_nodes
}

/// A helper function that performs the core Perft test recursively.
/// It generates all possible moves for the board at the current depth and counts the number of nodes.
/// For deeper levels, it recursively calls itself to count all possible move sequences.
#[allow(unused_assignments)]
pub fn inner_perft<const DIV: bool>(board: &Board, depth: usize) -> usize {
    let mut total: usize = 0;

    if !DIV && depth <= 1 {
        enumerate_legal_moves::<ALL_MOVES, _>(board, |_| -> bool {
            total += 1;
            true
        });
        return total;
    }

    enumerate_legal_moves::<ALL_MOVES, _>(board, |mv| -> bool {
        let mut nodes: usize = 0;
        if DIV && depth == 1 {
            nodes = 1;
        } else {
            let board_res: Board = board.make_move(mv);
            nodes = if depth == 1 {
                1
            } else {
                inner_perft::<false>(&board_res, depth - 1)
            };
        }

        total += nodes;

        if DIV && nodes > 0 {
            println!("{} -> {}", mv, nodes);
        }

        true
    });

    total
}

fn main() {
    for (fen, correct_count, depth) in PERFT_TEST {
        let board: Board = Board::from_str(fen).unwrap();
        println!("");
        println!("{fen}");

        let nodes: usize = perft::<false>(&board, depth);
        assert_eq!(nodes, correct_count as usize, "Perft Test Failed");
    }
}
