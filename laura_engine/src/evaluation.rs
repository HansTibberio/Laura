/*
    Laura: A multi-threaded UCI chess engine written in Rust.

    Copyright (C) 2024-2025 HansTibberio <hanstiberio@proton.me>

    Laura is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Laura is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Laura. If not, see <https://www.gnu.org/licenses/>.
*/

// src/evaluation.rs

//! Static board evaluation.

use laura_core::{
    get_bishop_attacks, get_knight_attacks, get_rook_attacks, BitBoard, Board, Color, Piece,
    PieceType, Square,
};
use std::ops::{AddAssign, Mul, Sub};

const WHITE: usize = Color::White as usize;
const BLACK: usize = Color::Black as usize;

#[derive(Debug, Clone, Copy)]
pub struct Value(i32, i32);

impl AddAssign for Value {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        Value(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        Value(self.0 * rhs.0, self.1 * rhs.1)
    }
}

const PIECE_VALUE: [Value; 6] = [
    // Pawns
    Value(82, 94),
    // Knights
    Value(337, 281),
    // Bishops
    Value(365, 297),
    // Rooks
    Value(477, 512),
    // Queens
    Value(1025, 936),
    // Kings
    Value(0, 0),
];

#[rustfmt::skip]
const PAWN_TABLE: [Value; 64] = [
    Value(0,0),      Value(0,0),     Value(0,0),     Value(0,0),     Value(0,0),     Value(0,0),     Value(0,0),     Value(0,0),
    Value(98,178),   Value(134,173), Value(61,158),  Value(95,134),  Value(68,147),  Value(126,132), Value(34,165),  Value(-11,187),
    Value(-6,94),    Value(7,100),   Value(26,85),   Value(31,67),   Value(65,56),   Value(56,53),   Value(25,82),   Value(-20, 84),
    Value(-14,32),   Value(13,24),   Value(6,13),    Value(21,5),    Value(23,-2),   Value(12,4),    Value(17,17),   Value(-23,17),
    Value(-27,13),   Value(-2,9),    Value(-5,-3),   Value(12,-7),   Value(17,-7),   Value(6, -8),   Value(10, 3),   Value(-25,-1),
    Value(-26,4),    Value(-4,7),    Value(-4,-6),   Value(-10,1),   Value(3,0),     Value(3,-5),    Value(33,-1),   Value(-12,-8),
    Value(-35,13),   Value(-1,8),    Value(-20,8),   Value(-23,10),  Value(-15,13),  Value(24,0),    Value(38,2),    Value(-22,-7),
    Value(0,0),      Value(0,0),     Value(0,0),     Value(0,0),     Value(0,0),     Value(0,0),     Value(0,0),     Value(0,0),
];

#[rustfmt::skip]
const KNIGHT_TABLE: [Value; 64] = [
    Value(-167,-58), Value(-89,-38), Value(-34,-13), Value(-49,-28), Value(61,-31),  Value(-97,-27), Value(-15,-63), Value(-107,-99), 
    Value(-73,-25),  Value(-41,-8),  Value(72,-25),  Value(36,-2),   Value(23,-9),   Value(62,-25),  Value(7,-24),   Value(-17,-52), 
    Value(-47,-24),  Value(60,-20),  Value(37,10),   Value(65,9),    Value(84,-1),   Value(129,-9),  Value(73,-19),  Value(44,-41), 
    Value(-9,-17),   Value(17,3),    Value(19,22),   Value(53,22),   Value(37,22),   Value(69,11),   Value(18,8),    Value(22,-18), 
    Value(-13,-18),  Value(4,-6),    Value(16,16),   Value(13,25),   Value(28,16),   Value(19,17),   Value(21,4),    Value(-8,-18), 
    Value(-23,-23),  Value(-9,-3),   Value(12,-1),   Value(10,15),   Value(19,10),   Value(17,-3),   Value(25,-20),  Value(-16,-22), 
    Value(-29,-42),  Value(-53,-20), Value(-12,-10), Value(-3,-5),   Value(-1,-2),   Value(18,-20),  Value(-14,-23), Value(-19,-44), 
    Value(-105,-29), Value(-21,-51), Value(-58,-23), Value(-33,-15), Value(-17,-22), Value(-28,-18), Value(-19,-50), Value(-23,-64),
];

#[rustfmt::skip]
const BISHOP_TABLE: [Value; 64] = [
    Value(-29,-14),  Value(4,-21),   Value(-82,-11), Value(-37,-8),  Value(-25,-7),  Value(-42,-9),  Value(7,-17),   Value(-8,-24), 
    Value(-26,-8),   Value(16,-4),   Value(-18,7),   Value(-13,-12), Value(30,-3),   Value(59,-13),  Value(18,-4),   Value(-47,-14), 
    Value(-16,2),    Value(37,-8),   Value(43,0),    Value(40,-1),   Value(35,-2),   Value(50,6),    Value(37,0),    Value(-2,4), 
    Value(-4,-3),    Value(5,9),     Value(19,12),   Value(50,9),    Value(37,14),   Value(37,10),   Value(7,3),     Value(-2,2), 
    Value(-6,-6),    Value(13,3),    Value(13,13),   Value(26,19),   Value(34,7),    Value(12,10),   Value(10,-3),   Value(4,-9), 
    Value(0,-12),    Value(15,-3),   Value(15,8),    Value(15,10),   Value(14,13),   Value(27,3),    Value(18,-7),   Value(10,-15), 
    Value(4,-14),    Value(15,-18),  Value(16,-7),   Value(0,-1),    Value(7,4),     Value(21,-9),   Value(33,-15),  Value(1,-27), 
    Value(-33,-23),  Value(-3,-9),   Value(-14,-23), Value(-21,-5),  Value(-13,-9),  Value(-12,-16), Value(-39,-5),  Value(-21,-17),
];

