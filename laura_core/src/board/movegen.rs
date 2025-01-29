use crate::castle_rights::*;
use crate::gen::king::get_king_attacks;
use crate::gen::knight::get_knight_attacks;
use crate::gen::pawn::get_pawn_attacks;
use crate::gen::rays::{bishop_rays, get_between, rook_rays};

#[cfg(not(feature = "bmi2"))]
use crate::gen::black_magics::{get_bishop_attacks, get_rook_attacks};
#[cfg(feature = "bmi2")]
use crate::gen::pext::{get_bishop_attacks, get_rook_attacks};

use crate::{BitBoard, Board, Move, MoveList, MoveType, Square};

// This file is responsible for generating legal moves for pieces, which is a core
// part of the chess engine's functionality. It works with bitboards and evaluates
// possible moves based on the current game state.
//
// This version of the move generation code has been adapted from the Chess engine Carp,
// with some optimizations and bug fixes aimed at improving performance and accuracy.
//
// This file contains code licensed under GPLv3.
// Source: https://github.com/dede1751/carp/blob/main/chess/src/movegen/gen.rs

/// Indicates that all moves, including both tactical and quiet moves, should be generated.
pub const ALL_MOVES: bool = true;

impl Board {
    /// Generates a list of legal moves for the board, with the option to include all moves or only tactical moves.
    /// The function checks for multiple attackers on the king; if there is more than one attacker, only king moves are generated.
    ///
    /// ### Type Parameter
    /// - `ALL_MOVES`: If `true`, all moves are generated; if `false`, only tactical moves (captures and threats) are included.
    ///
    /// ### Returns
    /// A `MoveList` containing the generated moves for the board.
    #[inline(always)]
    pub fn gen_moves<const ALL_MOVES: bool>(&self) -> MoveList {
        let mut move_list: MoveList = MoveList::default();

        self.gen_king_moves::<ALL_MOVES>(&mut move_list);

        let attackers: u32 = self.checkers.count_bits();

        // If the king is under attack by more than one piece, only king moves are generated
        // to avoid exposing the king to check.
        if attackers > 1 {
            return move_list;
        }

        // If there's exactly one attacker, generate moves to get out of check
        let check_mask: BitBoard = if attackers == 1 {
            get_between(self.allied_king().to_square(), self.checkers.to_square()) | self.checkers
        } else {
            BitBoard::FULL // No check, so no restrictions on the moves
        };

        // Get the diagonal and linear pins on the board
        let (diagonal_pins, linear_pins) = self.pinners();
        let all_pins: BitBoard = diagonal_pins | linear_pins;

        self.gen_pawn_attacks(check_mask, diagonal_pins, linear_pins, &mut move_list);
        self.gen_pawn_moves::<ALL_MOVES>(check_mask, diagonal_pins, linear_pins, &mut move_list);
        self.gen_knights_moves::<ALL_MOVES>(check_mask, all_pins, &mut move_list);
        self.gen_bishops_moves::<ALL_MOVES>(check_mask, diagonal_pins, linear_pins, &mut move_list);
        self.gen_rooks_moves::<ALL_MOVES>(check_mask, linear_pins, diagonal_pins, &mut move_list);

        // Return the list of generated moves
        move_list
    }

    /// Generates moves for the king piece, considering both capture and quiet moves based on the `ALL_MOVES` parameter.
    /// This function checks potential attack and safe squares for the king, ensuring the move does not put the king in check.
    #[inline(always)]
    pub fn gen_king_moves<const ALL_MOVES: bool>(&self, move_list: &mut MoveList) {
        let king_bitboard: BitBoard = self.allied_king();
        let src: Square = king_bitboard.to_square();
        let attacks: BitBoard = get_king_attacks(src);
        let blockers: BitBoard = self.combined_bitboard().pop_square(src);

        let enemy_targets: BitBoard = attacks & self.enemy_presence();
        let quiet_targets: BitBoard = attacks & !self.combined_bitboard();

        // Process capture moves for the king: iterate over the enemy target squares
        for dest in enemy_targets {
            if !self.attacked_square(dest, blockers) {
                move_list.push(Move::new(src, dest, MoveType::Capture));
            }
        }

        if ALL_MOVES {
            // Process quiet moves for the king: iterate over the quiet target squares
            for dest in quiet_targets {
                if !self.attacked_square(dest, blockers) {
                    move_list.push(Move::new(src, dest, MoveType::Quiet));
                }
            }

            // If the king is not in check, consider castling as a move
            if self.checkers.is_empty() {
                if self.castling.has_kingside(self.side) {
                    self.gen_castle_moves::<KING_SIDE>(move_list);
                }
                if self.castling.has_queenside(self.side) {
                    self.gen_castle_moves::<QUEEN_SIDE>(move_list);
                }
            }
        }
    }

