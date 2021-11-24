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

use core::cmp::Ordering;
use core::f32;

use crate::colourblock;
use crate::colourset::ColourSet;
use crate::math::{Sym3x3, Vec3, Vec4};
use crate::{ColourWeights, Format};

use super::ColourFitImpl;

const MAX_ITERATIONS: usize = 8;

pub struct ClusterFit<'a> {
    colourset: &'a ColourSet,
    format: Format,
    weights: Vec4,
    num_iterations: usize,
    principle: Vec3,
    order: [[u8; 16]; MAX_ITERATIONS],
    points_weights: [Vec4; 16],
    xsum_wsum: Vec4,
    best_error: Vec4,
    best_compressed: [u8; 8],
}

impl<'a> ClusterFit<'a> {
    pub fn new(
        colourset: &'a ColourSet,
        format: Format,
        weights: ColourWeights,
        iterate: bool,
    ) -> Self {
        let mut fit = ClusterFit {
            colourset,
            format,
            weights: Vec4::new(weights[0], weights[1], weights[2], 1.0),
            num_iterations: if iterate { MAX_ITERATIONS } else { 1 },
            principle: Vec3::new(0.0, 0.0, 0.0),
            order: [[0u8; 16]; MAX_ITERATIONS],
            points_weights: [Vec4::new(0.0, 0.0, 0.0, 0.0); 16],
            xsum_wsum: Vec4::new(0.0, 0.0, 0.0, 0.0),
            best_error: Vec4::new(f32::MAX, f32::MAX, f32::MAX, f32::MAX),
            best_compressed: [0u8; 8],
        };

        // get the covariance matrix
        let covariance =
            Sym3x3::weighted_covariance(fit.colourset.points(), fit.colourset.weights());

        // get the principle component
        fit.principle = covariance.principle_component();

        fit
    }

    fn construct_ordering(&mut self, axis: &Vec3, iteration: usize) -> bool {
        // cache some values
        let count = self.colourset.count();
        let values = self.colourset.points();

        // build list of dot products
        let mut dps = [(0usize, f32::MAX); 16];
        for i in 0..count {
            dps[i] = (i, values[i].dot(axis));
        }

        // sort fn for floats - NaN & Inf are pushed to the end of the list
        fn fcmp(a: f32, b: f32) -> Ordering {
            match (a, b) {
                (x, y) if !x.is_finite() && !y.is_finite() => Ordering::Equal,
                (x, _) if !x.is_finite() => Ordering::Greater,
                (_, y) if !y.is_finite() => Ordering::Less,
                (_, _) => a.partial_cmp(&b).unwrap(),
            }
        }

        // sort numbered list based on dot product value
        dps.sort_unstable_by(|a, b| fcmp(a.1, b.1));

        // this is our ordering now
        for (a, b) in self.order[iteration].iter_mut().zip(dps.iter()) {
            *a = b.0 as u8;
        }

        // check if this ordering is unique (does not run on iteration 0)
        for i in 0..iteration {
            let mut same = true;
            for j in 0..self.order[iteration].len() {
                if self.order[iteration][j] != self.order[i][j] {
                    same = false;
                    break;
                }
            }

            if same {
                return false;
            }
        }

        // copy the ordering and weigh all the points
        let unweighted = self.colourset.points();
        let weights = self.colourset.weights();
        self.xsum_wsum = Vec4::new(0.0, 0.0, 0.0, 0.0);
        for i in 0..count {
            let j = self.order[iteration][i] as usize;
            let p = Vec4::new(unweighted[j].x(), unweighted[j].y(), unweighted[j].z(), 1.0);
            let w = Vec4::new(weights[j], weights[j], weights[j], weights[j]);
            let x = p * w;
            self.points_weights[i] = x;
            self.xsum_wsum += x;
        }

        true
    }
}

