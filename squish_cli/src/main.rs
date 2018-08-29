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

use ddsfile::{AlphaMode, Dds, D3D10ResourceDimension, D3DFormat, DxgiFormat};
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
    },

    /// Deompress a DDS file to PNG
    #[structopt(name = "decompress")]
    Decompress {
        /// Output file (PNG)
        #[structopt(short = "o", long = "output", parse(from_os_str))]
        outfile: Option<PathBuf>,

        /// Input file (DDS)
        #[structopt(name = "INFILE", parse(from_os_str))]
        infile: PathBuf,
    }
}

fn main() {
    match Opt::from_args() {
        Opt::Compress{outfile, infile, format} => compress_file(outfile, infile, format),
        Opt::Decompress{outfile, infile} => decompress_file(outfile, infile),
    };
}

fn format_to_dxgiformat(f: Format) -> DxgiFormat {
    match f {
        Format::Bc1 => DxgiFormat::BC1_UNorm_sRGB,
        Format::Bc2 => DxgiFormat::BC2_UNorm_sRGB,
        Format::Bc3 => DxgiFormat::BC3_UNorm_sRGB,
    }
}

fn dxgiformat_to_format(d: DxgiFormat) -> Format {
    match d {
        DxgiFormat::BC1_UNorm_sRGB => Format::Bc1,
        DxgiFormat::BC2_UNorm_sRGB => Format::Bc2,
        DxgiFormat::BC3_UNorm_sRGB => Format::Bc3,
        _ => panic!("Unsupported DXGI format!"),
    }
}

fn d3dformat_to_format(d: D3DFormat) -> Format {
    match d {
        D3DFormat::DXT1 => Format::Bc1,
        D3DFormat::DXT3 => Format::Bc2,
        D3DFormat::DXT5 => Format::Bc3,
        _ => panic!("Unsupported D3D format!"),
    }
}

fn compress_file(outfile: Option<PathBuf>, infile: PathBuf, format: Format) {
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
        "jpg" | "jpeg" => image::jpeg::read(&infile),
        "png" => image::png::read(&infile),
        _ => panic!("Unrecognized image format. Supported formats are PNG and JPEG"),
    };

    let mut buf = vec![
        0u8; format.compressed_size(image.width, image.height)
    ];
    format.compress(
        &image.data,
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
        None, // caps2
        false, // is_cubemap 
        D3D10ResourceDimension::Texture2D, 
        alphamode
    ).unwrap();
    dds.data = buf;

    let mut outfile = File::create(outfile).expect("Failed to create output file");
    dds.write(&mut outfile).unwrap();
}

fn decompress_file(outfile: Option<PathBuf>, infile: PathBuf) {
    let outfile = outfile.unwrap_or(
        PathBuf::new()
            .with_file_name(
                infile.file_name()
                    .unwrap_or(OsStr::new("output")))
            .with_extension("png")
    );

    let mut infile = File::open(&infile).expect("Failed to open file");
    let dds = Dds::read(&mut infile).unwrap();

    let d3dformat = D3DFormat::try_from_pixel_format(&dds.header.spf);
    let format;
    if let Some(header10) = dds.header10 {
        if header10.resource_dimension != D3D10ResourceDimension::Texture2D {
            panic!("Only images with resource dimension Texture2D are supported");
        }

        format = dxgiformat_to_format(header10.dxgi_format)
    } else {
        format = d3dformat_to_format(d3dformat.unwrap());
    }

    let width = dds.header.width as usize;
    let height = dds.header.height as usize;
    let mut decompressed = vec![0u8; 4*width*height];

    format.decompress(
        &dds.data,
        width,
        height,
        &mut decompressed
    );

    image::png::write(&outfile, width as u32, height as u32, &decompressed);
}