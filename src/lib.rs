// Copyright (c) 2006 Simon Brown <si@sjbrown.co.uk>
// Copyright (c) 2018 Jan Solanti <jhs@psonet.com>
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the 
// "Software"), to	deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to 
// permit persons to whom the Software is furnished to do so, subject to 
// the following conditions:
//
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF 
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY 
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, 
// TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE 
// SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


//! A pure Rust DXT1/3/5 compressor and decompressor based on Simon Brown's
//! **libsquish**

extern crate byteorder;

mod math;
mod colourblock;
mod colourset;

/// Defines a compression format
#[derive(PartialEq, Eq)]
pub enum Format {
    Dxt1,
    Dxt3,
    Dxt5,
}

/// Defines a compression algorithm
pub enum CompressionAlgorithm {
    /// Fast, low quality
    ColourRangeFit,

    /// Slow, high quality (default)
    ColourClusterFit,

    /// Very slow, very high quality
    ColourIterativeClusterFit,
}

/// A block of owned compressed data. Variants are for different block sizes
enum CompressedBlock {
    Dxt1([u8; 8]),
    Dxt3([u8; 8], [u8; 8]),
    Dxt5([u8; 16]),
}

impl Default for CompressionAlgorithm {
    fn default() -> Self { CompressionAlgorithm::ColourClusterFit }
}

/// RGB colour channel weights for use in block fitting
pub type ColourWeights = [f32; 3];

/// Weights based on the perceived brightness of each colour channel
pub const COLOUR_WEIGHTS_PERCEPTUAL: ColourWeights = [0.2126, 0.7152, 0.0722];

pub struct CompressorParams {
    /// The compression algorithm to be used
    pub algorithm: CompressionAlgorithm,

    /// Weigh the relative importance of each colour channel when fitting
    /// (defaults to equal weights)
    pub weights: Option<ColourWeights>,

    /// Weigh colour by alpha during cluster fit (defaults to false)
    ///
    /// This can significantly increase perceived quality for images that are rendered
    /// using alpha blending.
    pub weigh_colour_by_alpha: bool,
}

impl Default for CompressorParams {
    fn default() -> Self {
        CompressorParams {
            algorithm: CompressionAlgorithm::default(),
            weights: None,
            weigh_colour_by_alpha: false,
        }
    }
}

/// Decompresses an image in memory
///
/// * `data`   - The compressed image data
/// * `width`  - The width of the source image
/// * `height` - The height of the source image
/// * `format` - The compression format
pub fn decompress(
    data: &[u8],
    width: usize,
    height: usize,
    format: Format,
) -> Vec<u8> {
    vec![]
}

/// Computes the amount of space in bytes needed for the compressed image
///
/// * `width`  - Width of the uncompressed image
/// * `height` - Height of the uncompressed image
/// * `format` - The desired compression format
///
pub fn compute_compressed_size(
    width: usize,
    height: usize,
    format: Format
) -> usize {
    // Number of blocks required for image of given dimensions
    let n_blocks = ((width + 3) / 4) * ((height + 3) / 4);

    let blocksize = bytes_per_block(format);

    n_blocks * blocksize
}

/// Compresses a 4x4 block of pixels
///
/// * `rgba`   - The uncompressed block of pixels
/// * `format` - The desired compression format
/// * `params` - Additional compressor parameters
fn compress_block(
    rgba: [[u8; 4]; 16],
    format: Format,
    params: Option<CompressorParams>
) -> () {
    compress_block_masked(rgba, 0xffffffff, format, params)
}

/// Compresses a 4x4 block of pixels, masking out some pixels e.g. for padding the
/// image to a multiple of the block size.
///
/// * `rgba`   - The uncompressed block of pixels
/// * `mask`   - The valid pixel mask
/// * `format` - The desired compression format
/// * `params` - Additional compressor parameters
fn compress_block_masked(
    rgba: [[u8; 4]; 16],
    mask: u32,
    format: Format,
    params: Option<CompressorParams>
) -> () {
    let params = params.unwrap_or(CompressorParams::default());
}

/// Decompresses a 4x4 block of pixels
///
/// * `rgba`   - The compressed block of pixels
/// * `format` - The compression format of the data
fn decompress_block(
    rgba: &CompressedBlock,
    format: Format,
) -> () {

}

/// Compresses an image in memory
///
/// * `rgba`   - The uncompressed pixel data
/// * `width`  - The width of the source image
/// * `height` - The height of the source image
/// * `format` - The desired compression format
/// * `params` - Additional compressor parameters
pub fn compress(
    rgba: &[u8],
    width: usize,
    height: usize,
    format: Format,
    params: Option<CompressorParams>
) -> Vec<u8> {
    vec![]
}

/// Returns how many bytes a 4x4 block of pixels will take after compression,
/// given the compression format
fn bytes_per_block(format: Format) -> usize {
    // Compressed block size in bytes
    match format {
        Format::Dxt1 => 8,
        Format::Dxt3 => 16,
        Format::Dxt5 => 16,
    } 
}

fn f32_to_i32_clamped(a: f32, limit: i32) -> i32 {
    (a.round() as i32).max(0).min(limit)
}


//--------------------------------------------------------------------------------
// Unit tests
//--------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_requirements_dxt1_exact() {
        let estimate = compute_compressed_size(16, 32, Format::Dxt1);
        assert_eq!(estimate, 256);
    }

    #[test]
    fn test_storage_requirements_dxt1_padded() {
        let estimate = compute_compressed_size(15, 30, Format::Dxt1);
        assert_eq!(estimate, 256);
    }

    #[test]
    fn test_storage_requirements_dxt3_exact() {
        let estimate = compute_compressed_size(16, 32, Format::Dxt3);
        assert_eq!(estimate, 512);
    }

    #[test]
    fn test_storage_requirements_dxt3_padded() {
        let estimate = compute_compressed_size(15, 30, Format::Dxt3);
        assert_eq!(estimate, 512);
    }

    #[test]
    fn test_storage_requirements_dxt5_exact() {
        let estimate = compute_compressed_size(16, 32, Format::Dxt5);
        assert_eq!(estimate, 512);
    }

    #[test]
    fn test_storage_requirements_dxt5_padded() {
        let estimate = compute_compressed_size(15, 30, Format::Dxt5);
        assert_eq!(estimate, 512);
    }
}
