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

use core::array::IntoIter;
use core::fmt;
use core::ops::{Deref, DerefMut};

use crate::Move;

#[cfg(target_pointer_width = "64")]
const MAX_MOVES: usize = 252;
#[cfg(target_pointer_width = "32")]
const MAX_MOVES: usize = 254;
#[cfg(target_pointer_width = "16")]
const MAX_MOVES: usize = 255;

// This implementation is based on the `MoveList` structure from Pleco,
// an efficient chess library, licensed under the MIT License.
// Copyright (c) 2022 Pleco
// Source: https://github.com/pleco-rs/Pleco/blob/main/pleco/src/core/move_list.rs

/// A container for storing and managing a list of [`Move`]s in a chess position.
///
/// `MoveList` efficiently holds up to `MAX_MOVES` moves, depending on the target architecture.
/// It keeps track of the number of moves currently stored, allowing for fast appending, clearing, and iteration.
///
/// # Notes
///
/// - `MAX_MOVES` may vary depending on the target platform's pointer width (`16`, `32`, or `64` bits).
/// - The list operates similarly to a small, fixed-capacity vector.
///
/// # Example
///
/// ```
/// # use laura_core::*;
///
/// let mut move_list = MoveList::default();
/// assert_eq!(move_list.len(), 0);
///
/// let mv = Move::new(Square::E2, Square::E3, MoveType::Quiet);
/// move_list.push(mv);
///
/// assert_eq!(move_list.len(), 1);
/// assert_eq!(move_list[0], mv);
/// ```
#[derive(Clone, Debug)]
pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl IntoIterator for MoveList {
    type Item = Move;
    type IntoIter = core::iter::Take<IntoIter<Move, MAX_MOVES>>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter(self.moves).take(self.len)
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;
    type IntoIter = core::slice::Iter<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.moves[..self.len].iter()
    }
}

impl Deref for MoveList {
    type Target = [Move];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl DerefMut for MoveList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl Default for MoveList {
    /// Creates a new, empty `MoveList` with all moves initialized to `Move::null()`.
    ///
    /// The list will have a length of `0` and a capacity of `MAX_MOVES`.
    /// All entries are pre-filled with `Move::null()` to ensure valid memory and avoid uninitialized data.
    ///
    /// # Returns
    ///
    /// A fresh `MoveList` ready to store moves.
    ///
    /// # Example
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let move_list = MoveList::default();
    /// assert_eq!(move_list.len(), 0);
    /// ```
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
        if self.is_empty() {
            return write!(f, "MoveList: (0 moves)");
        }

        writeln!(f, "MoveList ({} moves):", self.len)?;
        for (index, mv) in self.moves.iter().take(self.len).enumerate() {
            writeln!(f, "{}: {}", index + 1, mv)?;
        }
        Ok(())
    }
}

impl MoveList {
    /// Adds a [`Move`] to the `MoveList`.
    ///
    /// If the list has not yet reached its maximum capacity (`MAX_MOVES`), the move is appended.
    /// If the list is full, the move is silently ignored.
    ///
    /// This function guarantees that the `MoveList` will never grow beyond its fixed capacity,
    /// avoiding out-of-bounds memory access.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mut move_list = MoveList::default();
    ///
    /// let mv1 = Move::new(Square::E2, Square::E4, MoveType::DoublePawn);
    /// let mv2 = Move::new(Square::D7, Square::D5, MoveType::DoublePawn);
    ///
    /// move_list.push(mv1);
    /// move_list.push(mv2);
    ///
    /// assert_eq!(move_list.len(), 2);
    /// assert_eq!(move_list[0], mv1);
    /// assert_eq!(move_list[1], mv2);
    /// ```
    ///
    /// # Expected behavior
    ///
    /// If the list reaches its maximum capacity, additional calls to `push` will have no effect.
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mut move_list = MoveList::default();
    /// const MAX_MOVES: usize = 252; // for 64 bits target pointer
    ///
    /// for _ in 0..(MAX_MOVES + 10) {
    ///     let mv = Move::new(Square::A2, Square::A3, MoveType::Quiet);
    ///     move_list.push(mv);
    /// }
    ///
    /// assert_eq!(move_list.len(), MAX_MOVES);
    /// ```
    #[inline(always)]
    pub fn push(&mut self, mv: Move) {
        if self.len < MAX_MOVES {
            self.moves[self.len] = mv;
            self.len += 1;
        }
    }

