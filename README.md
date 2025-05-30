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

The project is divided into two main components:

- **`laura_core`** – A fast and efficient legal [move generator][laura-link] for chess engines.
- **`laura_engine`** – The UCI-compatible chess engine currently under active development.

## Installation & Compilation

You have two main options to install and run **Laura**:

### 1. Compile from Source

To build **Laura** optimized for your machine, ensure you have [Rust][rust-link] installed, then run:

``` bash
git clone https://github.com/HansTibberio/Laura.git
cd Laura
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

For even better performance on supported CPUs, enable the `bmi2` feature:

``` bash
RUSTFLAGS="-C target-cpu=native" cargo build --release --features bmi2
```

### 2. Download Precompiled Binary

You can also download a precompiled binary from the [Releases][release-link] section. Just choose the version that matches your system and run the binary directly.

## Features

-   Hand-crafted static evaluation function
-   Super-fast legal move generation
-   Fail-soft negamax with alpha-beta pruning
-   Iterative deepening
-   Aspiration windows
-   Principal variation search (PVS)
-   Lockless transposition table
-   Lazy SMP (multithreaded search)
-   Advanced move ordering:
    -   Killer heuristic
    -   MVV/LVA (Most Valuable Victim / Least Valuable Attacker)
-   Mate distance pruning
-   Internal iterative reductions (IIR)
-   Quiescence search

## License

This project is licensed under **GPLv3**. See the [LICENSE][license-link] file for details.

[build-link]:https://github.com/HansTibberio/Laura/actions/workflows/build.yml
[build-badge]:https://img.shields.io/github/actions/workflow/status/HansTibberio/Laura/build.yml?style=for-the-badge
[license-link]:https://github.com/hanstibberio/Laura/blob/master/LICENSE
[license-badge]:https://img.shields.io/github/license/hanstibberio/laura?style=for-the-badge&label=license&color=success
[release-link]:https://github.com/HansTibberio/Laura/releases/latest
[release-badge]:https://img.shields.io/github/v/release/HansTibberio/Laura?label=official%20release&style=for-the-badge

[uci-link]:https://en.wikipedia.org/wiki/Universal_Chess_Interface
[rust-link]:https://www.rust-lang.org/
[laura-link]:https://github.com/HansTibberio/Laura/tree/master/laura_core
