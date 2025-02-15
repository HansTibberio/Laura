# <div align="center"> Laura-Core</div>

<div  align="center"> 

[![License][license-badge]][license-link]

</div>

**Laura-Core** a fast and efficient move generator for chess engines.

## Features
- **Bitboards** for efficient board representation.  
- **Zobrist Hashing** 
- **Black Magic Bitboards** for rapid move generation of rooks, bishops, and queens.  
- **PEXT Bitboards** as an alternative for efficient sliding piece move generation.  
- **Full legal or only tactical move generation** depending on requirements.  
- **FEN support**: Initialize the board from a FEN string.  
- **Move execution** to update the board state dynamically.  
- **Null move support** for search optimizations like quiescence search.  
- **UCI move execution**: Apply moves directly from a UCI-compliant string.

## **Usage**

### **Seting up the initial board**

```rust
use laura_core::Board;

fn main() {
    let board: Board = Board::default();
    assert_eq!(board.to_fen(), "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
}
```

### **Initialize a board from a FEN string**

You can create a `Board` from a FEN (Forsyth-Edwards Notation) string using the `FromStr` trait:

```rust
use std::str::FromStr;
use laura_core::Board;

fn main() {
    let fen: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let board: Board = Board::from_str(fen).unwrap();
    assert_eq!(board, Board::kiwipete());
}
```

### **Generate all legal moves**

To generate moves from a given position, use the gen_moves function along with one of the predefined constants:

- ALL_MOVES: Generates all legal moves.
- QUIET_MOVES: Generates only quiet moves (non-capturing moves).
- TACTICAL_MOVES: Generates tactical moves (captures and queen promotions).

**Example: Generating all legal moves**

This example starts from the default position and generate all legal moves.

```rust 
use laura_core::{gen_moves, Board, MoveList, ALL_MOVES};

fn main() {
    let board: Board = Board::default();
    let moves: MoveList = gen_moves::<ALL_MOVES>(&board);
    assert_eq!(moves.len(), 20);
}
```

**Example: Generating only quiet moves**

```rust 
use laura_core::{gen_moves, Board, MoveList, QUIET_MOVES};

fn main() {
    let board: Board = Board::kiwipete();
    let moves: MoveList = gen_moves::<QUIET_MOVES>(&board);
    assert_eq!(moves.len(), 40);
}
```

**Example: Generating only tactical moves**

```rust 
use laura_core::{gen_moves, Board, MoveList, TACTICAL_MOVES};

fn main() {
    let board: Board = Board::kiwipete();
    let moves: MoveList = gen_moves::<TACTICAL_MOVES>(&board);
    assert_eq!(moves.len(), 8);
}
```

### **Execute moves**

You can apply a move to the board using UCI (Universal Chess Interface) notation:

```rust
use laura_core::Board;

fn main() {
    let board: Board = Board::default();
    let new: Board = board.make_uci_move("e2e4").unwrap();
    assert_eq!(new.to_fen(), "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
}
```

This executes the move e2e4 (pawn to e4) and assert the updated board position.

## **License**

This project is licensed under **GPLv3**. See the [LICENSE][license-link] file for details.

[license-link]:https://github.com/hanstibberio/Laura/blob/master/LICENSE

[license-badge]:https://img.shields.io/github/license/hanstibberio/laura?style=for-the-badge&label=license&color=success
