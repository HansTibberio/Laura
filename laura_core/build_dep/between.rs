use std::io::Result;
use std::io::Write;
use std::mem::transmute;

use super::types::{BitBoard, Square};

/// Calculates the `BitBoard` representing all squares between two squares on a chessboard.
///
/// Given two squares, `start` and `end`, this function returns a `BitBoard` with all squares
/// that lie between them in a straight line along a rank, file, or diagonal. If the squares are 
/// not aligned or are the same, an empty `BitBoard` is returned.
pub fn squares_between(start: Square, end: Square) -> BitBoard {
    if start == end {
        return BitBoard::EMPTY;
    }

    let mut bitboard: BitBoard = BitBoard::EMPTY;
    let (start_rank, start_file) = (start.rank() as i8, start.file() as i8);
    let (end_rank, end_file) = (end.rank() as i8, end.file() as i8);

    let (dr, df) = match (end_rank - start_rank, end_file - start_file) {
        (0, df) if df != 0 => (0, df.signum()),
        (dr, 0) if dr != 0 => (dr.signum(), 0),
        (dr, df) if dr.abs() == df.abs() => (dr.signum(), df.signum()),
        _ => return BitBoard::EMPTY,
    };
    let mut new_rank: i8 = start_rank + dr;
    let mut new_file: i8 = start_file + df;

    while (new_rank, new_file) != (end_rank, end_file) {
        if (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
            let square: Square =
                Square::from_file_rank(unsafe { transmute(new_file as u8) }, unsafe {
                    transmute(new_rank as u8)
                });
            bitboard = bitboard.set_square(square);
        } else {
            break;
        }
        new_rank += dr;
        new_file += df;
    }
    bitboard = bitboard.set_square(end);
    bitboard
}

/// Generates a table of `BitBoard`s that represent the squares between any two squares on a chessboard.
///
/// This function creates a 2D array where each entry contains a `BitBoard` representing the squares 
/// lying between a pair of `Square`s. This is useful for precomputing move paths for sliding pieces 
/// like rooks, bishops, and queens.
pub fn gen_between() -> [[BitBoard; Square::NUM_SQUARES]; Square::NUM_SQUARES] {
    let mut table: [[BitBoard; 64]; 64] =
        [[BitBoard::EMPTY; Square::NUM_SQUARES]; Square::NUM_SQUARES];
    for start in 0..Square::NUM_SQUARES {
        for end in 0..Square::NUM_SQUARES {
            let start_square = Square::from_index(start);
            let end_square: Square = Square::from_index(end);
            table[start][end] = squares_between(start_square, end_square);
        }
    }

    table
}

/// Writes a precomputed table of `BitBoard`s to a Rust source file in the form of a 2D constant array.
///
/// This function takes a bidimensional array of `BitBoard`s and writes it as a Rust array declaration
/// with a given `name`. Each entry is written as a `u64` value.
pub fn write_between(
    name: &str,
    table: &[[BitBoard; Square::NUM_SQUARES]; Square::NUM_SQUARES],
    out: &mut impl Write,
) -> Result<()> {
    write!(
        out,
        "const {}_ARRAY: [[u64; {}]; {}] = [\n",
        name,
        Square::NUM_SQUARES,
        Square::NUM_SQUARES
    )?;

    for row in table {
        write!(out, "    [")?;
        for entry in row {
            write!(out, "{}, ", entry.0)?;
        }
        write!(out, "],\n")?;
    }

    write!(out, "];\n")?;
    Ok(())
}
