/*
    Laura: A multi-threaded UCI chess engine written in Rust.

    Copyright (C) 2024-2026 HansTibberio <hanstiberio@proton.me>

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

use crate::sse::SEE;
use crate::tables::HistoryTable;
use laura_core::{Board, Color, Move, MoveList, quiet_moves, tactical_moves};

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub enum Stage {
    TTMove,
    GenCaptures,
    GoodCaptures,
    Killers,
    Quiets,
    BadCaptures,
    Done,
}

pub struct MovePicker {
    tt_move: Option<Move>,
    killer_move: [Option<Move>; 2],
    killer_index: usize,
    stage: Stage,

    good_captures: MoveList,
    bad_captures: MoveList,
    quiets: MoveList,

    index: usize,
    pub skip_quiets: bool,
}

impl MovePicker {
    pub fn new(tt_move: Option<Move>, killer_move: [Option<Move>; 2]) -> Self {
        Self {
            tt_move,
            killer_move,
            stage: Stage::TTMove,
            good_captures: MoveList::default(),
            bad_captures: MoveList::default(),
            quiets: MoveList::default(),
            index: 0,
            skip_quiets: false,
            killer_index: 0,
        }
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn next(&mut self, position: &Board, history: &HistoryTable) -> Option<Move> {
        loop {
            match self.stage {
                Stage::Done => {
                    return None;
                }
                Stage::TTMove => {
                    self.stage = Stage::GenCaptures;
                    if let Some(tt_move) = self.tt_move {
                        return Some(tt_move);
                    }
                }
                Stage::GenCaptures => {
                    self.generate_and_score_captures(position);
                    self.stage = Stage::GoodCaptures;
                    self.index = 0;
                }
                Stage::GoodCaptures => {
                    if let Some(mv) = self.next_from_list(&self.good_captures.clone()) {
                        return Some(mv);
                    }
                    self.stage = if self.skip_quiets {
                        Stage::BadCaptures
                    } else {
                        Stage::Killers
                    };
                    self.index = 0;
                }
                Stage::Killers => {
                    if self.index == 0 {
                        self.quiets = quiet_moves!(position);
                        self.score_quiets(position, history);
                    }

                    for i in self.killer_index..2 {
                        self.killer_index += 1;
                        match self.killer_move[i] {
                            Some(killer)
                                if Some(killer) != self.tt_move
                                    && self.quiets.contains(&killer) =>
                            {
                                return Some(killer);
                            }
                            _ => continue,
                        }
                    }

                    self.stage = Stage::Quiets;
                }
                Stage::Quiets => {
                    if let Some(mv) = self.next_from_list(&self.quiets.clone()) {
                        return Some(mv);
                    }
                    self.stage = Stage::BadCaptures;
                    self.index = 0;
                }
                Stage::BadCaptures => {
                    if let Some(mv) = self.next_from_list(&self.bad_captures.clone()) {
                        return Some(mv);
                    }
                    self.stage = Stage::Done;
                    self.index = 0;
                }
            }
        }
    }

    fn generate_and_score_captures(&mut self, position: &Board) {
        let all_captures: MoveList = tactical_moves!(position);

        self.good_captures.clear();
        self.bad_captures.clear();

        for mv in all_captures.iter() {
            if Some(*mv) == self.tt_move {
                continue;
            }

            let is_good: bool = SEE::see(position, *mv, 0);

            if is_good {
                self.good_captures.push(*mv);
            } else {
                self.bad_captures.push(*mv);
            }
        }
    }

    fn next_from_list(&mut self, list: &MoveList) -> Option<Move> {
        while self.index < list.len() {
            let mv: Move = list[self.index];
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

    fn score_quiets(&mut self, position: &Board, history: &HistoryTable) {
        let color: Color = position.side();

        self.quiets
            .sort_unstable_by_key(|mv| -history.get_score(*mv, color));
    }
}

#[cfg(test)]
mod test {
    use super::MovePicker;
    use crate::sse::SEE;
    use crate::tables::HistoryTable;
    use laura_core::Board;
    use std::str::FromStr;

    #[test]
    fn test_good_bad_captures() {
        let board: Board = Board::from_str("4k3/4p3/8/1p1p4/r3Q3/8/8/4K3 w - - 0 1").unwrap();
        let mut picker: MovePicker = MovePicker::new(None, [None, None]);
        let history: HistoryTable = HistoryTable::default();

        while let Some(mv) = picker.next(&board, &history) {
            println!("{} - Stage: {:?}", mv, picker.stage);
        }
    }

    #[test]
    fn test_see_ordering() {
        let board: Board =
            Board::from_str("rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/7P/P1P1P3/RNBQKBNR w KQkq e6 0 1")
                .unwrap();
        let mut picker: MovePicker = MovePicker::new(None, [None, None]);

        picker.generate_and_score_captures(&board);

        println!("Good captures: {}", picker.good_captures.len());
        for mv in picker.good_captures.iter() {
            let see: bool = SEE::see(&board, *mv, 0);
            println!("Good: {} (SEE: {})", mv, see);
            assert_eq!(see, true);
        }
        println!("Bad captures: {}", picker.bad_captures.len());
        for mv in picker.bad_captures.iter() {
            let see: bool = SEE::see(&board, *mv, 0);
            println!("Bad: {} (SEE: {})", mv, see);
            assert_eq!(see, false);
        }
    }
}
