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

use core::{mem, u8};

use byteorder::{ByteOrder, LittleEndian};

use f32_to_i32_clamped;
use math::Vec3;

/// Convert a colour value to a little endian u16
fn pack_565(colour: &Vec3) -> u16 {
    let r = f32_to_i32_clamped(31.0 * colour.x(), 31) as u8;
    let g = f32_to_i32_clamped(63.0 * colour.y(), 63) as u8;
    let b = f32_to_i32_clamped(31.0 * colour.z(), 31) as u8;

    LittleEndian::read_u16(&[(g << 5) | b, (r << 3) | (g >> 3)])
}

fn write_block(a: u16, b: u16, indices: &[u8; 16], block: &mut [u8]) {
    // write endpoints
    LittleEndian::write_u16(&mut &mut block[0..2], a);
    LittleEndian::write_u16(&mut &mut block[2..4], b);

    // write 2-bit LUT indices
    let mut packed = [0u8; 4];
    for i in 0..packed.len() {
        packed[i] = ((indices[4 * i + 3] & 0x03) << 6)
            | ((indices[4 * i + 2] & 0x03) << 4)
            | ((indices[4 * i + 1] & 0x03) << 2)
            | (indices[4 * i] & 0x03);
    }

    block[4..].copy_from_slice(&packed);
}

pub fn write3(start: &Vec3, end: &Vec3, indices: &[u8; 16], block: &mut [u8]) {
    let mut a = pack_565(start);
    let mut b = pack_565(end);

    let mut remapped = *indices;

    if a > b {
        // swap a, b and indices referring to them
        mem::swap(&mut a, &mut b);
        for index in &mut remapped[..] {
            *index = match *index {
                0 => 1,
                1 => 0,
                x => x,
            };
        }
    }

    write_block(a, b, &remapped, block);
}

pub fn write4(start: &Vec3, end: &Vec3, indices: &[u8; 16], block: &mut [u8]) {
    let mut a = pack_565(start);
    let mut b = pack_565(end);

    // remap indices
    let mut remapped = [0u8; 16];
    if a < b {
        mem::swap(&mut a, &mut b);
        for (mut remapped, index) in remapped.iter_mut().zip(indices) {
            *remapped = (index ^ 0x01) & 0x03;
        }
    } else if a > b {
        // use indices as-is
        remapped = *indices;
    }
    // if a == b, use index 0 for everything, i.e. no need to do anything

    write_block(a, b, &remapped, block);
}

/// Convert a little endian 565-packed colour to 8bpc RGBA
fn unpack_565(packed: &[u8]) -> [u8; 4] {
    assert!(packed.len() == 2);
    // get components
    let value: u16 = u16::from(packed[0]) | (u16::from(packed[1]) << 8);
    let r = ((value >> 11) & 0x1F) as u8;
    let g = ((value >> 5) & 0x3F) as u8;
    let b = (value & 0x1F) as u8;

    // scale up to 8 bits
    let r = (r << 3) | (r >> 2);
    let g = (g << 2) | (g >> 4);
    let b = (b << 3) | (b >> 2);

    [r, g, b, 255u8]
}

/// Decompress a BC1/2/3 block to 4x4 RGBA pixels
pub fn decompress(bytes: &[u8], is_bc1: bool) -> [[u8; 4]; 16] {
    assert!(bytes.len() == 8);

    let mut codes = [0u8; 16];

    // unpack endpoints
    let a = LittleEndian::read_u16(&bytes[0..2]);
    let b = LittleEndian::read_u16(&bytes[2..4]);
    codes[0..4].copy_from_slice(&unpack_565(&bytes[0..2]));
    codes[4..8].copy_from_slice(&unpack_565(&bytes[2..4]));

    // generate intermediate values
    for i in 0..4 {
        let c = u32::from(codes[i]);
        let d = u32::from(codes[4 + i]);

        if is_bc1 && (a <= b) {
            codes[8 + i] = ((c + d) / 2) as u8;
            codes[12 + i] = 0;
        } else {
            codes[8 + i] = ((2 * c + d) / 3) as u8;
            codes[12 + i] = ((c + 2 * d) / 3) as u8;
        }
    }

    // fill in alpha for intermediate values
    codes[8 + 3] = u8::MAX;
    codes[12 + 3] = if is_bc1 && (a <= b) { 0u8 } else { u8::MAX };

    // unpack LUT indices
    let mut indices = [0u8; 16];
    for i in 0..4 {
        let ind = &mut indices[4 * i..4 * i + 4];
        let packed = bytes[4 + i];

        ind[0] = packed & 0x03;
        ind[1] = (packed >> 2) & 0x03;
        ind[2] = (packed >> 4) & 0x03;
        ind[3] = (packed >> 6) & 0x03;
    }

    let mut rgba = [[0u8; 4]; 16];
    for i in 0..rgba.len() {
        let offset = 4 * indices[i] as usize;
        let length = rgba[i].len();

        rgba[i].copy_from_slice(&codes[offset..(offset + length)])
    }

    rgba
}
