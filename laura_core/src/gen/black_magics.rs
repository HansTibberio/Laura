use crate::{BitBoard, BlackMagic, Square};

// Includes the pre-generated files containing the slider attack bitboards and black magic numbers.
// These files are created at build time and are dynamically included at compile-time into the current
// Rust module.
include!(concat!(env!("OUT_DIR"), "/sliders_attacks.rs"));
include!(concat!(env!("OUT_DIR"), "/rook_bmagics.rs"));
include!(concat!(env!("OUT_DIR"), "/bishop_bmagics.rs"));

/// The shift constant used for rook magic numbers. This value is used to compute the final index for
/// a given square, based on its blockers and magic number.
const ROOK_SHIFT: usize = 12;

/// The shift constant used for bishop magic numbers.
pub const BISHOP_SHIFT: usize = 9;

/// Struct representing a single black magic entry for a slider piece (rook or bishop).
///
/// This entry is used in combination with a blocker bitboard to quickly compute valid attacks for the slider.
pub struct BlackMagicEntry {
    magic: u64,
    not_mask: u64,
    offset: usize,
}

/// Calculates the index of a slider piece's attack bitboard based on the given blockers and magic number.
#[inline]
fn magic_index(magic: &BlackMagicEntry, shift: usize, blockers: BitBoard) -> usize {
    let relevant_blockers: u64 = blockers.0 | magic.not_mask;
    let hash: u64 = relevant_blockers.wrapping_mul(magic.magic);
    magic.offset + (hash >> (Square::NUM_SQUARES - shift)) as usize
}

/// Gets the attack bitboard for a rook from a given square, considering the positions of blockers.
///
/// This function uses the magic number technique to quickly compute the valid attack squares for a rook.
/// The precomputed magic numbers for rooks are used to generate the attack bitboard for the given square,
/// considering the positions of blockers (other pieces on the board).
#[inline]
pub fn get_rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let magic: &BlackMagicEntry = ROOK_BLACK_MAGICS.get_unchecked(square.to_index());
        BitBoard(*SLIDER_ATTACKS.get_unchecked(magic_index(magic, ROOK_SHIFT, blockers)))
    }
}

/// Gets the attack bitboard for a bishop from a given square, considering the positions of blockers.
///
/// This function follows the same approach as `get_rook_attacks`, but is designed for bishop attacks.
/// It uses the magic number technique to quickly compute the valid attack squares for a bishop,
/// considering the positions of blockers (other pieces on the board).
#[inline]
pub fn get_bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let magic: &BlackMagicEntry = BISHOP_BLACK_MAGICS.get_unchecked(square.to_index());
        BitBoard(*SLIDER_ATTACKS.get_unchecked(magic_index(magic, BISHOP_SHIFT, blockers)))
    }
}

#[test]
fn bishop_magic_attacks() {
    let blockers: BitBoard = BitBoard(76631562411574272);
    let bitboard: BitBoard = get_bishop_attacks(Square::E4, blockers);
    println!("{}\n{}", blockers, bitboard);
    assert_eq!(bitboard, BitBoard(72695482583352320));

    let blockers: BitBoard = BitBoard(1099782160384);
    let bitboard: BitBoard = get_bishop_attacks(Square::B7, blockers);
    println!("{}\n{}", blockers, bitboard);
    assert_eq!(bitboard, BitBoard(360293502375952384));
}

#[test]
fn rook_magic_attacks() {
    let blockers: BitBoard = BitBoard(144115188075921408);
    let bitboard: BitBoard = get_rook_attacks(Square::A8, blockers);
    println!("{}\n{}", blockers, bitboard);
    assert_eq!(bitboard, BitBoard(144397766876004352));

    let blockers: BitBoard = BitBoard(4503600181022721);
    let bitboard: BitBoard = get_rook_attacks(Square::E4, blockers);
    println!("{}\n{}", blockers, bitboard);
    assert_eq!(bitboard, BitBoard(4521261322473472));
}
