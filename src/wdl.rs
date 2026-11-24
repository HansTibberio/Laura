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

// src/wdl.rs

//! Win/Draw/Loss model and UCI score normalization.

use laura_core::Board;

// Material count used as the WDL model input:
// P = 1, N = 3, B = 3, R = 5, Q = 9 (both sides).
#[inline]
pub fn wdl_material(board: &Board) -> i32 {
    let pawns: i32 = board.pawns().count_bits() as i32;
    let minors: i32 = (board.knights() | board.bishops()).count_bits() as i32;
    let rooks: i32 = board.rooks().count_bits() as i32;
    let queens: i32 = board.queens().count_bits() as i32;
    pawns + 3 * minors + 5 * rooks + 9 * queens
}

const AS: [f64; 4] = [-683.16337758, 2054.38886077, -2012.52971720, 754.95189068];
const BS: [f64; 4] = [10.07175585, 60.61936450, -102.68327937, 140.70486988];

// Internal value equal to 1 pawn (100 cp) at the reference material = 58.
pub const NORMALIZE_TO_PAWN_VALUE: i32 = 114;

#[inline]
fn win_rate_params(material: i32) -> (f64, f64) {
    let x: f64 = material.clamp(17, 78) as f64 / 58.0;
    let a: f64 = ((AS[0] * x + AS[1]) * x + AS[2]) * x + AS[3];
    let b: f64 = ((BS[0] * x + BS[1]) * x + BS[2]) * x + BS[3];
    (a, b)
}

// Win probability (per mille) for the side to move.
#[inline]
fn win_rate(v: i32, material: i32) -> i32 {
    let (a, b) = win_rate_params(material);
    (0.5 + 1000.0 / (1.0 + ((a - v as f64) / b).exp())) as i32
}

// (win, draw, loss) in per mille
pub fn wdl(v: i32, material: i32) -> (i32, i32, i32) {
    let w: i32 = win_rate(v, material);
    let l: i32 = win_rate(-v, material);
    (w, 1000 - w - l, l)
}

// Normalized score for UCI: 100 cp = 50 % win chance.
pub fn to_cp(v: i32, material: i32) -> i32 {
    let (a, _) = win_rate_params(material);
    (100.0 * v as f64 / a).round() as i32
}

#[cfg(test)]
mod test {
    use super::{wdl, wdl_material};
    use laura_core::Board;

    #[test]
    fn startpos_material() {
        assert_eq!(wdl_material(&Board::default()), 78);
    }

    #[test]
    fn wdl_sums_to_1000() {
        for v in [-2000, -300, -100, 0, 100, 300, 2000] {
            for m in [10, 17, 40, 58, 78, 90] {
                let (w, d, l) = wdl(v, m);
                assert_eq!(w + d + l, 1000);
                assert!(w >= 0 && d >= 0 && l >= 0);
            }
        }
    }
}
