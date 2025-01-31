# <div align="center"> Laura-Core</div>

<div  align="center"> 

[![License][license-badge]][license-link]

</div>

**Laura Core** a fast and efficient move generator for chess engines using bitboards.

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

### **Initialize a board from a FEN string**

```rust
use std::str::FromStr;
use laura_core::Board;

fn main() {
    let fen: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board: Board = Board::from_str(fen).unwrap();

    println!("{}", board);
}
```

### **Generate all legal moves**
```rust 
use laura_core::{Board, MoveList};

fn main() {
	let board: Board = Board::default();
    let moves: MoveList = board.gen_moves::<true>();
    
    for mv in moves {
        println!("{}", mv);
    }
}
```

### **Execute moves**
```rust
use laura_core::Board;

fn main() {
    let board: Board = Board::default();

    let new: Board = board.make_uci_move("e2e4").unwrap();
    println!("{}", new);
}
```

## **License**

This project is licensed under **GPLv3**. See the [LICENSE][license-link] file for details.

[license-link]:https://github.com/hanstibberio/Laura/blob/master/LICENSE

[license-badge]:https://img.shields.io/github/license/hanstibberio/laura?style=for-the-badge&label=license&color=success