    /// Returns a slice containing the moves currently stored in the `MoveList`.
    ///
    /// Only the first `len` moves are included; unused slots in the internal array are excluded.
    ///
    /// This allows efficient and safe iteration over the moves without exposing the uninitialized (or `Move::null()`) entries.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mut move_list = MoveList::default();
    ///
    /// let mv1 = Move::new(Square::E2, Square::E4, MoveType::DoublePawn);
    /// let mv2 = Move::new(Square::D7, Square::D5, MoveType::DoublePawn);
    ///
    /// move_list.push(mv1);
    /// move_list.push(mv2);
    ///
    /// let moves = move_list.as_slice();
    ///
    /// assert_eq!(moves.len(), 2);
    /// assert_eq!(moves[0], mv1);
    /// assert_eq!(moves[1], mv2);
    /// ```
    ///
    /// # Expected behavior
    ///
    /// Even though the underlying array has capacity for `MAX_MOVES`,  
    /// only the portion up to the current `len` is returned.
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let move_list = MoveList::default();
    /// assert!(move_list.as_slice().is_empty());
    /// ```
    #[inline(always)]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    /// Returns a mutable slice containing the moves currently stored in the `MoveList`.
    ///
    /// Only the first `len` moves are included; unused slots beyond `len` are excluded.
    ///
    /// This allows safe in-place modification of the stored moves without risking access
    /// to uninitialized (`Move::null()`) entries.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mut move_list = MoveList::default();
    ///
    /// let mv1 = Move::new(Square::E2, Square::E4, MoveType::DoublePawn);
    /// let mv2 = Move::new(Square::D7, Square::D5, MoveType::DoublePawn);
    ///
    /// move_list.push(mv1);
    /// move_list.push(mv2);
    ///
    /// // Modify the first move
    /// move_list.as_mut_slice()[0] = Move::new(Square::E2, Square::E3, MoveType::Quiet);
    ///
    /// assert_eq!(move_list[0].get_dest(), Square::E3);
    /// ```
    ///
    /// # Expected behavior
    ///
    /// The returned mutable slice is always exactly `len` elements long,  
    /// regardless of the full array capacity.
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mut move_list = MoveList::default();
    /// assert_eq!(move_list.as_mut_slice().len(), 0);
    /// ```
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [Move] {
        &mut self.moves[0..self.len]
    }

    /// Returns the number of moves currently stored in the `MoveList`.
    ///
    /// This represents the number of valid moves in the list,  
    /// which may be less than the maximum capacity (`MAX_MOVES`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mut move_list = MoveList::default();
    /// assert_eq!(move_list.len(), 0);
    ///
    /// move_list.push(Move::new(Square::E2, Square::E4, MoveType::DoublePawn));
    /// assert_eq!(move_list.len(), 1);
    ///
    /// move_list.push(Move::new(Square::D7, Square::D5, MoveType::DoublePawn));
    /// assert_eq!(move_list.len(), 2);
    /// ```
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the `MoveList` contains no moves.
    ///
    /// This checks whether the list is currently empty (`len == 0`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mut move_list = MoveList::default();
    /// assert!(move_list.is_empty());
    ///
    /// move_list.push(Move::new(Square::E2, Square::E4, MoveType::DoublePawn));
    /// assert!(!move_list.is_empty());
    ///
    /// move_list.clear();
    /// assert!(move_list.is_empty());
    /// ```
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Clears all moves from the `MoveList`.
    ///
    /// This resets the list to an empty state by setting the length to zero.
    /// The underlying move data is not overwritten, but will be replaced as new moves are added.
    ///
    /// # Examples
    ///
    /// ```
    /// # use laura_core::*;
    ///
    /// let mut move_list = MoveList::default();
    ///
    /// move_list.push(Move::new(Square::E2, Square::E4, MoveType::DoublePawn));
    /// move_list.push(Move::new(Square::D7, Square::D5, MoveType::DoublePawn));
    ///
    /// assert_eq!(move_list.len(), 2);
    ///
    /// move_list.clear();
    ///
    /// assert!(move_list.is_empty());
    /// assert_eq!(move_list.len(), 0);
    /// ```
    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }
}
