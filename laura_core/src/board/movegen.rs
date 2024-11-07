use super::board::Board;

use crate::color::Color;
use crate::move_list::MoveList;
use crate::bitboard::BitBoard;
use crate::moves::{Move, MoveType};
use crate::piece::PieceType;
use crate::gen::magics;
use crate::gen::king;
use crate::gen::knight;
use crate::gen::pawn;
use crate::square::Square;

/// Retrieves the attack bitboard for a bishop located on a specific square, considering blockers.
/// 
/// The bitboard returned represents all potential attack squares for the bishop, based on
/// precomputed magic bitboard tables for optimal performance.
pub fn get_bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    magics::get_bishop_attacks(square, blockers)
}

/// Retrieves the attack bitboard for a rook located on a specific square, considering blockers.
/// 
/// The bitboard returned represents all potential attack squares for the rook, using precomputed
/// magic bitboard tables for efficiency.
pub fn get_rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    magics::get_rook_attacks(square, blockers)
}

/// Retrieves the attack bitboard for a king located on a specific square.
/// This function returns a bitboard representing all adjacent squares the king can move to or attack.
pub fn get_king_attacks(square: Square) -> BitBoard {
    king::get_king_attacks(square)
}

/// Retrieves the attack bitboard for a knight located on a specific square.
/// This function returns a bitboard representing all potential attack or move squares for the knight,
/// based on the knight's unique "L"-shaped movement pattern.
pub fn get_knight_attacks(square: Square) -> BitBoard {
    knight::get_knight_attacks(square)
}

/// Retrieves the attack bitboard for a pawn located on a specific square, considering the pawn's color.
/// This function returns the attack bitboard for the pawn, with the attack direction depending
/// on whether the pawn is white or black.
pub fn get_pawn_attacks(color: Color, square: Square) -> BitBoard {
    match color {
        Color::White => {
            pawn::get_pawn_attacks(Color::White, square)
        }
        Color::Black => {
            pawn::get_pawn_attacks(Color::Black, square)
        }
    }
}

/// Retrieves the attack bitboard for a given piece type on a specific square, considering blockers if necessary.
/// For pieces like bishops, rooks, and queens, blockers are taken into account to limit the attack range.
/// The queen's attack is a combination of bishop and rook attacks.
pub fn get_attack_by_piece(piece_type: PieceType, square: Square, blockers: BitBoard) -> BitBoard {
    match piece_type {
        PieceType::Knight => knight::get_knight_attacks(square),
        PieceType::Bishop => magics::get_bishop_attacks(square, blockers),
        PieceType::Rook => magics::get_rook_attacks(square, blockers),
        PieceType::Queen => magics::get_bishop_attacks(square, blockers) | magics::get_rook_attacks(square, blockers),
        PieceType::King => king::get_king_attacks(square),
        PieceType::Pawn => todo!()
    }
}

/// Indicates that all moves, including both tactical and quiet moves, should be generated.
pub const ALL: bool = true;

/// Indicates that only tactical moves (captures and threats) should be generated.
pub const TACTICALS: bool = false;

impl Board {
    
    /// Generates a list of legal moves for the board, with the option to include all moves or only tactical moves.
    /// The function checks for multiple attackers on the king; if there is more than one attacker, only king moves are generated.
    /// 
    /// ### Type Parameter
    /// - `ALL`: If `true`, all moves are generated; if `false`, only tactical moves are included.
    ///
    /// ### Returns
    /// A `MoveList` containing the generated moves for the board.
    pub fn gen_moves<const ALL: bool>(&self) -> MoveList {
        let mut move_list: MoveList = MoveList::default();

        self.gen_king_moves::<ALL>(&mut move_list);

        let attackers: u32 = self.checkers.count_bits();
        if attackers > 1 {
            return move_list;
        }

        todo!();

        move_list
    }

    /// Generates moves for the king piece, considering both capture and quiet moves based on the `ALL` parameter.
    /// This function checks potential attack and safe squares for the king, ensuring the move does not put the king in check.
    pub fn gen_king_moves<const ALL: bool>(&self, move_list: &mut MoveList){
        let king_bitboard: BitBoard = self.allied_king();
        let src: Square = king_bitboard.to_square();
        let attacks: BitBoard = get_king_attacks(src);
        let blockers: BitBoard = self.combined_bitboard().pop_square(src);
        
        for dest in attacks & self.enemy_presence() {
            if !self.attacked_square(dest, blockers) {
                move_list.push(Move::new(src, dest, MoveType::Capture));
            }
        }

        if ALL {
            for dest in attacks & !self.combined_bitboard() {
                if !self.attacked_square(dest, blockers) {
                    move_list.push(Move::new(src, dest, MoveType::Quiet));
                }
            }
        }
    }

    /// Identifies pieces that are pinning allied pieces to the king along diagonal or orthogonal lines.
    /// This function uses bishop and rook attack bitboards to detect potential pins by enemy bishops, rooks, or queens.
    /// Currently a placeholder function; intended for future functionality related to move validation.
    fn pinners(&self) {
        let king_square: Square = self.allied_king().to_square();
        let presence_bitboard: BitBoard = self.combined_bitboard();

        let allied_diagonal_pins: BitBoard = get_bishop_attacks(king_square, presence_bitboard) & self.allied_presence();
        let allied_orthogonal_pins: BitBoard = get_rook_attacks(king_square, presence_bitboard) & self.allied_presence();

        let diagonal_blockers_removed: BitBoard = presence_bitboard & !allied_diagonal_pins;
        let orthogonal_blockers_removed: BitBoard = presence_bitboard & !allied_orthogonal_pins;

        let diagonal_attackers: BitBoard = get_bishop_attacks(king_square, diagonal_blockers_removed) & self.enemy_queen_bishops();
        println!("{}", diagonal_attackers);
        let orthogonal_attackers: BitBoard = get_rook_attacks(king_square, orthogonal_blockers_removed) & self.enemy_queen_rooks();
        println!("{}", orthogonal_attackers);

        todo!()
    }

}

#[test]
fn test_gen_king_moves(){
    use std::str::FromStr;

    let board: Board = Board::from_str("8/4R3/1Q1n4/1P3P2/b5Np/2K3q1/5p2/k2r4 w - - 0 1").unwrap();
    let mut move_list: MoveList = MoveList::default();
    board.gen_king_moves::<ALL>(&mut move_list);
    println!("{}\n{}", board, move_list);
    board.pinners();
}