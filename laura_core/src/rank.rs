use std::mem::transmute;
use std::fmt;

/// Enum representing the ranks (rows) on a chessboard.
/// Ranks are numbered from 'One' (1) to 'Eight' (8).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
#[repr(u8)]
pub enum Rank {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

/// Implementing `Display` for `Rank` to convert the enum to a string representation (One-Eight).
impl fmt::Display for Rank {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::One => write!(f, "One"),
            Self::Two => write!(f, "Two"),
            Self::Three => write!(f, "Three"),
            Self::Four => write!(f, "Four"),
            Self::Five => write!(f, "Five"),
            Self::Six => write!(f, "Six"),
            Self::Seven => write!(f, "Seven"),
            Self::Eight => write!(f, "Eight"),
        }
    }
}

impl Rank {
    
    /// Total number of ranks (8 in standard chess).
    pub const NUM_RANKS: usize = 8;
    /// Array containing all possible ranks (One to Eight).
    pub const ALL: [Self; 8] = [Self::One, Self::Two, Self::Three, Self::Four, Self::Five, Self::Six, Self::Seven, Self::Eight];

    /// Converts an index (0-7) to the corresponding `Rank`.
    #[inline]
    pub const fn from_index(index: usize) -> Rank {
        unsafe { transmute(index as u8 & 7) }
    }
    
    /// Converts a `Rank` into its corresponding index (0 for One, 7 for Eight).
    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    /// Gets rank above, wraps Eight->One
    #[inline]
    pub const fn up(self) -> Self {
        unsafe { transmute((self as u8 + 1) & 7) }
    }

    /// Gets rank below, wraps One->Eight
    #[inline]
    pub const fn down(self) -> Self {
        unsafe { transmute((self as u8).wrapping_sub(1) & 7) }
    }
}

#[test]
fn test() {
    let rank: Rank = Rank::from_index(6);
    println!("Rank: {}, Index: {}", rank, rank.to_index());
    println!("Down: {}, Up: {}", rank.down(), rank.up());
}