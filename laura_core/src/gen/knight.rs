use std::mem::transmute;

use crate::{BitBoard, File, Rank, Square};

/// Contains the movement deltas for a knight, relative to its current position.
/// These deltas represent the possible knight moves in terms of changes in rank and file.
const KNIGHT_DELTAS: [(i8, i8); 8] = [
    (-2, -1),
    (-2, 1),
    (-1, -2),
    (-1, 2),
    (1, -2),
    (1, 2),
    (2, -1),
    (2, 1),
];

/// Precomputed knight attack bitboards for all 64 squares on a chessboard.
/// Each element corresponds to a square, and the bitboard represents the knight's potential
/// attacks from that square. This allows for fast lookups of knight attacks.
pub const KNIGHT_ATTACKS: [BitBoard; 64] = [
    BitBoard(132096),
    BitBoard(329728),
    BitBoard(659712),
    BitBoard(1319424),
    BitBoard(2638848),
    BitBoard(5277696),
    BitBoard(10489856),
    BitBoard(4202496),
    BitBoard(33816580),
    BitBoard(84410376),
    BitBoard(168886289),
    BitBoard(337772578),
    BitBoard(675545156),
    BitBoard(1351090312),
    BitBoard(2685403152),
    BitBoard(1075839008),
    BitBoard(8657044482),
    BitBoard(21609056261),
    BitBoard(43234889994),
    BitBoard(86469779988),
    BitBoard(172939559976),
    BitBoard(345879119952),
    BitBoard(687463207072),
    BitBoard(275414786112),
    BitBoard(2216203387392),
    BitBoard(5531918402816),
    BitBoard(11068131838464),
    BitBoard(22136263676928),
    BitBoard(44272527353856),
    BitBoard(88545054707712),
    BitBoard(175990581010432),
    BitBoard(70506185244672),
    BitBoard(567348067172352),
    BitBoard(1416171111120896),
    BitBoard(2833441750646784),
    BitBoard(5666883501293568),
    BitBoard(11333767002587136),
    BitBoard(22667534005174272),
    BitBoard(45053588738670592),
    BitBoard(18049583422636032),
    BitBoard(145241105196122112),
    BitBoard(362539804446949376),
    BitBoard(725361088165576704),
    BitBoard(1450722176331153408),
    BitBoard(2901444352662306816),
    BitBoard(5802888705324613632),
    BitBoard(11533718717099671552),
    BitBoard(4620693356194824192),
    BitBoard(288234782788157440),
    BitBoard(576469569871282176),
    BitBoard(1224997833292120064),
    BitBoard(2449995666584240128),
    BitBoard(4899991333168480256),
    BitBoard(9799982666336960512),
    BitBoard(1152939783987658752),
    BitBoard(2305878468463689728),
    BitBoard(1128098930098176),
    BitBoard(2257297371824128),
    BitBoard(4796069720358912),
    BitBoard(9592139440717824),
    BitBoard(19184278881435648),
    BitBoard(38368557762871296),
    BitBoard(4679521487814656),
    BitBoard(9077567998918656),
];

/// Retrieves the precomputed attack `BitBoard` for a knight located on a specific square.
///
/// This function provides a `BitBoard` that represents all potential attack
/// squares for a knight positioned on the given square. The knight attack patterns
/// are precomputed and stored in the `KNIGHT_ATTACKS` array for efficient access.
#[inline(always)]
pub fn get_knight_attacks(square: Square) -> BitBoard {
    unsafe { *KNIGHT_ATTACKS.get_unchecked(square.to_index()) }
}

/// Generates the attack bitboard for a knight on the given `square`.
/// This function computes the knight's valid moves based on the current rank and file
/// of the knight's position, using the predefined movement deltas.
pub fn gen_knight_attacks(square: Square) -> BitBoard {
    let mut attacks: BitBoard = BitBoard::EMPTY;
    let rank: i8 = square.rank() as i8;
    let file: i8 = square.file() as i8;

    for (dr, df) in KNIGHT_DELTAS.iter() {
        let new_rank: i8 = rank + dr;
        let new_file: i8 = file + df;

        if (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
            let new_square: Square =
                Square::from_file_rank(unsafe { transmute::<u8, File>(new_file as u8) }, unsafe {
                    transmute::<u8, Rank>(new_rank as u8)
                });

            attacks = attacks.set_square(new_square);
        }
    }

    attacks
}

/// Generates the full knight attack table for all squares on the board.
/// The table is an array where each index corresponds to a square, and the value is a
/// precomputed `BitBoard` representing the knight's attack pattern from that square.
pub fn gen_knight_attack_table() -> [BitBoard; 64] {
    let mut table: [BitBoard; 64] = [BitBoard::EMPTY; Square::NUM_SQUARES];

    for square in 0..Square::NUM_SQUARES {
        let sq: Square = Square::from_index(square);
        table[square] = gen_knight_attacks(sq);
    }

    table
}

#[test]
fn test_gen_attacks() {
    let attack: BitBoard = gen_knight_attacks(Square::C1);
    println!("{}", attack);
}

#[test]
fn test_get_attacks() {
    let attack: BitBoard = get_knight_attacks(Square::C1);
    println!("{}", attack);
}

#[test]
fn gen_attacks() {
    let attacks: [BitBoard; 64] = gen_knight_attack_table();
    println!("{:?}", attacks);
}
