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

//! A pure Rust BC1/2/3 compressor and decompressor based on Simon Brown's
//! **libsquish**

#![no_std]

mod alpha;
mod colourblock;
mod colourfit;
mod colourset;
mod math;

use crate::colourfit::{ClusterFit, ColourFit, RangeFit, SingleColourFit};
use crate::colourset::ColourSet;

/// Defines a compression format
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Bc1,
    Bc2,
    Bc3,
}

/// Defines a compression algorithm
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    /// Fast, low quality
    RangeFit,

    /// Slow, high quality
    ClusterFit,

    /// Very slow, very high quality
    IterativeClusterFit,
}

impl Default for Algorithm {
    fn default() -> Self {
        Algorithm::ClusterFit
    }
}

/// RGB colour channel weights for use in block fitting
pub type ColourWeights = [f32; 3];

/// Uniform weights for each colour channel
pub const COLOUR_WEIGHTS_UNIFORM: ColourWeights = [1.0, 1.0, 1.0];

/// Weights based on the perceived brightness of each colour channel
pub const COLOUR_WEIGHTS_PERCEPTUAL: ColourWeights = [0.2126, 0.7152, 0.0722];

#[derive(Clone, Copy)]
pub struct Params {
    /// The compression algorithm to be used
    pub algorithm: Algorithm,

    /// Weigh the relative importance of each colour channel when fitting
    /// (defaults to perceptual weights)
    pub weights: ColourWeights,

    /// Weigh colour by alpha during cluster fit (defaults to false)
    ///
    /// This can significantly increase perceived quality for images that are rendered
    /// using alpha blending.
    pub weigh_colour_by_alpha: bool,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            algorithm: Algorithm::default(),
            weights: COLOUR_WEIGHTS_PERCEPTUAL,
            weigh_colour_by_alpha: false,
        }
    }
}

/// Returns number of blocks needed for an image of given dimension
pub fn num_blocks(size: usize) -> usize {
    (size + 3) / 4
}

impl Format {
    /// Decompresses an image in memory
    ///
    /// * `data`   - The compressed image data
    /// * `width`  - The width of the source image
    /// * `height` - The height of the source image
    /// * `output` - Space to store the decompressed image
    pub fn decompress(self, data: &[u8], width: usize, height: usize, output: &mut [u8]) {
        let blocks_wide = num_blocks(width);
        let blocks_high = num_blocks(height);
        let block_size = self.block_size();

        // loop over blocks
        for y in 0..blocks_high {
            for x in 0..blocks_wide {
                // decompress the block
                let bidx = (x + y * blocks_wide) * block_size;
                let rgba = self.decompress_block(&data[bidx..bidx + block_size]);

                // write the decompressed pixels to the correct image location
                for py in 0..4 {
                    for px in 0..4 {
                        // get target location
                        let sx = 4 * x + px;
                        let sy = 4 * y + py;

                        if sx < width && sy < height {
                            for i in 0..4 {
                                output[4 * (sx + sy * width) + i] = rgba[px + py * 4][i];
                            }
                        }
                    }
                }
            }
        }
    }

    /// Returns how many bytes a 4x4 block of pixels will compress into
    fn block_size(self) -> usize {
        // Compressed block size in bytes
        match self {
            Format::Bc1 => 8,
            Format::Bc2 => 16,
            Format::Bc3 => 16,
        }
    }

    /// Computes the amount of space in bytes needed for an image of given size,
    /// accounting for padding to a multiple of 4x4 pixels
    ///
    /// * `width`  - Width of the uncompressed image
    /// * `height` - Height of the uncompressed image
    pub fn compressed_size(self, width: usize, height: usize) -> usize {
        // Number of blocks required for image of given dimensions
        let blocks = num_blocks(width) * num_blocks(height);
        blocks * self.block_size()
    }

