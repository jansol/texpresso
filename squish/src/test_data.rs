//! This module provides static data for unit test.
//! Most test data is available as public constants with a naming scheme of `<BCn>_<pattern-name>`.
//! Currently two patterns are used: `GRAY` and `COLOUR`.

/// A data set for testing holds the encoded and decoded values for a single 4x4 block of pixels.
#[derive(Debug)]
pub struct TestDataSet {
    pub encoded: &'static [u8],
    pub decoded: &'static [u8],
}

/// The test-pattern is a gray-scale checkerboard of size 4x4 starting with 0xFF in the top-left.
/// On top of that, the four middle pixels are set to 0x7F.
/// BC1 data created with AMD Compressonator v4.1.5083.
pub const BC1_GRAY: TestDataSet = TestDataSet {
    encoded: &[0x00, 0x00, 0xFF, 0xFF, 0x11, 0x68, 0x29, 0x44],
    decoded: &add_alpha_to_rgb(
        &expand_single_to_rgb(&[
            0xFF, 0x00, 0xFF, 0x00, // row 0
            0x00, 0x7F, 0x7F, 0xFF, // row 1
            0xFF, 0x7F, 0x7F, 0x00, // row 2
            0x00, 0xFF, 0x00, 0xFF, // row 3
        ]),
        &[0xFF; 16],
    ),
};

/// Provides a colour test pattern without alpha information, i.e. in RGB format.
const COLOUR_BLOCK_RGB: [u8; 4 * 4 * 3] = [
    0xFF, 0x96, 0x4A, 0xFF, 0x96, 0x4A, // row 0, left half
    0xFF, 0x96, 0x4A, 0xFF, 0x96, 0x4A, // row 0, right half
    0xFF, 0x78, 0x34, 0xFF, 0x78, 0x34, // row 1, left half
    0xFF, 0x78, 0x34, 0xFF, 0x78, 0x34, // row 1, right half
    0xFF, 0x69, 0x29, 0xFF, 0x69, 0x29, // row 2, left half
    0xFF, 0x69, 0x29, 0xFF, 0x69, 0x29, // row 2, right half
    0xFF, 0x69, 0x29, 0xFF, 0x69, 0x29, // row 3, left half
    0xFF, 0x69, 0x29, 0xFF, 0x69, 0x29, // row 3, right half
];

/// Provides a gray test pattern a gray-scale checkerboard of size 4x4 starting with 0xFF in the top-left.
/// On top of that, the four middle pixels are set to 0x55.
const GRAY_BLOCK_LUMA: [u8; 4 * 4] = [
    0xFF, 0x00, 0xFF, 0x00, // row 0
    0x00, 0x55, 0x55, 0xFF, // row 1
    0xFF, 0x55, 0x55, 0x00, // row 2
    0x00, 0xFF, 0x00, 0xFF, // row 3
];

/// A colour test-pattern (RGB) with the first row in one colour,
/// the second in another and the third and last row in a third colour.
/// The alpha value is 0xFF for all pixels.
/// BC1 data created with AMD Compressonator v4.1.5083 and is the same as libsquish.
pub const BC1_COLOUR: TestDataSet = TestDataSet {
    encoded: &[0xA9, 0xFC, 0x45, 0xFB, 0x00, 0xFF, 0x55, 0x55],
    decoded: &add_alpha_to_rgb(&COLOUR_BLOCK_RGB, &[0xFF; 16]),
};

/// A slightly different gray pattern to BC1_GRAY with a changed middle gray value.
/// The alpha value is starting with 0x00 and increases by 0x11 steps.
pub const BC2_GRAY: TestDataSet = TestDataSet {
    encoded: &[
        0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE, // Alpha
        0xFF, 0xFF, 0x00, 0x00, 0x44, 0x3D, 0x7C, 0x11, // Colour
    ],
    decoded: &add_alpha_to_rgb(&expand_single_to_rgb(&GRAY_BLOCK_LUMA), &LINEAR_RAMP),
};

/// The same test pattern as BC1_COLOUR, but with different alpha values.
/// The alpha value is starting with 0x00 and increases by 0x11 steps.
pub const BC2_COLOUR: TestDataSet = TestDataSet {
    encoded: &[
        0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE, // Alpha
        0xA9, 0xFC, 0x45, 0xFB, 0x00, 0xFF, 0x55, 0x55, // Colour
    ],
    decoded: &add_alpha_to_rgb(&COLOUR_BLOCK_RGB, &LINEAR_RAMP),
};

/// Internal data set for storing 16 alpha values from 8 bytes of BC3 alpha encoding.
const BC3_ALPHA_DECODED: [u8; 4 * 4] = [
    0x00, 0x24, 0x48, 0x6D, // row 0
    0x91, 0xB6, 0xDB, 0xFF, // row 1
    0x00, 0x24, 0x48, 0x6D, // row 2
    0x91, 0xB6, 0xDB, 0xFF, // row 3
];