#[rustfmt::skip]
const ROOK_TABLE: [Value; 64] = [
    Value(32,13),    Value(42,10),   Value(32,18),   Value(51,15),   Value(63,12),   Value(9,12),    Value(31,8),    Value(43,5), 
    Value(27,11),    Value(32,13),   Value(58,13),   Value(62,11),   Value(80,-3),   Value(67,3),    Value(26,8),    Value(44,3), 
    Value(-5,7),     Value(19,7),    Value(26,7),    Value(36,5),    Value(17,4),    Value(45,-3),   Value(61,-5),   Value(16,-3), 
    Value(-24,4),    Value(-11,3),   Value(7,13),    Value(26,1),    Value(24,2),    Value(35,1),    Value(-8,-1),   Value(-20,2), 
    Value(-36,3),    Value(-26,5),   Value(-12,8),   Value(-1,4),    Value(9,-5),    Value(-7,-6),   Value(6,-8),    Value(-23,-11), 
    Value(-45,-4),   Value(-25,0),   Value(-16,-5),  Value(-17,-1),  Value(3,-7),    Value(0,-12),   Value(-5,-8),   Value(-33,-16), 
    Value(-44,-6),   Value(-16,-6),  Value(-20,0),   Value(-9,2),    Value(-1,-9),   Value(11,-9),   Value(-6,-11),  Value(-71,-3), 
    Value(-19,-9),   Value(-13,2),   Value(1,3),     Value(17,-1),   Value(16,-5),   Value(7,-13),   Value(-37,4),   Value(-26,-20),
];

#[rustfmt::skip]
const QUEEN_TABLE: [Value; 64] = [
    Value(-28,-9),   Value(0,22),    Value(29,22),   Value(12,27),   Value(59,27),   Value(44,19),   Value(43,10),   Value(45,20), 
    Value(-24,-17),  Value(-39,20),  Value(-5,32),   Value(1,41),    Value(-16,58),  Value(57,25),   Value(28,30),   Value(54,0), 
    Value(-13,-20),  Value(-17,6),   Value(7,9),     Value(8,49),    Value(29,47),   Value(56,35),   Value(47,19),   Value(57,9), 
    Value(-27,3),    Value(-27,22),  Value(-16,24),  Value(-16,45),  Value(-1,57),   Value(17,40),   Value(-2,57),   Value(1,36), 
    Value(-9,-18),   Value(-26,28),  Value(-9,19),   Value(-10,47),  Value(-2,31),   Value(-4,34),   Value(3,39),    Value(-3,23), 
    Value(-14,-16),  Value(2,-27),   Value(-11,15),  Value(-2,6),    Value(-5,9),    Value(2,17),    Value(14,10),   Value(5,5), 
    Value(-35,-22),  Value(-8,-23),  Value(11,-30),  Value(2,-16),   Value(8,-16),   Value(15,-23),  Value(-3,-36),  Value(1,-32), 
    Value(-1,-33),   Value(-18,-28), Value(-9,-22),  Value(10,-43),  Value(-15,-5),  Value(-25,-32), Value(-31,-20), Value(-50,-41),
];

#[rustfmt::skip]
const KING_TABLE: [Value; 64] = [
    Value(-65,-74),  Value(23,-35),  Value(16,-18),  Value(-15,-18), Value(-56,-11), Value(-34,15),  Value(2,4),     Value(13,-17), 
    Value(29,-12),   Value(-1,17),   Value(-20,14),  Value(-7,17),   Value(-8,17),   Value(-4,38),   Value(-38,23),  Value(-29,11), 
    Value(-9,10),    Value(24,17),   Value(2,23),    Value(-16,15),  Value(-20,20),  Value(6,45),    Value(22,44),   Value(-22,13), 
    Value(-17,-8),   Value(-20,22),  Value(-12,24),  Value(-27,27),  Value(-30,26),  Value(-25,33),  Value(-14,26),  Value(-36,3), 
    Value(-49,-18),  Value(-1,-4),   Value(-27,21),  Value(-39,24),  Value(-46,27),  Value(-44,23),  Value(-33,9),   Value(-51,-11), 
    Value(-14,-19),  Value(-14,-3),  Value(-22,11),  Value(-46,21),  Value(-44,23),  Value(-30,16),  Value(-15,7),   Value(-27,-9), 
    Value(1,-27),    Value(7,-11),   Value(-8,4),    Value(-64,13),  Value(-43,14),  Value(-16,4),   Value(9,-5),    Value(8,-17), 
    Value(-15,-53),  Value(36,-34),  Value(12,-21),  Value(-54,-11), Value(8,-28),   Value(-28,-14), Value(24,-24),  Value(14,-43),
];

