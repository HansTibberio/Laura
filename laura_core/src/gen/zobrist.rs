use crate::gen::random::Xoshiro256PlusPlus;
use crate::{CastleRights, Piece, Square};

/// Generates unique `u64` keys for Zobrist hashing, using the Xoshiro256++
/// pseudorandom number generator. These keys are used to hash different board states
/// in a chess game, including piece positions, en passant squares, castling rights, and
/// the side to move.
///
/// The function outputs constants in Rust code format, representing the following:
/// - A 2D array `KEY_PIECE_SQUARE` that hashes pieces at each square.
/// - A 1D array `KEY_ENPASSANT` that hashes en passant squares.
/// - A 1D array `KEY_CASTLE` that hashes castling rights.
/// - A single `u64` constant `KEY_SIDE` that hashes which side is to move.
#[allow(dead_code)]
pub fn generate_unique_u64_keys(seed: [u64; 4]) {
    // Initialize the PRNG using the provided seed (A 4-element array of `u64` values).
    let mut prng: Xoshiro256PlusPlus = Xoshiro256PlusPlus::new(seed);

    // Generate Zobrist keys for piece-square combinations and print them in Rust constant format.
    println!("pub const KEY_PIECE_SQUARE: [[u64; Square::NUM_SQUARES]; Piece::NUM_PIECES] = [");
    for _ in 0..Piece::NUM_PIECES {
        print!("    [");
        for j in 0..Square::NUM_SQUARES {
            print!("{}", prng.next_u64());
            if j != 63 {
                print!(", ");
            }
        }
        println!("],");
    }
    println!("];");

    // Generate Zobrist keys for en passant squares and print them as a Rust constant array.
    println!("\npub const KEY_ENPASSANT: [u64; Square::NUM_SQUARES] = [");
    print!("    ");
    for i in 0..Square::NUM_SQUARES {
        print!("{}", prng.next_u64());
        if i != 63 {
            print!(", ");
        }
    }
    println!("\n];");

    // Generate Zobrist keys for castling rights and print them as a Rust constant array.
    println!("\npub const KEY_CASTLE: [u64; CastleRights::NUM_CASTLING_RIGHTS] = [");
    print!("    ");
    for i in 0..CastleRights::NUM_CASTLING_RIGHTS {
        print!("{}", prng.next_u64());
        if i != 15 {
            print!(", ");
        }
    }
    println!("\n];");

    // Generate the Zobrist key for the side to move and print it as a Rust constant.
    print!("\npub const KEY_SIDE: u64 = {};", prng.next_u64());
}

#[test]
fn generate_zobrist() {
    let seed: [u64; 4] = [
        0x1a2b3c4d5e6f7,
        0x1122334455667788,
        0x99aabbccddeeff00,
        0x2233445566778899,
    ];
    generate_unique_u64_keys(seed);
}
