# Changelog

All notable changes to this project will be documented in this file.  
This project follows [Keep a Changelog][changelog-link] and adheres to [Semantic Versioning][semver-link].

---

## [Unreleased]

---

## [0.3.0] - 2025-04-26

### Added
- New utility functions in the **BitBoard** module:
  - `right()`, `right_for()`, `left()`, `left_for()`, `up_left_for()`, and `up_right_for()`.
- New methods in the **MoveList** module:
  - `as_slice()`, `as_mut_slice()`, and `clear()`
- Expanded and improved documentation for **BitBoard** and **MoveList**, now including usage examples.
- New macros for generating legal moves:
  - `legal_moves!`: Generates all legal moves for a given board position.
  - `quiet_moves!`: Generates only quiet (non-capturing) moves.
  - `tactical_moves!`: Generates only tactical (capturing and promoting) moves.
- Introduced the `MoveFilter` trait along with the `QuietMoves`, `TacticalMoves`, and `AllMoves` structs,
  offering more flexibility and control over move generation.
- Implemented `Deref` and `DerefMut` for **MoveList** for more ergonomic access.

### Changed
- Rewrote the `Iterator` implementation for `BitBoard` to improve clarity and flexibility.
- Replaced the previous move generation constants with the new `MoveFilter` trait and associated
  structs(`QuietMoves`, `TacticalMoves`, `AllMoves`).
- Renamed the old `moves()` method in **MoveList** to `as_slice()`.

### Removed
- `BitBoard::to_square()` now returns an `Option<Square>` instead of `Square`, improving safety when dealing with empty
  bitboards.
- The previous move generation approach based on constants has been fully replaced by the `MoveFilter`
  system. This change may require users to update how they generate moves.

---

## [0.2.2] - 2025-03-27

### Fixed
- Optimized move generation performance by fixing a bottleneck in the code.

---

## [0.2.1] - 2025-03-21

### Changed
- Improved and expanded documentation across all modules and functions.
- Refactored code to enhance readability and maintainability.

### Fixed
- Fixed all **Clippy** and **cargo fmt** warnings, ensuring cleaner code aligned with Rust best practices.

---

## [0.2.0] - 2025-02-20

### Added
- Added full **`no_std`** support in `laura_core`, allowing compatibility with other `no_std` crates or projects.
- Added new functions to the **Bitboard** module:  
  - `up_left()` and `up_right()` for more efficient diagonal move calculations for pawns.  
- Added new functions to the **Square** module:  
  - `right_color()` and `left_color()` to determine the direction of adjacent squares based on the color of the side to move.  
- Introduced **internal macros** to optimize the new move generator.

### Changed
- Replaced the legal move generator with a more efficient version, improving move generation performance.
- Updated the **README** with new usage examples and improved documentation.
- Modified board printing characters:  
  - **Previous:** Unicode  
  - **Now:** ASCII (for better compatibility across different terminals).

### Fixed
- Fixed a bug in `movelist.rs` that affected the printing of the move list in certain positions.

---

## [0.1.0] - 2025-02-01

### Initial Release
- First functional release of `laura_core`, with basic features implemented, including **pext bitboards**.


[changelog-link]:https://keepachangelog.com/en/1.1.0/
[semver-link]:https://semver.org/