const PASSED_PAWN: Value = Value(40, 80);
const ISOLATED_PAWN: Value = Value(-3, -15);
const DOUBLED_PAWN: [Value; 8] = [
    Value(-15, -30), // File A
    Value(-10, -25), // File B
    Value(-7, -15),  // File C
    Value(-4, -10),  // File D
    Value(-4, -10),  // File E
    Value(-7, -15),  // File F
    Value(-10, -25), // File G
    Value(-15, -30), // File H
];
const CONNECTED_PAWN_BONUS: [Value; 8] = [
    Value(0, 0),   // Rank One
    Value(5, 10),  // Rank Two
    Value(15, 20), // Rank Three
    Value(20, 25), // Rank Four
    Value(25, 30), // Rank Five
    Value(35, 35), // Rank Six
    Value(40, 40), // Rank Seven
    Value(0, 0),   // Rank Eight
];
const CENTRAL_PAWN_BONUS: Value = Value(10, 0);
const OUTPOST_KNIGHT_BONUS: [Value; 8] = [
    Value(0, 0),   // Rank One
    Value(0, 0),   // Rank Two
    Value(0, 0),   // Rank Three
    Value(20, 25), // Rank Four
    Value(25, 30), // Rank Five
    Value(35, 35), // Rank Six
    Value(40, 40), // Rank Seven
    Value(0, 0),   // Rank Eight
];
const KNIGHT_MOBILITY_BONUS: [Value; 9] = [
    Value(-40, -35), // 0
    Value(-20, -25), // 1
    Value(-5, -15),  // 2
    Value(0, -5),    // 3
    Value(5, 0),     // 4
    Value(10, 5),    // 5
    Value(15, 10),   // 6
    Value(30, 15),   // 7
    Value(40, 20),   // 8
];
const BISHOP_PAIR: Value = Value(30, 60);
const OUTPOST_BISHOP_BONUS: [Value; 8] = [
    Value(0, 0),   // Rank One
    Value(0, 0),   // Rank Two
    Value(0, 0),   // Rank Three
    Value(20, 25), // Rank Four
    Value(25, 30), // Rank Five
    Value(35, 35), // Rank Six
    Value(40, 40), // Rank Seven
    Value(0, 0),   // Rank Eight
];
const BISHOP_MOBILITY_BONUS: [Value; 14] = [
    Value(-60, -80), // 0
    Value(-30, -50), // 1
    Value(-20, -30), // 2
    Value(-10, -15), // 3
    Value(-5, -10),  // 4
    Value(0, -5),    // 5
    Value(5, 0),     // 6
    Value(10, 5),    // 7
    Value(15, 10),   // 8
    Value(20, 15),   // 9
    Value(25, 20),   // 10
    Value(30, 25),   // 11
    Value(35, 30),   // 12
    Value(40, 35),   // 13
];
const OPEN_FILE_ROOK: [Value; 2] = [Value(10, 0), Value(15, 10)];
const ROOK_MOBILITY_BONUS: [Value; 15] = [
    Value(-30, -60), // 0
    Value(-20, -40), // 1
    Value(-15, -25), // 2
    Value(-10, -15), // 3
    Value(-5, -10),  // 4
    Value(0, -5),    // 5
    Value(5, 0),     // 6
    Value(10, 5),    // 7
    Value(15, 10),   // 8
    Value(20, 15),   // 9
    Value(25, 20),   // 10
    Value(30, 25),   // 11
    Value(35, 30),   // 12
    Value(40, 35),   // 13
    Value(45, 40),   // 14
];
const OUTPOST_ROOK_BONUS: [Value; 8] = [
    Value(0, 0),   // Rank One
    Value(0, 0),   // Rank Two
    Value(0, 0),   // Rank Three
    Value(20, 25), // Rank Four
    Value(25, 30), // Rank Five
    Value(35, 35), // Rank Six
    Value(40, 40), // Rank Seven
    Value(0, 0),   // Rank Eight
];
const QUEEN_MOBILITY_BONUS: [Value; 28] = [
    Value(-60, -80), // 0
    Value(-50, -70),
    Value(-40, -60),
    Value(-35, -50),
    Value(-30, -40),
    Value(-25, -30),
    Value(-20, -25),
    Value(-15, -20),
    Value(-10, -15),
    Value(-5, -10),
    Value(0, -5), // 10
    Value(5, 0),
    Value(10, 5),
    Value(15, 10),
    Value(20, 15),
    Value(25, 20),
    Value(30, 25),
    Value(35, 30),
    Value(40, 35),
    Value(45, 40),
    Value(50, 45),
    Value(55, 50),
    Value(60, 55),
    Value(65, 60),
    Value(70, 65),
    Value(75, 70),
    Value(80, 75),
    Value(85, 80), // 28
];
const OPEN_FILE_KING: [Value; 2] = [Value(-15, -5), Value(-20, -0)];
const SHIELD_PENALTY: [Value; 4] = [Value(-60, 0), Value(-40, 0), Value(-20, 0), Value(5, 0)];
const PAWN_STORM: [Value; 8] = [
    Value(0, 0),
    Value(-20, 0),
    Value(-15, 0),
    Value(-10, 0),
    Value(-5, 0),
    Value(0, 0),
    Value(0, 0),
    Value(0, 0),
];
const TEMPO: i32 = 20;

