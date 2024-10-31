use super::random::Xoshiro256PlusPlus;
use super::sliders::Slider;

use crate::bitboard::BitBoard;
use crate::square::Square;


/*  From:
**  Magical Bitboards and How to Find Them: Sliding move generation in chess by Analog Hors
**  https://analog-hors.github.io/site/magic-bitboards/
*/

/// Represents an error during the generation of a magic bitboard table.
#[derive(Clone, Copy, Debug)]
struct TableError;

/// Holds the parameters for calculating a unique magic index for move generation.
/// Each `MagicEntry` includes a mask for relevant blocker bits, a magic number for hashing,
/// and a shift value to define the table size.
#[derive(Clone, Copy, Debug)]
pub struct MagicEntry {
    mask: BitBoard,
    magic: u64,
    shift: u8,
}

/// Computes the magic index for a given set of blockers using the provided `MagicEntry`.
/// This index is used to retrieve moves from a precomputed magic move table.
fn magic_index(entry: &MagicEntry, blockers: BitBoard) -> usize {
    let blockers: BitBoard = blockers & entry.mask;
    let hash: u64 = blockers.0.wrapping_mul(entry.magic);
    let index: usize = (hash >> entry.shift) as usize;

    index
}

/// Searches for a unique magic number and corresponding table for the specified square and slider piece.
/// This function iterates to find a magic number that yields unique indices for all possible blocker configurations.
fn find_magic(slider: &Slider, square: Square, index: u8, prng: &mut Xoshiro256PlusPlus) -> (MagicEntry, Vec<BitBoard>){
    let mask: BitBoard = slider.relevant_blockers(square);
    let shift: u8 = 64 - index;

    loop {
        let magic: u64 = prng.next() & prng.next() & prng.next();
        let magic_entry: MagicEntry = MagicEntry { mask, magic, shift };
        if let Ok(table) = gen_table(slider, square, &magic_entry) {
            return(magic_entry, table);
        }
    }
}

/// Generates a move table for a sliding piece on a given square using the specified magic entry.
/// This table maps blocker configurations to possible moves.
fn gen_table(slider: &Slider, square: Square, magic_entry: &MagicEntry) -> Result<Vec<BitBoard>, TableError> {
    let index: u8 = 64 - magic_entry.shift;
    let mut table: Vec<BitBoard> = vec![BitBoard::EMPTY; 1 << index];
    let mut blockers: BitBoard = BitBoard::EMPTY;

    loop {
        let moves: BitBoard = slider.moves(square, blockers);
        let table_entry: &mut BitBoard = &mut table[magic_index(magic_entry, blockers)];
        if table_entry.is_empty() {
            *table_entry = moves;
        } else if *table_entry != moves {
            return Err(TableError);
        }

        blockers.0 = blockers.0.wrapping_sub(magic_entry.mask.0) & magic_entry.mask.0;
        if blockers.is_empty() {
            break;
        }
    }

    Ok(table)
}

/// Generates magic numbers and move tables for each square on the board for a given piece type
/// (e.g., rook or bishop). This function outputs a Rust constant array for the generated magic numbers.
fn gen_magics(slider: &Slider, name: &str, prng: &mut Xoshiro256PlusPlus) {
    println!(
        "pub const {}_MAGICS: [MagicEntry; Square::NUM_SQUARES] = [",
        name
    );
    let mut table_size: usize = 0;
    for square_index in 0..Square::NUM_SQUARES {
        let square: Square = Square::from_index(square_index);
        let index: u8 = slider.relevant_blockers(square).count_bits() as u8;
        let (entry, table) = find_magic(slider, square, index, prng);
        println!(
            "    Magic!(0x{:016X}, 0x{:016X}, {}, {}),",
            entry.mask.0, entry.magic, entry.shift, table_size
        );
        table_size += table.len();
    }

    println!("];");
    println!(
        "pub const {}_TABLE_SIZE: usize = {};",
        name, table_size
    );
}

#[test]
fn test_gen_magics(){
    use super::sliders::{ROOK, BISHOP};

    let mut prng: Xoshiro256PlusPlus = Xoshiro256PlusPlus::default();
    gen_magics(&ROOK, "ROOK", &mut prng);
    gen_magics(&BISHOP, "BISHOP", &mut prng);
}
