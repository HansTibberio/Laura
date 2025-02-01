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

use std::mem::transmute;

use crate::{BitBoard, File, Rank, Square};

/// The possible relative moves a king can make on a chessboard.
/// Each tuple represents the change in rank (row) and file (column) for each direction the king can move.
/// These 8 directions include moves horizontally, vertically, and diagonally.
const KING_DELTAS: [(i8, i8); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

/// Precomputed bitboards representing the possible king attacks from every square on the chessboard.
/// Each element in the array corresponds to one square on the board (indexed by `Square`),
/// and the bitboard marks all squares that are attacked by a king from that square.
pub const KING_ATTACKS: [BitBoard; Square::NUM_SQUARES] = [
    BitBoard(770),
    BitBoard(1797),
    BitBoard(3594),
    BitBoard(7188),
    BitBoard(14376),
    BitBoard(28752),
    BitBoard(57504),
    BitBoard(49216),
    BitBoard(197123),
    BitBoard(460039),
    BitBoard(920078),
    BitBoard(1840156),
    BitBoard(3680312),
    BitBoard(7360624),
    BitBoard(14721248),
    BitBoard(12599488),
    BitBoard(50463488),
    BitBoard(117769984),
    BitBoard(235539968),
    BitBoard(471079936),
    BitBoard(942159872),
    BitBoard(1884319744),
    BitBoard(3768639488),
    BitBoard(3225468928),
    BitBoard(12918652928),
    BitBoard(30149115904),
    BitBoard(60298231808),
    BitBoard(120596463616),
    BitBoard(241192927232),
    BitBoard(482385854464),
    BitBoard(964771708928),
    BitBoard(825720045568),
    BitBoard(3307175149568),
    BitBoard(7718173671424),
    BitBoard(15436347342848),
    BitBoard(30872694685696),
    BitBoard(61745389371392),
    BitBoard(123490778742784),
    BitBoard(246981557485568),
    BitBoard(211384331665408),
    BitBoard(846636838289408),
    BitBoard(1975852459884544),
    BitBoard(3951704919769088),
    BitBoard(7903409839538176),
    BitBoard(15806819679076352),
    BitBoard(31613639358152704),
    BitBoard(63227278716305408),
    BitBoard(54114388906344448),
    BitBoard(216739030602088448),
    BitBoard(505818229730443264),
    BitBoard(1011636459460886528),
    BitBoard(2023272918921773056),
    BitBoard(4046545837843546112),
    BitBoard(8093091675687092224),
    BitBoard(16186183351374184448),
    BitBoard(13853283560024178688),
    BitBoard(144959613005987840),
    BitBoard(362258295026614272),
    BitBoard(724516590053228544),
    BitBoard(1449033180106457088),
    BitBoard(2898066360212914176),
    BitBoard(5796132720425828352),
    BitBoard(11592265440851656704),
    BitBoard(4665729213955833856),
];

/// Retrieves the precomputed attack `BitBoard` for a king located on a specific square.
///
/// This function provides a `BitBoard` representing all potential attack
/// squares for a king positioned on the specified square. The king's attack patterns
/// are precomputed and stored in the `KING_ATTACKS` array for quick access.
#[inline(always)]
pub fn get_king_attacks(square: Square) -> BitBoard {
    unsafe { *KING_ATTACKS.get_unchecked(square.to_index()) }
}

/// Generates the bitboard representing all the squares a king can attack from the given square.
///
/// The function computes the king's potential moves by iterating through the possible relative moves
/// defined in `KING_DELTAS`, ensuring that the resulting squares are within the valid board bounds.
pub fn gen_king_attacks(square: Square) -> BitBoard {
    let mut attacks: BitBoard = BitBoard::EMPTY;
    let rank: i8 = square.rank() as i8;
    let file: i8 = square.file() as i8;

    for (dr, df) in KING_DELTAS.iter() {
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

/// Generates the entire attack table for a king, where each index corresponds to a square
/// on the chessboard and stores the bitboard of attackable squares from that position.
pub fn gen_king_attack_table() -> [BitBoard; 64] {
    let mut table: [BitBoard; Square::NUM_SQUARES] = [BitBoard::EMPTY; Square::NUM_SQUARES];

    for square in BitBoard::FULL {
        table[square.to_index()] = gen_king_attacks(square);
    }

    table
}

#[test]
fn test_gen_attacks() {
    let attack: BitBoard = gen_king_attacks(Square::A2);
    println!("{}", attack);
}

#[test]
fn test_get_attacks() {
    let attack: BitBoard = get_king_attacks(Square::A2);
    println!("{}", attack);
}

#[test]
fn gen_attacks() {
    let attacks: [BitBoard; 64] = gen_king_attack_table();
    println!("{:?}", attacks);
}