    /// Generates moves for the knight piece, considering both capture and quiet moves based
    /// on the `ALL_MOVES` parameter.
    ///
    /// The function takes into account the knight's possible attacks and checks if the move
    /// is within the bounds of the board and valid based on the given check and pin masks.
    #[inline(always)]
    fn gen_knights_moves<const ALL_MOVES: bool>(
        &self,
        check_mask: BitBoard,
        pin_mask: BitBoard,
        move_list: &mut MoveList,
    ) {
        // Get the knight's positions, excluding pinned knights (they can't move)
        let knights: BitBoard = self.allied_knights() & !pin_mask;

        for src in knights {
            let attacks: BitBoard = get_knight_attacks(src) & check_mask;

            // Process capture moves for the knight
            for dest in attacks & self.enemy_presence() {
                move_list.push(Move::new(src, dest, MoveType::Capture));
            }

            // If `ALL_MOVES` is true, process quiet (non-capture) moves for the knight
            if ALL_MOVES {
                for dest in attacks & !self.combined_bitboard() {
                    move_list.push(Move::new(src, dest, MoveType::Quiet));
                }
            }
        }
    }

    /// Generates moves for the bishop and queen pieces, considering both capture and quiet moves
    /// based on the `ALL_MOVES` parameter.
    ///
    /// The function computes the possible diagonal attacks for the bishops and queens,
    /// ensuring that moves do not land on blocked squares or put the king in check.
    #[inline(always)]
    fn gen_bishops_moves<const ALL_MOVES: bool>(
        &self,
        check_mask: BitBoard,
        slide_mask: BitBoard,
        pinned_mask: BitBoard,
        move_list: &mut MoveList,
    ) {
        // Combine the bishops and queens bitboards, excluding pinned pieces
        let bishops: BitBoard = (self.allied_bishops() | self.allied_queens()) & !pinned_mask;
        let blockers: BitBoard = self.combined_bitboard();

        for src in bishops {
            let mut attacks: BitBoard = get_bishop_attacks(src, blockers) & check_mask;

            // Apply the slide mask to restrict the bishop's (or queen's) movement
            // Even though the piece is pinned, it can still move along the diagonal if not obstructed
            if slide_mask.get_square(src) {
                attacks &= slide_mask;
            }

            // Process capture moves for the bishop/queen
            for dest in attacks & self.enemy_presence() {
                move_list.push(Move::new(src, dest, MoveType::Capture));
            }

            // If `ALL_MOVES` is true, process quiet (non-capture) moves for the bishop/queen
            if ALL_MOVES {
                for dest in attacks & !self.combined_bitboard() {
                    move_list.push(Move::new(src, dest, MoveType::Quiet));
                }
            }
        }
    }

    /// Generates rook moves (and queen moves, as they share movement logic) for the board.
    /// 
    /// This function checks for sliding attacks, considering blockers and pinned pieces,
    /// and adds legal moves to the `move_list` based on the `ALL_MOVES` parameter.
    #[inline(always)]
    fn gen_rooks_moves<const ALL_MOVES: bool>(
        &self,
        check_mask: BitBoard,
        slide_mask: BitBoard,
        pinned_mask: BitBoard,
        move_list: &mut MoveList,
    ) {
        // Combine the rooks and queens and filter out pinned pieces that cannot move freely
        let rooks: BitBoard = (self.allied_rooks() | self.allied_queens()) & !pinned_mask;
        let blockers: BitBoard = self.combined_bitboard();

        for src in rooks {
            let mut attacks: BitBoard = get_rook_attacks(src, blockers) & check_mask;

            // Apply the slide mask to restrict rook movement to valid squares, if applicable
            // Even though the piece is pinned, it can still move along the file or rank if not obstructed
            if slide_mask.get_square(src) {
                attacks &= slide_mask;
            }

            // Iterate through the squares and add capture moves
            for dest in attacks & self.enemy_presence() {
                move_list.push(Move::new(src, dest, MoveType::Capture));
            }

             // If `ALL_MOVES` is enabled, add quiet moves as well
            if ALL_MOVES {
                for dest in attacks & !self.combined_bitboard() {
                    move_list.push(Move::new(src, dest, MoveType::Quiet));
                }
            }
        }
    }

