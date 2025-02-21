use std::str::FromStr;

use laura_core::*;

#[test]
fn test_bitboard() {
    let bitboard: BitBoard = BitBoard(2097152);
    assert_eq!(bitboard.to_square(), Square::F3);
    println!("{}", bitboard);
    let bitboard: BitBoard = bitboard.set_square(Square::G6);
    println!("{}", bitboard);
    assert_eq!(bitboard.get_square(Square::G6), true);
    let bitboard: BitBoard = bitboard.set_square(Square::B5);
    assert_eq!(bitboard.count_bits(), 3);
    println!("{}", bitboard);
    let bitboard: BitBoard = bitboard.pop_square(Square::G6);
    assert_eq!(bitboard.count_bits(), 2);
    println!("{}", bitboard);
}

#[test]
fn test_file_from_index() {
    let file: File = File::from_index(4);
    println!("File: {}, Index: {}", file, file.to_index());
    println!("Left: {}, Right: {}", file.left(), file.right())
}

#[test]
fn test_movelist_push() {
    use crate::MoveType;
    use crate::Square;

    let mut list: MoveList = MoveList::default();
    assert_eq!(list.is_empty(), true);

    list.push(Move::new(Square::E2, Square::E3, MoveType::Quiet));
    list.push(Move::new(Square::D7, Square::D5, MoveType::DoublePawn));
    println!("{}", list);
    assert_eq!(list.len(), 2);
}

#[test]
fn test_movelist_iter() {
    use crate::{Board, MoveList};

    let board: Board = Board::default();
    let moves: MoveList = gen_moves::<ALL_MOVES>(&board);
    assert_eq!(moves.len(), 20);

    for mv in moves {
        println!("{}", mv);
    }
}

#[test]
fn test_piece_new() {
    let piece: Piece = Piece::new(PieceType::King, Color::White);
    println!(
        "Char: '{}' Color: {}, Type: {}",
        piece,
        piece.color(),
        piece.piece_type()
    );

    let piece: Option<Piece> = Piece::from_index(12);
    println!("{:?}", piece);
}

#[test]
fn test_piece_from() {
    let c: char = 'N';
    let piece: Piece = Piece::try_from(c).unwrap();
    println!(
        "Char: '{}' Color: {}, Type: {}",
        piece,
        piece.color(),
        piece.piece_type()
    );
}

#[test]
fn test_rank_from_index() {
    let rank: Rank = Rank::from_index(6);
    println!("Rank: {}, Index: {}", rank, rank.to_index());
    println!("Down: {}, Up: {}", rank.down(), rank.up());
}

#[test]
fn test_square() {
    let square: Square = Square::from_str("f5").unwrap();
    assert_eq!(square.file(), File::F);
    assert_eq!(square.rank(), Rank::Five);
    assert_eq!(square.to_index(), 37);
    assert_eq!(square.up(), Square::F6);
    assert_eq!(square.down(), Square::F4);
    assert_eq!(square.left(), Square::E5);
    assert_eq!(square.right(), Square::G5);
    assert_eq!(square.forward(Color::Black), Square::F4);
    assert_eq!(square.backward(Color::Black), Square::F6);
}

#[test]
fn test_castling() {
    let castle_rights: CastleRights = CastleRights::from_str("KQkq").unwrap();
    assert_eq!(castle_rights.has_kingside(Color::White), true);
    assert_eq!(castle_rights.has_queenside(Color::White), true);
    assert_eq!(castle_rights.has_kingside(Color::Black), true);
    assert_eq!(castle_rights.has_queenside(Color::Black), true);
    println!("{}", castle_rights);
    let castle_rights: CastleRights = castle_rights.update(Square::H1, Square::H5);
    let castle_rights: CastleRights = castle_rights.update(Square::E8, Square::E6);
    assert_eq!(castle_rights.has_kingside(Color::White), false);
    assert_eq!(castle_rights.has_queenside(Color::White), true);
    assert_eq!(castle_rights.has_kingside(Color::Black), false);
    assert_eq!(castle_rights.has_queenside(Color::Black), false);
    println!("{}", castle_rights);
}

#[test]
fn test_castling_from_string() {
    let castle_rights: CastleRights = CastleRights::from_str("Kk").unwrap();
    assert_eq!(castle_rights.has_kingside(Color::White), true);
    assert_eq!(castle_rights.has_queenside(Color::White), false);
    assert_eq!(castle_rights.has_kingside(Color::Black), true);
    assert_eq!(castle_rights.has_queenside(Color::Black), false);
    println!("{}", castle_rights);
}

