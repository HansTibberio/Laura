use std::fmt;
use std::mem::transmute;

/// Enum representing the files (columns) on a chessboard.
/// Files are labeled from 'A' to 'H'.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
#[repr(u8)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

/// Implementing `Display` for `File` to convert the enum to a string representation (A-H).
impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::E => write!(f, "E"),
            Self::F => write!(f, "F"),
            Self::G => write!(f, "G"),
            Self::H => write!(f, "H"),
        }
    }
}

impl File {
    /// Total number of files (8 in standard chess).
    pub const NUM_FILES: usize = 8;
    /// Array containing all possible files (A to H).
    pub const ALL: [Self; Self::NUM_FILES] = [
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::E,
        Self::F,
        Self::G,
        Self::H,
    ];

    /// Converts an index (0-7) to the corresponding `File`.
    #[inline]
    pub const fn from_index(index: usize) -> File {
        unsafe { transmute(index as u8 & 7) }
    }

    /// Converts a `File` into its corresponding index (0 for A, 7 for H).
    #[inline]
    pub const fn to_index(self) -> usize {
        self as usize
    }

    /// Gets file to the right, wraps H->A
    #[inline]
    pub const fn right(self) -> Self {
        unsafe { transmute((self as u8 + 1) & 7) }
    }

    /// Gets file to the left, wraps A->H
    #[inline]
    pub const fn left(self) -> Self {
        unsafe { transmute((self as u8).wrapping_sub(1) & 7) }
    }
}

#[test]
fn test() {
    let file: File = File::from_index(4);
    println!("File: {}, Index: {}", file, file.to_index());
    println!("Left: {}, Right: {}", file.left(), file.right())
}
