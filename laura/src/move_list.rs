use crate::moves::Move;

#[cfg(target_pointer_width = "128")]
pub const MAX_MOVES: usize = 248;
#[cfg(target_pointer_width = "64")]
pub const MAX_MOVES: usize = 252;
#[cfg(target_pointer_width = "32")]
pub const MAX_MOVES: usize = 254;
#[cfg(any(target_pointer_width = "16", target_pointer_width = "8",))]
pub const MAX_MOVES: usize = 255;

/// A struct that holds a list of `Move` objects for a given position in a chess game.
/// 
/// From Pleco: https://github.com/pleco-rs/Pleco/blob/main/pleco/src/core/move_list.rs
/// 
/// The `MoveList` allows storing and managing moves, and tracks the current number of moves.
/// 
/// ### Fields
/// - `index`: An array of `Move` objects, up to `MAX_MOVES`.
/// - `len`: The current number of moves stored in the list.
#[derive(Clone, Debug)]
pub struct MoveList {
    index: [Move; MAX_MOVES],
    len: usize,
}

impl Default for MoveList {
    /// Creates a new, empty `MoveList` initialized with the default values.
    ///
    /// ### Returns
    /// A `MoveList` with all moves set to `Move::null()` and length set to 0.
    #[inline]
    fn default() -> Self {
        MoveList {
            index: [Move::null(); MAX_MOVES],
            len: 0,
        }
    }
}

impl MoveList {
    
    /// Adds a move to the list.
    ///
    /// If the list is already at maximum capacity, the move is not added.
    #[inline(always)]
    pub fn push(&mut self, mv: Move) {
        self.push_move(mv);
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

    /// Adds a move to the list if there is available space.
    #[inline(always)]
    fn push_move(&mut self, mv: Move) {
        if self.len() < MAX_MOVES {
            unsafe { self.unchecked_push_move(mv) }
        }
    }

    /// Unsafely adds a move to the list without checking if the list is at capacity.
    ///
    /// ## Safety
    /// This function assumes that the length of the list (`len`) is less than `MAX_MOVES`.
    #[inline(always)]
    unsafe fn unchecked_push_move(&mut self, mv: Move) {
        let end: &mut Move = self.index.get_unchecked_mut(self.len);
        *end = mv;
        self.len += 1;
    }
}

#[test]
fn test_list(){
    use crate::square::Square;
    use crate::moves::MoveType;

    let mut list: MoveList = MoveList::default();
    assert_eq!(list.is_empty(), true);

    list.push(Move::new(Square::E2, Square::E3, MoveType::Quiet));
    list.push(Move::new(Square::D7, Square::D5, MoveType::DoublePawn));
    assert_eq!(list.len(), 2);
}