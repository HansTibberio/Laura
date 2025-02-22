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
use core::ops::Not;

/// Enum representing the color of a piece/player in a chess game.
/// It can either be `White` or `Black`, corresponding to the two players in a chess game.
///
/// The `Color` enum is used to distinguish between the two sides in a chess game, 
/// where one player controls the white pieces and the other controls the black pieces.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub enum Color {
    /// Represents the White side.
    White,

    /// Represents the Black side.
    Black,
}

impl Not for Color {
    type Output = Color;

    /// Returns the opposite color.
    /// If the current color is `White`, it returns `Black`, and vice versa.
    fn not(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// Implements printing for the `Color` enum.
/// Converts the enum to a user-friendly string when printed.
impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Color::White => "w",
                Color::Black => "b",
            }
        )
    }
}
