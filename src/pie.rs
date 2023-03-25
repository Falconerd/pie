//! This library provides the following functions:
//! read    - Read a PIE file from disk and decode it.
//! write   - Encode and write a PIE file to disk.
//! decode  - Decode the raw bytes of a PIE image from memory.
//! encode  - Encode an RGBA buffer into a PIE image in memory.

use std::{fs::File, io::Read, process::exit, collections::HashMap};

const FLAG_PALETTE: u8      = 1 << 0;
const FLAG_TRANSPARENCY: u8 = 1 << 1;
const HEADER_SIZE: usize = 11;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PixelFormat {
    RGB, RGBA,
}

#[derive(Debug, PartialEq)]
pub struct DecodedPIE {
    pub width: u16,
    pub height: u16,
    pub format: PixelFormat,
    pub pixels: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct EncodedPIE {
    pub width: u16,
    pub height: u16,
    pub indices: Vec<u8>,
    pub palette: Option<Palette>,
}

// TODO: Use proper errors.
#[derive(Debug, PartialEq)]
pub enum DecodeError {
    SWR
}

#[derive(Debug, PartialEq)]
pub enum EncodeError {
    SWR
}

#[derive(Debug, PartialEq)]
pub struct Palette {
    format: PixelFormat,
    colors: Vec<u8>, // Stride will be 4 for RGBA, 3 for RGB.
}

pub fn write(path: &str, width: u16, height: u16, format: PixelFormat, palette: Option<&Palette>, pixels: Vec<u8>) -> Result<bool, EncodeError> {
    Ok(true)
}

/// Encode an array of RGB or RGBA bytes into an EncodedPIE.
/// Note that an EncodedPIE struct is not the same format as a saved .PIE file.
/// To get the correct format for saving, use the write function.
pub fn encode(width: u16, height: u16, pixel_bytes: &[u8], palette: Option<&Palette>) -> Result<EncodedPIE, EncodeError> {
    let mut encoded = EncodedPIE {
        width, height,
        indices: Vec::new(),
        palette: None
    };

    // If palette is not included, it must be created on the fly.
    if palette.is_none() {
    }

    // 1. Chunk the Vec<u8> into RGB (3) or RGBA (4) bytes.
    // 2. 
}

fn rle(data: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let mut count = 1;
        while i + count < data.len() && data[i] == data[i + count] && count < 255 {
            count += 1;
        }
        encoded.push(count as u8);
        encoded.push(data[i]);
        i += count;
    }
    encoded
}

fn palette_from_pixels(width: u16, height: u16, pixel_bytes: &[u8]) -> Palette {
    let mut palette = Palette {
        format: if pixel_bytes.len() > (width * height * 3).into() { PixelFormat::RGBA } else { PixelFormat::RGB },
        colors: Vec::new()
    };

    match palette.format {
        PixelFormat::RGB => {
            let mut colors = HashMap::new();
            for (index, _) in pixel_bytes.iter().enumerate().step_by(3) {
                let color = pixel_bytes[index] << 16 | pixel_bytes[index + 1] << 8 | pixel_bytes[index + 2];
                if !colors.contains_key(&color) {
                    colors.insert(color, ());
                    palette.colors.push(pixel_bytes[index]);
                    palette.colors.push(pixel_bytes[index + 1]);
                    palette.colors.push(pixel_bytes[index + 2]);
                }
            }
        },
        PixelFormat::RGBA => {
            let mut colors = HashMap::new();
            for (index, _) in pixel_bytes.iter().enumerate().step_by(4) {
                let color = pixel_bytes[index] << 24 | pixel_bytes[index + 1] << 16 | pixel_bytes[index + 2] << 8 | pixel_bytes[index + 3];
                if !colors.contains_key(&color) {
                    colors.insert(color, ());
                    palette.colors.push(pixel_bytes[index]);
                    palette.colors.push(pixel_bytes[index + 1]);
                    palette.colors.push(pixel_bytes[index + 2]);
                    palette.colors.push(pixel_bytes[index + 3]);
                }
            }
        },
    }

    palette
}

/// Read a PIE file from disk and decode it into a DecodedPIE.
/// Palette is required if not included in the image.
/// # Arguments
/// * `path` - A string slice that is a path to the file on disk.
/// * `palette` - An optional palette that must be included if the PIE file was saved with an
/// external palette.
pub fn read(path: &str, palette: Option<&Palette>) -> Result<DecodedPIE, DecodeError> {
    let mut file = File::open(path).expect("Could not open file");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Could not read file");

    decode(&bytes, palette)
}

/// Decode raw bytes from PIE format into a DecodedPIE.
/// Palette is required if not included in the data.
/// * `bytes` - The raw bytes including header, index data, and optionally palette.
/// * `palette` - An optional palette that must be included if the PIE file was saved with an
/// external palette.
pub fn decode(bytes: &[u8], maybe_palette: Option<&Palette>) -> Result<DecodedPIE, DecodeError> {
    let mut decoded = DecodedPIE {
        width: 0, height: 0,
        format: PixelFormat::RGBA, pixels: vec![]
    };

    // Get palette.
    let mut palette = Palette {
        format: PixelFormat::RGB,
        colors: Vec::new(),
    };

    assert!(bytes[0] == 'P' as u8 && bytes[1] == 'I' as u8 && bytes[2] == 'E' as u8);
    decoded.width = u16::from_be_bytes([bytes[4], bytes[5]]);
    decoded.height = u16::from_be_bytes([bytes[6], bytes[7]]);
    let flags = bytes[8];

    let has_embedded_palette = flags & FLAG_PALETTE > 0;
    let has_transparency = flags & FLAG_TRANSPARENCY > 0;
    palette.format = if has_transparency {
        PixelFormat::RGBA
    } else {
        PixelFormat::RGB
    };

    let data_length = u16::from_be_bytes([bytes[9], bytes[10]]);

    if has_embedded_palette {
        let step = if has_transparency { 4 } else { 3 };
        for (index, _) in bytes.iter().skip(HEADER_SIZE + (data_length * 2) as usize).enumerate().step_by(step) {
            let absolute_index = HEADER_SIZE + (data_length * 2) as usize + index;
            println!("index: {}, absolute_index: {}", index, absolute_index);
            for i in 0..step {
                palette.colors.push(bytes[absolute_index + step - i]);
            }
        }
        palette.colors.iter().for_each(|x| println!("{:06X}", x));
    } else if let Some(p) = maybe_palette {
        palette.format = p.format;
        palette.colors = p.colors.to_owned();
    } else {
        return Err(DecodeError::SWR);
    }

    for i in (HEADER_SIZE..(HEADER_SIZE + (data_length * 2) as usize)).step_by(2) {
        let run_length = bytes[i];
        let color_index = bytes[i + 1] as usize;
        let color = palette.colors.get(color_index).expect("Coud not get color");

        for _ in 0..run_length {
            decoded.pixels.push(*color);
        }
    }

    Ok(decoded)
}
