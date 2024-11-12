use std::io::Result;
use std::io::Write;
use std::mem::transmute;

use super::types::BitBoard;
use super::types::Square;

/// A macro to create a `MagicEntry` instance with the provided parameters.
///
/// This macro simplifies the initialization of `MagicEntry` structs by directly
/// mapping the values to the struct's fields, which represent various components
/// of the magic bitboard setup for chess engines or similar applications.
macro_rules! Magic {
    ($mk: expr, $mg: expr, $s: expr, $o: expr) => {
        MagicEntry {
            mask: $mk,
            magic: $mg,
            shift: $s,
            offset: $o,
        }
    };
}

/// Structure representing a single entry in a magic bitboard table.
///
/// The `MagicEntry` struct encapsulates the bitmask, magic multiplier,
/// shift amount, and offset needed for efficient board indexing and lookup.
/// Each entry allows for rapid access to precomputed move sets, enabling
/// efficient move generation.
pub struct MagicEntry {
    pub mask: u64,
    pub magic: u64,
    pub shift: u8,
    pub offset: u32,
}

/// Array of precomputed `MagicEntry` instances for rook moves.
///
/// Each entry in `ROOK_MAGICS` is indexed by a square on the chess board,
/// where each `MagicEntry` defines the bitmask, magic number, shift, and
/// offset required to efficiently generate possible rook moves on that square.
pub const ROOK_MAGICS: [MagicEntry; Square::NUM_SQUARES] = [
    Magic!(0x000101010101017E, 0x0880008060400032, 52, 0),
    Magic!(0x000202020202027C, 0x6880102000C00080, 53, 4096),
    Magic!(0x000404040404047A, 0x0080088220001004, 53, 6144),
    Magic!(0x0008080808080876, 0x5080280044801000, 53, 8192),
    Magic!(0x001010101010106E, 0x008008000C000680, 53, 10240),
    Magic!(0x002020202020205E, 0x0880020044008003, 53, 12288),
    Magic!(0x004040404040403E, 0x0100008200210004, 53, 14336),
    Magic!(0x008080808080807E, 0x2080002041000080, 52, 16384),
    Magic!(0x0001010101017E00, 0x5040800020904000, 53, 20480),
    Magic!(0x0002020202027C00, 0x4808400020100440, 54, 22528),
    Magic!(0x0004040404047A00, 0x0045001041002000, 54, 23552),
    Magic!(0x0008080808087600, 0x4842802800801000, 54, 24576),
    Magic!(0x0010101010106E00, 0x0205001100040800, 54, 25600),
    Magic!(0x0020202020205E00, 0x3282000A0008501C, 54, 26624),
    Magic!(0x0040404040403E00, 0x0002004409080200, 54, 27648),
    Magic!(0x0080808080807E00, 0x901C800700024080, 53, 28672),
    Magic!(0x00010101017E0100, 0x0980084020044000, 53, 30720),
    Magic!(0x00020202027C0200, 0x0001060040208200, 54, 32768),
    Magic!(0x00040404047A0400, 0x0082020010A04480, 54, 33792),
    Magic!(0x0008080808760800, 0x0010008010080081, 54, 34816),
    Magic!(0x00101010106E1000, 0x001101002800101C, 54, 35840),
    Magic!(0x00202020205E2000, 0x1002010100080400, 54, 36864),
    Magic!(0x00404040403E4000, 0x4100840001080A10, 54, 37888),
    Magic!(0x00808080807E8000, 0x83206200011040A4, 53, 38912),
    Magic!(0x000101017E010100, 0x0800400080002C90, 53, 40960),
    Magic!(0x000202027C020200, 0x0020400080200484, 54, 43008),
    Magic!(0x000404047A040400, 0x0160410100200A10, 54, 44032),
    Magic!(0x0008080876080800, 0x0010080180100080, 54, 45056),
    Magic!(0x001010106E101000, 0x0806002A00100420, 54, 46080),
    Magic!(0x002020205E202000, 0x00820200803C0080, 54, 47104),
    Magic!(0x004040403E404000, 0x0811002100020024, 54, 48128),
    Magic!(0x008080807E808000, 0x0000800080046100, 53, 49152),
    Magic!(0x0001017E01010100, 0x0120004000808000, 53, 51200),
    Magic!(0x0002027C02020200, 0x0020802901004000, 54, 53248),
    Magic!(0x0004047A04040400, 0x00218200AA0010C0, 54, 54272),
    Magic!(0x0008087608080800, 0x301100D821005001, 54, 55296),
    Magic!(0x0010106E10101000, 0x0000980080806C00, 54, 56320),
    Magic!(0x0020205E20202000, 0xC480810200800C00, 54, 57344),
    Magic!(0x0040403E40404000, 0x0201000401000200, 54, 58368),
    Magic!(0x0080807E80808000, 0x0832284102000084, 53, 59392),
    Magic!(0x00017E0101010100, 0x8803832840058000, 53, 61440),
    Magic!(0x00027C0202020200, 0x0010082000404000, 54, 63488),
    Magic!(0x00047A0404040400, 0x0002A00010008080, 54, 64512),
    Magic!(0x0008760808080800, 0x0010420021120008, 54, 65536),
    Magic!(0x00106E1010101000, 0x8041000800110024, 54, 66560),
    Magic!(0x00205E2020202000, 0x2002001008020084, 54, 67584),
    Magic!(0x00403E4040404000, 0x01001008722C0001, 54, 68608),
    Magic!(0x00807E8080808000, 0x2200008041020004, 53, 69632),
    Magic!(0x007E010101010100, 0x20028D020E20C200, 53, 71680),
    Magic!(0x007C020202020200, 0x4020A00280400080, 54, 73728),
    Magic!(0x007A040404040400, 0x000012C020820200, 54, 74752),
    Magic!(0x0076080808080800, 0x0000402200691200, 54, 75776),
    Magic!(0x006E101010101000, 0x0811009490180100, 54, 76800),
    Magic!(0x005E202020202000, 0x104080C200940080, 54, 77824),
    Magic!(0x003E404040404000, 0x0804302201A80400, 54, 78848),
    Magic!(0x007E808080808000, 0x00000100408C2200, 53, 79872),
    Magic!(0x7E01010101010100, 0x0200802045020612, 52, 81920),
    Magic!(0x7C02020202020200, 0x0340002090824501, 53, 86016),
    Magic!(0x7A04040404040400, 0xC804144840A00101, 53, 88064),
    Magic!(0x7608080808080800, 0x801020D001005419, 53, 90112),
    Magic!(0x6E10101010101000, 0x1111001C82101801, 53, 92160),
    Magic!(0x5E20202020202000, 0x8049000400980601, 53, 94208),
    Magic!(0x3E40404040404000, 0x04310000A4520001, 53, 96256),
    Magic!(0x7E80808080808000, 0x040100002A014081, 52, 98304),
];