/// The same test pattern as BC2_GRAY, but with with 255/7 steps in the alpha values.
/// Note that the alpha values are rounded down.
pub const BC3_GRAY: TestDataSet = TestDataSet {
    encoded: &[
        0x24, 0xDB, 0x86, 0xC6, 0xE6, 0x86, 0xC6, 0xE6, // Alpha
        0xFF, 0xFF, 0x00, 0x00, 0x44, 0x3D, 0x7C, 0x11, // Colour
    ],
    decoded: &add_alpha_to_rgb(&expand_single_to_rgb(&GRAY_BLOCK_LUMA), &BC3_ALPHA_DECODED),
};

/// The same test pattern as BC2_COLOUR, but with 255/7 steps in the alpha values.
/// Note that the alpha values are rounded down.
pub const BC3_COLOUR: TestDataSet = TestDataSet {
    encoded: &[
        0x24, 0xDB, 0x86, 0xC6, 0xE6, 0x86, 0xC6, 0xE6, // Alpha
        0xA9, 0xFC, 0x45, 0xFB, 0x00, 0xFF, 0x55, 0x55, // Colour
    ],
    decoded: &add_alpha_to_rgb(&COLOUR_BLOCK_RGB, &BC3_ALPHA_DECODED),
};

/// The BC1_GRAY test pattern, but as BC4 with BC3 alpha encoding.
pub const BC4_GRAY: TestDataSet = TestDataSet {
    encoded: &[0x7F, 0x84, 0xF7, 0x6D, 0xE0, 0x07, 0xEC, 0xFB],
    decoded: &add_alpha_to_rgb(
        &expand_single_to_rgb(&[
            0xFF, 0x00, 0xFF, 0x00, // row 0
            0x00, 0x7F, 0x7F, 0xFF, // row 1
            0xFF, 0x7F, 0x7F, 0x00, // row 2
            0x00, 0xFF, 0x00, 0xFF, // row 3
        ]),
        &[0xFF; 16],
    ),
};

/// The BC1_GRAY test pattern for the red channel and the inverse pattern for the green channel.
pub const BC5_GRAY: TestDataSet = TestDataSet {
    encoded: &[
        0x7F, 0x84, 0xF7, 0x6D, 0xE0, 0x07, 0xEC, 0xFB, // Alpha 0
        0x7F, 0x84, 0xBE, 0x7F, 0xC0, 0x06, 0x7E, 0xDF, // Alpha 1
    ],
    decoded: &add_alpha_to_rgb(
        &[
            0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00, // row 0
            0x00, 0xFF, 0x00, 0x7F, 0x7F, 0x00, 0x7F, 0x7F, 0x00, 0xFF, 0x00, 0x00, // row 1
            0xFF, 0x00, 0x00, 0x7F, 0x7F, 0x00, 0x7F, 0x7F, 0x00, 0x00, 0xFF, 0x00, // row 2
            0x00, 0xFF, 0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0x00, // row 3
        ],
        &[0xFF; 16],
    ),
};

/// Expands an array with a single value per pixel to an array with this value expanded
/// into the RGB channels.
const fn expand_single_to_rgb(input: &[u8; 4 * 4]) -> [u8; 4 * 4 * 3] {
    let mut output = [0u8; 4 * 4 * 3];
    let mut i = 0;
    // for loops are not available in const functions at the time of writing
    while i < input.len() {
        output[i * 3 + 0] = input[i]; // R
        output[i * 3 + 1] = input[i]; // G
        output[i * 3 + 2] = input[i]; // B
        i += 1;
    }
    output
}

/// Provides a linear value ramp with 16 entries, useful as alpha values.
/// Starts at 0 and increases in 0x11 steps until 0xFF.
const LINEAR_RAMP: [u8; 4 * 4] = [
    0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF,
];

/// Appends a list of 16 alpha values to the RGB values.
/// I.e. each RGB is extended to RGBA with the alpha value from the list.
const fn add_alpha_to_rgb(input: &[u8; 4 * 4 * 3], alpha_values: &[u8; 4 * 4]) -> [u8; 4 * 4 * 4] {
    let mut output = [0u8; 4 * 4 * 4];
    let mut i = 0;
    // for loops are not available in const functions at the time of writing
    while i < 4 * 4 {
        output[i * 4 + 0] = input[i * 3 + 0]; // R
        output[i * 4 + 1] = input[i * 3 + 1]; // G
        output[i * 4 + 2] = input[i * 3 + 2]; // B
        output[i * 4 + 3] = alpha_values[i]; //A
        i += 1;
    }
    output
}
