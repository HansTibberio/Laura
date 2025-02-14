/*
    Laura-Core: a fast and efficient move generator for chess engines.

    Copyright (C) 2024-2025 HansTibberio <hanstiberio@proton.me>

    Laura-Core is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Laura-Core is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Laura-Core. If not, see <https://www.gnu.org/licenses/>.
*/

use crate::{BitBoard, Square};

// This implementation of PEXT bitboards is based on the work developed in Cozy-Chess, licensed under the MIT License.
// Copyright (c) 2021 analog-hors
// Source: https://github.com/analog-hors/cozy-chess/blob/master/types/src/sliders/pext.rs

// Includes pre-generated files containing the slider attack bitboards and the PEXT (Parallel Bit Extraction)
// data. These files are created at build time and are dynamically included into the current module at compile-time.
include!(concat!(env!("OUT_DIR"), "/sliders_attacks.rs"));
include!(concat!(env!("OUT_DIR"), "/pext_data.rs"));

/// Executes the PEXT (Parallel Bit Extraction) operation on two 64-bit integers. This function uses the x86_64
/// architecture's intrinsic to perform the PEXT operation, which extracts specific bits from one integer according
/// to a mask and returns them in the result. This is used to efficiently compute attack bitboards based on blockers.
///
/// ## Safety:
/// This function uses a raw FFI call to access the `x86_64::_pext_u64` intrinsic, which is platform-specific and unsafe.
fn pext(a: u64, mask: u64) -> u64 {
    unsafe { core::arch::x86_64::_pext_u64(a, mask) }
}

/// Represents a single PEXT entry for a slider piece's attack data. This structure is used to store the necessary
/// information for performing a PEXT operation to compute the attack bitboard of a slider piece (rook or bishop).
struct PextEntry {
    offset: usize,
    mask: BitBoard,
}

/// Contains the PEXT data for rook and bishop pieces. This structure stores precomputed information for each square
/// on the chessboard regarding the PEXT operations required to compute valid attacks for sliders (rooks and bishops).
///
/// The data is divided into two arrays, one for rooks and one for bishops, with each entry containing an `offset` and `mask`.
/// Additionally, the total size of the table is stored.
#[allow(dead_code)]
struct PextIndexData {
    rook_data: [PextEntry; Square::NUM_SQUARES],
    bishop_data: [PextEntry; Square::NUM_SQUARES],
    table_size: usize,
}

/// Computes the index of the attack bitboard for a slider piece (rook or bishop) based on the blocker positions
/// using the PEXT data.
#[inline]
fn pext_index(index_data: &PextEntry, blockers: BitBoard) -> usize {
    let index: u64 = pext(blockers.0, index_data.mask.0);
    index_data.offset + index as usize
}

/// Retrieves the attack bitboard for a rook from a given square, considering the positions of blockers.
///
/// This function uses the PEXT operation and precomputed data to efficiently calculate the attack bitboard for a
/// rook piece.
#[inline]
pub fn get_rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let index_data: &PextEntry = PEXT_DATA.rook_data.get_unchecked(square as usize);
        BitBoard(*SLIDER_ATTACKS.get_unchecked(pext_index(index_data, blockers)))
    }
}

/// Retrieves the attack bitboard for a bishop from a given square, considering the positions of blockers.
///
/// This function is similar to `get_rook_attacks`, but is designed for bishop pieces. It uses the PEXT operation
/// to efficiently compute the attack bitboard for a bishop, considering the positions of blockers.
#[inline]
pub fn get_bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    unsafe {
        let index_data: &PextEntry = PEXT_DATA.bishop_data.get_unchecked(square as usize);
        BitBoard(*SLIDER_ATTACKS.get_unchecked(pext_index(index_data, blockers)))
    }
}
