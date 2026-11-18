/*
    Laura: A multi-threaded UCI chess engine written in Rust.

    Copyright (C) 2024-2026 HansTibberio <hanstiberio@proton.me>

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

// src/legality.rs

//! Constant-time legality check for a single move (TT move, killers...).

use laura_core::{
    BitBoard, Board, Color, Move, MoveType, Piece, PieceType, Rank, Square, get_bishop_attacks,
    get_bishop_rays, get_king_attacks, get_knight_attacks, get_pawn_attacks, get_rook_attacks,
    get_rook_rays,
};

// Castling geometry (standard chess). laura_core keeps these as pub(crate).
const KING_FROM: [Square; 2] = [Square::E1, Square::E8];
const KS_DEST: [Square; 2] = [Square::G1, Square::G8];
const KS_MEDIUM: [Square; 2] = [Square::F1, Square::F8];
const KS_EMPTY: [BitBoard; 2] = [BitBoard(0x60), BitBoard(0x60 << 56)]; // F, G
const QS_DEST: [Square; 2] = [Square::C1, Square::C8];
const QS_MEDIUM: [Square; 2] = [Square::D1, Square::D8];
const QS_EMPTY: [BitBoard; 2] = [BitBoard(0x0E), BitBoard(0x0E << 56)]; // B, C, D

const START_RANK: [Rank; 2] = [Rank::Two, Rank::Seven];
const PROMO_RANK: [Rank; 2] = [Rank::Eight, Rank::One];

// Returns true if mv is a legal move on board.
#[inline]
pub fn is_legal_move(board: &Board, mv: Move) -> bool {
    if mv.is_null() {
        return false;
    }

    // Flags 0b0110 and 0b0111 are unused: `get_type()` would transmute them
    // into an invalid `MoveType`, so they must be rejected before calling it.
    let flag: u16 = mv.flag();
    if flag == 0b0110 || flag == 0b0111 {
        return false;
    }

    let side: Color = board.side();
    let us: usize = side as usize;
    let src: Square = mv.get_src();
    let dest: Square = mv.get_dest();
    let dest_bb: BitBoard = dest.to_bitboard();

    // 1. There must be one of our pieces on the source square
    let piece: Piece = match board.piece_on(src) {
        Some(piece) if piece.color() == side => piece,
        _ => return false,
    };

    // 2. The destination can never hold one of our pieces (also rejects src == dest)
    if !(board.allied_presence() & dest_bb).is_empty() {
        return false;
    }

    let occ: BitBoard = board.combined_bitboard();
    let move_type: MoveType = mv.get_type();
    let piece_type: PieceType = piece.piece_type();

    // 3. Castling has its own complete rule set
    if mv.is_castle() {
        return piece_type == PieceType::King
            && src == KING_FROM[us]
            && board.checkers.is_empty()
            && castling_ok(board, mv.is_king_castle(), dest, us, occ);
    }

    let is_ep: bool = move_type == MoveType::EnPassant;
    let dest_enemy: bool = !(board.enemy_presence() & dest_bb).is_empty();

    // 4. The capture flag must match what is on the destination square
    if !is_ep && mv.is_capture() != dest_enemy {
        return false;
    }

    // 5. Piece geometry
    if piece_type == PieceType::Pawn {
        // Promotion flag <=> destination on the last rank
        if (dest.rank() == PROMO_RANK[us]) != mv.is_promotion() {
            return false;
        }

        let geometry_ok: bool = match move_type {
            // Single push (dest is known to be empty: not ally, not enemy)
            MoveType::Quiet
            | MoveType::PromotionKnight
            | MoveType::PromotionBishop
            | MoveType::PromotionRook
            | MoveType::PromotionQueen => dest == src.forward(side),

            MoveType::DoublePawn => {
                let mid: Square = src.forward(side);
                src.rank() == START_RANK[us] && !occ.get_square(mid) && dest == mid.forward(side)
            }

            MoveType::Capture
            | MoveType::CapPromoKnight
            | MoveType::CapPromoBishop
            | MoveType::CapPromoRook
            | MoveType::CapPromoQueen => get_pawn_attacks(side, src).get_square(dest),

            MoveType::EnPassant => {
                board.enpassant_square == Some(dest) && get_pawn_attacks(side, src).get_square(dest)
            }

            _ => false,
        };

        if !geometry_ok {
            return false;
        }
    } else {
        // Pieces only use Quiet / Capture encodings
        if move_type != MoveType::Quiet && move_type != MoveType::Capture {
            return false;
        }

        let attacks: BitBoard = match piece_type {
            PieceType::Knight => get_knight_attacks(src),
            PieceType::Bishop => get_bishop_attacks(src, occ),
            PieceType::Rook => get_rook_attacks(src, occ),
            PieceType::Queen => get_bishop_attacks(src, occ) | get_rook_attacks(src, occ),
            PieceType::King => get_king_attacks(src),
            PieceType::Pawn => unreachable!(),
        };

        if !attacks.get_square(dest) {
            return false;
        }

        // King: the destination must not be attacked once the king leaves `src`
        if piece_type == PieceType::King {
            return !board.attacked_square(dest, occ.pop_square(src));
        }
    }

    // 6. King safety for non-king moves
    let in_check: bool = !board.checkers.is_empty();
    if board.checkers.count_bits() > 1 {
        return false; // Double check: only king moves
    }

    let king: Square = board.allied_king().to_square().unwrap();

    // Fast path: not in check and the piece is not on a line with our king,
    // so moving it cannot uncover an attack (en passant also removes a pawn).
    if !in_check && !is_ep && !(get_bishop_rays(king) | get_rook_rays(king)).get_square(src) {
        return true;
    }

    let mut captured: BitBoard = dest_bb;
    let mut occ_after: BitBoard = (occ ^ src.to_bitboard()) | dest_bb;
    if is_ep {
        let victim: BitBoard = dest.forward(!side).to_bitboard();
        captured = victim;
        occ_after ^= victim;
    }

    if in_check {
        // Any attacker (including the leaper that gives check) must be gone
        (board.attackers(king, occ_after) & !captured).is_empty()
    } else {
        // Only sliders can be uncovered
        let diagonal: BitBoard = get_bishop_attacks(king, occ_after) & board.enemy_queen_bishops();
        let linear: BitBoard = get_rook_attacks(king, occ_after) & board.enemy_queen_rooks();
        ((diagonal | linear) & !captured).is_empty()
    }
}

#[inline(always)]
fn castling_ok(board: &Board, king_side: bool, dest: Square, us: usize, occ: BitBoard) -> bool {
    if king_side {
        board.castling.has_kingside(board.side())
            && dest == KS_DEST[us]
            && (occ & KS_EMPTY[us]).is_empty()
            && !board.attacked_square(KS_MEDIUM[us], occ)
            && !board.attacked_square(dest, occ)
    } else {
        board.castling.has_queenside(board.side())
            && dest == QS_DEST[us]
            && (occ & QS_EMPTY[us]).is_empty()
            && !board.attacked_square(QS_MEDIUM[us], occ)
            && !board.attacked_square(dest, occ)
    }
}

#[cfg(test)]
mod test {
    use super::is_legal_move;
    use laura_core::{AllMoves, Board, Move, MoveList, gen_moves};
    use std::str::FromStr;
    use std::time::Instant;

    const FENS: [&str; 8] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        "8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3",
        "8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1",
    ];

    // Collects every position reachable in `depth` plies (with duplicates).
    fn collect(board: &Board, depth: usize, out: &mut Vec<Board>) {
        out.push(*board);
        if depth == 0 {
            return;
        }
        for mv in gen_moves::<AllMoves>(board).iter() {
            collect(&board.make_move(*mv), depth - 1, out);
        }
    }

    fn positions() -> Vec<Board> {
        let mut out: Vec<Board> = Vec::new();
        for fen in FENS {
            let mut tree: Vec<Board> = Vec::new();
            collect(&Board::from_str(fen).unwrap(), 2, &mut tree);
            // Keep a spread-out sample to keep the test fast
            out.extend(tree.into_iter().step_by(7));
        }
        out
    }

    /// Every possible 16-bit move is checked against the generator.
    #[test]
    fn matches_generator_for_every_encoding() {
        let boards: Vec<Board> = positions();
        for board in boards.iter() {
            let legal: MoveList = gen_moves::<AllMoves>(board);
            for raw in 0..=u16::MAX {
                let mv: Move = Move(raw);
                let expected: bool = !mv.is_null() && legal.contains(&mv);
                assert_eq!(
                    is_legal_move(board, mv),
                    expected,
                    "{} | move {} (raw {raw})",
                    board.to_fen(),
                    mv
                );
            }
        }
        println!("checked {} positions x 65536 encodings", boards.len());
    }

    #[test]
    fn bench_against_generator() {
        let boards: Vec<Board> = positions();
        // Typical TT hit: the stored move is legal
        let probes: Vec<(Board, Move)> = boards
            .iter()
            .filter_map(|b| gen_moves::<AllMoves>(b).iter().last().map(|m| (*b, *m)))
            .collect();

        let rounds: usize = 200;
        let start: Instant = Instant::now();
        let mut hits: usize = 0;
        for _ in 0..rounds {
            for (b, m) in probes.iter() {
                hits += std::hint::black_box(gen_moves::<AllMoves>(b).contains(m)) as usize;
            }
        }
        let old = start.elapsed();

        let start: Instant = Instant::now();
        let mut hits_new: usize = 0;
        for _ in 0..rounds {
            for (b, m) in probes.iter() {
                hits_new += std::hint::black_box(is_legal_move(b, *m)) as usize;
            }
        }
        let new = start.elapsed();

        assert_eq!(hits, hits_new);
        let n = (rounds * probes.len()) as f64;
        println!(
            "gen_moves+contains: {:.1} ns/call | is_legal_move: {:.1} ns/call | x{:.1}",
            old.as_nanos() as f64 / n,
            new.as_nanos() as f64 / n,
            old.as_nanos() as f64 / new.as_nanos() as f64
        );
    }
}