pub fn evaluate(board: &Board) -> i32 {
    let eval: Value = evaluate_pieces(board);

    // Game phase calculation from Stockfish
    let phase: i32 = phase(board);

    // Interpolated evaluation from White perspective
    let mut eval: i32 = (eval.0 * phase + (eval.1 * (128 - phase))) / 128;
    eval += TEMPO;

    if board.side == Color::Black {
        eval = -eval;
    }

    eval
}

fn evaluate_pieces(board: &Board) -> Value {
    let mut eval: Value = Value(0, 0);
    eval += evaluate_pawns::<WHITE>(board) - evaluate_pawns::<BLACK>(board);
    eval += evaluate_king_pawns::<WHITE>(board) - evaluate_king_pawns::<BLACK>(board);
    eval += evaluate_knights::<WHITE>(board) - evaluate_knights::<BLACK>(board);
    eval += evaluate_bishops::<WHITE>(board) - evaluate_bishops::<BLACK>(board);
    eval += evaluate_rooks::<WHITE>(board) - evaluate_rooks::<BLACK>(board);
    eval += evaluate_queens::<WHITE>(board) - evaluate_queens::<BLACK>(board);
    eval += evaluate_kings::<WHITE>(board) - evaluate_kings::<BLACK>(board);

    eval
}

fn evaluate_pawns<const COLOR: usize>(board: &Board) -> Value {
    let mut eval: Value = Value(0, 0);
    let pawns: BitBoard = board.pieces_bitboard[PieceType::PAWN] & board.sides_bitboard[COLOR];
    let enemy_pawns: BitBoard =
        board.pieces_bitboard[PieceType::PAWN] & board.sides_bitboard[COLOR ^ 1];
    let connected: BitBoard = connected_pawns::<COLOR>(pawns);

    for square in pawns {
        eval += PIECE_VALUE[PieceType::PAWN];
        eval += PAWN_TABLE[square.to_index() ^ (56 * COLOR)];

        // Passed pawn bonus
        if enemy_pawns.0 & PASSED_PAWN_MASKS[COLOR][square.to_index()] == 0 {
            eval += PASSED_PAWN;
        }
        // Isolated pawn penalties
        if pawns.0 & ISOLATED_PAWN_MASKS[square.file().to_index()] == 0 {
            eval += ISOLATED_PAWN;
        }
        // Penalty for doubled pawns
        if pawns.0 & DOUBLED_PAWN_MASK[square.to_index()] != 0 {
            eval += DOUBLED_PAWN[square.file().to_index()];
        }

        //Double supported pawn bonus
        if (pawns.0 & DOUBLE_SUPPORTED_PAWN_MASKS[COLOR][square.to_index()]).count_ones() >= 2 {
            eval += CONNECTED_PAWN_BONUS[square.rank().to_index() ^ (7 * COLOR)]
        }

        // Central pawn bonus
        if pawns.0 & CENTER_MASK != 0 {
            eval += CENTRAL_PAWN_BONUS;
        }
    }

    // Connected pawn bonus
    for square in connected {
        eval += CONNECTED_PAWN_BONUS[square.rank().to_index() ^ (7 * COLOR)]
    }

    eval
}

fn connected_pawns<const COLOR: usize>(pawns: BitBoard) -> BitBoard {
    let phalanx: BitBoard = pawns.left_for::<COLOR>() | pawns.right_for::<COLOR>();
    let supported: BitBoard = pawns.up_left_for::<COLOR>() | pawns.up_right_for::<COLOR>();
    pawns & (phalanx | supported)
}

