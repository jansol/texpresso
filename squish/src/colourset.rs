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

use math::*;
use Format;

pub struct ColourSet {
    count: usize,
    points: [Vec3; 16],
    weights: [f32; 16],
    remap: [i8; 16],
    transparent: bool,
}

impl ColourSet {
    pub fn new(rgba: &[[u8; 4]; 16], mask: u32, format: Format, alpha_weighted: bool) -> ColourSet {
        let mut set = ColourSet {
            count: 0,
            points: [Vec3::new(0f32, 0f32, 0f32); 16],
            weights: [0f32; 16],
            remap: [0i8; 16],
            transparent: false,
        };

        // create the minimal set
        for i in 0..rgba.len() {
            // disabled pixels are transparent black
            let bit = 1u32 << i;
            if (mask & bit) == 0 {
                set.remap[i] = -1;
                continue;
            }

            // DXT uses binary alpha
            if (format == Format::Bc1) && (rgba[i][3] < 128u8) {
                set.remap[i] = -1;
                set.transparent = true;
                continue;
            }

            // loop over previous points in case the colour is duplicated in this block
            for j in 0..rgba.len() {
                // no duplicates found, store new point
                if j == i {
                    // normalise coordinates to [0,1]
                    let x = f32::from(rgba[i][0]) / 255f32;
                    let y = f32::from(rgba[i][1]) / 255f32;
                    let z = f32::from(rgba[i][2]) / 255f32;

                    // ensure weight is always nonzero even when alpha is not
                    let w = (i32::from(rgba[i][3]) + 1) as f32 / 256f32;

                    // store point
                    set.points[set.count] = Vec3::new(x, y, z);
                    set.weights[set.count] = if alpha_weighted { w } else { 1f32 };
                    set.remap[i] = set.count as i8;

                    // move to next pixel
                    set.count += 1;
                    break;
                }

                // check for duplicates
                let oldbit = 1u32 << j;
                let duplicate = ((mask & oldbit) != 0)
                    && (rgba[i][0] == rgba[j][0])
                    && (rgba[i][1] == rgba[j][1])
                    && (rgba[i][2] == rgba[j][2])
                    && (format != Format::Bc1 || rgba[j][3] >= 128u8);
                if duplicate {
                    // get index of duplicate
                    let index = set.remap[j];

                    // ensure weight is always nonzero even when alpha is not
                    let w = (i32::from(rgba[i][3]) + 1) as f32 / 256f32;

                    // map this point to its duplicate and increase the duplicate's weight
                    set.weights[index as usize] += if alpha_weighted { w } else { 1f32 };
                    set.remap[i] = index;

                    // move to next pixel
                    break;
                }
            }
        }

        // square root the weights
        for w in set.weights.iter_mut() {
            *w = w.sqrt();
        }

        set
    }

    pub fn is_transparent(&self) -> bool {
        self.transparent
    }

    pub fn points(&self) -> &[Vec3] {
        &self.points[..self.count]
    }

    pub fn weights(&self) -> &[f32] {
        &self.weights[..self.count]
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn remap_indices(&self, source: &[u8; 16], target: &mut [u8; 16]) {
        for (i, target) in target.iter_mut().enumerate() {
            let j = self.remap[i];

            if j == -1 {
                // palette has 4 elements, last one is transparent black if transparency is used
                *target = 3;
            } else {
                *target = source[j as usize];
            }
        }
    }
}
