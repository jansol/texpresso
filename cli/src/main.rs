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

use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use ddsfile::{AlphaMode, D3D10ResourceDimension, D3DFormat, Dds, DxgiFormat};
use texpresso::{Algorithm, Format, Params, COLOUR_WEIGHTS_PERCEPTUAL};

mod image;

#[derive(Clone, ValueEnum)]
enum Profile {
    Speed,
    Balanced,
    Quality,
}

#[derive(Clone, ValueEnum)]
enum CliFormat {
    Bc1,
    Bc2,
    Bc3,
    Bc4,
    Bc5,
}

#[derive(Parser)]
#[command(version, about)]
enum Opt {
    /// Compress a PNG or JPEG file to DDS
    #[command(name = "compress")]
    Compress {
        /// Output file (DDS)
        #[arg(short = 'o', long = "output")]
        outfile: Option<PathBuf>,

        /// Input file (PNG, JPG)
        #[arg(name = "INFILE")]
        infile: PathBuf,

        /// Compression format
        #[arg(short = 'f', long = "format")]
        format: CliFormat,

        /// Compressor profile (speed, balanced, quality).
        #[arg(short = 'p', long = "profile", default_value = "balanced")]
        profile: Profile,

        /// Weigh colours by alpha while fitting. Can improve perceived quality in alpha-blended images.
        #[arg(long = "weigh-colour-by-alpha")]
        weigh_colour_by_alpha: bool,

        // TODO: replace with something nicer
        /// Colour weights to be used for matching colours during fitting.
        #[arg(short = 'w', long = "weights")]
        weights: Vec<f32>,
    },

    /// Deompress a DDS file to PNG
    #[command(name = "decompress")]
    Decompress {
        /// Output file (PNG)
        #[clap(short = 'o', long = "output")]
        outfile: Option<PathBuf>,

        /// Input file (DDS)
        #[clap(name = "INFILE")]
        infile: PathBuf,
    },
}

fn main() {
    match Opt::parse() {
        Opt::Compress {
            outfile,
            infile,
            format,
            profile,
            weigh_colour_by_alpha,
            weights,
        } => {
            let w;
            if weights.is_empty() {
                w = COLOUR_WEIGHTS_PERCEPTUAL;
            } else if weights.len() == 3 {
                w = [weights[0], weights[1], weights[2]];
            } else {
                panic!("Weights must have 3 values");
            }
            let params = Params {
                algorithm: profile.into(),
                weights: w,
                weigh_colour_by_alpha,
            };
            compress_file(outfile, &infile, format.into(), params)
        }
        Opt::Decompress { outfile, infile } => decompress_file(outfile, &infile),
    };
}

fn compress_file(outfile: Option<PathBuf>, infile: &Path, format: Format, params: Params) {
    let outfile = outfile.unwrap_or_else(|| {
        PathBuf::new()
            .with_file_name(infile.file_name().unwrap_or_else(|| OsStr::new("output")))
            .with_extension("dds")
    });
    let in_ext = infile
        .extension()
        .expect("Input filename has no extension, can't guess type")
        .to_string_lossy()
        .to_owned()
        .to_lowercase();
    let image = match in_ext.as_str() {
        "jpg" | "jpeg" => image::jpeg::read(infile),
        "png" => image::png::read(infile),
        _ => panic!("Unrecognized image format. Supported formats are PNG and JPEG"),
    };

    let mut buf = vec![0u8; format.compressed_size(image.width, image.height)];
    format.compress(&image.data, image.width, image.height, params, &mut buf);

    let alphamode = if format == Format::Bc1 {
        AlphaMode::PreMultiplied
    } else {
        AlphaMode::Straight
    };
    let mut dds = Dds::new_dxgi(ddsfile::NewDxgiParams {
        height: image.height as u32,
        width: image.width as u32,
        depth: None,
        format: format_to_dxgiformat(format),
        mipmap_levels: None,
        array_layers: None,
        caps2: None,
        is_cubemap: false,
        resource_dimension: D3D10ResourceDimension::Texture2D,
        alpha_mode: alphamode,
    })
    .unwrap();
    dds.data = buf;

    let mut outfile = File::create(outfile).expect("Failed to create output file");
    dds.write(&mut outfile).unwrap();
}

fn decompress_file(outfile: Option<PathBuf>, infile: &Path) {
    let outfile = outfile.unwrap_or_else(|| {
        PathBuf::new()
            .with_file_name(infile.file_name().unwrap_or_else(|| OsStr::new("output")))
            .with_extension("png")
    });

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
    let mut decompressed = vec![0u8; 4 * width * height];

    format.decompress(&dds.data, width, height, &mut decompressed);

    image::png::write(&outfile, width as u32, height as u32, &decompressed);
}

impl Into<Algorithm> for Profile {
    fn into(self) -> Algorithm {
        match self {
            Profile::Speed => Algorithm::RangeFit,
            Profile::Balanced => Algorithm::ClusterFit,
            Profile::Quality => Algorithm::IterativeClusterFit,
        }
    }
}

impl Into<Format> for CliFormat {
    fn into(self) -> Format {
        match self {
            CliFormat::Bc1 => Format::Bc1,
            CliFormat::Bc2 => Format::Bc2,
            CliFormat::Bc3 => Format::Bc3,
            CliFormat::Bc4 => Format::Bc4,
            CliFormat::Bc5 => Format::Bc5,
        }
    }
}

fn format_to_dxgiformat(f: Format) -> DxgiFormat {
    match f {
        Format::Bc1 => DxgiFormat::BC1_UNorm_sRGB,
        Format::Bc2 => DxgiFormat::BC2_UNorm_sRGB,
        Format::Bc3 => DxgiFormat::BC3_UNorm_sRGB,
        Format::Bc4 => DxgiFormat::BC4_UNorm,
        Format::Bc5 => DxgiFormat::BC5_UNorm,
    }
}

fn dxgiformat_to_format(d: DxgiFormat) -> Format {
    match d {
        DxgiFormat::BC1_UNorm_sRGB => Format::Bc1,
        DxgiFormat::BC2_UNorm_sRGB => Format::Bc2,
        DxgiFormat::BC3_UNorm_sRGB => Format::Bc3,
        DxgiFormat::BC4_UNorm => Format::Bc4,
        DxgiFormat::BC5_UNorm => Format::Bc5,
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

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Opt::command().debug_assert();
}
