mod build_dep;

use std::env::var;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use build_dep::between::*;
use build_dep::magics::*;
use build_dep::types::BitBoard;

/// Sets up a buffered writer for a given filename in the output directory specified by `OUT_DIR`.
fn create_out_file(filename: &str) -> BufWriter<File> {
    let mut out_path: PathBuf = var("OUT_DIR").unwrap().into();
    out_path.push(filename);
    BufWriter::new(File::create(out_path).unwrap())
}

/// Main function to generate and write precomputed magic move tables for rooks and bishops.
///
/// This function creates move tables for rook and bishop pieces by specifying their respective
/// movement deltas, then uses `make_table` to populate the moves. It outputs each table to
/// a `.rs` file using `write_table` to generate Rust source files containing the precomputed moves.
fn main() {
    // Directional deltas for rook movement (up, left, down, right).
    const ROOK_DELTAS: [(i8, i8); 4] = [(1, 0), (0, -1), (-1, 0), (0, 1)];

    // Directional deltas for bishop movement (up-right, up-left, down-left, down-right).
    const BISHOP_DELTAS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, -1), (-1, 1)];

    // Generate rook and bishop magic move tables using specified deltas and magic numbers.
    let rook_table: Vec<BitBoard> = make_table(ROOK_TABLE_SIZE, &ROOK_DELTAS, &ROOK_MAGICS);
    let bishop_table: Vec<BitBoard> = make_table(BISHOP_TABLE_SIZE, &BISHOP_DELTAS, &BISHOP_MAGICS);

    // Write rook magic numbers to "rook_magics.rs" file in OUT_DIR
    let mut rook_magic_file: BufWriter<File> = create_out_file("rook_magics.rs");
    write_magic("ROOK", &ROOK_MAGICS, &mut rook_magic_file).unwrap();

    // Write bishop magic numbers to "bishop_magics.rs" file in OUT_DIR
    let mut bishop_magic_file: BufWriter<File> = create_out_file("bishop_magics.rs");
    write_magic("BISHOP", &BISHOP_MAGICS, &mut bishop_magic_file).unwrap();

    // Write rook move table to "rook_attacks.rs" file in OUT_DIR
    let mut rook_attacks_file: BufWriter<File> = create_out_file("rook_attacks.rs");
    write_table("ROOK", &rook_table, &mut rook_attacks_file).unwrap();

    // Write bishop move table to "bishop_attacks.rs" file in OUT_DIR
    let mut bishop_attacks_file: BufWriter<File> = create_out_file("bishop_attacks.rs");
    write_table("BISHOP", &bishop_table, &mut bishop_attacks_file).unwrap();

    // Generates a 2D table of `BitBoard`s for all pairs of squares on the chessboard,
    // representing the squares between them for straight-line moves.
    let between_table: [[BitBoard; 64]; 64] = gen_between();

    // Writes the `between_table` array to "between_array.rs" file in OUT_DIR
    let mut between_file: BufWriter<File> = create_out_file("between_array.rs");
    write_between("BETWEEN", &between_table, &mut between_file).unwrap();
}