fn evaluate_king_pawns<const COLOR: usize>(board: &Board) -> Value {
    let mut eval: Value = Value(0, 0);
    let pawns: BitBoard = board.pieces_bitboard[PieceType::PAWN] & board.sides_bitboard[COLOR];
    let enemy_pawns: BitBoard =
        board.pieces_bitboard[PieceType::PAWN] & board.sides_bitboard[COLOR ^ 1];
    let king: Square = (board.pieces_bitboard[PieceType::KING] & board.sides_bitboard[COLOR])
        .to_square()
        .unwrap();

    if king.rank().to_index() ^ (7 * COLOR) <= 1 {
        // Open/Semi-open file penalty
        if pawns & king.file().to_bitboard() == BitBoard::EMPTY {
            let open: usize = (enemy_pawns & king.file().to_bitboard() == BitBoard::EMPTY) as usize;
            eval += OPEN_FILE_KING[open];
        }

        // King Shelter
        let shield_mask: BitBoard = BitBoard(KING_SHELTER_MASK[COLOR][king.to_index()]);
        let shield_count: usize = (pawns & shield_mask).count_bits() as usize;
        eval += SHIELD_PENALTY[shield_count];

        // Pawn Storm
        for pawn in shield_mask {
            let enemy_pawn: BitBoard = pawn.file().to_bitboard() & enemy_pawns;
            let distance: usize = if let Some(square) = enemy_pawn.to_square_nearest::<COLOR>() {
                king.rank().to_index().abs_diff(square.rank().to_index())
            } else {
                7
            };

            eval += PAWN_STORM[distance];
        }
    }

    eval
}
fn evaluate_knights<const COLOR: usize>(board: &Board) -> Value {
    let mut eval: Value = Value(0, 0);
    let knights: BitBoard = board.pieces_bitboard[PieceType::KNIGHT] & board.sides_bitboard[COLOR];
    let enemy_pawns: BitBoard =
        board.pieces_bitboard[PieceType::PAWN] & board.sides_bitboard[COLOR ^ 1];
    let pawns: BitBoard = board.pieces_bitboard[PieceType::PAWN] & board.sides_bitboard[COLOR];
    let outpost: BitBoard = knights & OUTPOST_MASK[COLOR];

    for square in knights {
        eval += PIECE_VALUE[PieceType::KNIGHT];
        eval += KNIGHT_TABLE[square.to_index() ^ (56 * COLOR)];

        // Knight mobility bonus/penalty
        let mobility_count: usize =
            (get_knight_attacks(square) & !board.sides_bitboard[COLOR]).count_bits() as usize;
        eval += KNIGHT_MOBILITY_BONUS[mobility_count];
    }

    // Knight Outpost Bonus
    for square in outpost {
        if enemy_pawns.0 & DOUBLE_SUPPORTED_PAWN_MASKS[COLOR ^ 1][square.to_index()] == 0 {
            let count: u32 =
                (pawns.0 & DOUBLE_SUPPORTED_PAWN_MASKS[COLOR][square.to_index()]).count_ones();

            // Extra bonus if the knight it's supported by two pawns
            eval += OUTPOST_KNIGHT_BONUS[square.rank().to_index() ^ (7 * COLOR)]
                * Value(count as i32, count as i32);
        }
    }

    eval
}

fn evaluate_bishops<const COLOR: usize>(board: &Board) -> Value {
    let mut eval: Value = Value(0, 0);
    let bishops: BitBoard = board.pieces_bitboard[PieceType::BISHOP] & board.sides_bitboard[COLOR];
    let enemy_pawns: BitBoard =
        board.pieces_bitboard[PieceType::PAWN] & board.sides_bitboard[COLOR ^ 1];
    let pawns: BitBoard = board.pieces_bitboard[PieceType::PAWN] & board.sides_bitboard[COLOR];
    let outpost: BitBoard = bishops & OUTPOST_MASK[COLOR];

    // Bishop Pair Bonus
    if (bishops & BitBoard::LIGHT_SQUARES).count_bits() == 1
        && (bishops & BitBoard::DARK_SQUARES).count_bits() == 1
    {
        eval += BISHOP_PAIR;
    }

    for square in bishops {
        eval += PIECE_VALUE[PieceType::BISHOP];
        eval += BISHOP_TABLE[square.to_index() ^ (56 * COLOR)];

        // Bishop mobility bonus/penalty
        let blockers: BitBoard = board.sides_bitboard[COLOR] | board.sides_bitboard[COLOR ^ 1];
        let mobility_count: usize = get_bishop_attacks(square, blockers).count_bits() as usize;
        eval += BISHOP_MOBILITY_BONUS[mobility_count];
    }

    // Bishop Outpost Bonus
    for square in outpost {
        if enemy_pawns.0 & DOUBLE_SUPPORTED_PAWN_MASKS[COLOR ^ 1][square.to_index()] == 0 {
            let count: u32 =
                (pawns.0 & DOUBLE_SUPPORTED_PAWN_MASKS[COLOR][square.to_index()]).count_ones();

            // Extra bonus if the bishop it's supported by two pawns
            eval += OUTPOST_BISHOP_BONUS[square.rank().to_index() ^ (7 * COLOR)]
                * Value(count as i32, count as i32);
        }
    }

    eval
}

fn evaluate_rooks<const COLOR: usize>(board: &Board) -> Value {
    let mut eval: Value = Value(0, 0);
    let rooks: BitBoard = board.pieces_bitboard[PieceType::ROOK] & board.sides_bitboard[COLOR];
    let pawns: BitBoard = board.pieces_bitboard[PieceType::PAWN] & board.sides_bitboard[COLOR];
    let enemy_pawns: BitBoard =
        board.pieces_bitboard[PieceType::PAWN] & board.sides_bitboard[COLOR ^ 1];
    let enemy_king: Square = (board.pieces_bitboard[PieceType::KING]
        & board.sides_bitboard[COLOR ^ 1])
        .to_square()
        .unwrap();
    let outpost: BitBoard = rooks & ROOK_OUTPOST_MASK[COLOR];

    for square in rooks {
        eval += PIECE_VALUE[PieceType::ROOK];
        eval += ROOK_TABLE[square.to_index() ^ (56 * COLOR)];

        // Open/Semi-open file bonus
        if pawns & square.file().to_bitboard() == BitBoard::EMPTY {
            let open: usize =
                (enemy_pawns & square.file().to_bitboard() == BitBoard::EMPTY) as usize;
            eval += OPEN_FILE_ROOK[open];
        }

        // 7 rank bonus
        if square.rank().to_index() ^ (7 * COLOR) == 6
            && enemy_king.rank().to_index() ^ (7 * COLOR) >= 6
        {
            eval += Value(5, 30);
        }

        // Rook mobility bonus/penalty
        let blockers: BitBoard = board.sides_bitboard[COLOR] | board.sides_bitboard[COLOR ^ 1];
        let mobility_count: usize = get_rook_attacks(square, blockers).count_bits() as usize;
        eval += ROOK_MOBILITY_BONUS[mobility_count];
    }

    // Rook Outpost Bonus
    for square in outpost {
        if enemy_pawns.0 & DOUBLE_SUPPORTED_PAWN_MASKS[COLOR ^ 1][square.to_index()] == 0 {
            let count: u32 =
                (pawns.0 & DOUBLE_SUPPORTED_PAWN_MASKS[COLOR][square.to_index()]).count_ones();

            // Extra bonus if the rook it's supported by two pawns
            eval += OUTPOST_ROOK_BONUS[square.rank().to_index() ^ (7 * COLOR)]
                * Value(count as i32, count as i32);
        }
    }

    eval
}

