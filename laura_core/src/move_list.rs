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

use std::array::IntoIter;
use std::fmt;

use crate::Move;

#[cfg(target_pointer_width = "64")]
pub const MAX_MOVES: usize = 252;
#[cfg(target_pointer_width = "32")]
pub const MAX_MOVES: usize = 254;
#[cfg(target_pointer_width = "16")]
pub const MAX_MOVES: usize = 255;

// This implementation is based on the `MoveList` structure from Pleco,
// an efficient chess library, licensed under the MIT License.
// Copyright (c) 2022 Pleco
// Source: https://github.com/pleco-rs/Pleco/blob/main/pleco/src/core/move_list.rs

/// A struct that holds a list of `Move` objects for a given position in a chess game.
///
/// The `MoveList` allows storing and managing moves, and tracks the current number of moves.
///
/// ### Fields
/// - `moves`: An array of `Move` objects, up to `MAX_MOVES`.
/// - `len`: The current number of moves stored in the list.
#[derive(Clone, Debug)]
pub struct MoveList {
    pub moves: [Move; MAX_MOVES],
    len: usize,
}

impl IntoIterator for MoveList {
    type Item = Move;
    type IntoIter = std::iter::Take<IntoIter<Move, MAX_MOVES>>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(self.moves).take(self.len)
    }
}

impl Default for MoveList {
    /// Creates a new, empty `MoveList` initialized with the default values.
    ///
    /// ### Returns
    /// A `MoveList` with all moves set to `Move::null()` and length set to 0.
    #[inline]
    fn default() -> Self {
        MoveList {
            moves: [Move::null(); MAX_MOVES],
            len: 0,
        }
    }
}

/// Implements the `fmt::Display` trait for `MoveList`, enabling formatted output.
///
/// This implementation formats the `MoveList` for display, showing the total
/// number of moves and listing each move sequentially. If the list is empty,
/// it displays "MoveList: (0) []" to indicate no moves are present.
impl fmt::Display for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.moves.is_empty() {
            return write!(f, "MoveList: (0) []");
        }

        writeln!(f, "MoveList ({} moves):", self.len)?;
        for (index, mv) in self.moves.iter().take(self.len).enumerate() {
            writeln!(f, "{}: {}", index + 1, mv)?;
        }
        Ok(())
    }
}

impl MoveList {
    /// Adds a move to the list.
    ///
    /// If the list is already at maximum capacity, the move is not added.
    #[inline(always)]
    pub fn push(&mut self, mv: Move) {
        self.moves[self.len] = mv;
        self.len += 1;
    }

    /// Returns the number of moves currently stored in the list.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Checks whether the move list is empty.
    ///
    ///`true` if the list is empty, `false` otherwise.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[test]
fn test_list() {
    use crate::moves::MoveType;
    use crate::square::Square;

    let mut list: MoveList = MoveList::default();
    assert_eq!(list.is_empty(), true);

    list.push(Move::new(Square::E2, Square::E3, MoveType::Quiet));
    list.push(Move::new(Square::D7, Square::D5, MoveType::DoublePawn));
    println!("{}", list);
    assert_eq!(list.len(), 2);
}

#[test]
fn test_movelist_iter() {
    use crate::{Board, MoveList};

    let board: Board = Board::default();
    let moves: MoveList = board.gen_moves::<true>();

    for mv in moves {
        println!("{}", mv);
    }
}