/// Total memory size allocated for rook move tables.
///
/// The `ROOK_TABLE_SIZE` constant defines the total number of bytes used
/// by the rook move table, enabling efficient indexing and move generation
/// without recalculating table bounds.
pub const ROOK_TABLE_SIZE: usize = 102400;

/// Array of precomputed `MagicEntry` instances for bishop moves.
///
/// Each entry in `BISHOP_MAGICS` corresponds to a specific square on the chess board,
/// detailing the bitmask, magic number, shift, and offset values necessary for quick
/// bishop move generation from that square.
pub const BISHOP_MAGICS: [MagicEntry; Square::NUM_SQUARES] = [
    Magic!(0x0040201008040200, 0x0860600201910090, 58, 0),
    Magic!(0x0000402010080400, 0x0008100400802044, 59, 64),
    Magic!(0x0000004020100A00, 0x3004440402404002, 59, 96),
    Magic!(0x0000000040221400, 0x0428204041000818, 59, 128),
    Magic!(0x0000000002442800, 0x0009104000038081, 59, 160),
    Magic!(0x0000000204085000, 0x02060A0220000000, 59, 192),
    Magic!(0x0000020408102000, 0x0210821010A42028, 59, 224),
    Magic!(0x0002040810204000, 0x8200104508205000, 58, 256),
    Magic!(0x0020100804020000, 0x1125600202080900, 59, 320),
    Magic!(0x0040201008040000, 0x0880124405420200, 59, 352),
    Magic!(0x00004020100A0000, 0x00900C1400860004, 59, 384),
    Magic!(0x0000004022140000, 0x8A00890401000800, 59, 416),
    Magic!(0x0000000244280000, 0x0201040421140080, 59, 448),
    Magic!(0x0000020408500000, 0x00000104A00C2000, 59, 480),
    Magic!(0x0002040810200000, 0x0802842205142004, 59, 512),
    Magic!(0x0004081020400000, 0x0120020222010480, 59, 544),
    Magic!(0x0010080402000200, 0x40C0100810814618, 59, 576),
    Magic!(0x0020100804000400, 0x0090052404280049, 59, 608),
    Magic!(0x004020100A000A00, 0x0061001004008290, 57, 640),
    Magic!(0x0000402214001400, 0x0403042804110001, 57, 768),
    Magic!(0x0000024428002800, 0x090C000610220910, 57, 896),
    Magic!(0x0002040850005000, 0x0221001020A01002, 57, 1024),
    Magic!(0x0004081020002000, 0x0218800D08215000, 59, 1152),
    Magic!(0x0008102040004000, 0x00020900404C1400, 59, 1184),
    Magic!(0x0008040200020400, 0x00044004A0080100, 59, 1216),
    Magic!(0x0010080400040800, 0x0010300224041482, 59, 1248),
    Magic!(0x0020100A000A1000, 0x8400220090008A02, 57, 1280),
    Magic!(0x0040221400142200, 0x4582008148018022, 55, 1408),
    Magic!(0x0002442800284400, 0x0810068004008402, 55, 1920),
    Magic!(0x0004085000500800, 0x0010404082031010, 57, 2432),
    Magic!(0x0008102000201000, 0xA008124212010C09, 59, 2560),
    Magic!(0x0010204000402000, 0x8024010002250500, 59, 2592),
    Magic!(0x0004020002040800, 0x0204212008440484, 59, 2624),
    Magic!(0x0008040004081000, 0xC102020200E14800, 59, 2656),
    Magic!(0x00100A000A102000, 0xA08C040200010200, 57, 2688),
    Magic!(0x0022140014224000, 0x0100020080080081, 55, 2816),
    Magic!(0x0044280028440200, 0x0840010100003040, 55, 3328),
    Magic!(0x0008500050080400, 0x0321010200070062, 57, 3840),
    Magic!(0x0010200020100800, 0x1028209280042201, 59, 3968),
    Magic!(0x0020400040201000, 0x0000808A08088220, 59, 4000),
    Magic!(0x0002000204081000, 0x000844100D000822, 59, 4032),
    Magic!(0x0004000408102000, 0x4004843008014A84, 59, 4064),
    Magic!(0x000A000A10204000, 0x8000802808008F01, 57, 4096),
    Magic!(0x0014001422400000, 0x0000004208000984, 57, 4224),
    Magic!(0x0028002844020000, 0x09202004E0800C01, 57, 4352),
    Magic!(0x0050005008040200, 0x0018010802002020, 57, 4480),
    Magic!(0x0020002010080400, 0x1222020202020408, 59, 4608),
    Magic!(0x0040004020100800, 0x2002048202008880, 59, 4640),
    Magic!(0x0000020408102000, 0x1852211002100020, 59, 4672),
    Magic!(0x0000040810204000, 0x5000808450029000, 59, 4704),
    Magic!(0x00000A1020400000, 0x5000D18088210320, 59, 4736),
    Magic!(0x0000142240000000, 0x80E00320C6080008, 59, 4768),
    Magic!(0x0000284402000000, 0x1012106450440200, 59, 4800),
    Magic!(0x0000500804020000, 0x0802042044010001, 59, 4832),
    Magic!(0x0000201008040200, 0x1020212B0A108080, 59, 4864),
    Magic!(0x0000402010080400, 0x0808080838802408, 59, 4896),
    Magic!(0x0002040810204000, 0x25004042100F2001, 58, 4928),
    Magic!(0x0004081020400000, 0x082814A201102880, 59, 4992),
    Magic!(0x000A102040000000, 0xC001020200840C40, 59, 5024),
    Magic!(0x0014224000000000, 0x0400008080420A01, 59, 5056),
    Magic!(0x0028440200000000, 0x0400000120042400, 59, 5088),
    Magic!(0x0050080402000000, 0x01061504102E0200, 59, 5120),
    Magic!(0x0020100804020000, 0x4220180210040108, 59, 5152),
    Magic!(0x0040201008040200, 0x0004281002408100, 58, 5184),
];