impl<'a> ColourFitImpl<'a> for ClusterFit<'a> {
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
        let count = self.colourset.count();
        let two = Vec4::new(2.0, 2.0, 2.0, 2.0);
        let one = Vec4::new(1.0, 1.0, 1.0, 1.0);
        let half_half2 = Vec4::new(0.5, 0.5, 0.5, 0.25);
        let zero = Vec4::new(0.0, 0.0, 0.0, 0.0);
        let half = Vec4::new(0.5, 0.5, 0.5, 0.5);
        let grid = Vec4::new(31.0, 63.0, 31.0, 0.0);
        let gridrcp = Vec4::new(1.0 / 31.0, 1.0 / 63.0, 1.0 / 31.0, 0.0);

        // check all possible clusters and iterate on the total order
        let mut best_start = zero;
        let mut best_end = zero;
        let mut best_error = self.best_error;
        let mut best_indices = [0u8; 16];
        let mut best_iteration = 0;
        let mut best_i = 0;
        let mut best_j = 0;

        // inital ordering is computed using principle axis
        let mut axis = self.principle;

        for iteration_index in 0..self.num_iterations {
            // generate new unique ordering, if possible
            if !self.construct_ordering(&axis, iteration_index) {
                break;
            }

            // first cluster [0,i) is at the start
            let mut part0 = zero;
            for i in 0..count {
                // second cluster [i,j) is halfway along
                let mut part1 = if i == 0 { self.points_weights[0] } else { zero };
                let jmin = if i == 0 { 1 } else { i };

                for j in jmin..=count {
                    // last cluster [j,count) is at the end
                    let part2 = self.xsum_wsum - part1 - part0;

                    // compute least squares term directly
                    let alphax_sum = part1 * half_half2 + part0;
                    let alpha2_sum = alphax_sum.splat_w();

                    let betax_sum = part1 * half_half2 + part2;
                    let beta2_sum = betax_sum.splat_w();

                    let alphabeta_sum = (part1 * half_half2).splat_w();

                    // compute the least-squares optimal points
                    let factor =
                        ((alpha2_sum * beta2_sum) - alphabeta_sum * alphabeta_sum).reciprocal();
                    let a = ((alphax_sum * beta2_sum) - betax_sum * alphabeta_sum) * factor;
                    let b = ((betax_sum * alpha2_sum) - alphax_sum * alphabeta_sum) * factor;

                    // clamp to the grid
                    let a = one.min(zero.max(a));
                    let b = one.min(zero.max(b));
                    let a = (grid * a + half).truncate() * gridrcp;
                    let b = (grid * b + half).truncate() * gridrcp;

                    // compute the error (we skip the constant xxsum)
                    let e1 = (a * a) * alpha2_sum + (b * b * beta2_sum);
                    let e2 = (a * b * alphabeta_sum) - a * alphax_sum;
                    let e3 = e2 - b * betax_sum;
                    let e4 = two * e3 + e1;

                    // apply the channel weights to the error term
                    let e5 = e4 * self.weights;
                    let error = e5.splat_x() + e5.splat_y() + e5.splat_z();

                    // keep the solution if it wins
                    if error.any_less_than(&best_error) {
                        best_start = a;
                        best_end = b;
                        best_i = i;
                        best_j = j;
                        best_error = error;
                        best_iteration = iteration_index;
                    }

                    // advance
                    if j < count {
                        part1 += self.points_weights[j];
                    }
                }

                // advance
                part0 += self.points_weights[i];
            }

            // stop if we didn't improve in this iteration
            if best_iteration != iteration_index {
                break;
            }

            // compute new axis for next iteration
            axis = (best_end - best_start).to_vec3();
        }

        // save the block if necessary
        if best_error.any_less_than(&self.best_error) {
            // remap indices
            let order = self.order[best_iteration];

            let mut unordered = [0u8; 16];
            for m in best_i..best_j {
                unordered[order[m] as usize] = 2;
            }
            for m in best_j..count {
                unordered[order[m] as usize] = 1;
            }

            self.colourset.remap_indices(&unordered, &mut best_indices);

            // generate the compressed blob
            let a = best_start.to_vec3();
            let b = best_end.to_vec3();
            colourblock::write3(&a, &b, &best_indices, &mut self.best_compressed);

            // save the error
            self.best_error = best_error;
        }
    }

    fn compress4(&mut self) {
        let count = self.colourset.count();
        let two = Vec4::new(2.0, 2.0, 2.0, 2.0);
        let one = Vec4::new(1.0, 1.0, 1.0, 1.0);
        let onethird_onethird2 = Vec4::new(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0, 1.0 / 9.0);
        let twothirds_twothirds2 = Vec4::new(2.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0, 4.0 / 9.0);
        let twoninths = Vec4::new(2.0 / 9.0, 2.0 / 9.0, 2.0 / 9.0, 2.0 / 9.0);
        let zero = Vec4::new(0.0, 0.0, 0.0, 0.0);
        let half = Vec4::new(0.5, 0.5, 0.5, 0.5);
        let grid = Vec4::new(31.0, 63.0, 31.0, 0.0);
        let gridrcp = Vec4::new(1.0 / 31.0, 1.0 / 63.0, 1.0 / 31.0, 0.0);

        // check all possible clusters and iterate on the total order
        let mut best_start = zero;
        let mut best_end = zero;
        let mut best_error = self.best_error;
        let mut best_indices = [0u8; 16];
        let mut best_iteration = 0;
        let mut best_i = 0;
        let mut best_j = 0;
        let mut best_k = 0;

        // inital ordering is computed using principle axis
        let mut axis = self.principle;

        for iteration_index in 0..self.num_iterations {
            // generate new unique ordering, if possible
            if !self.construct_ordering(&axis, iteration_index) {
                break;
            }

            // first cluster [0,i) is at the start
            let mut part0 = zero;
            for i in 0..count {
                // second cluster [i,j) is one third along
                let mut part1 = zero;

                for j in i..=count {
                    // third cluster [j, k) is two thirds along
                    let mut part2 = if j == 0 { self.points_weights[0] } else { zero };
                    let kmin = if j == 0 { 1 } else { j };

                    for k in kmin..=count {
                        // last cluster [k, count) is at the end
                        let part3 = self.xsum_wsum - part2 - part1 - part0;

                        // compute least squares terms directly
                        let alphax_sum =
                            part2 * onethird_onethird2 + (part1 * twothirds_twothirds2 + part0);
                        let alpha2_sum = alphax_sum.splat_w();

                        let betax_sum =
                            part1 * onethird_onethird2 + (part2 * twothirds_twothirds2 + part3);
                        let beta2_sum = betax_sum.splat_w();

                        let alphabeta_sum = twoninths * (part1 + part2).splat_w();

                        // compute the least-squares optimal points
                        let factor =
                            ((alpha2_sum * beta2_sum) - alphabeta_sum * alphabeta_sum).reciprocal();
                        let a = ((alphax_sum * beta2_sum) - betax_sum * alphabeta_sum) * factor;
                        let b = ((betax_sum * alpha2_sum) - alphax_sum * alphabeta_sum) * factor;

                        // clamp to the grid
                        let a = one.min(zero.max(a));
                        let b = one.min(zero.max(b));
                        let a = (grid * a + half).truncate() * gridrcp;
                        let b = (grid * b + half).truncate() * gridrcp;

                        // compute the error (we skip the constant xxsum)
                        let e1 = (a * a) * alpha2_sum + (b * b * beta2_sum);
                        let e2 = (a * b * alphabeta_sum) - a * alphax_sum;
                        let e3 = e2 - b * betax_sum;
                        let e4 = two * e3 + e1;

                        // apply the channel weights to the error term
                        let e5 = e4 * self.weights;
                        let error = e5.splat_x() + e5.splat_y() + e5.splat_z();

                        // keep the solution if it wins
                        if error.any_less_than(&best_error) {
                            best_start = a;
                            best_end = b;
                            best_i = i;
                            best_j = j;
                            best_k = k;
                            best_error = error;
                            best_iteration = iteration_index;
                        }

                        // advance
                        if k < count {
                            part2 += self.points_weights[k];
                        }
                    }

                    // advance
                    if j < count {
                        part1 += self.points_weights[j];
                    }
                }

                // advance
                part0 += self.points_weights[i];
            }

            // stop if we didn't improve in this iteration
            if best_iteration != iteration_index {
                break;
            }

            // compute new axis for next iteration
            axis = (best_end - best_start).to_vec3();
        }

        // save the block if necessary
        if best_error.any_less_than(&self.best_error) {
            // remap indices
            let order = self.order[best_iteration];

            let mut unordered = [0u8; 16];
            for m in best_i..best_j {
                unordered[order[m] as usize] = 2;
            }
            for m in best_j..count {
                unordered[order[m] as usize] = 3;
            }
            for m in best_k..count {
                unordered[order[m] as usize] = 1;
            }

            self.colourset.remap_indices(&unordered, &mut best_indices);

            // generate the compressed blob
            let a = best_start.to_vec3();
            let b = best_end.to_vec3();
            colourblock::write4(&a, &b, &best_indices, &mut self.best_compressed);

            // save the error
            self.best_error = best_error;
        }
    }
}
