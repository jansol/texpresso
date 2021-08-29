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

use core::f32;

mod vec3;
pub use self::vec3::*;

mod vec4;
pub use self::vec4::*;

/// A type representing a symmetric 3x3 matrix
pub struct Sym3x3 {
    x: [f32; 6],
}

/// Symmetric eigensystem solver algorithm from http://www.geometrictools.com/Documentation/EigenSymmetric3x3.pdf
impl Sym3x3 {
    pub fn new(s: f32) -> Self {
        Self {
            x: [s, s, s, s, s, s],
        }
    }

    pub fn weighted_covariance(points: &[Vec3], weights: &[f32]) -> Self {
        assert!(points.len() == weights.len());

        // compute the centroid
        let total: f32 = weights.iter().sum();
        let centroid: Vec3 = points.iter().zip(weights).map(|(p, &w)| p * w).sum();

        let centroid = if total > f32::EPSILON {
            centroid / total
        } else {
            centroid
        };

        // accumulate the covariance matrix
        let mut covariance = Sym3x3::new(0.0);

        for (p, &w) in points.iter().zip(weights) {
            let a: Vec3 = p - &centroid;
            let b = a * w;

            covariance.x[..][0] += a.x() * b.x();
            covariance.x[..][1] += a.x() * b.y();
            covariance.x[..][2] += a.x() * b.z();
            covariance.x[..][3] += a.y() * b.y();
            covariance.x[..][4] += a.y() * b.z();
            covariance.x[..][5] += a.z() * b.z();
        }

        covariance
    }

    pub fn principle_component(&self) -> Vec3 {
        const POWER_ITERATION_COUNT: usize = 8;

        let row0 = Vec4::new(self.x[0], self.x[1], self.x[2], 0.0);
        let row1 = Vec4::new(self.x[1], self.x[3], self.x[4], 0.0);
        let row2 = Vec4::new(self.x[2], self.x[4], self.x[5], 0.0);
        let mut v = Vec4::new(1.0, 1.0, 1.0, 1.0);

        for _ in 0..POWER_ITERATION_COUNT {
            // matrix multiplication
            let w = row0 * v.splat_x();
            let w = row1 * v.splat_y() + w;
            let w = row2 * v.splat_z() + w;

            // Construct Vec4 with max component from xyz in all channels
            let a = w.x().max(w.y().max(w.z()));
            let a = Vec4::new(a, a, a, a);

            v = w * a.reciprocal();
        }

        v.to_vec3()
    }
}


pub fn f32_to_i32_clamped(a: f32, limit: i32) -> i32 {
    libm::roundf(a).max(0.0).min(limit as f32) as i32
}