/// The size of the lookup table for bishop moves in chess.
///
/// This constant represents the number of entries needed to store precomputed
/// moves for a bishop on an 8x8 chessboard. The size is determined based on
/// the possible blocker configurations and the efficiency of lookup using
/// magic bitboards.
pub const BISHOP_TABLE_SIZE: usize = 5248;

/// Calculates the magic index for a given square and blocker configuration.
///
/// This function takes in a `MagicEntry` and a set of blockers (`BitBoard`),
/// then computes a unique index for accessing precomputed moves in a magic
/// bitboard table. The index calculation involves applying the magic
/// number multiplication and shifting as defined in the magic bitboard
/// algorithm.
fn magic_index(entry: &MagicEntry, blockers: BitBoard) -> usize {
    let blockers: u64 = blockers.0 & entry.mask;
    let hash: u64 = blockers.wrapping_mul(entry.magic);
    let index: usize = (hash >> entry.shift) as usize;

    entry.offset as usize + index
}

/// Computes all possible moves for a piece based on deltas and board blockers.
///
/// Given movement `deltas`, the starting `square`, and a `blockers` bitboard,
/// this function calculates the valid moves for a sliding piece (e.g., bishop,
/// rook) by iterating in each delta direction until a blocker or board edge is
/// reached. Moves are added to a `BitBoard` result and returned.
fn moves(deltas: &[(i8, i8)], square: Square, blockers: BitBoard) -> BitBoard {
    let mut moves: BitBoard = BitBoard::EMPTY;
    let rank: i8 = square.rank() as i8;
    let file: i8 = square.file() as i8;

    for &(dr, df) in deltas {
        let mut new_rank: i8 = rank + dr;
        let mut new_file: i8 = file + df;

        while (0..8).contains(&new_rank) && (0..8).contains(&new_file) {
            let new_square: Square =
                Square::from_file_rank(unsafe { transmute(new_file as u8) }, unsafe {
                    transmute(new_rank as u8)
                });
            let target_bitboard: BitBoard = new_square.to_bitboard();
            moves |= target_bitboard;

            if target_bitboard & blockers != BitBoard::EMPTY {
                break;
            }

            new_rank += dr;
            new_file += df;
        }
    }

    moves
}

