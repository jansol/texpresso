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

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use png::{BitDepth, ColorType, Transformations};

use super::RawImage;

pub fn read(path: &Path) -> RawImage {
    let file = File::open(path).expect("Failed to open file");
    let mut decoder = png::Decoder::new(file);
    decoder.set_transformations(Transformations::EXPAND);

    let mut reader = decoder
        .read_info()
        .expect("Failed to read PNG header. Is this really a PNG file?");

    // Preallocate the output buffer.
    let mut buf = vec![0; reader.output_buffer_size()];

    // Read the next frame. Currently this function should only called once.
    reader.next_frame(&mut buf).unwrap();

    let info = reader.info();
    if info.bit_depth != BitDepth::Eight {
        panic!("Only images with 8 bits per channel are supported");
    }

    // expand to rgba
    buf = match info.color_type {
        ColorType::Grayscale => buf[..]
            .iter()
            .flat_map(|&r| vec![r, r, r, 255])
            .collect::<Vec<u8>>(),
        ColorType::GrayscaleAlpha => buf[..]
            .chunks(2)
            .flat_map(|rg| vec![rg[0], rg[0], rg[0], rg[1]])
            .collect::<Vec<u8>>(),
        ColorType::Rgb => buf[..]
            .chunks(3)
            .flat_map(|rgb| vec![rgb[0], rgb[1], rgb[2], 255])
            .collect::<Vec<u8>>(),
        ColorType::Rgba => buf,
        _ => unreachable!(),
    };

    RawImage {
        width: info.width as usize,
        height: info.height as usize,
        data: buf,
    }
}

pub fn write(path: &Path, width: u32, height: u32, data: &[u8]) {
    let file = File::create(path).expect("Unable to create file");
    let w = &mut BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(data).unwrap();
}
