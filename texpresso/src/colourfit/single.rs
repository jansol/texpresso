// Copyright (c) 2006 Simon Brown <si@sjbrown.co.uk>
// Copyright (c) 2018-2021 Jan Solanti <jhs@psonet.com>
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

use core::u32;

use crate::colourblock;
use crate::colourset::ColourSet;
use crate::math::{f32_to_i32_clamped, Vec3};
use crate::Format;

use super::single_lut::*;
use super::ColourFitImpl;

pub struct SingleColourFit<'a> {
    colourset: &'a ColourSet,
    format: Format,
    start: Vec3,
    end: Vec3,
    index: u8,
    error: u32,
    best_error: u32,
    best_compressed: [u8; 8],
}

impl<'a> SingleColourFit<'a> {
    pub fn new(colourset: &'a ColourSet, format: Format) -> Self {
        SingleColourFit {
            colourset,
            format,
            start: Vec3::new(0.0, 0.0, 0.0),
            end: Vec3::new(0.0, 0.0, 0.0),
            index: 0,
            error: u32::MAX,
            best_error: u32::MAX,
            best_compressed: [0u8; 8],
        }
    }

    fn compute_endpoints(&mut self, lut: [&[SingleColourLookup; 256]; 3]) {
        // get colour for this block
        let colour = [
            f32_to_i32_clamped(self.colourset.points()[0].x() * 255.0, 255),
            f32_to_i32_clamped(self.colourset.points()[0].y() * 255.0, 255),
            f32_to_i32_clamped(self.colourset.points()[0].z() * 255.0, 255),
        ];

        // check each index combination (endpoint and intermediate)
        self.error = u32::MAX;
        for index in 0..2 {
            // check the error for this codebook index
            let mut error = 0u32;
            let mut sources = [
                &lut[0][0].sources[0],
                &lut[1][0].sources[0],
                &lut[2][0].sources[0],
            ];

            for channel in 0..3 {
                // grab the lookup table and index for this channel
                let lookup = &lut[channel];
                let target = colour[channel];

                // store a reference to the source for this channel
                sources[channel] = &lookup[target as usize].sources[index];

                // accumulate the error
                let diff = u32::from(sources[channel].error);
                error += diff * diff;
            }

            // keep these if the error is lower
            if error < self.error {
                self.start = Vec3::new(
                    f32::from(sources[0].start) / 31.0,
                    f32::from(sources[1].start) / 63.0,
                    f32::from(sources[2].start) / 31.0,
                );
                self.end = Vec3::new(
                    f32::from(sources[0].end) / 31.0,
                    f32::from(sources[1].end) / 63.0,
                    f32::from(sources[2].end) / 31.0,
                );
                self.index = 2 * index as u8;
                self.error = error;
            }
        }
    }
}

impl<'a> ColourFitImpl<'a> for SingleColourFit<'a> {
    fn is_bc1(&self) -> bool {
        self.format == Format::Bc1
    }

    fn is_transparent(&self) -> bool {
        self.colourset.is_transparent()
    }

    fn best_compressed(&'a self) -> &'a [u8] {
        &self.best_compressed
    }

    fn compress3(&mut self) {
        // build lookup table
        let lut = [&LOOKUP_5_3, &LOOKUP_6_3, &LOOKUP_5_3];

        // find best endpoints and index
        self.compute_endpoints(lut);

        // build the block if we win
        if self.error < self.best_error {
            // remap the indices
            let mut indices = [0u8; 16];
            self.colourset
                .remap_indices(&[self.index; 16], &mut indices);

            // build the compressed blob
            colourblock::write3(&self.start, &self.end, &indices, &mut self.best_compressed);

            // save the error
            self.best_error = self.error;
        }
    }

    fn compress4(&mut self) {
        // build lookup table
        let lut = [&LOOKUP_5_4, &LOOKUP_6_4, &LOOKUP_5_4];

        // find best endpoints and index
        self.compute_endpoints(lut);

        // build the block if we win
        if self.error < self.best_error {
            // remap the indices
            let mut indices = [0u8; 16];
            self.colourset
                .remap_indices(&[self.index; 16], &mut indices);

            // build the compressed blob
            colourblock::write4(&self.start, &self.end, &indices, &mut self.best_compressed);

            // save the error
            self.best_error = self.error;
        }
    }
}