/// Builds a precomputed table of moves for a sliding piece using magic bitboards.
///
/// This function iterates over each square and blocker configuration to
/// populate a move table using a specified set of movement `deltas` and
/// magic entries for each square. The resulting table is stored in a
/// `Vec<BitBoard>`, where each entry is a precomputed `BitBoard` of moves
/// for a given square/blocker configuration.
pub fn make_table(
    size: usize,
    deltas: &[(i8, i8)],
    magics: &[MagicEntry; Square::NUM_SQUARES],
) -> Vec<BitBoard> {
    let mut table: Vec<BitBoard> = vec![BitBoard::EMPTY; size];

    for square_index in 0..Square::NUM_SQUARES {
        let magic_entry: &MagicEntry = &magics[square_index];
        let mask: BitBoard = BitBoard(magic_entry.mask);
        let square: Square = Square::from_index(square_index);
        let mut blockers: BitBoard = BitBoard::EMPTY;
        loop {
            let moves: BitBoard = moves(deltas, square, blockers);
            table[magic_index(magic_entry, blockers)] = moves;

            blockers.0 = blockers.0.wrapping_sub(mask.0) & mask.0;
            if blockers.is_empty() {
                break;
            }
        }
    }

    table
}

/// Writes a precomputed moves table to an output in Rust source format.
///
/// This function takes a table of `BitBoard`s and writes it as a Rust array
/// with the specified `name` to the provided output writer. This is useful for
/// generating a source file that includes a precomputed move table for a sliding
/// chess piece.
pub fn write_table(name: &str, table: &[BitBoard], out: &mut impl Write) -> Result<()> {
    write!(out, "const {}_ATTACKS: [u64; {}] = [", name, table.len())?;

    for entry in table {
        write!(out, "{},", entry.0)?;
    }
    write!(out, "];")?;
    Ok(())
}

/// Writes a Rust array declaration for magic entries to the specified output writer.
///
/// The function outputs a constant Rust array in source format, with each `MagicEntry`
/// represented in hexadecimal format for readability and conciseness.
pub fn write_magic(
    name: &str,
    magics: &[MagicEntry; Square::NUM_SQUARES],
    out: &mut impl Write,
) -> Result<()> {
    write!(
        out,
        "const {}_MAGICS: [MagicEntry; Square::NUM_SQUARES] = [",
        name
    )?;

    for entry in magics {
        write!(
            out,
            "Magic!(0x{:016X}, 0x{:016X}, {}, {}), ",
            entry.mask, entry.magic, entry.shift, entry.offset
        )?;
    }
    write!(out, "];")?;
    Ok(())
}
