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

use crate::castle_rights::*;
use crate::get_king_attacks;
use crate::get_knight_attacks;
use crate::get_pawn_attacks;
use crate::{get_between, get_bishop_rays, get_rook_rays};

#[cfg(not(feature = "bmi2"))]
use crate::{get_bishop_attacks, get_rook_attacks};
#[cfg(feature = "bmi2")]
use crate::{get_bishop_attacks, get_rook_attacks};

use crate::{BitBoard, Board, Call_Handler, Enumerate_Moves, Move, MoveList, MoveType, Square};

// This file is responsible for generating legal moves for pieces, which is a core
// part of the chess engine's functionality. It works with bitboards and evaluates
// possible moves based on the current game state.
//
// This version of the move generation code has been adapted from the chess engine Belette.
// with en passant move generation inspired by Cozy-Chess.
//
// This file contains code licensed under GPLv3 and MIT.
// - Belette (GPLv3): https://github.com/vincentbab/Belette/blob/main/src/movegen.h
// - Cozy-Chess (MIT): https://github.com/analog-hors/cozy-chess/blob/master/cozy-chess/src/board/movegen/mod.rs

// Constants defining the types of moves for the move generation process.
// - QUIET_MOVES represents standard, non-capturing moves.
// - TACTICAL_MOVES represents capturing or special moves, such as en passant or queen promotions.
// - ALL_MOVES is a combination of both quiet and tactical moves.
pub const QUIET_MOVES: usize = 1;
pub const TACTICAL_MOVES: usize = 2;
pub const ALL_MOVES: usize = QUIET_MOVES | TACTICAL_MOVES;

/// Generates a list of legal moves for the given board based on the specified move types.
///  
/// This function enumerates all legal moves for the provided board, considering the move types
/// defined by the `ALL_MOVES` constant. It collects these moves in a `MoveList` and returns it.
#[inline(always)]
pub fn gen_moves<const ALL_MOVES: usize>(board: &Board) -> MoveList {
    let mut move_list: MoveList = MoveList::default();
    enumerate_legal_moves::<ALL_MOVES, _>(board, |mv| -> bool {
        move_list.push(mv);
        true
    });
    move_list
}

/// Enumerates all legal moves for the given board based on the specified move types.
///  
/// This function evaluates the current game state (including check conditions) and generates
/// all possible legal moves for each piece on the board. The move types to be generated are
/// determined by the `ALL_MOVES` constant.
#[inline(always)]
pub fn enumerate_legal_moves<const ALL_MOVES: usize, F>(board: &Board, mut handler: F) -> bool
where
    F: FnMut(Move) -> bool,
{
    let (diagonal_pins, linear_pins) = pinners(board);
    match board.checkers.count_bits() {
        0 => {
            Enumerate_Moves!(false, board, diagonal_pins, linear_pins, handler);
            if ALL_MOVES & QUIET_MOVES != 0 {
                enumerate_castling_moves(board, &mut handler);
            }
        }
        1 => {
            Enumerate_Moves!(true, board, diagonal_pins, linear_pins, handler);
        }
        _ => {}
    }
    enumerate_king_moves::<ALL_MOVES, F>(board, board.allied_king().to_square(), &mut handler);
    true
}

