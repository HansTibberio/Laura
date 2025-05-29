/*
    Laura: A multi-threaded UCI chess engine written in Rust.

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

// src/transposition.rs

//! Lockless Transposition Table.

use std::{
    alloc::{alloc_zeroed, Layout},
    mem::MaybeUninit,
    ptr,
    sync::atomic::{AtomicU16, AtomicU64, AtomicU8, Ordering},
    thread,
};

use laura_core::{gen_moves, AllMoves, Board, Move};

use crate::config::{
    AGE_MASK, AGE_OFFSET, BOUNDTYPE_MASK, BOUND_OFFSET, DATA_MASK, ENTRIES_PER_CELL, KEY_MASK,
    KEY_WRAPPER_MASK, MEGABYTE, PV_NODE_MASK, TTMATE,
};

fn normalize_score(score: i32, ply: i32) -> i16 {
    if score >= TTMATE {
        return (score + ply) as i16;
    } else if score <= -TTMATE {
        return (score - ply) as i16;
    }
    score as i16
}

fn unnormalize_score(score: i32, ply: i32) -> i32 {
    if score >= TTMATE {
        return score - ply;
    } else if score <= -TTMATE {
        return score + ply;
    }
    score
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BoundType {
    None = 0,
    Exact = 1,
    LowerBound = 2,
    UpperBound = 3,
}

#[derive(Debug, Clone, Copy)]
pub struct PackedData(u8);

impl PackedData {
    #[inline(always)]
    pub fn new(age: u8, bound: BoundType, pv_node: bool) -> Self {
        Self((age << AGE_OFFSET) | ((bound as u8) << BOUND_OFFSET) | pv_node as u8)
    }

    #[inline(always)]
    pub fn age(&self) -> u8 {
        self.0 >> AGE_OFFSET
    }

    #[inline(always)]
    pub fn bound(&self) -> BoundType {
        match (self.0 & BOUNDTYPE_MASK) >> BOUND_OFFSET {
            0 => BoundType::None,
            1 => BoundType::Exact,
            2 => BoundType::LowerBound,
            3 => BoundType::UpperBound,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn pv_node(&self) -> bool {
        self.0 & PV_NODE_MASK != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EntryHit {
    pub mv: Move,
    pub score: i32,
    pub evaluation: i32,
    pub depth: usize,
    pub bound: BoundType,
    pub pv_node: bool,
}

impl EntryHit {
    #[inline(always)]
    /// Directly return the best legal move, if one exists, from an EntryHit.
    pub fn legal_move(&self, board: &Board) -> Option<Move> {
        if !self.mv.is_null() && gen_moves::<AllMoves>(board).contains(&self.mv) {
            Some(self.mv)
        } else {
            None
        }
    }
}

/// 10 Bytes Entry
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    key: u16,         // 2 Bytes
    mv: Move,         // 2 Bytes
    score: i16,       // 2 Bytes
    evaluation: i16,  // 2 Bytes
    depth: u8,        // 1 Byte
    data: PackedData, // 1 Byte
}

impl Entry {
    #[inline(always)]
    #[allow(clippy::wrong_self_convention)]
    pub fn to_ne_bytes(&self) -> [u8; 16] {
        let packed: Packed128 = Packed128 { entry: *self };
        unsafe { packed.raw.to_ne_bytes() }
    }

    #[inline(always)]
    pub fn from_ne_bytes(bytes: [u8; 16]) -> Self {
        let raw: u128 = u128::from_ne_bytes(bytes);
        unsafe { Packed128 { raw }.entry }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
union Packed128 {
    raw: u128,
    entry: Entry,
}

#[repr(C, align(32))]
#[derive(Debug, Default)]
pub struct Cell {
    d0: AtomicU64,
    d1: AtomicU64,
    d2: AtomicU64,
    k0: AtomicU16,
    k1: AtomicU16,
    k2: AtomicU16,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            d0: AtomicU64::new(0),
            d1: AtomicU64::new(0),
            d2: AtomicU64::new(0),
            k0: AtomicU16::new(0),
            k1: AtomicU16::new(0),
            k2: AtomicU16::new(0),
        }
    }

    pub fn load(&self, index: usize) -> Entry {
        let (data, key) = match index {
            0 => (
                self.d0.load(Ordering::Relaxed),
                self.k0.load(Ordering::Relaxed),
            ),
            1 => (
                self.d1.load(Ordering::Relaxed),
                self.k1.load(Ordering::Relaxed),
            ),
            2 => (
                self.d2.load(Ordering::Relaxed),
                self.k2.load(Ordering::Relaxed),
            ),
            _ => panic!("Invalid Index"),
        };

        let raw: u128 = (u128::from(key) << 64) | u128::from(data);
        unsafe { Packed128 { raw }.entry }
    }

    pub fn store(&self, index: usize, entry: Entry) {
        let raw: u128 = unsafe { Packed128 { entry }.raw };
        let data: u64 = (raw & DATA_MASK) as u64;
        let key: u16 = ((raw >> 64) & KEY_MASK) as u16;

        match index {
            0 => {
                self.d0.store(data, Ordering::Relaxed);
                self.k0.store(key, Ordering::Relaxed);
            }
            1 => {
                self.d1.store(data, Ordering::Relaxed);
                self.k1.store(key, Ordering::Relaxed);
            }
            2 => {
                self.d2.store(data, Ordering::Relaxed);
                self.k2.store(key, Ordering::Relaxed);
            }
            _ => panic!("Invalid Index"),
        }
    }
}

#[derive(Debug, Default)]
pub struct TranspositionTable {
    entries: Vec<Cell>,
    age: AtomicU8,
}

impl TranspositionTable {
    pub fn resize(&mut self, megabytes: usize) {
        let len: usize = megabytes * MEGABYTE / size_of::<Cell>();
        let layout: Layout = Layout::array::<Cell>(len).expect("Invalid layout");

        // Allocate zeroed memory manually
        let raw_ptr: *mut Cell = unsafe { alloc_zeroed(layout) as *mut Cell };

        if raw_ptr.is_null() {
            panic!("Failed to allocate memory");
        }

        // SAFETY: We allocated len * size_of::<Cell>() bytes, zeroed
        let final_vec: Vec<Cell> = unsafe { Vec::from_raw_parts(raw_ptr, len, len) };

        self.entries = final_vec;
    }

    pub fn clear(&mut self, threads: usize) {
        let len: usize = self.entries.len();
        let ptr: *mut MaybeUninit<u8> = self.entries.as_mut_ptr() as *mut MaybeUninit<u8>;

        self.parallel_clear(ptr, threads, len);
        self.age.store(0, Ordering::Relaxed);
    }

    fn parallel_clear(&self, ptr: *mut MaybeUninit<u8>, threads: usize, len: usize) {
        unsafe {
            let base_ptr: *mut u8 = ptr as *mut u8;
            let total_bytes: usize = len * size_of::<Cell>();
            let chunk_size: usize = total_bytes.div_ceil(threads);

            thread::scope(|s| {
                for chunk_offset in (0..total_bytes).step_by(chunk_size) {
                    let start_addr: usize = base_ptr.add(chunk_offset) as usize;
                    let size: usize = chunk_size.min(total_bytes - chunk_offset);

                    s.spawn(move || {
                        let start: *mut u8 = start_addr as *mut u8;
                        ptr::write_bytes(start, 0, size);
                    });
                }
            })
        }
    }

    #[inline(always)]
    fn index(&self, key: u64) -> usize {
        let key: u128 = key as u128;
        let len: u128 = self.entries.len() as u128;

        ((key * len) >> 64) as usize
    }

    pub fn hash_full(&self) -> usize {
        let mut counter: usize = 0;
        let age: u8 = self.age.load(Ordering::Relaxed);
        for cell in self.entries.iter().take(2000) {
            for index in 0..ENTRIES_PER_CELL {
                let entry: Entry = cell.load(index);
                counter +=
                    (entry.data.bound() != BoundType::None && entry.data.age() == age) as usize;
            }
        }

        counter / (2 * ENTRIES_PER_CELL)
    }

    pub fn age(&self) {
        let new_age: u8 = (self.age.load(Ordering::Relaxed) + 1) & AGE_MASK;
        self.age.store(new_age, Ordering::Relaxed);
    }

    pub fn prefetch(&self, key: u64) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};

            let index: usize = self.index(key);
            let ptr: *const i8 = &self.entries[index] as *const Cell as *const i8;

            _mm_prefetch::<_MM_HINT_T0>(ptr);
        }

        #[cfg(not(target_arch = "x86_64"))]
        let _ = key;
    }
}

impl TranspositionTable {
    pub fn probe(&self, key: u64, ply: usize) -> Option<EntryHit> {
        let index: usize = self.index(key);
        let key: u16 = wrap_key(key);

        let cell: &Cell = unsafe { self.entries.get_unchecked(index) };

        for index in 0..ENTRIES_PER_CELL {
            let entry: Entry = cell.load(index);

            if entry.key != key {
                continue;
            }

            return Some(EntryHit {
                mv: entry.mv,
                score: unnormalize_score(entry.score as i32, ply as i32),
                evaluation: entry.evaluation as i32,
                depth: entry.depth as usize,
                bound: entry.data.bound(),
                pv_node: entry.data.pv_node(),
            });
        }

        None
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert(
        &self,
        key: u64,
        mut best_move: Move,
        score: i32,
        evaluation: i32,
        depth: usize,
        bound: BoundType,
        pv_node: bool,
        ply: usize,
    ) {
        let index: usize = self.index(key);
        let key: u16 = wrap_key(key);
        let age: u8 = self.age.load(Ordering::Relaxed);

        let cell: &Cell = unsafe { self.entries.get_unchecked(index) };

        let mut entry: Entry = cell.load(0);
        let mut store_index: usize = 0;

        if !(entry.key == 0 || entry.key == key) {
            for index in 1..ENTRIES_PER_CELL {
                let internal: Entry = cell.load(index);

                if internal.key == 0 || internal.key == key {
                    entry = internal;
                    store_index = index;
                    break;
                }
            }
        }

        if best_move == Move::null() && entry.key == key {
            best_move = entry.mv;
        }

        if entry.key != key
            || bound == BoundType::Exact && entry.data.bound() != BoundType::Exact
            || (depth + 4) + 2 * pv_node as usize > entry.depth as usize
        {
            let new: Entry = Entry {
                key,
                mv: best_move,
                score: normalize_score(score, ply as i32),
                evaluation: evaluation as i16,
                depth: depth as u8,
                data: PackedData::new(age, bound, pv_node),
            };

            cell.store(store_index, new);
        }
    }
}

#[inline(always)]
fn wrap_key(key: u64) -> u16 {
    (key & KEY_WRAPPER_MASK) as u16
}

fn parallel_clear(ptr: *mut MaybeUninit<u8>, threads: usize, len: usize) {
    unsafe {
        let base_ptr: *mut u8 = ptr as *mut u8;
        let total_bytes: usize = len * size_of::<u8>();
        let chunk_size: usize = total_bytes.div_ceil(threads);

        thread::scope(|s| {
            for chunk_offset in (0..total_bytes).step_by(chunk_size) {
                let start_addr: usize = base_ptr.add(chunk_offset) as usize;
                let size: usize = chunk_size.min(total_bytes - chunk_offset);

                s.spawn(move || {
                    let start: *mut u8 = start_addr as *mut u8;
                    ptr::write_bytes(start, 0, size);
                });
            }
        })
    }
}

#[cfg(test)]
mod test {
    use std::{
        alloc::{alloc_zeroed, Layout},
        sync::atomic::AtomicU8,
        time::Instant,
    };

    use laura_core::{Move, MoveType, Square};

    use crate::transposition::{BoundType, Cell, Entry, PackedData};

    use super::{parallel_clear, TranspositionTable, MEGABYTE};

    #[test]
    fn test_table() {
        let mut ttable: TranspositionTable = TranspositionTable::default();
        ttable.resize(1);
        assert_eq!(ttable.entries.len(), 32_768);
    }

    #[test]
    fn test_cell() {
        let cell: Cell = Cell::new();
        assert_eq!(std::mem::size_of::<Cell>(), 32);
        assert_eq!(std::mem::align_of::<Cell>(), 32);

        let entry: Entry = Entry {
            key: 0xABCD,
            mv: Move::new(Square::A2, Square::A5, MoveType::Quiet),
            score: 20,
            evaluation: -12,
            depth: 15,
            data: PackedData::new(3, BoundType::LowerBound, true),
        };

        cell.store(0, entry);
        let retrieved: Entry = cell.load(0);
        assert_eq!(retrieved.key, 43981);
        assert_eq!(retrieved.mv, Move(2056));
        assert_eq!(retrieved.data.bound(), BoundType::LowerBound);
    }

    #[test]
    fn test_write() {
        let cell: Cell = Cell::new();
        assert_eq!(std::mem::size_of::<Cell>(), 32);
        assert_eq!(std::mem::align_of::<Cell>(), 32);

        let entry: Entry = Entry {
            key: 0xABCD,
            mv: Move::new(Square::A2, Square::A5, MoveType::Quiet),
            score: 20,
            evaluation: -12,
            depth: 15,
            data: PackedData::new(3, BoundType::LowerBound, true),
        };

        for i in 0..3 {
            cell.store(i, entry);
            let retrieved: Entry = cell.load(i);
            assert_eq!(entry.to_ne_bytes(), retrieved.to_ne_bytes());
            assert_eq!(retrieved.key, 43981);
            assert_eq!(retrieved.mv, Move(2056));
            assert_eq!(retrieved.data.bound(), BoundType::LowerBound);
        }
    }

    #[test]
    fn test_resize() {
        let mut tt: TranspositionTable = TranspositionTable {
            entries: Vec::new(),
            age: AtomicU8::new(0),
        };

        let start: Instant = Instant::now();
        tt.resize(16);
        println!("Full initialization in {} µs", start.elapsed().as_micros());
    }

    #[test]
    fn test_parallel_clear() {
        let mut vector: Vec<u8> = vec![1u8; 1024];
        let len: usize = vector.len();
        parallel_clear(vector.as_mut_ptr().cast(), 1, len);

        for (index, byte) in vector.iter().enumerate() {
            assert_eq!(*byte, 0, "Error on index {index}");
        }

        let mut vector: Vec<u8> = vec![1u8; 2048];
        let len: usize = vector.len();
        parallel_clear(vector.as_mut_ptr().cast(), 2, len);

        for (index, byte) in vector.iter().enumerate() {
            assert_eq!(*byte, 0, "Error on index {index}");
        }

        let mut vector: Vec<u8> = vec![1u8; 4096];
        let len: usize = vector.len();
        parallel_clear(vector.as_mut_ptr().cast(), 5, len);

        for (index, byte) in vector.iter().enumerate() {
            assert_eq!(*byte, 0, "Error on index {index}");
        }
    }

    #[test]
    fn test_alloc_zeroed() {
        let len: usize = MEGABYTE;
        let layout: Layout = Layout::array::<u8>(len).unwrap();
        let raw_ptr: *mut u8 = unsafe { alloc_zeroed(layout) };

        if raw_ptr.is_null() {
            panic!("Failed to allocate memory");
        }

        let vec: Vec<u8> = unsafe { Vec::from_raw_parts(raw_ptr, len, len) };
        for (index, byte) in vec.iter().enumerate() {
            assert_eq!(*byte, 0, "Error on index {index}");
        }
    }
}
