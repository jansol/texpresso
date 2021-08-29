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

use core::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

use super::Vec3;

#[derive(Copy, Clone, PartialEq)]
pub struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn z(&self) -> f32 {
        self.z
    }

    pub fn w(&self) -> f32 {
        self.w
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    pub fn splat_x(&self) -> Vec4 {
        Vec4::new(self.x, self.x, self.x, self.x)
    }

    pub fn splat_y(&self) -> Vec4 {
        Vec4::new(self.y, self.y, self.y, self.y)
    }

    pub fn splat_z(&self) -> Vec4 {
        Vec4::new(self.z, self.z, self.z, self.z)
    }

    pub fn splat_w(&self) -> Vec4 {
        Vec4::new(self.w, self.w, self.w, self.w)
    }

    pub fn max(&self, other: Vec4) -> Vec4 {
        Vec4::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
            self.w.max(other.w),
        )
    }

    pub fn min(&self, other: Vec4) -> Vec4 {
        Vec4::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
            self.w.min(other.w),
        )
    }

    pub fn reciprocal(&self) -> Vec4 {
        Vec4::new(1.0 / self.x, 1.0 / self.y, 1.0 / self.z, 1.0 / self.w)
    }

    pub fn any_less_than(&self, other: &Vec4) -> bool {
        self.x < other.x || self.y < other.y || self.z < other.z || self.w < other.w
    }

    pub fn truncate(&self) -> Vec4 {
        Vec4::new(
            libm::truncf(self.x),
            libm::truncf(self.y),
            libm::truncf(self.z),
            libm::truncf(self.w)
        )
    }
}

impl Add for Vec4 {
    type Output = Vec4;

    fn add(self, other: Vec4) -> Vec4 {
        Vec4::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
            self.w + other.w,
        )
    }
}

impl<'a> Add for &'a Vec4 {
    type Output = Vec4;

    fn add(self, other: &'a Vec4) -> Vec4 {
        Vec4::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
            self.w + other.w,
        )
    }
}

impl<'a> Add<Vec4> for &'a Vec4 {
    type Output = Vec4;

    fn add(self, other: Vec4) -> Vec4 {
        Vec4::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
            self.w + other.w,
        )
    }
}

impl<'a> Add<&'a Vec4> for Vec4 {
    type Output = Vec4;

    fn add(self, other: &'a Vec4) -> Vec4 {
        Vec4::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
            self.w + other.w,
        )
    }
}

impl Add<f32> for Vec4 {
    type Output = Vec4;

    fn add(self, other: f32) -> Vec4 {
        Vec4::new(
            self.x + other,
            self.y + other,
            self.z + other,
            self.w + other,
        )
    }
}

impl<'a> Add<f32> for &'a Vec4 {
    type Output = Vec4;

    fn add(self, other: f32) -> Vec4 {
        Vec4::new(
            self.x + other,
            self.y + other,
            self.z + other,
            self.w + other,
        )
    }
}

impl AddAssign<Vec4> for Vec4 {
    fn add_assign(&mut self, other: Vec4) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
        self.w += other.w;
    }
}

impl<'a> AddAssign<&'a Vec4> for Vec4 {
    fn add_assign(&mut self, other: &'a Vec4) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
        self.w += other.w;
    }
}

impl AddAssign<f32> for Vec4 {
    fn add_assign(&mut self, other: f32) {
        self.x += other;
        self.y += other;
        self.z += other;
        self.w += other;
    }
}

impl Sub for Vec4 {
    type Output = Vec4;

    fn sub(self, other: Vec4) -> Vec4 {
        Vec4::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
            self.w - other.w,
        )
    }
}

impl<'a> Sub for &'a Vec4 {
    type Output = Vec4;

    fn sub(self, other: &'a Vec4) -> Vec4 {
        Vec4::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
            self.w - other.w,
        )
    }
}

impl<'a> Sub<Vec4> for &'a Vec4 {
    type Output = Vec4;

    fn sub(self, other: Vec4) -> Vec4 {
        Vec4::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
            self.w - other.w,
        )
    }
}

impl<'a> Sub<&'a Vec4> for Vec4 {
    type Output = Vec4;

    fn sub(self, other: &'a Vec4) -> Vec4 {
        Vec4::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
            self.w - other.w,
        )
    }
}

impl Sub<f32> for Vec4 {
    type Output = Vec4;

    fn sub(self, other: f32) -> Vec4 {
        Vec4::new(
            self.x - other,
            self.y - other,
            self.z - other,
            self.w - other,
        )
    }
}

impl<'a> Sub<f32> for &'a Vec4 {
    type Output = Vec4;

    fn sub(self, other: f32) -> Vec4 {
        Vec4::new(
            self.x - other,
            self.y - other,
            self.z - other,
            self.w - other,
        )
    }
}

impl SubAssign<Vec4> for Vec4 {
    fn sub_assign(&mut self, other: Vec4) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
        self.w -= other.w;
    }
}

impl<'a> SubAssign<&'a Vec4> for Vec4 {
    fn sub_assign(&mut self, other: &'a Vec4) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
        self.w -= other.w;
    }
}

impl SubAssign<f32> for Vec4 {
    fn sub_assign(&mut self, other: f32) {
        self.x -= other;
        self.y -= other;
        self.z -= other;
        self.w -= other;
    }
}

impl Mul for Vec4 {
    type Output = Vec4;

    fn mul(self, other: Vec4) -> Vec4 {
        Vec4::new(
            self.x * other.x,
            self.y * other.y,
            self.z * other.z,
            self.w * other.w,
        )
    }
}

impl<'a> Mul for &'a Vec4 {
    type Output = Vec4;

    fn mul(self, other: &'a Vec4) -> Vec4 {
        Vec4::new(
            self.x * other.x,
            self.y * other.y,
            self.z * other.z,
            self.w * other.w,
        )
    }
}

impl<'a> Mul<Vec4> for &'a Vec4 {
    type Output = Vec4;

    fn mul(self, other: Vec4) -> Vec4 {
        Vec4::new(
            self.x * other.x,
            self.y * other.y,
            self.z * other.z,
            self.w * other.w,
        )
    }
}

impl<'a> Mul<&'a Vec4> for Vec4 {
    type Output = Vec4;

    fn mul(self, other: &'a Vec4) -> Vec4 {
        Vec4::new(
            self.x * other.x,
            self.y * other.y,
            self.z * other.z,
            self.w * other.w,
        )
    }
}

impl Mul<f32> for Vec4 {
    type Output = Vec4;

    fn mul(self, other: f32) -> Vec4 {
        Vec4::new(
            self.x * other,
            self.y * other,
            self.z * other,
            self.w * other,
        )
    }
}

impl<'a> Mul<f32> for &'a Vec4 {
    type Output = Vec4;

    fn mul(self, other: f32) -> Vec4 {
        Vec4::new(
            self.x * other,
            self.y * other,
            self.z * other,
            self.w * other,
        )
    }
}

impl MulAssign<Vec4> for Vec4 {
    fn mul_assign(&mut self, other: Vec4) {
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
        self.w *= other.w;
    }
}

impl<'a> MulAssign<&'a Vec4> for Vec4 {
    fn mul_assign(&mut self, other: &'a Vec4) {
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
        self.w *= other.w;
    }
}

impl MulAssign<f32> for Vec4 {
    fn mul_assign(&mut self, other: f32) {
        self.x *= other;
        self.y *= other;
        self.z *= other;
        self.w *= other;
    }
}
