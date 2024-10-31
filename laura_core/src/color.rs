use std::fmt;
use std::ops::Not;


/// Enum representing the color of a piece/player in a chess game.
/// It can either be `White` or `Black`.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Debug, Hash)]
pub enum Color {
    White,
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
                Color::White => "White",
                Color::Black => "Black",
            }
        )
    }
}