/// Enumerates the normal pawn moves for the given board, considering quiet moves and tactical moves.
///  
/// This function handles the generation of all possible pawn normal moves, including:
/// - Single and double pushes, with special handling for pawns on the second or seventh ranks.
/// - Normal captures, considering any pins and the presence of enemy pieces.
#[inline(always)]
fn enumerate_pawn_normal_moves<const IN_CHECK: bool, const ALL_MOVES: usize, F>(
    board: &Board,
    src: BitBoard,
    diagonal_pins: BitBoard,
    linear_pins: BitBoard,
    handler: &mut F,
) -> bool
where
    F: FnMut(Move) -> bool,
{
    const RANK_7: [BitBoard; 2] = [BitBoard::RANK_7, BitBoard::RANK_2];
    const RANK_3: [BitBoard; 2] = [BitBoard::RANK_3, BitBoard::RANK_6];

    //Single & Double Push
    if ALL_MOVES & QUIET_MOVES != 0 {
        let pawns: BitBoard = src & !RANK_7[board.side as usize] & !diagonal_pins;

        // Non-promotion single pawn pushes.
        let mut single_push: BitBoard = ((pawns & !linear_pins).forward(board.side)
            | ((pawns & linear_pins).forward(board.side) & linear_pins))
            & !board.combined_bitboard();

        let mut double_push: BitBoard = (single_push & RANK_3[board.side as usize])
            .forward(board.side)
            & !board.combined_bitboard();

        if IN_CHECK {
            single_push &= check_mask::<IN_CHECK>(board);
            double_push &= check_mask::<IN_CHECK>(board);
        }

        for dest in single_push {
            let src: Square = dest.backward(board.side);
            Call_Handler!(handler, src, dest, Quiet);
        }

        for dest in double_push {
            let src: Square = (dest.backward(board.side)).backward(board.side);
            Call_Handler!(handler, src, dest, DoublePawn);
        }
    }

    // Normal Captures (Non promotions)
    if ALL_MOVES & TACTICAL_MOVES != 0 {
        let pawns: BitBoard = src & !RANK_7[board.side as usize] & !linear_pins;
        let mut capture_left: BitBoard = ((pawns & !diagonal_pins).up_left(board.side)
            | ((pawns & diagonal_pins).up_left(board.side) & diagonal_pins))
            & board.enemy_presence();
        let mut capture_right: BitBoard = ((pawns & !diagonal_pins).up_right(board.side)
            | ((pawns & diagonal_pins).up_right(board.side) & diagonal_pins))
            & board.enemy_presence();

        if IN_CHECK {
            capture_left &= check_mask::<IN_CHECK>(board);
            capture_right &= check_mask::<IN_CHECK>(board);
        }

        for dest in capture_left {
            let src: Square = dest.backward(board.side).right_color(board.side);
            Call_Handler!(handler, src, dest, Capture);
        }

        for dest in capture_right {
            let src: Square = dest.backward(board.side).left_color(board.side);
            Call_Handler!(handler, src, dest, Capture);
        }
    }

    true
}

/// Enumerates all pawn promotion moves for the given board, considering both quiet promotions
/// and capture promotions.
///
/// It handles both:
/// - Capture promotions, where pawns capture an enemy piece diagonally and promote.
/// - Quiet promotions, where pawns advance forward and promote without capturing.
#[inline(always)]
fn enumerate_pawn_promotion_moves<const IN_CHECK: bool, const ALL_MOVES: usize, F>(
    board: &Board,
    src: BitBoard,
    diagonal_pins: BitBoard,
    linear_pins: BitBoard,
    handler: &mut F,
) -> bool
where
    F: FnMut(Move) -> bool,
{
    const RANK_7: [BitBoard; 2] = [BitBoard::RANK_7, BitBoard::RANK_2];

    let pawns_to_promote: BitBoard = src & RANK_7[board.side as usize];

    if pawns_to_promote.0 != 0 {
        // Capture Promotions
        {
            let pawns: BitBoard = pawns_to_promote & !linear_pins;
            let mut capture_left_prom: BitBoard = ((pawns & !diagonal_pins).up_left(board.side)
                | ((pawns & diagonal_pins).up_left(board.side) & diagonal_pins))
                & board.enemy_presence();
            let mut capture_right_prom: BitBoard = ((pawns & !diagonal_pins).up_right(board.side)
                | ((pawns & diagonal_pins).up_right(board.side) & diagonal_pins))
                & board.enemy_presence();

            if IN_CHECK {
                capture_left_prom &= check_mask::<IN_CHECK>(board);
                capture_right_prom &= check_mask::<IN_CHECK>(board);
            }

            for dest in capture_left_prom {
                let src: Square = dest.backward(board.side).right_color(board.side);
                enumerate_promotions::<ALL_MOVES, F>(src, dest, handler, true);
            }

            for dest in capture_right_prom {
                let src: Square = dest.backward(board.side).left_color(board.side);
                enumerate_promotions::<ALL_MOVES, F>(src, dest, handler, true);
            }
        }

        // Quiet Promotions
        {
            let pawns: BitBoard = pawns_to_promote & !diagonal_pins;
            let mut quiet_promotions: BitBoard = ((pawns & !linear_pins).forward(board.side)
                | ((pawns & linear_pins).forward(board.side) & linear_pins))
                & !board.combined_bitboard();

            if IN_CHECK {
                quiet_promotions &= check_mask::<IN_CHECK>(board);
            }

            for dest in quiet_promotions {
                let src: Square = dest.backward(board.side);
                enumerate_promotions::<ALL_MOVES, F>(src, dest, handler, false);
            }
        }
    }
    true
}

