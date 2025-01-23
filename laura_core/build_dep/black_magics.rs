use std::io::Result;
use std::io::Write;

use super::sliders::Slider;
use super::types::{BitBoard, Square};

//  All the magic numbers and offsets were published by Volker Annuss & Niklas Fiekas.
//  Credits to the original creators.
//  (https://www.talkchess.com/forum3/viewtopic.php?p=726160#p726160)

/// Defines a structure to store black magic bitboard entries.
///
/// The `BlackMagicEntry` structure is used to store precomputed magic numbers and offsets
/// for efficient bitboard manipulation in the chess engine. These values are essential for
/// calculating sliding piece attacks (rooks and bishops) using magic bitboards.
///
/// ## Fields
/// - `magic`: A 64-bit unsigned integer representing the magic number used for bitboard hashing.
/// - `offset`: A 64-bit unsigned integer representing the offset into the attack table.
pub struct BlackMagicEntry {
    magic: u64,
    offset: u64,
}

/// Macro to create a `BlackMagicEntry` instance.
///
/// This macro simplifies the creation of `BlackMagicEntry` objects by accepting a magic
/// number and an offset as arguments.
///
/// ## Example
/// ```rust
/// let entry = BlackMagic!(0x80280013FF84FFFF, 10890);
/// ```
macro_rules! BlackMagic {
    ($mg: expr, $o: expr) => {
        BlackMagicEntry {
            magic: $mg,
            offset: $o,
        }
    };
}

/// Precomputed black magic numbers and offsets for rook moves.
///
/// The `ROOK_BLACK_MAGICS` array contains magic bitboard entries for each square on a
/// standard chessboard (0-63). These values are optimized for fast attack generation.
#[rustfmt::skip]
pub const ROOK_BLACK_MAGICS: [BlackMagicEntry; Square::NUM_SQUARES] = [
    BlackMagic!(0x80280013FF84FFFF, 10890), BlackMagic!(0x5FFBFEFDFEF67FFF, 50579),
    BlackMagic!(0xFFEFFAFFEFFDFFFF, 62020), BlackMagic!(0x003000900300008A, 67322),
    BlackMagic!(0x0050028010500023, 80251), BlackMagic!(0x0020012120A00020, 58503),
    BlackMagic!(0x0030006000C00030, 51175), BlackMagic!(0x0058005806B00002, 83130),
    BlackMagic!(0x7FBFF7FBFBEAFFFC, 50430), BlackMagic!(0x0000140081050002, 21613),
    BlackMagic!(0x0000180043800048, 72625), BlackMagic!(0x7FFFE800021FFFB8, 80755),
    BlackMagic!(0xFFFFCFFE7FCFFFAF, 69753), BlackMagic!(0x00001800C0180060, 26973),
    BlackMagic!(0x4F8018005FD00018, 84972), BlackMagic!(0x0000180030620018, 31958),
    BlackMagic!(0x00300018010C0003, 69272), BlackMagic!(0x0003000C0085FFFF, 48372),
    BlackMagic!(0xFFFDFFF7FBFEFFF7, 65477), BlackMagic!(0x7FC1FFDFFC001FFF, 43972),
    BlackMagic!(0xFFFEFFDFFDFFDFFF, 57154), BlackMagic!(0x7C108007BEFFF81F, 53521),
    BlackMagic!(0x20408007BFE00810, 30534), BlackMagic!(0x0400800558604100, 16548),
    BlackMagic!(0x0040200010080008, 46407), BlackMagic!(0x0010020008040004, 11841),
    BlackMagic!(0xFFFDFEFFF7FBFFF7, 21112), BlackMagic!(0xFEBF7DFFF8FEFFF9, 44214),
    BlackMagic!(0xC00000FFE001FFE0, 57925), BlackMagic!(0x4AF01F00078007C3, 29574),
    BlackMagic!(0xBFFBFAFFFB683F7F, 17309), BlackMagic!(0x0807F67FFA102040, 40143),
    BlackMagic!(0x200008E800300030, 64659), BlackMagic!(0x0000008780180018, 70469),
    BlackMagic!(0x0000010300180018, 62917), BlackMagic!(0x4000008180180018, 60997),
    BlackMagic!(0x008080310005FFFA, 18554), BlackMagic!(0x4000188100060006, 14385),
    BlackMagic!(0xFFFFFF7FFFBFBFFF, 00000), BlackMagic!(0x0000802000200040, 38091),
    BlackMagic!(0x20000202EC002800, 25122), BlackMagic!(0xFFFFF9FF7CFFF3FF, 60083),
    BlackMagic!(0x000000404B801800, 72209), BlackMagic!(0x2000002FE03FD000, 67875),
    BlackMagic!(0xFFFFFF6FFE7FCFFD, 56290), BlackMagic!(0xBFF7EFFFBFC00FFF, 43807),
    BlackMagic!(0x000000100800A804, 73365), BlackMagic!(0x6054000A58005805, 76398),
    BlackMagic!(0x0829000101150028, 20024), BlackMagic!(0x00000085008A0014, 09513),
    BlackMagic!(0x8000002B00408028, 24324), BlackMagic!(0x4000002040790028, 22996),
    BlackMagic!(0x7800002010288028, 23213), BlackMagic!(0x0000001800E08018, 56002),
    BlackMagic!(0xA3A80003F3A40048, 22809), BlackMagic!(0x2003D80000500028, 44545),
    BlackMagic!(0xFFFFF37EEFEFDFBE, 36072), BlackMagic!(0x40000280090013C1, 04750),
    BlackMagic!(0xBF7FFEFFBFFAF71F, 06014), BlackMagic!(0xFFFDFFFF777B7D6E, 36054),
    BlackMagic!(0x48300007E8080C02, 78538), BlackMagic!(0xAFE0000FFF780402, 28745),
    BlackMagic!(0xEE73FFFBFFBB77FE, 08555), BlackMagic!(0x0002000308482882, 01009),
];

