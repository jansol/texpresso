# Squish_cli
[![Build Status](https://travis-ci.org/jansol/squish-rs.svg?branch=master)](https://travis-ci.org/jansol/squish-rs)

A commandline utility for compressing images to DDS files using BC1/2/3/4/5. Serves mainly as a usage example of squish.

## Installation
```
cargo install squish_cli
```

## Usage
Compress image to DDS:
```
squish compress infile.png -f BC1
```

Decompress DDS file (only PNG output is supported for now)
```
squish decompress infile.dds
```

For more details:
```
squish help
```

## Todo
* [ ] Move from structopt to clap 3 (once it's released)
