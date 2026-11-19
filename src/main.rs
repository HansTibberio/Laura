/*
    Laura: A multi-threaded UCI chess engine written in Rust.

    Copyright (C) 2024-2026 HansTibberio <hanstiberio@proton.me>

    Laura is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Laura is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Laura. If not, see <https://www.gnu.org/licenses/>.
*/

// src/main.rs

use laura_engine::{BENCH_DEPTH, bench, uci_listener, uci_start};

fn main() {
    // `laura bench [depth]`: run the benchmark and exit (used by OpenBench)
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("bench") {
        let depth: usize = args
            .get(2)
            .and_then(|depth| depth.parse().ok())
            .unwrap_or(BENCH_DEPTH);
        bench(depth);
        return;
    }

    uci_start();
    uci_listener();
}
