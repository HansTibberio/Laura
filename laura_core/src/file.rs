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

use core::fmt;
use core::mem::transmute;

/// Enum representing the files (columns) on a chessboard.
/// Files are labeled from 'A' to 'H', with 'A' being the leftmost column and 'H' the rightmost.
/// The `File` enum is used to identify the specific column a piece resides on during the game.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
#[repr(u8)]
pub enum File {
    /// The 'A' file, the leftmost column on the chessboard.
    A,

    /// The 'B' file.
    B,

    /// The 'C' file.
    C,

    /// The 'D' file.
    D,

    /// The 'E' file.
    E,

    /// The 'F' file.
    F,

    /// The 'G' file.
    G,

    /// The 'H' file, the rightmost column on the chessboard.
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
