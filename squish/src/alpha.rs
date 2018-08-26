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


use std::u32;

use ::f32_to_i32_clamped;


pub fn compress_alpha_dxt3(
    rgba: &[[u8; 4]; 16],
    mask: u32,
    block: &mut Vec<u8>
) {
    let mut tmp = [0u8; 8];
    for i in 0..tmp.len() {
        // quantise down to 4 bits
        let alpha1 = rgba[2*i][3] as f32 * (15.0/255.0);
        let alpha2 = rgba[2*i + 1][3] as f32 * (15.0/255.0);
        let mut quant1 = f32_to_i32_clamped(alpha1, 15) as u8;
        let mut quant2 = f32_to_i32_clamped(alpha2, 15) as u8;

        // set alpha to zero where masked
        let bit1 = 1 << (2*i);
        let bit2 = 1 << (2*i + 1);
        if (mask & bit1) == 0 {
            quant1 = 0;
        }
        if (mask & bit2) == 0 {
            quant2 = 0;
        }

        // pack into byte
        tmp[i] = quant1 | (quant2 << 4)
    }

    block.extend_from_slice(&tmp);
}

pub fn decompress_alpha_dxt3(rgba: &mut [[u8; 4]; 16], bytes: &[u8]) {
    assert!(bytes.len() == 8);

    // unpack alpha values pairwise
    for i in 0..bytes.len() {
        let quant = bytes[i];

        // unpack
        let lo = quant & 0x0F;
        let hi = quant & 0xF0;
        
        // convert back up to bytes
        rgba[2*i][3] = lo | (lo << 4);
        rgba[2*i + 1][3] = hi | (hi << 4);
    }
}

fn fix_range(min: &mut u8, max: &mut u8, steps: u8) {
    if (*max-*min) < steps {
        *max = (*min as i32 + steps as i32).min(255) as u8;
    }
    if (*max-*min) < steps {
        *min = (*max as i32 - steps as i32).max(0) as u8;
    }
}

fn fit_codes(
    rgba: &[[u8; 4]; 16],
    mask: u32,
    codes: &[u8; 8],
    indices: &mut [u8; 16]
) -> u32 {
    let mut err = 0;

    // fit each alpha value to the codebook
    for i in 0..16 {
        // check if pixel is valid
        let bit = 1 << i;
        if (mask & bit) == 0 {
            // use the first code
            indices[i] = 0;
            continue;
        }

        let value = rgba[i][3];
        let mut least = u32::MAX;
        let mut index = 0;
        for j in 0..8 {
            // get squared error from this code
            let dist = value as i32 - codes[j] as i32;
            let dist = (dist*dist) as u32;

            // compare with best so far
            if dist < least {
                least = dist;
                index = j as u8;
            }
        }

        // save this index and accumulate the error
        indices[i] = index;
        err += least;
    }

    err
}

fn write_alpha_block(
    alpha0: u8,
    alpha1: u8,
    indices: &[u8; 16],
    block: &mut Vec<u8>
) {
    // write endpoints
    block.extend_from_slice(&[alpha0, alpha1]);

    // pack the indices with 3 bits each
    for i in 0..2 {
        // pack 8 3-bit values
        let mut value = 0u32;
        for j in 0..8 {
            let index = indices[8*i+j] as u32;
            value |= index << 3*j;
        }

        // store in 3 bytes
        let mut tmp = [0u8; 3];
        for j in 0..tmp.len() {
            tmp[j] = ((value >> 8*j) & 0xff) as u8;
        }
        block.extend_from_slice(&tmp);
    }
}

fn write_alpha_block5(
    alpha0: u8,
    alpha1: u8,
    indices: &[u8; 16],
    block: &mut Vec<u8>
) {
    if alpha0 > alpha1 {
        // invert indices
        let mut swapped = *indices;
        for index in &mut swapped[..] {
            *index = match *index {
                0 => 1,
                1 => 0,
                x @ 2...5 => 7-x,
                x => x
            }
        }

        // write with endpoints swapped
        write_alpha_block(alpha1, alpha0, &swapped, block);
    } else {
        // write as-is
        write_alpha_block(alpha0, alpha1, indices, block);
    }
}

