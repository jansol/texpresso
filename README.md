A pure Rust texture compression suite

This library started out as `squish-rs`, a pure rust port of Simon Brown's libsquish but was forked and renamed to avoid confusion as the API and format support is expanded beyond the original C++ library.

# Crates in This Workspace
* `texpresso` - The library itself
* `texpresso_cli` - A command-line utility for compressing and decompressing textures. Also serves as a usage example for the library.

# Roadmap
### Library
* S3 Texture Compression formats
  [x] DXT1 aka BC1: 8bpc RGB with optional binary alpha
  [x] DXT3 aka BC2: 8bpc RGB with paletted alpha
  [x] DXT5 aka BC3: 8bpc RGB with interpolated alpha
* 3Dc variants / RGTC (Red-Green Texture Compression)
  [x] ATI1 aka RGTC1 aka BC4: 8-bit grayscale
  [x] ATI2 aka RGTC2 aka BC5: two 8-bit channels
* Direct3D 11 additions / BPTC (Block-Partition Texture Compression?)
  [ ] BPTC\_ALPHA aka BC6h: 16-bit HDR RGB
  [ ] BPTC aka BC7: 8-bit RGB with optional alpha
* Ericsson Texture Compression (common in mobile chips)
  [ ] R11\_EAC: 11-bit grayscale
  [ ] RG11\_EAC: two 11-bit channels
  [ ] ETC2: 8bpc RGB
  [ ] ETC2\_EAC: 8bpc RGB + 11-bit alpha
  [ ] PUNCHTHROUGH\_ALPHA1\_ETC2: 8bpc RGB with punchthrough alpha
* Adaptable Scalable Texture Compression (common in modern mobile chips)
  * the texture compression format to end all texture compression formats
  * needs further investigating due to vast complexity and e.g. supporting non-square blocks
  [ ] ASTC\_LDR
  [ ] ASTC\_HDR
* Compatibility / ease of use
  [x] no\_std
  [ ] support compiling for GPU targets via [rust-gpu](https://shader.rs/)
  [ ] Support 1-4 input channels without requiring padding in the calling code

### CLI
* non-texture formats
  [x] Read PNG
  [x] Read JPG
  [x] Write PNG
* Containers for compressed textures
  [x] Read DDS
  [x] Write DDS
  [ ] Read KTX
  [ ] Write KTX
  [ ] Read KTX2
  [ ] Write KTX2
* Target platforms
  [ ] Support encoding on GPU via Vulkan
* Maintenance
  [ ] Migrate to clap 3

