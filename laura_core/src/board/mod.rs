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

#[allow(clippy::module_inception)]
pub mod board;
pub mod lookups;
pub mod movegen;
pub mod movemaker;
