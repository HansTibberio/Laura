use laura_core::{Board, Move, MoveType, Square};
use std::str::FromStr;

#[test]
fn test_san_castling() {
    let board: Board = Board::from_str("r3k3/8/8/8/8/8/8/4K2R w Kq - 0 1").unwrap();
    let mv: Move = Move::new(Square::E1, Square::G1, MoveType::KingCastle);
    assert_eq!(board.to_san(mv), "O-O");

    let board: Board = Board::from_str("r3k3/8/8/8/8/8/8/5RK1 b q - 0 1").unwrap();
    let mv: Move = Move::new(Square::E8, Square::C8, MoveType::QueenCastle);
    assert_eq!(board.to_san(mv), "O-O-O");
}

#[test]
fn test_san_promotion() {
    let board: Board = Board::from_str("8/P3k3/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let mv: Move = Move::new(Square::A7, Square::A8, MoveType::PromotionQueen);
    assert_eq!(board.to_san(mv), "a8=Q");

    let board: Board = Board::from_str("8/4K3/8/8/8/8/p3k3/8 b - - 0 1").unwrap();
    let mv: Move = Move::new(Square::A2, Square::A1, MoveType::PromotionRook);
    assert_eq!(board.to_san(mv), "a1=R");
}

#[test]
fn test_san_check() {
    let board: Board = Board::from_str("8/k7/7Q/6R1/8/8/8/K7 w - - 0 1").unwrap();
    let mv: Move = Move::new(Square::G5, Square::G7, MoveType::Quiet);
    assert_eq!(board.to_san(mv), "Rg7+");

    let board: Board = Board::from_str("8/k5R1/7Q/8/8/8/8/K7 b - - 0 1").unwrap();
    let mv: Move = Move::new(Square::A7, Square::A8, MoveType::Quiet);
    assert_eq!(board.to_san(mv), "Ka8");
    assert_eq!(board.checkers.count_bits(), 1);

    let board: Board = Board::from_str("k7/6R1/7Q/8/8/8/8/K7 w - - 0 1").unwrap();
    let mv: Move = Move::new(Square::H6, Square::H8, MoveType::Quiet);
    assert_eq!(board.to_san(mv), "Qh8#");
}

#[test]
fn test_san_disambiguation() {
    let board: Board = Board::from_str("2kr3r/8/8/R7/4Q2Q/8/8/R1K4Q w - - 0 1").unwrap();
    let mv: Move = Move::new(Square::A1, Square::A3, MoveType::Quiet);
    assert_eq!(board.to_san(mv), "R1a3");
    let mv: Move = Move::new(Square::H4, Square::E1, MoveType::Quiet);
    assert_eq!(board.to_san(mv), "Qh4e1");

    let board: Board = Board::from_str("2kr3r/8/8/R7/4Q2Q/8/8/R1K4Q b - - 0 1").unwrap();
    let mv: Move = Move::new(Square::D8, Square::F8, MoveType::Quiet);
    assert_eq!(board.to_san(mv), "Rdf8");
}

#[test]
fn test_san_promotion_capture() {
    let board: Board = Board::from_str("1r6/P3k3/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let mv: Move = Move::new(Square::A7, Square::B8, MoveType::CapPromoQueen);
    assert_eq!(board.to_san(mv), "axb8=Q");
}

#[test]
fn test_san_pawn_capture() {
    let board: Board = Board::from_str("4k3/8/2p5/3P4/8/8/8/4K3 w - - 0 1").unwrap();
    let mv: Move = Move::new(Square::D5, Square::C6, MoveType::Capture);
    assert_eq!(board.to_san(mv), "dxc6");
}

#[test]
fn test_san_en_passant_checks() {
    let board: Board = Board::from_str("8/3k4/8/2pP4/8/8/8/4K3 w - c6 0 1").unwrap();
    let mv: Move = Move::new(Square::D5, Square::C6, MoveType::EnPassant);
    assert_eq!(board.to_san(mv), "dxc6+");
}

#[test]
fn test_san_promotion_mates() {
    let board: Board = Board::from_str("1r2k3/P6R/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let mv: Move = Move::new(Square::A7, Square::B8, MoveType::CapPromoQueen);
    assert_eq!(board.to_san(mv), "axb8=Q#");
}

#[test]
fn test_san_ilegal_move() {
    let board: Board = Board::default();
    let mv: Move = Move::new(Square::E2, Square::E4, MoveType::DoublePawn);
    assert_ne!(board.to_san(mv), "Na1xc2#");
    assert_eq!(board.to_san(mv), "e4");
}