#[test]
fn test_bishop_magic_attacks() {
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
fn test_rook_magic_attacks() {
    let blockers: BitBoard = BitBoard(144115188075921408);
    let bitboard: BitBoard = get_rook_attacks(Square::A8, blockers);
    println!("{}\n{}", blockers, bitboard);
    assert_eq!(bitboard, BitBoard(144397766876004352));

    let blockers: BitBoard = BitBoard(4503600181022721);
    let bitboard: BitBoard = get_rook_attacks(Square::E4, blockers);
    println!("{}\n{}", blockers, bitboard);
    assert_eq!(bitboard, BitBoard(4521261322473472));
}

#[test]
fn test_get_king_attacks() {
    let attack: BitBoard = get_king_attacks(Square::A2);
    assert_eq!(attack, BitBoard(197123));
    println!("{}", attack);
}

#[test]
fn test_get_knight_attacks() {
    let attack: BitBoard = get_knight_attacks(Square::C1);
    assert_eq!(attack, BitBoard(659712));
    println!("{}", attack);
}

#[test]
fn test_pawn_get_attacks() {
    let square: Square = Square::E3;
    let color: Color = Color::White;
    let pawn_attack: BitBoard = get_pawn_attacks(color, square);
    assert_eq!(pawn_attack, BitBoard(671088640));
    println!("Pawn attacks ({}) from E3: {}", color, pawn_attack);
}

#[test]
fn test_bishop_ray() {
    let square: Square = Square::B6;
    let king_ray: BitBoard = get_bishop_rays(square);
    assert_eq!(king_ray, BitBoard(577868148797087808));
    println!("{}", king_ray);
}

#[test]
fn test_rook_ray() {
    let square: Square = Square::B6;
    let king_ray: BitBoard = get_rook_rays(square);
    assert_eq!(king_ray, BitBoard(144956323094725122));
    println!("{}", king_ray);
}

#[test]
fn test_make_move() {
    let board: Board = Board::default();
    assert_eq!(
        board.to_fen(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    );
    println!("{}", board);
    let mv: Move = Move::new(Square::E2, Square::E4, MoveType::DoublePawn);
    let board: Board = board.make_move(mv);
    assert_eq!(
        board.to_fen(),
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
    );
    println!("{}", board);
    let mv: Move = Move::new(Square::C7, Square::C5, MoveType::DoublePawn);
    let board: Board = board.make_move(mv);
    assert_eq!(
        board.to_fen(),
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2"
    );
    println!("{}", board);
    let mv: Move = Move::new(Square::G1, Square::F3, MoveType::Quiet);
    let board: Board = board.make_move(mv);
    assert_eq!(
        board.to_fen(),
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2"
    );
    println!("{}", board);
}

#[test]
fn test_null_move() {
    let board: Board = Board::default();
    println!("{}", board);
    let board: Board = board.null_move();
    println!("{}", board);
}

#[test]
fn test_uci_move() {
    let board: Board = Board::default();
    let board: Board = board.make_uci_move("e2e4").unwrap();
    assert_eq!(
        board.to_fen(),
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
    );
    println!("{}", board);
}

#[test]
fn test_find_move() {
    let board: Board = Board::default();
    let mv: &str = "d2d4";
    println!("{}", board.find_move(mv).unwrap());
}

#[test]
fn test_default_moves() {
    let board: Board = Board::default();
    let move_list: MoveList = gen_moves::<{ ALL_MOVES }>(&board);
    assert_eq!(move_list.len(), 20);
    for mv in move_list {
        println!("{mv} -> {:?}", mv.get_type());
    }
}

#[test]
fn test_quiet_moves() {
    use std::str::FromStr;

    let board: Board =
        Board::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    let move_list: MoveList = gen_moves::<{ QUIET_MOVES }>(&board);
    assert_eq!(move_list.len(), 40);
    println!("{board}");
    for mv in move_list {
        println!("{mv} -> {:?}", mv.get_type());
    }
}

#[test]
fn test_tactical_moves() {
    use std::str::FromStr;

    let board: Board =
        Board::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    let move_list: MoveList = gen_moves::<{ TACTICAL_MOVES }>(&board);
    assert_eq!(move_list.len(), 8);
    println!("{board}");
    for mv in move_list {
        println!("{mv} -> {:?}", mv.get_type());
    }
}

#[test]
fn test_board() {
    let board: Board = Board::new();
    println!("{}", board);
}

#[test]
fn test_board_from_string() {
    let board: Board =
        Board::from_str("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    assert_eq!(board.side(), Color::White);
    assert_eq!(board.enpassant_square, None);
    assert_eq!(board.zobrist(), Zobrist(0x9076b588b1b0450a));
    println!("{}", board);
}

#[test]
fn test_board_default() {
    let board: Board =
        Board::from_str("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let board_default: Board = Board::default();
    println!("{}", board);
    assert_eq!(board, board_default);
    assert_eq!(board.side(), Color::White);
    assert_eq!(board.enpassant_square, None);
    assert_eq!(board.zobrist(), Zobrist(0xc18ae40f70a32d9b));
}
