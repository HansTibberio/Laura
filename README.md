<div  align="center">

![logo](logo.png)

</div>

# <div align="center">Laura</div>

<div align="center">

[![Build][build-badge]][build-link]
[![Release][release-badge]][release-link]
[![License][license-badge]][license-link]

</div>

**Laura** is a multi-threaded, [UCI][uci-link]-compatible chess engine written in Rust, designed with a focus on speed, modularity, and tactical strength.

The engine is built on top of [laura_core][laura_core-link], a dedicated high-performance move generation library developed as a separate project.

## Installation & Compilation

You have two main options to install and run **Laura**:

### 1. Compile from Source

To build **Laura** optimized for your machine, ensure you have [Rust][rust-link] installed, then clone the repository:

``` bash
git clone https://github.com/HansTibberio/Laura.git
cd Laura
```

Linux/macOS/Git Bash/WSL

``` bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

Windows PowerShell

``` bash
$env:RUSTFLAGS="-C target-cpu=native"
cargo build --release
```

For even better performance on supported CPUs, enable the `bmi2` feature:

``` bash
cargo build --release --features bmi2
```

### 2. Download Precompiled Binary

You can also download a precompiled binary from the [Releases][release-link] section. Just choose the version that matches your system and run the binary directly.

## Features

### Board Representation & Move Generation

-   High-performance legal move generation via **laura_core** (Black Magic / PEXT bitboards)
-   Staged move generation (tactical moves first, quiet moves only when needed)

### Search

-   Fail-soft negamax with alpha-beta pruning
-   Principal Variation Search (PVS)
-   Iterative deepening
-   Aspiration windows with progressive widening
-   Quiescence search (with check evasions)
-   Check extensions
-   Singular Extensions:
    -   Double extensions (limited per search path)
    -   Multi-cut pruning
    -   Negative extensions
-   Internal Iterative Reductions (IIR)
-   Mate distance pruning
-   Search stack with per-ply static evaluation and move history
-   *Improving* heuristic (static eval compared against 2 and 4 plies back)
-   TT-adjusted static evaluation
-   Draw detection: fifty-move rule, repetitions and insufficient material

### Pruning & Reductions

-   Reverse Futility Pruning (RFP, *improving*-aware)
-   Razoring
-   Null Move Pruning (NMP, depth-adaptive reduction with zugzwang guard)
-   Futility Pruning
-   Late Move Pruning (LMP)
-   History Leaf Pruning
-   SEE pruning for quiet moves and captures
-   Late Move Reductions (LMR), logarithmic table adjusted by:
    -   PV / cut-node status
    -   *Improving* flag
    -   Checks given
    -   Quiet history score
-   Quiescence search pruning:
    -   Delta pruning
    -   SEE pruning

### Move Ordering

-   Staged move picker:
    -   TT move
    -   Good captures (MVV + capture history, lazy SEE)
    -   Killer moves (2 slots)
    -   Countermove
    -   Quiets (butterfly history + continuation history)
    -   Bad captures
-   History heuristics with gravity:
    -   Butterfly history
    -   Capture history
    -   Continuation history (1, 2 and 4 plies)
-   Static Exchange Evaluation (SEE)

### Evaluation

-   Hand-crafted, tapered evaluation (midgame / endgame phase interpolation)
-   Material and piece-square tables
-   Pawn structure: passed, isolated, doubled, connected/phalanx and central pawns
-   Piece mobility (knights, bishops, rooks, queens)
-   Outposts (knights, bishops, rooks)
-   Bishop pair
-   Rooks on open / semi-open files and on the 7th rank
-   King safety: pawn shelter, open files and pawn storms
-   Tempo bonus

### Transposition Table

-   Lockless, shared across threads
-   32-byte buckets with 3 compact 10-byte entries
-   Age- and depth-aware replacement policy
-   Stores static evaluation and PV-node flag
-   Best move preserved on fail-low updates
-   Memory prefetching
-   Multithreaded clearing

### Parallelism & Time Management

-   Lazy SMP (multithreaded search) with staggered start depths
-   Best-move selection by voting among threads at the deepest completed iteration
-   Soft / hard time limits with move overhead
-   Supports `depth`, `nodes`, `movetime`, `wtime/btime/winc/binc`, `movestogo` and `infinite`

### UCI Interface

-   Options: `Hash` (up to 1 TB) and `Threads` (up to 512)
-   Extra commands: `perft`, `dperft` (divided perft), `eval`, `print`

## License

This project is licensed under **GPLv3**. See the [LICENSE][license-link] file for details.

[build-link]:https://github.com/HansTibberio/Laura/actions/workflows/build.yml
[build-badge]:https://img.shields.io/github/actions/workflow/status/HansTibberio/Laura/build.yml
[license-link]:https://github.com/hanstibberio/Laura/blob/master/LICENSE
[license-badge]:https://img.shields.io/github/license/hanstibberio/laura?label=license&color=success
[release-link]:https://github.com/HansTibberio/Laura/releases/latest
[release-badge]:https://img.shields.io/github/v/release/HansTibberio/Laura?label=official%20release

[uci-link]:https://en.wikipedia.org/wiki/Universal_Chess_Interface
[rust-link]:https://www.rust-lang.org/
[laura_core-link]:https://github.com/HansTibberio/laura_core