/// Precomputed black magic numbers and offsets for bishop moves.
///
/// Similar to `ROOK_BLACK_MAGICS`, the `BISHOP_BLACK_MAGICS` array contains precomputed
/// magic numbers and offsets for each square, but specifically for bishops.
#[rustfmt::skip]
pub const BISHOP_BLACK_MAGICS: [BlackMagicEntry; Square::NUM_SQUARES] = [
    BlackMagic!(0xA7020080601803D8, 60984), BlackMagic!(0x13802040400801F1, 66046),
    BlackMagic!(0x0A0080181001F60C, 32910), BlackMagic!(0x1840802004238008, 16369),
    BlackMagic!(0xC03FE00100000000, 42115), BlackMagic!(0x24C00BFFFF400000, 00835),
    BlackMagic!(0x0808101F40007F04, 18910), BlackMagic!(0x100808201EC00080, 25911),
    BlackMagic!(0xFFA2FEFFBFEFB7FF, 63301), BlackMagic!(0x083E3EE040080801, 16063),
    BlackMagic!(0xC0800080181001F8, 17481), BlackMagic!(0x0440007FE0031000, 59361),
    BlackMagic!(0x2010007FFC000000, 18735), BlackMagic!(0x1079FFE000FF8000, 61249),
    BlackMagic!(0x3C0708101F400080, 68938), BlackMagic!(0x080614080FA00040, 61791),
    BlackMagic!(0x7FFE7FFF817FCFF9, 21893), BlackMagic!(0x7FFEBFFFA01027FD, 62068),
    BlackMagic!(0x53018080C00F4001, 19829), BlackMagic!(0x407E0001000FFB8A, 26091),
    BlackMagic!(0x201FE000FFF80010, 15815), BlackMagic!(0xFFDFEFFFDE39FFEF, 16419),
    BlackMagic!(0xCC8808000FBF8002, 59777), BlackMagic!(0x7FF7FBFFF8203FFF, 16288),
    BlackMagic!(0x8800013E8300C030, 33235), BlackMagic!(0x0420009701806018, 15459),
    BlackMagic!(0x7FFEFF7F7F01F7FD, 15863), BlackMagic!(0x8700303010C0C006, 75555),
    BlackMagic!(0xC800181810606000, 79445), BlackMagic!(0x20002038001C8010, 15917),
    BlackMagic!(0x087FF038000FC001, 08512), BlackMagic!(0x00080C0C00083007, 73069),
    BlackMagic!(0x00000080FC82C040, 16078), BlackMagic!(0x000000407E416020, 19168),
    BlackMagic!(0x00600203F8008020, 11056), BlackMagic!(0xD003FEFE04404080, 62544),
    BlackMagic!(0xA00020C018003088, 80477), BlackMagic!(0x7FBFFE700BFFE800, 75049),
    BlackMagic!(0x107FF00FE4000F90, 32947), BlackMagic!(0x7F8FFFCFF1D007F8, 59172),
    BlackMagic!(0x0000004100F88080, 55845), BlackMagic!(0x00000020807C4040, 61806),
    BlackMagic!(0x00000041018700C0, 73601), BlackMagic!(0x0010000080FC4080, 15546),
    BlackMagic!(0x1000003C80180030, 45243), BlackMagic!(0xC10000DF80280050, 20333),
    BlackMagic!(0xFFFFFFBFEFF80FDC, 33402), BlackMagic!(0x000000101003F812, 25917),
    BlackMagic!(0x0800001F40808200, 32875), BlackMagic!(0x084000101F3FD208, 04639),
    BlackMagic!(0x080000000F808081, 17077), BlackMagic!(0x0004000008003F80, 62324),
    BlackMagic!(0x08000001001FE040, 18159), BlackMagic!(0x72DD000040900A00, 61436),
    BlackMagic!(0xFFFFFEFFBFEFF81D, 57073), BlackMagic!(0xCD8000200FEBF209, 61025),
    BlackMagic!(0x100000101EC10082, 81259), BlackMagic!(0x7FBAFFFFEFE0C02F, 64083),
    BlackMagic!(0x7F83FFFFFFF07F7F, 56114), BlackMagic!(0xFFF1FFFFFFF7FFC1, 57058),
    BlackMagic!(0x0878040000FFE01F, 58912), BlackMagic!(0x945E388000801012, 22194),
    BlackMagic!(0x0840800080200FDA, 70880), BlackMagic!(0x100000C05F582008, 11140),
];

