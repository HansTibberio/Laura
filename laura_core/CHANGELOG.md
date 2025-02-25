# Changelog

All notable changes to this project will be documented in this file.  
This project follows [Keep a Changelog][changelog-link] and adheres to [Semantic Versioning][semver-link].

## [Unreleased]

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