/// Enumerates all possible pawn promotion moves for a given pawn that has reached the promotion rank.
/// The function handles both capture and quiet promotions, allowing the pawn to promote to any of the four pieces:
/// Queen, Rook, Bishop, or Knight.
///  
/// It handles:
/// - Tactical moves (capture and quiet promotions) to Queen.
/// - Quiet moves to Rook, Bishop, or Knight.
#[inline(always)]
fn enumerate_promotions<const ALL_MOVES: usize, F>(
    src: Square,
    dest: Square,
    handler: &mut F,
    capture: bool,
) -> bool
where
    F: FnMut(Move) -> bool,
{
    macro_rules! Call_Promotion {
        ($promo_type:ident, $cap_type:ident) => {
            if capture {
                Call_Handler!(handler, src, dest, $cap_type);
            } else {
                Call_Handler!(handler, src, dest, $promo_type);
            }
        };
    }

    if ALL_MOVES & TACTICAL_MOVES != 0 {
        Call_Promotion!(PromotionQueen, CapPromoQueen);
    }

    if ALL_MOVES & QUIET_MOVES != 0 {
        Call_Promotion!(PromotionRook, CapPromoRook);
        Call_Promotion!(PromotionBishop, CapPromoBishop);
        Call_Promotion!(PromotionKnight, CapPromoKnight);
    }

    true
}

/// Enumerates all possible pawn en passant moves for the given board, considering the current game state.
/// This function checks if any pawn can capture an enemy pawn using the en passant rule and ensures that
/// the move does not expose the king to any attacks.
///
/// The function works by first identifying the en passant square, then checking which pawns can attack
/// that square. It ensures that performing an en passant capture does not leave the king vulnerable to
/// attacks by rooks, queens, or bishops.
#[inline(always)]
fn enumerate_pawn_en_passant_moves<F>(
    board: &Board,
    src: BitBoard,
    linear_pins: BitBoard,
    handler: &mut F,
) -> bool
where
    F: FnMut(Move) -> bool,
{
    let pawns: BitBoard = src & !linear_pins;
    let king_square: Square = board.allied_king().to_square();

    // En Passant captures
    if let Some(en_passant) = board.enpassant_square {
        let dest: Square = en_passant;
        let victim: Square = en_passant.forward(!board.side);

        // Check which pawns can capture en passant.
        for src in pawns & get_pawn_attacks(!board.side, dest) {
            // Simulate the board after en passant capture.
            let blockers: BitBoard =
                board.combined_bitboard() ^ victim.to_bitboard() ^ src.to_bitboard()
                    | dest.to_bitboard();

            // Ensure en passant does not expose the king to a rook or queen attack.
            let king_ray: bool =
                !(get_rook_rays(king_square) & board.enemy_queen_rooks()).is_empty();
            if king_ray
                && !(get_rook_attacks(king_square, blockers) & board.enemy_queen_rooks()).is_empty()
            {
                continue;
            }

            // Ensure en passant does not expose the king to a bishop or queen attack.
            let king_ray: bool =
                !(get_bishop_rays(king_square) & board.enemy_queen_bishops()).is_empty();
            if king_ray
                && !(get_bishop_attacks(king_square, blockers) & board.enemy_queen_bishops())
                    .is_empty()
            {
                continue;
            }

            Call_Handler!(handler, src, dest, EnPassant);
        }
    }
    true
}

