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

use core::f32;

use crate::colourblock;
use crate::colourset::ColourSet;
use crate::math::{Sym3x3, Vec3};
use crate::{ColourWeights, Format};

use super::ColourFitImpl;

pub struct RangeFit<'a> {
    colourset: &'a ColourSet,
    format: Format,
    weights: Vec3,
    start: Vec3,
    end: Vec3,
    indices: [u8; 16],
    best_error: f32,
    best_compressed: [u8; 8],
}

impl<'a> RangeFit<'a> {
    pub fn new(colourset: &'a ColourSet, format: Format, weights: ColourWeights) -> Self {
        let mut fit = RangeFit {
            colourset,
            format,
            weights: Vec3::new(weights[0], weights[1], weights[2]),
            start: Vec3::new(0.0, 0.0, 0.0),
            end: Vec3::new(0.0, 0.0, 0.0),
            indices: [0u8; 16],
            best_error: f32::MAX,
            best_compressed: [0u8; 8],
        };

        // cache some values
        let values = fit.colourset.points();
        let weights = fit.colourset.weights();
        let count = fit.colourset.count();

        // get the covariance matrix
        let covariance = Sym3x3::weighted_covariance(values, weights);

        // get the principle component
        let principle = covariance.principle_component();

        let mut start = Vec3::new(0.0, 0.0, 0.0);
        let mut end = Vec3::new(0.0, 0.0, 0.0);
        if count > 0 {
            // compute the range
            start = values[0];
            end = start;
            let mut min = start.dot(&principle);
            let mut max = min;
            for &val in values.iter().take(count).skip(1) {
                let dot = val.dot(&principle);
                if dot < min {
                    start = val;
                    min = dot;
                } else if dot > max {
                    end = val;
                    max = dot;
                }
            }
        }

        // clamp the output to [0, 1]
        let one = Vec3::new(1.0, 1.0, 1.0);
        let zero = Vec3::new(0.0, 0.0, 0.0);
        start = one.min(zero.max(start));
        end = one.min(zero.max(end));

        // clamp to the grid and save
        let grid = Vec3::new(31.0, 63.0, 31.0);
        let gridrcp = Vec3::new(1.0 / 31.0, 1.0 / 63.0, 1.0 / 31.0);
        let half = Vec3::new(0.5, 0.5, 0.5);
        fit.start = (grid * start + half).truncate() * gridrcp;
        fit.end = (grid * end + half).truncate() * gridrcp;

        fit
    }

    fn compression_helper(&mut self, codes: &[Vec3]) -> bool {
        // cache some values
        let count = self.colourset.count();
        let values = self.colourset.points();

        // match each point to the closest code
        let mut closest = [0u8; 16];
        let mut error = 0f32;
        for i in 0..count {
            // find the closest code
            let mut dist = f32::MAX;
            let mut idx = 0;

            for (j, code) in codes.iter().enumerate() {
                let d = (self.weights * (values[i] - code)).length2();
                if d < dist {
                    dist = d;
                    idx = j;
                }
            }

            // save the index
            closest[i] = idx as u8;

            // accumulate the error
            error += dist;
        }

        // save this scheme if it wins
        if error < self.best_error {
            // remap the indices
            self.colourset.remap_indices(&closest, &mut self.indices);

            // save the error
            self.best_error = error;

            return true;
        }

        false
    }
}

impl<'a> ColourFitImpl<'a> for RangeFit<'a> {
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
        // create a codebook
        let codes = [self.start, self.end, self.start * 0.5 + self.end * 0.5];

        if self.compression_helper(&codes) {
            // build the best compressed blob
            colourblock::write3(
                &self.start,
                &self.end,
                &self.indices,
                &mut self.best_compressed,
            );
        }
    }

    fn compress4(&mut self) {
        // create a codebook
        let codes = [
            self.start,
            self.end,
            self.start * (2.0 / 3.0) + self.end * (1.0 / 3.0),
            self.start * (1.0 / 3.0) + self.end * (2.0 / 3.0),
        ];

        if self.compression_helper(&codes) {
            // build the best compressed blob
            colourblock::write4(
                &self.start,
                &self.end,
                &self.indices,
                &mut self.best_compressed,
            );
        }
    }
}
