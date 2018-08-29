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


extern crate ddsfile;
extern crate jpeg_decoder;
extern crate png;
extern crate squish;
#[macro_use]
extern crate structopt;

use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;

use ddsfile::{AlphaMode, Dds, D3D10ResourceDimension, DxgiFormat};
use squish::{Format, Params};
use structopt::StructOpt;

mod image;

#[derive(StructOpt)]
#[structopt(name = "squish", about = "A BC1/2/3 compressor and decompressor")]
enum Opt {
    /// Compress a PNG or JPEG file to DDS
    #[structopt(name = "compress")]
    Compress {
        /// Output file (DDS)
        #[structopt(short = "o", long = "output", parse(from_os_str))]
        outfile: Option<PathBuf>,
    
        /// Input file (PNG, JPG)
        #[structopt(name = "INFILE", parse(from_os_str))]
        infile: PathBuf,
    
        /// Compression format (BC1, BC2 or BC3)
        #[structopt(short = "f", long = "format")]
        format: Format,
    }
}

fn main() {
    let (outfile, infile, format) = match Opt::from_args() {
        Opt::Compress{outfile, infile, format} => (outfile, infile, format),
    };

    let outfile = outfile.unwrap_or(
        PathBuf::new()
            .with_file_name(
                infile.file_name()
                    .unwrap_or(OsStr::new("output")))
            .with_extension("dds")
    );
    let in_ext = infile.extension()
                    .expect("Input filename has no extension, can't guess type")
                    .to_string_lossy()
                    .to_owned()
                    .to_lowercase();
    let image = match in_ext.as_str() {
        "jpg" | "jpeg" => image::jpeg::read(infile),
        "png" => image::png::read(infile),
        _ => panic!("Unrecognized image format. Supported formats are PNG and JPEG"),
    };

    let mut buf = vec![
        0u8; format.compressed_size(image.width, image.height)
    ];
    format.compress(
        &image.data[..],
        image.width,
        image.height,
        Params::default(),
        &mut buf
    );

    let alphamode = if format == Format::Bc1 {
        AlphaMode::PreMultiplied
    } else {
        AlphaMode::Straight
    };
    let mut dds = Dds::new_dxgi(
        image.height as u32, 
        image.width as u32, 
        None, // depth
        format_to_dxgiformat(format), 
        None, // mipmap_levels
        None, // array_layers 
        None, // caps3
        false, // is_cubemap 
        D3D10ResourceDimension::Texture2D, 
        alphamode
    ).unwrap();
    dds.data = buf;

    let mut outfile = File::create(outfile).expect("Failed to create output file");
    dds.write(&mut outfile).unwrap();
}

fn format_to_dxgiformat(f: Format) -> DxgiFormat {
    match f {
        Format::Dxt1 => DxgiFormat::BC1_UNorm_sRGB,
        Format::Dxt3 => DxgiFormat::BC2_UNorm_sRGB,
        Format::Dxt5 => DxgiFormat::BC3_UNorm_sRGB,
    }
}