    #[inline(always)]
    fn gen_castle_moves<const KING_SIDE: usize>(&self, move_list: &mut MoveList) {
        let side: usize = self.side as usize;
        let src: Square = SOURCE[side];
        let dest: Square = DESTINATION[KING_SIDE][side];
        let move_type: MoveType = CASTLE_TYPE[KING_SIDE];

        if (self.combined_bitboard() & PRESENCE[KING_SIDE][side]).is_empty()
            && !self.attacked_square(MEDIUM[KING_SIDE][side], self.combined_bitboard())
            && !self.attacked_square(dest, self.combined_bitboard())
        {
            move_list.push(Move::new(src, dest, move_type));
        }
    }

    #[inline(always)]
    fn gen_pawn_moves<const ALL_MOVES: bool>(
        &self,
        check_mask: BitBoard,
        diagonal_pins: BitBoard,
        linear_pins: BitBoard,
        move_list: &mut MoveList,
    ) {
        let pawns: BitBoard = self.allied_pawns() & !diagonal_pins;
        let empty: BitBoard = !self.combined_bitboard();
        let target: BitBoard = empty & check_mask;
        let single_push: BitBoard = pawns & target.forward(!self.side);
        let promotion_mask: BitBoard =
            single_push & BitBoard::PROMOTION_RANKS[self.side as usize];

        for src in promotion_mask {
            let dest: Square = src.forward(self.side);
            if !linear_pins.get_square(src) || linear_pins.get_square(dest) {
                move_list.push(Move::new(src, dest, MoveType::PromotionQueen));

                if ALL_MOVES {
                    move_list.push(Move::new(src, dest, MoveType::PromotionKnight));
                    move_list.push(Move::new(src, dest, MoveType::PromotionRook));
                    move_list.push(Move::new(src, dest, MoveType::PromotionBishop));
                }
            }
        }

        if !ALL_MOVES {
            return;
        }

        let quiet_mask: BitBoard = single_push & !BitBoard::PROMOTION_RANKS[self.side as usize];

        for src in quiet_mask {
            let dest: Square = src.forward(self.side);
            if !linear_pins.get_square(src) || linear_pins.get_square(dest) {
                move_list.push(Move::new(src, dest, MoveType::Quiet));
            }
        }

        let double_push: BitBoard = pawns
            & (empty & target.forward(!self.side)).forward(!self.side)
            & BitBoard::PAWN_START[self.side as usize];

        for src in double_push {
            let dest: Square = src.forward(self.side).forward(self.side);
            if !linear_pins.get_square(src) || linear_pins.get_square(dest) {
                move_list.push(Move::new(src, dest, MoveType::DoublePawn));
            }
        }
    }

    #[inline(always)]
    fn gen_pawn_attacks(
        &self,
        check_mask: BitBoard,
        diagonal_pins: BitBoard,
        linear_pins: BitBoard,
        move_list: &mut MoveList,
    ) {
        let pawns: BitBoard = self.allied_pawns() & !linear_pins;
        let targets: BitBoard = self.enemy_presence() & check_mask;
        let king_square: Square = self.allied_king().to_square();

        for src in pawns & BitBoard::PROMOTION_RANKS[self.side as usize] {
            let mut attacks: BitBoard = get_pawn_attacks(self.side, src);

            if diagonal_pins.get_square(src) {
                attacks &= diagonal_pins
            }

            for dest in attacks & targets {
                move_list.push(Move::new(src, dest, MoveType::CapPromoQueen));
                move_list.push(Move::new(src, dest, MoveType::CapPromoRook));
                move_list.push(Move::new(src, dest, MoveType::CapPromoBishop));
                move_list.push(Move::new(src, dest, MoveType::CapPromoKnight));
            }
        }

        // En Pasant
        if let Some(en_passant) = self.enpassant_square {
            let dest: Square = en_passant;
            let victim: Square = en_passant.forward(!self.side);

            for src in pawns & get_pawn_attacks(!self.side, dest) {
                let blockers: BitBoard =
                    self.combined_bitboard() ^ victim.to_bitboard() ^ src.to_bitboard()
                        | dest.to_bitboard();

                let king_ray: bool =
                    !(rook_rays(king_square) & self.enemy_queen_rooks()).is_empty();
                if king_ray
                    && !(get_rook_attacks(king_square, blockers) & self.enemy_queen_rooks())
                        .is_empty()
                {
                    continue;
                }

                let king_ray: bool =
                    !(bishop_rays(king_square) & self.enemy_queen_bishops()).is_empty();
                if king_ray
                    && !(get_bishop_attacks(king_square, blockers) & self.enemy_queen_bishops())
                        .is_empty()
                {
                    continue;
                }

                move_list.push(Move::new(src, dest, MoveType::EnPassant));
            }
        }

        for src in pawns & !BitBoard::PROMOTION_RANKS[self.side as usize] {
            let mut attacks: BitBoard = get_pawn_attacks(self.side, src);

            if diagonal_pins.get_square(src) {
                attacks &= diagonal_pins
            }

            for dest in attacks & targets {
                move_list.push(Move::new(src, dest, MoveType::Capture));
            }
        }
    }

