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

#[allow(clippy::module_inception)]
pub mod board;
pub mod lookups;
pub mod movegen;
pub mod movemaker;

const MAX_FEN_LENGTH: usize = 128;

#[derive(Debug)]
pub struct FenBuffer {
    buf: [u8; MAX_FEN_LENGTH],
    pos: usize,
}

impl FenBuffer {
    fn new() -> Self {
        Self {
            buf: [0; MAX_FEN_LENGTH],
            pos: 0,
        }
    }

    fn to_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.pos]).unwrap_or("")
    }
}

impl fmt::Write for FenBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes: &[u8] = s.as_bytes();
        let len: usize = bytes.len();

        if self.pos + len > MAX_FEN_LENGTH {
            return Err(core::fmt::Error);
        }

        self.buf[self.pos..self.pos + len].copy_from_slice(bytes);
        self.pos += len;
        Ok(())
    }
}

impl fmt::Display for FenBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl PartialEq<&str> for FenBuffer {
    fn eq(&self, other: &&str) -> bool {
        self.to_str() == *other
    }
}
