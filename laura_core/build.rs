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

mod build_dep;

use std::env::var;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use build_dep::between::*;
#[cfg(not(feature = "bmi2"))]
use build_dep::black_magics::*;
#[cfg(feature = "bmi2")]
use build_dep::pext::*;

#[cfg(not(feature = "bmi2"))]
use build_dep::sliders::BISHOP_SLIDER;
#[cfg(not(feature = "bmi2"))]
use build_dep::sliders::ROOK_SLIDER;
use build_dep::types::BitBoard;

/// Sets up a buffered writer for a given filename in the output directory specified by `OUT_DIR`.
fn create_out_file(filename: &str) -> BufWriter<File> {
    let mut out_path: PathBuf = var("OUT_DIR").unwrap().into();
    out_path.push(filename);
    BufWriter::new(File::create(out_path).unwrap())
}

/// Main function for generating and writing the necessary bitboard data,
/// including the black magic numbers for rooks and bishops, attacks for sliders,
/// and between-square tables to corresponding output files.
///
/// The function first checks the feature flag `bmi2` to determine whether to use the BMI2
/// instructions, or to use black magic numbers (for systems not supporting BMI2).
fn main() {
    #[cfg(not(feature = "bmi2"))]
    {
        // Generate attack bitboards and black magic numbers for non-BMI2 feature enabled systems
        let mut attacks: [BitBoard; TABLE_SIZE] = [BitBoard::EMPTY; TABLE_SIZE];
        let rook_bmagics: BlackMagics =
            BlackMagics::gen(&mut attacks, ROOK_BLACK_MAGICS, ROOK_SHIFT, ROOK_SLIDER);
        let bishop_bmagics: BlackMagics = BlackMagics::gen(
            &mut attacks,
            BISHOP_BLACK_MAGICS,
            BISHOP_SHIFT,
            BISHOP_SLIDER,
        );

        // Create a file writer for rook black magic numbers and write them
        let mut rook_bmagic_file: BufWriter<File> = create_out_file("rook_bmagics.rs");
        write_bmagics(rook_bmagics, "ROOK", &mut rook_bmagic_file).unwrap();

        // Create a file writer for bishop black magic numbers and write them
        let mut bishop_bmagic_file: BufWriter<File> = create_out_file("bishop_bmagics.rs");
        write_bmagics(bishop_bmagics, "BISHOP", &mut bishop_bmagic_file).unwrap();

        // Create a file writer for slider attack bitboards and write them
        let mut sliders_attacks: BufWriter<File> = create_out_file("sliders_attacks.rs");
        write_attacks(&attacks, &mut sliders_attacks).unwrap();
    }

    #[cfg(feature = "bmi2")]
    {
        // Generate Pext data and attack bitboards for BMI2-optimized systems
        let pext_data: PextIndexData = gen_pext();
        let mut pext_writer: BufWriter<File> = create_out_file("pext_data.rs");
        write_pext(pext_data, &mut pext_writer).unwrap();

        // Generate attack bitboards for sliders (rooks and bishops)
        let mut attacks: [BitBoard; TABLE_SIZE] = [BitBoard::EMPTY; TABLE_SIZE];
        gen_attacks(&mut attacks);
        let mut sliders_attacks: BufWriter<File> = create_out_file("sliders_attacks.rs");
        write_attacks(&attacks, &mut sliders_attacks).unwrap();
    }

    // Generates a 2D table of `BitBoard`s for all pairs of squares on the chessboard,
    // representing the squares between them for straight-line moves.
    let between_table: [[BitBoard; 64]; 64] = gen_between();

    // Writes the `between_table` array to "between_array.rs" file in OUT_DIR
    let mut between_file: BufWriter<File> = create_out_file("between_array.rs");
    write_between("BETWEEN", &between_table, &mut between_file).unwrap();
}
