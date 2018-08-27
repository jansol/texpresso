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


use std::path::PathBuf;
use std::fs::File;

use png::{BitDepth, ColorType, Decoder, HasParameters, Transformations};

use super::RawImage;

pub fn read(path: PathBuf) -> RawImage {
    let file = File::open(path).expect("Failed to open file");
    let mut decoder = Decoder::new(file);
    decoder.set(Transformations::EXPAND);

    let (info, mut reader) = decoder.read_info()
        .expect("Failed to read PNG header. Is this really a PNG file?");
    if info.bit_depth != BitDepth::Eight {
        panic!("Only images with 8 bits per channel are supported");
    }

    let channels = match info.color_type {
        ColorType::Grayscale => 1,
        ColorType::GrayscaleAlpha => 2,
        ColorType::RGB => 3,
        ColorType::RGBA => 4,
        ColorType::Indexed => {
            panic!("Image should be de-indexed already");
        }
    };

    // Preallocate the output buffer.
    let mut buf = vec![0; info.buffer_size()];

    // Read the next frame. Currently this function should only called once.
    reader.next_frame(&mut buf).unwrap();

    // duck tape missing channels in
    buf = match channels {
        1 => buf[..].iter()
            .flat_map(|&r| vec![r, 0, 0, 255])
            .collect::<Vec<u8>>(),
        2 => buf[..].chunks(2)
            .flat_map(|rg| vec![rg[0], rg[1], 0, 255])
            .collect::<Vec<u8>>(),
        3 => buf[..].chunks(3)
            .flat_map(|rgb| vec![rgb[0], rgb[1], rgb[2], 255])
            .collect::<Vec<u8>>(),
        4 => buf,
        _ => unreachable!()
    };

    RawImage {
        width: info.width as usize,
        height: info.height as usize,
        data: buf,
    }
}
