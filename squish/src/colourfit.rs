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

mod cluster;
mod range;
mod single;
mod single_lut;

pub use self::cluster::ClusterFit;
pub use self::range::RangeFit;
pub use self::single::SingleColourFit;

pub trait ColourFit<'a> {
    fn compress(&'a mut self, block: &mut [u8]);
}

pub trait ColourFitImpl<'a> {
    fn is_bc1(&self) -> bool;
    fn is_transparent(&self) -> bool;
    fn compress3(&mut self);
    fn compress4(&mut self);
    fn best_compressed(&'a self) -> &'a [u8];
}

impl<'a, T> ColourFit<'a> for T
where
    T: ColourFitImpl<'a>,
{
    fn compress(&'a mut self, block: &mut [u8]) {
        if self.is_bc1() {
            self.compress3();
            if !self.is_transparent() {
                self.compress4();
            }
        } else {
            self.compress4();
        }

        block.copy_from_slice(self.best_compressed());
    }
}