    /// Compresses a 4x4 block of pixels, masking out some pixels e.g. for padding the
    /// image to a multiple of the block size.
    ///
    /// * `rgba`   - The uncompressed block of pixels
    /// * `mask`   - The valid pixel mask
    /// * `params` - Additional compressor parameters
    /// * `output` - Storage for the compressed block
    fn compress_block_masked(
        self,
        rgba: [[u8; 4]; 16],
        mask: u32,
        params: Params,
        output: &mut [u8],
    ) {
        // compress alpha separately if necessary
        if self == Format::Bc2 {
            alpha::compress_bc2(&rgba, mask, &mut output[..8]);
        } else if self == Format::Bc3 {
            alpha::compress_bc3(&rgba, mask, &mut output[..8]);
        }

        // create the minimal point set
        let colours = ColourSet::new(&rgba, mask, self, params.weigh_colour_by_alpha);

        let colour_offset = if self == Format::Bc1 { 0 } else { 8 };
        let colour_block = &mut output[colour_offset..colour_offset + 8];

        // compress with appropriate compression algorithm
        if colours.count() == 1 {
            // Single colour fit can't handle fully transparent blocks, hence the
            // set has to contain at least 1 colour. It's also not very useful for
            // anything more complex so we only use it for blocks of uniform colour.
            let mut fit = SingleColourFit::new(&colours, self);
            fit.compress(colour_block);
        } else if (params.algorithm == Algorithm::RangeFit) || (colours.count() == 0) {
            let mut fit = RangeFit::new(&colours, self, params.weights);
            fit.compress(colour_block);
        } else {
            let iterate = params.algorithm == Algorithm::IterativeClusterFit;
            let mut fit = ClusterFit::new(&colours, self, params.weights, iterate);
            fit.compress(colour_block);
        }
    }

    /// Decompresses a 4x4 block of pixels
    ///
    /// * `block`  - The compressed block of pixels
    /// * `output` - Storage for the decompressed block of pixels
    fn decompress_block(self, block: &[u8]) -> [[u8; 4]; 16] {
        // get reference to the actual colour block
        let colour_offset = if self == Format::Bc1 { 0 } else { 8 };
        let colour_block = &block[colour_offset..colour_offset + 8];

        // decompress colour
        let mut rgba = colourblock::decompress(colour_block, self == Format::Bc1);

        // decompress alpha separately if necessary
        if self == Format::Bc2 {
            alpha::decompress_bc2(&mut rgba, &block[..8]);
        } else if self == Format::Bc3 {
            alpha::decompress_bc3(&mut rgba, &block[..8]);
        }

        rgba
    }

    /// Compresses an image in memory
    ///
    /// * `rgba`   - The uncompressed pixel data
    /// * `width`  - The width of the source image
    /// * `height` - The height of the source image
    /// * `params` - Additional compressor parameters
    /// * `output` - Output buffer for the compressed image. Ensure that this has
    /// at least as much space available as `compute_compressed_size` suggests.
    pub fn compress(
        self,
        rgba: &[u8],
        width: usize,
        height: usize,
        params: Params,
        output: &mut [u8],
    ) {
        assert!(output.len() >= self.compressed_size(width, height));

        let block_size = self.block_size();
        let blocks_high = num_blocks(height);
        let blocks_wide = num_blocks(width);

        // loop over blocks, rounding size to next multiple of 4
        for y in 0..blocks_high {
            for x in 0..blocks_wide {
                // build the 4x4 block of pixels
                let mut source_rgba = [[0u8; 4]; 16];
                let mut mask = 0u32;
                for py in 0..4 {
                    for px in 0..4 {
                        let index = 4 * py + px;

                        // get position in source image
                        let sx = 4 * x + px;
                        let sy = 4 * y + py;

                        // enable pixel if within bounds
                        if sx < width && sy < height {
                            // copy pixel value
                            let src_index = 4 * (width * sy + sx);
                            source_rgba[index].copy_from_slice(&rgba[src_index..src_index + 4]);

                            // enable pixel
                            mask |= 1 << index;
                        }
                    }
                }

                // compress block into output
                let offset = x * block_size + y * blocks_wide * block_size;
                let block = &mut output[offset..offset + block_size];
                self.compress_block_masked(source_rgba, mask, params, block);
            }
        }
    }
}

//--------------------------------------------------------------------------------
// Unit tests
//--------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_requirements_bc1_exact() {
        let estimate = Format::Bc1.compressed_size(16, 32);
        assert_eq!(estimate, 256);
    }

    #[test]
    fn test_storage_requirements_bc1_padded() {
        let estimate = Format::Bc1.compressed_size(15, 30);
        assert_eq!(estimate, 256);
    }

    #[test]
    fn test_storage_requirements_bc2_exact() {
        let estimate = Format::Bc2.compressed_size(16, 32);
        assert_eq!(estimate, 512);
    }

    #[test]
    fn test_storage_requirements_bc2_padded() {
        let estimate = Format::Bc2.compressed_size(15, 30);
        assert_eq!(estimate, 512);
    }

    #[test]
    fn test_storage_requirements_bc3_exact() {
        let estimate = Format::Bc3.compressed_size(16, 32);
        assert_eq!(estimate, 512);
    }

    #[test]
    fn test_storage_requirements_bc3_padded() {
        let estimate = Format::Bc3.compressed_size(15, 30);
        assert_eq!(estimate, 512);
    }
}