fn write_alpha_block7(
    alpha0: u8,
    alpha1: u8,
    indices: &[u8; 16],
    block: &mut Vec<u8>
) {
    if alpha0 < alpha1 {
        // invert indices
        let mut swapped = *indices;
        for index in &mut swapped[..] {
            *index = match *index {
                0 => 1,
                1 => 0,
                x => 9-x
            }
        }

        // write with endpoints swapped
        write_alpha_block(alpha1, alpha0, &swapped, block);
    } else {
        // write as-is
        write_alpha_block(alpha0, alpha1, indices, block);
    }
}

pub fn compress_alpha_dxt5(
    rgba: &[[u8; 4]; 16],
    mask: u32,
    block: &mut Vec<u8>
) {
    // get range for 5-alpha and 7-alpha interpolation
    let mut min5 = 255u8;
    let mut max5 = 0u8;
    let mut min7 = 255u8;
    let mut max7 = 0u8;

    for i in 0..rgba.len() {
        // skip masked-out bits
        let bit = 1 << i;
        if (mask & bit) == 0 {
            continue;
        }

        // incorporate into the min/max
        let value = rgba[i][3];
        min7 = min7.min(value);
        max7 = max7.max(value);

        if value != 0 {
            min5 = min5.min(value);
        }
        if value != 255 {
            max5 = max5.max(value);
        }
    }

    // handle the case that no valid range was found
    if min5 > max5 {
        min5 = max5;
    }
    if min7 > max7 {
        min7 = max7;
    }

    // fix range to be the minimum in both cases
    fix_range(&mut min5, &mut max5, 5);
    fix_range(&mut min7, &mut max7, 7);

    // set up the 5-alpha codebook
    let mut codes5 = [0u8; 8];
    codes5[0] = min5;
    codes5[1] = max5;
    for i in 1..5i32 {
        codes5[1+i as usize] = (((5 - i)*min5 as i32 + i*max5 as i32)/5) as u8;
    }
    codes5[6] = 0;
    codes5[7] = 255;

    // set up the 7-alpha codebook
    let mut codes7 = [0u8; 8];
    codes7[0] = min5;
    codes7[1] = max5;
    for i in 1..7i32 {
        codes7[1+i as usize] = (((7 - i)*min7 as i32 + i*max7 as i32)/7) as u8;
    }

    // fit the data to both codebooks
    let mut indices5 = [0u8; 16];
    let mut indices7 = [0u8; 16];
    let err5 = fit_codes(rgba, mask, &codes5, &mut indices5);
    let err7 = fit_codes(rgba, mask, &codes7, &mut indices7);

    // save the block with the least error
    if err5 <= err7 {
        write_alpha_block5(min5, max5, &indices5, block);
    } else {
        write_alpha_block7(min7, max7, &indices7, block);
    }
}

pub fn decompress_alpha_dxt5(rgba: &mut [[u8; 4]; 16], bytes: &[u8]) {
    assert!(bytes.len() == 8);

    // get endpoint values
    let alpha0 = bytes[0] as i32;
    let alpha1 = bytes[1] as i32;

    // build the codebook
    let mut codes = [0u8; 8];
    codes[0] = alpha0 as u8;
    codes[1] = alpha1 as u8;
    if alpha0 <= alpha1 {
        // use 5-alpha codebook
        for i in 1..5i32 {
            codes[1+i as usize] = (((5 - i)*alpha0 + i*alpha1)/5) as u8
        }
        codes[6] = 0;
        codes[7] = 255;
    } else {
        // use 7-alpha codebook
        for i in 1..7i32 {
            codes[1+i as usize] = (((7 - i)*alpha0 + i*alpha1)/7) as u8;
        }
    }

    // decode the indices
    let mut indices = [0u8; 16];
    for i in 0..2 {
        // grab 3 bytes
        let mut value = 0i32;
        for j in 0..3 {
            let byte = bytes[2 + 3*i + j] as i32;
            value |= byte << 8*j;
        }

        // unpack 8 3-bit values from it
        for j in 0..8 {
            let index = (value >> 3*j) & 0x07;
            indices[8*i + j] = index as u8;
        }
    }

    // write out the indexed codebook values
    for i in 0..rgba.len() {
        rgba[i][3] = codes[indices[i] as usize];
    }
}