/// Enumerates all possible pawn moves for the given board, including normal moves, promotions,
/// and en passant captures.
/// The function handles different types of pawn moves based on the game state and the `ALL_MOVES` constant.
#[inline(always)]
fn enumerate_pawn_moves<const IN_CHECK: bool, const ALL_MOVES: usize, F>(
    board: &Board,
    src: BitBoard,
    diagonal_pins: BitBoard,
    linear_pins: BitBoard,
    handler: &mut F,
) -> bool
where
    F: FnMut(Move) -> bool,
{
    enumerate_pawn_normal_moves::<IN_CHECK, ALL_MOVES, F>(
        board,
        src,
        diagonal_pins,
        linear_pins,
        handler,
    );
    enumerate_pawn_promotion_moves::<IN_CHECK, ALL_MOVES, F>(
        board,
        src,
        diagonal_pins,
        linear_pins,
        handler,
    );
    if ALL_MOVES & TACTICAL_MOVES != 0 {
        enumerate_pawn_en_passant_moves::<F>(board, src, linear_pins, handler);
    }
    true
}

/// Enumerates all possible castling moves for the current side, both kingside and queenside castling.
/// The function checks if castling is available and whether the king and relevant squares are not under attack,
/// and if there are no obstructions between the king and the rook.
#[inline(always)]
fn enumerate_castling_moves<F>(board: &Board, handler: &mut F) -> bool
where
    F: FnMut(Move) -> bool,
{
    // King Side Castling
    if board.castling.has_kingside(board.side) {
        let side: usize = board.side as usize;
        let src: Square = SOURCE[side];
        let dest: Square = DESTINATION[KING_SIDE][side];

        if (board.combined_bitboard() & PRESENCE[KING_SIDE][side]).is_empty()
            && !board.attacked_square(MEDIUM[KING_SIDE][side], board.combined_bitboard())
            && !board.attacked_square(dest, board.combined_bitboard())
        {
            Call_Handler!(handler, src, dest, KingCastle);
        }
    }
    // Queen Side Castling
    if board.castling.has_queenside(board.side) {
        let side: usize = board.side as usize;
        let src: Square = SOURCE[side];
        let dest: Square = DESTINATION[QUEEN_SIDE][side];

        if (board.combined_bitboard() & PRESENCE[QUEEN_SIDE][side]).is_empty()
            && !board.attacked_square(MEDIUM[QUEEN_SIDE][side], board.combined_bitboard())
            && !board.attacked_square(dest, board.combined_bitboard())
        {
            Call_Handler!(handler, src, dest, QueenCastle);
        }
    }

    true
}

/// Enumerates all legal king moves, ensuring the king does not move into an attacked square.
/// The function considers both tactical (captures) and quiet moves based on the `ALL_MOVES` flag.
#[inline(always)]
fn enumerate_king_moves<const ALL_MOVES: usize, F>(
    board: &Board,
    src: Square,
    handler: &mut F,
) -> bool
where
    F: FnMut(Move) -> bool,
{
    // Get all possible king moves, avoiding squares occupied by allied pieces.
    let mut king: BitBoard = get_king_attacks(src) & !board.allied_presence();
    let blockers: BitBoard = board.combined_bitboard().pop_square(src);

    if ALL_MOVES == TACTICAL_MOVES {
        king &= board.enemy_presence()
    }
    if ALL_MOVES == QUIET_MOVES {
        king &= !board.enemy_presence()
    }

    // Iterate through the possible king moves and ensure the king does not move into check.
    for dest in king {
        if !board.attacked_square(dest, blockers) {
            if board.enemy_presence().get_square(dest) {
                Call_Handler!(handler, src, dest, Capture);
            } else {
                Call_Handler!(handler, src, dest, Quiet);
            }
        }
    }
    true
}

/// Enumerates all legal knight moves, considering possible checks and move type constraints.
/// This function ensures that pinned knights cannot move and filters moves based on tactical
/// or quiet move generation flags.
#[inline(always)]
fn enumerate_knight_moves<const IN_CHECK: bool, const ALL_MOVES: usize, F>(
    board: &Board,
    src: BitBoard,
    diagonal_pins: BitBoard,
    linear_pins: BitBoard,
    handler: &mut F,
) -> bool
where
    F: FnMut(Move) -> bool,
{
    // Remove pinned knights from the move generation.
    let knights: BitBoard = src & !(diagonal_pins | linear_pins);

    for src in knights {
        let mut attacks: BitBoard = get_knight_attacks(src) & !board.allied_presence();

        // If in check, restrict moves to those that block or capture the checking piece.
        if IN_CHECK {
            attacks &= check_mask::<IN_CHECK>(board);
        }

        if ALL_MOVES == TACTICAL_MOVES {
            attacks &= board.enemy_presence();
        }

        if ALL_MOVES == QUIET_MOVES {
            attacks &= !board.enemy_presence();
        }

        for dest in attacks {
            if board.enemy_presence().get_square(dest) {
                Call_Handler!(handler, src, dest, Capture);
            } else {
                Call_Handler!(handler, src, dest, Quiet);
            }
        }
    }
    true
}