fn evaluate_queens<const COLOR: usize>(board: &Board) -> Value {
    let mut eval: Value = Value(0, 0);
    let queens: BitBoard = board.pieces_bitboard[PieceType::QUEEN] & board.sides_bitboard[COLOR];

    for square in queens {
        eval += PIECE_VALUE[PieceType::QUEEN];
        eval += QUEEN_TABLE[square.to_index() ^ (56 * COLOR)];

        // Queen mobility bonus/penalty
        let blockers: BitBoard = board.sides_bitboard[COLOR] | board.sides_bitboard[COLOR ^ 1];
        let mobility_count: usize = (get_rook_attacks(square, blockers)
            | get_bishop_attacks(square, blockers))
        .count_bits() as usize;
        eval += QUEEN_MOBILITY_BONUS[mobility_count];
    }

    eval
}

fn evaluate_kings<const COLOR: usize>(board: &Board) -> Value {
    let mut eval: Value = Value(0, 0);
    let king: Square = (board.pieces_bitboard[PieceType::KING] & board.sides_bitboard[COLOR])
        .to_square()
        .unwrap();

    eval += KING_TABLE[king.to_index() ^ (56 * COLOR)];

    eval
}

fn non_pawn_material(piece: Piece) -> i32 {
    match piece.piece_type() {
        PieceType::Pawn => 0,
        PieceType::Knight => 781,
        PieceType::Bishop => 825,
        PieceType::Rook => 1276,
        PieceType::Queen => 2538,
        PieceType::King => 0,
    }
}

fn phase(board: &Board) -> i32 {
    const MG_LIMIT: i32 = 15258;
    const EG_LIMIT: i32 = 3915;

    let mut npm: i32 = 0;

    for square in BitBoard::FULL {
        if let Some(piece) = board.piece_on(square) {
            npm += non_pawn_material(piece);
        }
    }

    npm = EG_LIMIT.max(MG_LIMIT.min(npm));

    ((npm - EG_LIMIT) * 128) / (MG_LIMIT - EG_LIMIT)
}

#[rustfmt::skip]
pub const PASSED_PAWN_MASKS: [[u64; 64]; 2] = [
    [
        217020518514230016, 506381209866536704, 1012762419733073408, 2025524839466146816, 4051049678932293632, 8102099357864587264, 16204198715729174528, 13889313184910721024,
        217020518514229248, 506381209866534912, 1012762419733069824, 2025524839466139648, 4051049678932279296, 8102099357864558592, 16204198715729117184, 13889313184910671872,
        217020518514032640, 506381209866076160, 1012762419732152320, 2025524839464304640, 4051049678928609280, 8102099357857218560, 16204198715714437120, 13889313184898088960,
        217020518463700992, 506381209748635648, 1012762419497271296, 2025524838994542592, 4051049677989085184, 8102099355978170368, 16204198711956340736, 13889313181676863488,
        217020505578799104, 506381179683864576, 1012762359367729152, 2025524718735458304, 4051049437470916608, 8102098874941833216, 16204197749883666432, 13889312357043142656,
        217017207043915776, 506373483102470144, 1012746966204940288, 2025493932409880576, 4050987864819761152, 8101975729639522304, 16203951459279044608, 13889101250810609664,
        216172782113783808, 504403158265495552, 1008806316530991104, 2017612633061982208, 4035225266123964416, 8070450532247928832, 16140901064495857664, 13835058055282163712,
        0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0,
        3, 7, 14, 28, 56, 112, 224, 192,
        771, 1799, 3598, 7196, 14392, 28784, 57568, 49344,
        197379, 460551, 921102, 1842204, 3684408, 7368816, 14737632, 12632256,
        50529027, 117901063, 235802126, 471604252, 943208504, 1886417008, 3772834016, 3233857728,
        12935430915, 30182672135, 60365344270, 120730688540, 241461377080, 482922754160, 965845508320, 827867578560,
        3311470314243, 7726764066567, 15453528133134, 30907056266268, 61814112532536, 123628225065072, 247256450130144, 211934100111552,
        847736400446211, 1978051601041159, 3956103202082318, 7912206404164636, 15824412808329272, 31648825616658544, 63297651233317088, 54255129628557504,
    ],
];

