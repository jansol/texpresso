# texpresso\_cli

A CLI utility for compressing images to GPU-readable texture formats

## Installation
```
cargo install texpresso_cli
```

## Usage
Compress image to DDS:
```
texpresso compress infile.png -f BC1
```

Decompress DDS file to PNG
```
texpresso decompress infile.dds
```

For more details:
```
texpresso help
```