    #[inline(always)]
    fn pinners(&self) -> (BitBoard, BitBoard) {
        let king_square: Square = self.allied_king().to_square();
        let blockers_mask: BitBoard = self.combined_bitboard();

        let diagonal_pinned: BitBoard =
            get_bishop_attacks(king_square, blockers_mask) & self.allied_presence();
        let linnear_pinned: BitBoard =
            get_rook_attacks(king_square, blockers_mask) & self.allied_presence();

        let diagonal_pinned_removed: BitBoard = blockers_mask & !diagonal_pinned;
        let linear_pinned_removed: BitBoard = blockers_mask & !linnear_pinned;

        let diagonal_attackers: BitBoard =
            get_bishop_attacks(king_square, diagonal_pinned_removed) & self.enemy_queen_bishops();
        let linear_attackers: BitBoard =
            get_rook_attacks(king_square, linear_pinned_removed) & self.enemy_queen_rooks();

        let mut diagonal_pins: BitBoard = BitBoard::EMPTY;
        for attacker in diagonal_attackers {
            let pin: BitBoard = get_between(king_square, attacker);
            diagonal_pins |= pin;
        }

        let mut linear_pins: BitBoard = BitBoard::EMPTY;
        for attacker in linear_attackers {
            let pin: BitBoard = get_between(king_square, attacker);
            linear_pins |= pin;
        }

        (diagonal_pins, linear_pins)
    }
}

#[test]
fn test_default_moves() {
    let board: Board = Board::default();
    let move_list: MoveList = board.gen_moves::<true>();
    println!("{}\n{}", board, move_list);
}

#[test]
fn test_gen_moves() {
    use std::str::FromStr;

    let board: Board = Board::from_str("8/4R3/1Q1n4/1P3P2/b5Np/2K3q1/5p2/k2r4 w - - 0 1").unwrap();
    let move_list: MoveList = board.gen_moves::<true>();
    println!("{}\n{}", board, move_list);
}

#[test]
fn test_castle_moves() {
    use std::str::FromStr;

    let board: Board =
        Board::from_str("r1n1k2r/p2ppp1p/8/8/8/8/P2PPP1P/R3KN1R b KQkq - 0 1").unwrap();
    let mut move_list: MoveList = MoveList::default();
    board.gen_castle_moves::<KING_SIDE>(&mut move_list);
    println!("{}\n{}", board, move_list);
}

#[test]
fn test_pin_moves() {
    use std::str::FromStr;

    let board: Board = Board::from_str("R2bk3/5p2/4r1B1/1Q6/8/4Q3/4R3/2K5 b - - 0 1").unwrap();
    println!("{}", board);

    let (diagonals, linears) = board.pinners();
    let pinned: BitBoard = (diagonals | linears) & board.allied_presence();
    println!(
        "Pinned Pieces: {}\nDiagonals: {}\nLinears: {}",
        pinned, diagonals, linears
    );

    assert!(pinned.get_square(Square::F7));
    assert!(pinned.get_square(Square::E6));
    assert!(pinned.get_square(Square::D8));
    assert!(diagonals.get_square(Square::G6));
    assert!(linears.get_square(Square::C8));
    assert!(linears.get_square(Square::E3));
    assert!(!linears.get_square(Square::E2));
}
