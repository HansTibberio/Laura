use crate::bitboard::BitBoard;
use crate::square::Square;
use crate::Magic;


// Include precomputed attack and magic tables for rooks and bishops.
// These tables are generated during the build process and stored in
// the specified output directory.
include!(concat!(env!("OUT_DIR"), "/rook_attacks.rs"));
include!(concat!(env!("OUT_DIR"), "/rook_magics.rs"));
include!(concat!(env!("OUT_DIR"), "/bishop_attacks.rs"));
include!(concat!(env!("OUT_DIR"), "/bishop_magics.rs"));

/// Represents a single entry in the magic bitboard table for efficient move generation.
///
/// Each `MagicEntry` provides the necessary information to quickly compute moves:
/// - `mask`: A bitmask isolating relevant board squares for the piece type.
/// - `magic`: The magic multiplier used to compress board occupancy into an index.
/// - `shift`: A shift amount to further compress the resulting index.
/// - `offset`: The starting index in the precomputed moves table for this entry.
pub struct MagicEntry {
    pub mask: u64,
    pub magic: u64,
    pub shift: u8,
    pub offset: u32,
}

/// Computes the magic bitboard index for a given square and blocker configuration.
///
/// Given a specific `MagicEntry` and a `BitBoard` representing blockers, this function
/// calculates a unique index by applying the magic multiplier and a shift operation.
/// This index is then used to look up precomputed moves for a piece in the attack tables.
#[inline]
pub fn magic_index(entry: &MagicEntry, blockers: BitBoard) -> usize {
    let blockers: u64 = blockers.0 & entry.mask;
    let hash: u64 = blockers.wrapping_mul(entry.magic);
    let index: usize = (hash >> entry.shift) as usize;

    entry.offset as usize + index
}

/// Retrieves the precomputed attack bitboard for a rook on a given square with blockers.
///
/// This function looks up the rook attack bitboard using magic bitboards and returns
/// possible moves by indexing into `ROOK_ATTACKS` with the computed magic index.
#[inline]
pub fn get_rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic: &MagicEntry = &ROOK_MAGICS[square.to_index()];
    BitBoard(ROOK_ATTACKS[magic_index(magic, blockers)])
}

/// Retrieves the precomputed attack bitboard for a bishop on a given square with blockers.
///
/// This function uses the magic bitboards to fetch possible bishop moves based on the
/// blockers and returns them as a `BitBoard` indexed into `BISHOP_ATTACKS`.
#[inline]
pub fn get_bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic: &MagicEntry = &BISHOP_MAGICS[square.to_index()];
    BitBoard(BISHOP_ATTACKS[magic_index(magic, blockers)])
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