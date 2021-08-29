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
use std::path::Path;

use jpeg_decoder::{Decoder, PixelFormat};

use super::RawImage;

pub fn read(path: &Path) -> RawImage {
    let file = File::open(path).expect("Failed to open file");
    let mut decoder = Decoder::new(file);
    decoder
        .read_info()
        .expect("Failed to read JPEG header. Is this really a JPEG file?");

    // Decode the image
    let info = decoder.info().unwrap();

    let mut buf = decoder.decode().unwrap();
    buf = match info.pixel_format {
        PixelFormat::L8 => buf[..]
            .iter()
            .flat_map(|&l| vec![l, l, l, 255u8])
            .collect::<Vec<u8>>(),
        PixelFormat::RGB24 => buf[..]
            .chunks(3)
            .flat_map(|rgb| vec![rgb[0], rgb[1], rgb[2], 255u8])
            .collect::<Vec<u8>>(),
        PixelFormat::CMYK32 => panic!("CMYK images are not supported!"),
    };

    RawImage {
        width: info.width as usize,
        height: info.height as usize,
        data: buf,
    }
}
