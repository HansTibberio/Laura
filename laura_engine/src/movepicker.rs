/*
    Laura: A single-threaded UCI chess engine written in Rust.

    Copyright (C) 2024-2025 HansTibberio <hanstiberio@proton.me>

    Laura is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Laura is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Laura. If not, see <https://www.gnu.org/licenses/>.
*/

// src/movepicker.rs

//! Move picker for search.

use laura_core::{quiet_moves, tactical_moves, Board, Move, MoveList, MoveType, Piece, PieceType};

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum Stage {
    TTMove,
    Captures,
    Killers,
    Quiets,
    Done,
}

pub struct MovePicker {
    tt_move: Option<Move>,
    killer_move: [Option<Move>; 2],
    killer_index: usize,
    stage: Stage,
    moves: MoveList,
    index: usize,
    pub skip_quiets: bool,
}

impl MovePicker {
    pub fn new(tt_move: Option<Move>, killer_move: [Option<Move>; 2]) -> Self {
        Self {
            tt_move,
            killer_move,
            stage: Stage::TTMove,
            moves: MoveList::default(),
            index: 0,
            skip_quiets: false,
            killer_index: 0,
        }
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn next(&mut self, position: &Board) -> Option<Move> {
        loop {
            if self.stage == Stage::Done {
                return None;
            }
            if self.stage == Stage::TTMove {
                self.stage = Stage::Captures;
                if let Some(tt_move) = self.tt_move {
                    return Some(tt_move);
                }
            }
            if self.stage == Stage::Captures {
                if self.index == 0 {
                    self.moves = tactical_moves!(position);
                }
                self.score_captures(position);
                if let Some(mv) = self.yield_once() {
                    return Some(mv);
                }
                self.stage = if self.skip_quiets {
                    Stage::Done
                } else {
                    Stage::Killers
                };
                self.index = 0;
            }
            if self.stage == Stage::Killers {
                let moves: MoveList = quiet_moves!(position);
                for i in self.killer_index..2 {
                    self.killer_index += 1;
                    match self.killer_move[i] {
                        Some(killer) if Some(killer) != self.tt_move && moves.contains(&killer) => {
                            return Some(killer);
                        }
                        _ => continue,
                    }
                }
                self.stage = Stage::Quiets;
            }
            if self.stage == Stage::Quiets {
                if self.index == 0 {
                    self.moves = quiet_moves!(position);
                }
                if let Some(mv) = self.yield_once() {
                    return Some(mv);
                }
                self.stage = Stage::Done;
                self.index = 0;
            }
        }
    }

    fn yield_once(&mut self) -> Option<Move> {
        while self.index < self.moves.len() {
            let mv: Move = self.moves[self.index];
            self.index += 1;
            if Some(mv) == self.tt_move
                || Some(mv) == self.killer_move[0]
                || Some(mv) == self.killer_move[1]
            {
                continue;
            }
            return Some(mv);
        }
        None
    }

    fn score_captures(&mut self, position: &Board) {
        self.moves.sort_unstable_by_key(|mv| {
            let victim_value: i32 = position
                .piece_on(mv.get_dest())
                .map(piece_value)
                .unwrap_or(0);
            let mut attacker_value: i32 = position
                .piece_on(mv.get_src())
                .map(piece_value)
                .unwrap_or(0);

            if mv.get_type() == MoveType::CapPromoQueen {
                attacker_value = 936
            }

            -(victim_value * 100 - attacker_value)
        });
    }
}

fn piece_value(piece: Piece) -> i32 {
    match piece.piece_type() {
        PieceType::Pawn => 94,
        PieceType::Knight => 281,
        PieceType::Bishop => 297,
        PieceType::Rook => 512,
        PieceType::Queen => 936,
        PieceType::King => 0,
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::MovePicker;
    use laura_core::Board;

    #[test]
    fn test_picker() {
        let board: Board = Board::from_str("2r1k3/1P6/8/8/5b2/6P1/P7/2Q3K1 w - - 0 1").unwrap();
        let mut picker: MovePicker = MovePicker::new(None, [None, None]);
        while let Some(mv) = picker.next(&board) {
            println!("{}", mv);
        }
    }

    #[test]
    fn test_captures() {
        let board: Board = Board::from_str("2r1k3/1P6/8/8/5b2/6P1/P7/2Q3K1 w - - 0 1").unwrap();
        let mut picker: MovePicker = MovePicker::new(None, [None, None]);
        picker.skip_quiets = true;
        while let Some(mv) = picker.next(&board) {
            println!("{}", mv);
        }
    }
}