/// Enumerates all legal bishop moves, considering check restrictions, pinning, and move type constraints.
///
/// - Bishops that are not pinned can move freely along diagonal lines.
/// - Pinned bishops can only move along the pinning diagonal.
/// - The function filters moves based on the requested move type (tactical or quiet).
/// - If the king is in check, moves are restricted to those that block or capture the checking piece.
///
#[inline(always)]
fn enumerate_bishop_moves<const IN_CHECK: bool, const ALL_MOVES: usize, F>(
    board: &Board,
    src: BitBoard,
    diagonal_pins: BitBoard,
    linear_pins: BitBoard,
    handler: &mut F,
) -> bool
where
    F: FnMut(Move) -> bool,
{
    // Non pinned Bishops|Queens
    let bishops: BitBoard = src & !linear_pins & !diagonal_pins;

    for src in bishops {
        let mut attacks: BitBoard =
            get_bishop_attacks(src, board.combined_bitboard()) & !board.allied_presence();

        if IN_CHECK {
            attacks &= check_mask::<IN_CHECK>(board);
        }

        if ALL_MOVES == TACTICAL_MOVES {
            attacks &= board.enemy_presence();
        }

        if ALL_MOVES == QUIET_MOVES {
            attacks &= !board.enemy_presence();
        }

        for dest in attacks {
            if board.enemy_presence().get_square(dest) {
                Call_Handler!(handler, src, dest, Capture);
            } else {
                Call_Handler!(handler, src, dest, Quiet);
            }
        }
    }

    // Pinned Bishops|Queens along diagonal lines.
    let bishops: BitBoard = src & !linear_pins & diagonal_pins;

    for src in bishops {
        let mut attacks: BitBoard = get_bishop_attacks(src, board.combined_bitboard())
            & !board.allied_presence()
            & diagonal_pins;

        if IN_CHECK {
            attacks &= check_mask::<IN_CHECK>(board);
        }

        if ALL_MOVES == TACTICAL_MOVES {
            attacks &= board.enemy_presence();
        }

        if ALL_MOVES == QUIET_MOVES {
            attacks &= !board.enemy_presence();
        }

        for dest in attacks {
            if board.enemy_presence().get_square(dest) {
                Call_Handler!(handler, src, dest, Capture);
            } else {
                Call_Handler!(handler, src, dest, Quiet);
            }
        }
    }
    true
}

/// Enumerates all legal rook moves, considering check restrictions, pinning, and move type constraints.
///
/// - Rooks that are not pinned can move freely along rank and file.
/// - Pinned rooks can only move along the pinning file or rank.
/// - The function filters moves based on the requested move type (tactical or quiet).
/// - If the king is in check, moves are restricted to those that block or capture the checking piece.
#[inline(always)]
fn enumerate_rook_moves<const IN_CHECK: bool, const ALL_MOVES: usize, F>(
    board: &Board,
    src: BitBoard,
    diagonal_pins: BitBoard,
    linear_pins: BitBoard,
    handler: &mut F,
) -> bool
where
    F: FnMut(Move) -> bool,
{
    // Non pinned Rooks|Queens
    let rooks: BitBoard = src & !diagonal_pins & !linear_pins;

    for src in rooks {
        let mut attacks: BitBoard =
            get_rook_attacks(src, board.combined_bitboard()) & !board.allied_presence();

        if IN_CHECK {
            attacks &= check_mask::<IN_CHECK>(board);
        }

        if ALL_MOVES == TACTICAL_MOVES {
            attacks &= board.enemy_presence();
        }

        if ALL_MOVES == QUIET_MOVES {
            attacks &= !board.enemy_presence();
        }

        for dest in attacks {
            if board.enemy_presence().get_square(dest) {
                Call_Handler!(handler, src, dest, Capture);
            } else {
                Call_Handler!(handler, src, dest, Quiet);
            }
        }
    }

    // Pinned Rooks|Queens along rank or file.
    let rooks: BitBoard = src & !diagonal_pins & linear_pins;

    for src in rooks {
        let mut attacks: BitBoard = get_rook_attacks(src, board.combined_bitboard())
            & !board.allied_presence()
            & linear_pins;

        if IN_CHECK {
            attacks &= check_mask::<IN_CHECK>(board);
        }

        if ALL_MOVES == TACTICAL_MOVES {
            attacks &= board.enemy_presence();
        }

        if ALL_MOVES == QUIET_MOVES {
            attacks &= !board.enemy_presence();
        }

        for dest in attacks {
            if board.enemy_presence().get_square(dest) {
                Call_Handler!(handler, src, dest, Capture);
            } else {
                Call_Handler!(handler, src, dest, Quiet);
            }
        }
    }
    true
}

