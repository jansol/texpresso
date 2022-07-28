# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [2.0.0] - 2022-07-29
### Added
- Fork/rename from squish-rs in interest of keeping the original RIIR code easily accessible and preventing confusion about what this library is

### Changed
- Use clap for parsing CLI arguments
- Update dependencies

## [2.0.0-beta1] - 2021-11-16
### Added
- CHANGELOG file
- Parallel compression of blocks
- Support for BC4 and BC5 formats

### Changed
- Updated to Rust 2021 Edition

### Removed
- Dependency on `byteorder`
- `FromStr` implementation on `Format`

### Fixed
- The library crate was not actually compiling as `no_std`


## [1.0.0] - 2018-09-10
### Added
- Full Rust reimplementation of the original libsquish code