#[rustfmt::skip]
pub const DOUBLE_SUPPORTED_PAWN_MASKS: [[u64; 64]; 2] = [
    [
        0, 0, 0, 0, 0, 0, 0, 0,
        2, 5, 10, 20, 40, 80, 160, 64,
        512, 1280, 2560, 5120, 10240, 20480, 40960, 16384,
        131072, 327680, 655360, 1310720, 2621440, 5242880, 10485760, 4194304,
        33554432, 83886080, 167772160, 335544320, 671088640, 1342177280, 2684354560, 1073741824,
        8589934592, 21474836480, 42949672960, 85899345920, 171798691840, 343597383680, 687194767360, 274877906944,
        2199023255552, 5497558138880, 10995116277760, 21990232555520, 43980465111040, 87960930222080, 175921860444160, 70368744177664,
        562949953421312, 1407374883553280, 2814749767106560, 5629499534213120, 11258999068426240, 22517998136852480, 45035996273704960, 18014398509481984
    ],
    [
        512, 1280, 2560, 5120, 10240, 20480, 40960, 16384,
        131072, 327680, 655360, 1310720, 2621440, 5242880, 10485760, 4194304,
        33554432, 83886080, 167772160, 335544320, 671088640, 1342177280, 2684354560, 1073741824,
        8589934592, 21474836480, 42949672960, 85899345920, 171798691840, 343597383680, 687194767360, 274877906944,
        2199023255552, 5497558138880, 10995116277760, 21990232555520, 43980465111040, 87960930222080, 175921860444160, 70368744177664,
        562949953421312, 1407374883553280, 2814749767106560, 5629499534213120, 11258999068426240, 22517998136852480, 45035996273704960, 18014398509481984,
        144115188075855872, 360287970189639680, 720575940379279360, 1441151880758558720, 2882303761517117440, 5764607523034234880, 11529215046068469760, 4611686018427387904,
        0, 0, 0, 0, 0, 0, 0, 0
    ],
];

#[rustfmt::skip]
pub const ISOLATED_PAWN_MASKS: [u64; 8] = [
    144680345676153346, 361700864190383365, 723401728380766730, 1446803456761533460, 2893606913523066920, 5787213827046133840, 11574427654092267680, 4629771061636907072,
];

#[rustfmt::skip]
pub const DOUBLED_PAWN_MASK: [u64; 64] = [
    72340172838076672, 144680345676153344, 289360691352306688, 578721382704613376, 1157442765409226752, 2314885530818453504, 4629771061636907008, 9259542123273814016,
    72340172838076417, 144680345676152834, 289360691352305668, 578721382704611336, 1157442765409222672, 2314885530818445344, 4629771061636890688, 9259542123273781376,
    72340172838011137, 144680345676022274, 289360691352044548, 578721382704089096, 1157442765408178192, 2314885530816356384, 4629771061632712768, 9259542123265425536,
    72340172821299457, 144680345642598914, 289360691285197828, 578721382570395656, 1157442765140791312, 2314885530281582624, 4629771060563165248, 9259542121126330496,
    72340168543109377, 144680337086218754, 289360674172437508, 578721348344875016, 1157442696689750032, 2314885393379500064, 4629770786759000128, 9259541573518000256,
    72339073326448897, 144678146652897794, 289356293305795588, 578712586611591176, 1157425173223182352, 2314850346446364704, 4629700692892729408, 9259401385785458816,
    72058697861366017, 144117395722732034, 288234791445464068, 576469582890928136, 1152939165781856272, 2305878331563712544, 4611756663127425088, 9223513326254850176,
    282578800148737, 565157600297474, 1130315200594948, 2260630401189896, 4521260802379792, 9042521604759584, 18085043209519168, 36170086419038336,
];
const CENTER_MASK: u64 = 103481868288;
#[rustfmt::skip]
pub const KING_SHELTER_MASK: [[u64; 64]; 2] = [
    [
        1792, 1792, 3584, 7168, 14336, 28672, 57344, 57344,
        458752, 458752, 917504, 1835008, 3670016, 7340032, 14680064, 14680064,
        117440512, 117440512, 234881024, 469762048, 939524096, 1879048192, 3758096384, 3758096384,
        30064771072, 30064771072, 60129542144, 120259084288, 240518168576, 481036337152, 962072674304, 962072674304,
        7696581394432, 7696581394432, 15393162788864, 30786325577728, 61572651155456, 123145302310912, 246290604621824, 246290604621824,
        1970324836974592, 1970324836974592, 3940649673949184, 7881299347898368, 15762598695796736, 31525197391593472, 63050394783186944, 63050394783186944,
        504403158265495552, 504403158265495552, 1008806316530991104, 2017612633061982208, 4035225266123964416, 8070450532247928832, 16140901064495857664, 16140901064495857664,
        0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0, 0, 0, 0, 0,
        7, 7, 14, 28, 56, 112, 224, 224,
        1792, 1792, 3584, 7168, 14336, 28672, 57344, 57344,
        458752, 458752, 917504, 1835008, 3670016, 7340032, 14680064, 14680064,
        117440512, 117440512, 234881024, 469762048, 939524096, 1879048192, 3758096384, 3758096384,
        30064771072, 30064771072, 60129542144, 120259084288, 240518168576, 481036337152, 962072674304, 962072674304,
        7696581394432, 7696581394432, 15393162788864, 30786325577728, 61572651155456, 123145302310912, 246290604621824, 246290604621824,
        1970324836974592, 1970324836974592, 3940649673949184, 7881299347898368, 15762598695796736, 31525197391593472, 63050394783186944, 63050394783186944,
    ]
];
pub const OUTPOST_MASK: [BitBoard; 2] = [BitBoard(16954728004976640), BitBoard(258708618240)];
pub const ROOK_OUTPOST_MASK: [BitBoard; 2] = [BitBoard(55102866016174080), BitBoard(840803009280)];

