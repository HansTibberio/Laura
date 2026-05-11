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

-   Hand-crafted static evaluation function
-   High-performance legal move generation via **laura_core**
-   Fail-soft negamax with alpha-beta pruning
-   Iterative deepening
-   Aspiration windows
-   Reverse Futility Pruning (RFP)
-   Null Move Pruning (NMP)
-   Futility Pruning
-   History Leaf Pruning
-   Late Move Pruning (LMP)
-   Principal variation search (PVS)
-   Late Move Reductions (LMR, basic implementation)
-   Lockless transposition table
-   Lazy SMP (multithreaded search)
-   Enhanced move ordering:
    -   TT move
    -   Good captures
    -   Killer moves
    -   Quiets (ordered via history heuristic table)
    -   Bad captures
-   Static Exchange Evaluation (SEE)
-   Mate distance pruning
-   Internal iterative reductions (IIR)
-   Quiescence search

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
