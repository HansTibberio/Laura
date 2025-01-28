use super::{
    sliders::{Slider, BISHOP_SLIDER, ROOK_SLIDER},
    types::{BitBoard, Square},
};
use std::io::Result;
use std::io::Write;

// This implementation of PEXT bitboards is inspired by and based on the work developed in Cozy-Chess, licensed under the MIT License.
// Copyright (c) 2021 analog-hors
// Source: https://github.com/analog-hors/cozy-chess/blob/master/types/src/sliders/pext.rs

// This attribute ensures that the code is compiled only if the target architecture is x86_64
// and the BMI2 instruction set is supported. If these conditions are not met, a compile-time error
// is triggered.
#[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
compile_error!(
    "This program requires support for BMI2 instructions on the x86_64 architecture. 
Please ensure you are using a CPU that supports BMI2 or enable BMI2 with appropriate compiler flags 
(e.g., RUSTFLAGS=\"-C target-cpu=native\" or RUSTFLAGS=\"-C target-feature=+bmi2\")."
);

/// Performs the Parallel Extract (PEXT) operation using the BMI2 instruction set.
///
/// This function uses the `core::arch::x86_64::_pext_u64` intrinsic, which is part of the BMI2
/// instruction set. It is only available on processors that support BMI2 instructions.
fn pext(a: u64, mask: u64) -> u64 {
    unsafe { core::arch::x86_64::_pext_u64(a, mask) }
}

/// Calculates the magic index for a given blocker configuration using the PEXT operation.
///
/// The function first applies the `pext` function to extract relevant bits from the `blockers` using
/// the provided mask from `index_data`.
/// The result is then adjusted by adding the `offset` stored in the `PextEntry` to compute the final index.
fn pext_index(index_data: &PextEntry, blockers: BitBoard) -> usize {
    let index: u64 = pext(blockers.0, index_data.mask.0);
    index_data.offset + index as usize
}

/// This macro is used to create a `PextEntry` structure with the specified offset and mask values.
///
/// Example Usage:
/// - The macro can be used to easily create `PextEntry` values for use in the `pext_index` function.
///
/// Notes:
/// - This macro simplifies the creation of `PextEntry` objects, making it easier to pass the necessary
///   values for the mask and offset to the relevant operations.
macro_rules! Pext {
    ($of: expr, $mk: expr) => {
        PextEntry {
            offset: $of,
            mask: BitBoard($mk),
        }
    };
}

/// This constant holds data related to the "PEXT" (parallel extract) operation for both rooks and bishops.
/// It contains two main arrays: `rook_data` and `bishop_data`, which represent precomputed bitboard data
/// for each of these chess pieces movement patterns.
/// 
/// Each entry in the arrays corresponds to a unique position on the chessboard and contains a 64-bit mask
/// for the PEXT operation, which is used to extract relevant bits for chessboard squares that are relevant
/// for rook or bishop movement. The data is structured with offsets and masks, allowing efficient extraction
/// of relevant bits for the given piece's movement.
/// 
/// The overall table size (`table_size`) defines the total number of entries, providing an upper bound on
/// how much data needs to be loaded or processed for these operations.
#[rustfmt::skip]
const PEXT_DATA: &PextIndexData = &PextIndexData { 
    rook_data: [
        Pext!(00000, 0x000101010101017E), Pext!(04096, 0x000202020202027C), Pext!(06144, 0x000404040404047A),
        Pext!(08192, 0x0008080808080876), Pext!(10240, 0x001010101010106E), Pext!(12288, 0x002020202020205E),
        Pext!(14336, 0x004040404040403E), Pext!(16384, 0x008080808080807E), Pext!(20480, 0x0001010101017E00),
        Pext!(22528, 0x0002020202027C00), Pext!(23552, 0x0004040404047A00), Pext!(24576, 0x0008080808087600),
        Pext!(25600, 0x0010101010106E00), Pext!(26624, 0x0020202020205E00), Pext!(27648, 0x0040404040403E00),
        Pext!(28672, 0x0080808080807E00), Pext!(30720, 0x00010101017E0100), Pext!(32768, 0x00020202027C0200),
        Pext!(33792, 0x00040404047A0400), Pext!(34816, 0x0008080808760800), Pext!(35840, 0x00101010106E1000),
        Pext!(36864, 0x00202020205E2000), Pext!(37888, 0x00404040403E4000), Pext!(38912, 0x00808080807E8000),
        Pext!(40960, 0x000101017E010100), Pext!(43008, 0x000202027C020200), Pext!(44032, 0x000404047A040400),
        Pext!(45056, 0x0008080876080800), Pext!(46080, 0x001010106E101000), Pext!(47104, 0x002020205E202000),
        Pext!(48128, 0x004040403E404000), Pext!(49152, 0x008080807E808000), Pext!(51200, 0x0001017E01010100), 
        Pext!(53248, 0x0002027C02020200), Pext!(54272, 0x0004047A04040400), Pext!(55296, 0x0008087608080800),
        Pext!(56320, 0x0010106E10101000), Pext!(57344, 0x0020205E20202000), Pext!(58368, 0x0040403E40404000),
        Pext!(59392, 0x0080807E80808000), Pext!(61440, 0x00017E0101010100), Pext!(63488, 0x00027C0202020200),
        Pext!(64512, 0x00047A0404040400), Pext!(65536, 0x0008760808080800), Pext!(66560, 0x00106E1010101000),
        Pext!(67584, 0x00205E2020202000), Pext!(68608, 0x00403E4040404000), Pext!(69632, 0x00807E8080808000),
        Pext!(71680, 0x007E010101010100), Pext!(73728, 0x007C020202020200), Pext!(74752, 0x007A040404040400),
        Pext!(75776, 0x0076080808080800), Pext!(76800, 0x006E101010101000), Pext!(77824, 0x005E202020202000),
        Pext!(78848, 0x003E404040404000), Pext!(79872, 0x007E808080808000), Pext!(81920, 0x7E01010101010100),
        Pext!(86016, 0x7C02020202020200), Pext!(88064, 0x7A04040404040400), Pext!(90112, 0x7608080808080800),
        Pext!(92160, 0x6E10101010101000), Pext!(94208, 0x5E20202020202000), Pext!(96256, 0x3E40404040404000),
        Pext!(98304, 0x7E80808080808000),
    ],
    bishop_data: [
        Pext!(102400, 0x0040201008040200), Pext!(102464, 0x0000402010080400), Pext!(102496, 0x0000004020100A00),
        Pext!(102528, 0x0000000040221400), Pext!(102560, 0x0000000002442800), Pext!(102592, 0x0000000204085000),
        Pext!(102624, 0x0000020408102000), Pext!(102656, 0x0002040810204000), Pext!(102720, 0x0020100804020000),
        Pext!(102752, 0x0040201008040000), Pext!(102784, 0x00004020100A0000), Pext!(102816, 0x0000004022140000),
        Pext!(102848, 0x0000000244280000), Pext!(102880, 0x0000020408500000), Pext!(102912, 0x0002040810200000),
        Pext!(102944, 0x0004081020400000), Pext!(102976, 0x0010080402000200), Pext!(103008, 0x0020100804000400),
        Pext!(103040, 0x004020100A000A00), Pext!(103168, 0x0000402214001400), Pext!(103296, 0x0000024428002800),
        Pext!(103424, 0x0002040850005000), Pext!(103552, 0x0004081020002000), Pext!(103584, 0x0008102040004000),
        Pext!(103616, 0x0008040200020400), Pext!(103648, 0x0010080400040800), Pext!(103680, 0x0020100A000A1000),
        Pext!(103808, 0x0040221400142200), Pext!(104320, 0x0002442800284400), Pext!(104832, 0x0004085000500800),
        Pext!(104960, 0x0008102000201000), Pext!(104992, 0x0010204000402000), Pext!(105024, 0x0004020002040800),
        Pext!(105056, 0x0008040004081000), Pext!(105088, 0x00100A000A102000), Pext!(105216, 0x0022140014224000),
        Pext!(105728, 0x0044280028440200), Pext!(106240, 0x0008500050080400), Pext!(106368, 0x0010200020100800),
        Pext!(106400, 0x0020400040201000), Pext!(106432, 0x0002000204081000), Pext!(106464, 0x0004000408102000),
        Pext!(106496, 0x000A000A10204000), Pext!(106624, 0x0014001422400000), Pext!(106752, 0x0028002844020000),
        Pext!(106880, 0x0050005008040200), Pext!(107008, 0x0020002010080400), Pext!(107040, 0x0040004020100800),
        Pext!(107072, 0x0000020408102000), Pext!(107104, 0x0000040810204000), Pext!(107136, 0x00000A1020400000),
        Pext!(107168, 0x0000142240000000), Pext!(107200, 0x0000284402000000), Pext!(107232, 0x0000500804020000),
        Pext!(107264, 0x0000201008040200), Pext!(107296, 0x0000402010080400), Pext!(107328, 0x0002040810204000),
        Pext!(107392, 0x0004081020400000), Pext!(107424, 0x000A102040000000), Pext!(107456, 0x0014224000000000),
        Pext!(107488, 0x0028440200000000), Pext!(107520, 0x0050080402000000), Pext!(107552, 0x0020100804020000),
        Pext!(107584, 0x0040201008040200),
    ],
    table_size: 107648
};

// Constant for the number of positions to shift when handling rook pieces.
pub const ROOK_SHIFT: usize = 12;

// Constant for the number of positions to shift when handling bishop pieces.
pub const BISHOP_SHIFT: usize = 9;

// Constant defining the total size of the PEXT data table.
pub const TABLE_SIZE: usize = 107648;

/// A structure representing an entry in the Pext index.
/// This is used for storing data associated with a piece movement in a chessboard,
/// specifically for a given square on the chessboard (offset) and a bitmask of relevant blockers.
struct PextEntry {
    offset: usize,
    mask: BitBoard,
}

/// A constant representing an empty entry in the Pext index.
/// This entry has an offset of 0 and an empty bitmask, indicating no relevant blockers.
/// It's used as a default value for initializing the Pext index.
const EMPTY_ENTRY: PextEntry = PextEntry {
    offset: 0,
    mask: BitBoard::EMPTY,
};

/// A structure to hold the Pext index data for rook and bishop movements.
/// The data is divided into two arrays, one for rook and one for bishop, each containing entries
/// for every square on the chessboard. The structure also tracks the total size of the Pext table.
pub struct PextIndexData {
    rook_data: [PextEntry; Square::NUM_SQUARES],
    bishop_data: [PextEntry; Square::NUM_SQUARES],
    table_size: usize,
}

/// Generates the Pext index data for rook and bishop piece movements.
/// This function initializes the Pext index for rook and bishop pieces by calculating the relevant blockers
/// for each square and updating the index entries with the correct offsets and bitmasks.
pub fn gen_pext() -> PextIndexData {
    let mut offset: usize = 0;

    let mut rook_data: [PextEntry; Square::NUM_SQUARES] = [EMPTY_ENTRY; Square::NUM_SQUARES];
    let mut i: usize = 0;
    while i < rook_data.len() {
        let square: Square = Square::from_index(i);
        let mask: BitBoard = Slider::relevant_blockers(&ROOK_SLIDER, square);
        rook_data[i] = PextEntry { offset, mask };
        offset += 1 << mask.count_bits();
        i += 1;
    }

    let mut bishop_data: [PextEntry; Square::NUM_SQUARES] = [EMPTY_ENTRY; Square::NUM_SQUARES];
    let mut i: usize = 0;
    while i < bishop_data.len() {
        let square: Square = Square::from_index(i);
        let mask: BitBoard = Slider::relevant_blockers(&BISHOP_SLIDER, square);
        bishop_data[i] = PextEntry { offset, mask };
        offset += 1 << mask.count_bits();
        i += 1;
    }

    PextIndexData {
        rook_data,
        bishop_data,
        table_size: offset,
    }
}

/// Generates the attack bitboards for both rook and bishop pieces for all squares on the chessboard.
/// This function calculates the possible attacks for both rooks and bishops by considering the relevant blockers
/// for each square and the movements of the piece. It then stores the generated bitboards in the `attacks` array.
pub fn gen_attacks(attacks: &mut [BitBoard; TABLE_SIZE]) {
    for square in 0..Square::NUM_SQUARES {
        let mask: BitBoard = Slider::relevant_blockers(&ROOK_SLIDER, Square::from_index(square));
        for index in 0..(1 << ROOK_SHIFT) {
            let blockers: BitBoard = mask.set_blockers(index);
            let index_data: &PextEntry = &PEXT_DATA.rook_data[square as usize];
            attacks[pext_index(&index_data, blockers)] =
                Slider::moves(&ROOK_SLIDER, Square::from_index(square), blockers)
        }
    }

    for square in 0..Square::NUM_SQUARES {
        let mask: BitBoard = Slider::relevant_blockers(&BISHOP_SLIDER, Square::from_index(square));
        for index in 0..(1 << ROOK_SHIFT) {
            let blockers: BitBoard = mask.set_blockers(index);
            let index_data: &PextEntry = &PEXT_DATA.bishop_data[square as usize];
            attacks[pext_index(&index_data, blockers)] =
                Slider::moves(&BISHOP_SLIDER, Square::from_index(square), blockers)
        }
    }
}

/// Writes the Pext index data to the provided output stream.
/// This function generates a serialized representation of the Pext index data, including
/// the rook and bishop movement data, as well as the table size. It writes the data in a
/// Rust constant format that can be used in the move generation.
pub fn write_pext(pext_data: PextIndexData, out: &mut impl Write) -> Result<()> {
    writeln!(out, "const PEXT_DATA: &PextIndexData = &PextIndexData {{ ")?;
    writeln!(out, "rook_data: [")?;
    for pext_entry in pext_data.rook_data {
        writeln!(
            out,
            "PextEntry {{ offset: {}, mask: BitBoard(0x{:016X}) }},",
            pext_entry.offset, pext_entry.mask.0,
        )?;
    }
    writeln!(out, "],")?;
    writeln!(out, "bishop_data: [")?;
    for pext_entry in pext_data.bishop_data {
        writeln!(
            out,
            "PextEntry {{ offset: {}, mask: BitBoard(0x{:016X}) }},",
            pext_entry.offset, pext_entry.mask.0,
        )?;
    }
    writeln!(out, "],")?;
    writeln!(out, "table_size: {}", pext_data.table_size,)?;
    writeln!(out, "}};",)?;

    Ok(())
}

/// Writes the attack bitboards to the provided output stream.
/// This function generates a serialized representation of the slider attack bitboards
/// in a constant format for use in other parts of the program. The bitboards represent
/// the possible attack patterns for each square on the chessboard based on the piece movements.
pub fn write_attacks(table: &[BitBoard; TABLE_SIZE], out: &mut impl Write) -> Result<()> {
    writeln!(out, "const SLIDER_ATTACKS: [u64; {}] = [", table.len())?;
    for attack in table {
        write!(out, "{}, ", attack.0)?;
    }
    write!(out, "];")?;

    Ok(())
}