/// Identifies all possible squares where a piece could be pinned to the king.
///
/// This function determines squares that are along a potential pinning line
/// between the king and an enemy sliding piece (bishop, rook, or queen). It does **not**
/// return the pinned pieces directly, but rather the bitboard of squares where a piece
/// could be pinned.
///
/// **How it works**:
/// 1. Determines which squares could potentially contain pinned pieces.
/// 2. Simulates removing those pieces to check if an enemy piece is attacking the king.
/// 3. Collects all such pinning paths and returns them as bitboards.
#[inline(always)]
fn pinners(board: &Board) -> (BitBoard, BitBoard) {
    let king_square: Square = board.allied_king().to_square();
    let blockers_mask: BitBoard = board.combined_bitboard();

    let probe: BitBoard = (get_bishop_rays(king_square) | get_rook_rays(king_square))
        & (board.enemy_queen_bishops() | board.enemy_queen_rooks());

    if probe.is_empty() {
        return (BitBoard::EMPTY, BitBoard::EMPTY);
    }

    // Identify squares along potential pinning paths (diagonal and linear).
    let diagonal_pinned: BitBoard =
        get_bishop_attacks(king_square, blockers_mask) & board.allied_presence();
    let linnear_pinned: BitBoard =
        get_rook_attacks(king_square, blockers_mask) & board.allied_presence();

    // Simulate removing those pieces to check if the king would be attacked.
    let diagonal_pinned_removed: BitBoard = blockers_mask & !diagonal_pinned;
    let linear_pinned_removed: BitBoard = blockers_mask & !linnear_pinned;

    // Find enemy attackers that could be pinning a piece along diagonal or linear paths.
    let diagonal_attackers: BitBoard =
        get_bishop_attacks(king_square, diagonal_pinned_removed) & board.enemy_queen_bishops();
    let linear_attackers: BitBoard =
        get_rook_attacks(king_square, linear_pinned_removed) & board.enemy_queen_rooks();

    // Get squares along the diagonal pinning line.
    let mut diagonal_pins: BitBoard = BitBoard::EMPTY;
    for attacker in diagonal_attackers {
        let pin: BitBoard = get_between(king_square, attacker);
        diagonal_pins |= pin;
    }

    // Get squares along the linear (orthogonal) pinning line.
    let mut linear_pins: BitBoard = BitBoard::EMPTY;
    for attacker in linear_attackers {
        let pin: BitBoard = get_between(king_square, attacker);
        linear_pins |= pin;
    }

    (diagonal_pins, linear_pins)
}

/// Generates a bitboard mask that restricts legal moves when the king is in check.
///
/// - If the king is in check, the mask includes only the squares between the king and the attacking piece,
///   as well as the square occupied by the checker. This ensures only blocking or capturing moves are considered.
/// - If the king is not in check, the mask allows movement to any square.
#[inline(always)]
fn check_mask<const IN_CHECK: bool>(board: &Board) -> BitBoard {
    if IN_CHECK {
        get_between(board.allied_king().to_square(), board.checkers.to_square()) | board.checkers
    } else {
        BitBoard::FULL
    }
}