#[cfg(test)]
mod test {
    use crate::evaluation::{
        connected_pawns, evaluate, Value, CONNECTED_PAWN_BONUS, DOUBLE_SUPPORTED_PAWN_MASKS,
        OUTPOST_MASK, WHITE,
    };
    use laura_core::{BitBoard, Board};
    use std::str::FromStr;

    #[test]
    fn position_evaluation_equal() {
        let board: Board = Board::default();
        assert_eq!(evaluate(&board), 20);
    }

    #[test]
    fn test_evaluation() {
        let board: Board =
            Board::from_str("r2r2k1/ppp2p1p/5qp1/3pnb2/1b1NpQ2/1PN1P3/P1PP1PPP/R3KB1R b KQ - 0 1")
                .unwrap();
        println!("Evaluation: {}", evaluate(&board));
    }

    #[test]
    fn passed_pawn() {
        let board: Board = Board::from_str("8/6k1/8/6p1/1P6/6P1/1K6/8 w - - 0 1").unwrap();
        println!("Evaluation: {}", evaluate(&board));
    }

    #[test]
    fn doubled_pawn() {
        let board: Board = Board::from_str("8/6k1/8/1P4p1/1P6/6P1/1K6/8 w - - 0 1").unwrap();
        println!("Evaluation: {}", evaluate(&board));
    }

    #[test]
    fn phalanx() {
        let board: Board =
            Board::from_str("rnbqkbnr/pppppppp/8/7P/8/8/PPPPPPP1/RNBQKBNR w KQkq - 0 1").unwrap();
        let pawns: BitBoard = board.allied_pawns();
        let phalanx: BitBoard = pawns.left_for::<WHITE>() | pawns.right_for::<WHITE>();
        println!("Phalanx: {}", phalanx);
        println!("{}", phalanx & pawns);
    }

    #[test]
    fn connected() {
        let board: Board = Board::from_str(
            "rnbqkbnr/p4p1p/2p1p1p1/1p1p4/1P6/P3PNP1/2PP1P1P/RNBQKB1R w KQkq - 0 1",
        )
        .unwrap();
        let pawns: BitBoard = board.allied_pawns();
        let connected: BitBoard = connected_pawns::<WHITE>(pawns);
        let mut eval: Value = Value(0, 0);
        for square in connected {
            eval += CONNECTED_PAWN_BONUS[square.rank().to_index() ^ (7 * WHITE)]
        }
        println!("{}", connected);
        println!("Connected Pawn Bonus: {:?}", eval);
    }

    #[test]
    fn double_supported() {
        let board: Board = Board::from_str(
            "rnbqkbnr/p4p1p/2p1p1p1/1p1p4/1P6/P3PNP1/2PP1P1P/RNBQKB1R w KQkq - 0 1",
        )
        .unwrap();
        let pawns: BitBoard = board.allied_pawns();
        let mut eval: Value = Value(0, 0);
        let mut supported: BitBoard = BitBoard::EMPTY;
        for square in pawns {
            if (pawns.0 & DOUBLE_SUPPORTED_PAWN_MASKS[WHITE][square.to_index()]).count_ones() >= 2 {
                supported = supported.set_square(square);
            }
        }
        for square in pawns {
            if (pawns.0 & DOUBLE_SUPPORTED_PAWN_MASKS[WHITE][square.to_index()]).count_ones() >= 2 {
                eval += CONNECTED_PAWN_BONUS[square.rank().to_index() ^ (7 * WHITE)]
            }
        }
        println!("{}", supported);
        println!("Double Supported Pawn Bonus: {:?}", eval);
    }

    #[test]
    fn generate_outpost_knight() {
        let board: Board =
            Board::from_str("r2r2k1/pp3pbp/3p2p1/3Np3/2P1P2q/P4P2/1P1Q2PP/1K1R3R w - - 2 21")
                .unwrap();
        let knights: BitBoard = board.allied_knights();
        let pawns: BitBoard = board.allied_pawns();
        let enemy_pawns: BitBoard = board.enemy_pawns();
        let candidate: BitBoard = knights & OUTPOST_MASK[WHITE];
        let mut outpost_knights: BitBoard = BitBoard::EMPTY;
        for square in candidate {
            if enemy_pawns.0 & DOUBLE_SUPPORTED_PAWN_MASKS[WHITE ^ 1][square.to_index()] == 0 {
                let count: u32 =
                    (pawns.0 & DOUBLE_SUPPORTED_PAWN_MASKS[WHITE][square.to_index()]).count_ones();
                assert_eq!(count, 2);
                outpost_knights = outpost_knights.set_square(square);
            }
        }
        println!("Outpost: {}", outpost_knights)
    }
}
