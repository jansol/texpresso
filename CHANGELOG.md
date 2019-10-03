# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]
### Added
- CHANGELOG file
- Parallel compression of blocks

### Changed
- Updated to Rust 2018 Edition

### Removed
- Dependency on `byteorder`
- `FromStr` implementation on `Format`

### Fixed
- The library crate was not actually compiling as `no_std`


## [1.0.0] - 2018-09-10
### Added
- Full Rust reimplementation of the original libsquish code