// Represents the bit shift amount used for rook magic numbers.
pub const ROOK_SHIFT: usize = 12;

// Represents the bit shift amount used for bishop magic numbers.
pub const BISHOP_SHIFT: usize = 9;

// The total size of the attack table used for precomputed slider moves.
pub const TABLE_SIZE: usize = 87988;

/// A structure that represents magic bitboards for efficiently calculating
/// sliding piece (rook/bishop) moves on a chessboard.
pub struct BlackMagics {
    magics: [BlackMagicEntry; Square::NUM_SQUARES],
    not_mask: [BitBoard; Square::NUM_SQUARES],
    shift: usize,
}

impl BlackMagics {
    /// Generates magic bitboards and attack tables for sliding pieces.
    ///
    /// Notes:
    /// - The method iterates over all squares on the chessboard and computes
    ///   the relevant blocker mask for each square.
    /// - For each blocker configuration, it calculates the magic index and stores
    ///   the precomputed moves in the attack table.
    pub fn gen(
        attacks: &mut [BitBoard; TABLE_SIZE],
        black_magics: [BlackMagicEntry; Square::NUM_SQUARES],
        shift: usize,
        slider: Slider,
    ) -> BlackMagics {
        let mut magics: BlackMagics = BlackMagics {
            magics: black_magics,
            not_mask: [BitBoard::EMPTY; Square::NUM_SQUARES],
            shift,
        };

        for square in 0..Square::NUM_SQUARES {
            let mask: BitBoard = slider.relevant_blockers(Square::from_index(square));
            magics.not_mask[square] = !mask;

            for index in 0..(1 << magics.shift) {
                let blockers: BitBoard = mask.set_blockers(index);
                let index: usize = magics.magic_index(square, blockers);

                attacks[index] = slider.moves(Square::from_index(square), blockers);
            }
        }

        magics
    }

    /// Calculates the magic index for a given square and blocker configuration.
    pub fn magic_index(&self, square: usize, blockers: BitBoard) -> usize {
        let black_magic: &BlackMagicEntry = &self.magics[square];

        let mut relevant_blocker: u64 = (blockers | self.not_mask[square]).0;
        relevant_blocker = relevant_blocker.wrapping_mul(black_magic.magic);
        relevant_blocker >>= 64 - self.shift;

        (relevant_blocker + black_magic.offset) as usize
    }
}

/// Writes the `BlackMagics` data as a constant array in Rust syntax.
///
/// The function generates an array of `BlackMagicEntry` for all squares.
/// Each entry is formatted using the `BlackMagic!` macro with the magic
/// number, not mask, and offset.
pub fn write_bmagics(bmagics: BlackMagics, name: &str, out: &mut impl Write) -> Result<()> {
    writeln!(
        out,
        "const {}_BLACK_MAGICS: [BlackMagicEntry; Square::NUM_SQUARES] = [",
        name
    )?;
    for index in 0..Square::NUM_SQUARES {
        writeln!(
            out,
            "BlackMagic!(0x{:016X}, 0x{:016X}, {}), ",
            bmagics.magics[index].magic, bmagics.not_mask[index].0, bmagics.magics[index].offset
        )?;
    }
    write!(out, "];")?;

    Ok(())
}

/// Writes the slider attack table as a static array in Rust syntax.
///
/// The function generates a `static` array in Rust syntax containing the
/// precomputed attack bitboards for all blocker configurations.
/// Each entry in the array corresponds to a specific blocker configuration
/// and is formatted as a `u64` value.
pub fn write_attacks(table: &[BitBoard; TABLE_SIZE], out: &mut impl Write) -> Result<()> {
    writeln!(out, "static SLIDER_ATTACKS: [u64; {}] = [", table.len())?;
    for attack in table {
        write!(out, "{}, ", attack.0)?;
    }
    write!(out, "];")?;

    Ok(())
}
