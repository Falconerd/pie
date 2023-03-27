//! This library provides the following functions:
//! read    - Read a PIE file from disk and decode it.
//! write   - Encode and write a PIE file to disk.
//! decode  - Decode the raw bytes of a PIE image from memory.
//! encode  - Encode an RGBA buffer into a PIE image in memory.
use std::{fs::{File, self}, io::Read, collections::HashMap};

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
    pub format: PixelFormat,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Palette {
    pub format: PixelFormat,
    pub colors: Vec<u8>, // Stride will be 4 for RGBA, 3 for RGB.
}

pub fn write(path: &str, width: u16, height: u16, embed_palette: bool, palette: Option<&Palette>, pixels: Vec<u8>) -> Result<bool, EncodeError> {
    let encoded = encode(width, height, &pixels, embed_palette, palette).expect("Failed to encode data.");
    let mut flags = 0;

    if encoded.indices.len() / 2 > u16::MAX as usize {
        return Err(EncodeError::SWR);
    }

    let mut bytes: Vec<u8> = vec!['P' as u8, 'I' as u8, 'E' as u8, 1];
    bytes.append(&mut width.to_be_bytes().to_vec());
    bytes.append(&mut height.to_be_bytes().to_vec());
    bytes.push(0); // Fill with flags later
    bytes.append(&mut ((encoded.indices.len() / 2) as u16).to_be_bytes().to_vec());
    bytes.append(&mut encoded.indices.to_vec());

    if embed_palette {
        flags |= FLAG_PALETTE;
        bytes.append(&mut encoded.palette.unwrap().colors.to_vec());
    }

    bytes[8] = flags;

    fs::write(path, &bytes).expect("Failed to write file.");
    Ok(true)
}

/// Encode an array of RGB or RGBA bytes into an EncodedPIE.
/// Note that an EncodedPIE struct is not the same format as a saved .PIE file.
/// To get the correct format for saving, use the write function.
pub fn encode(width: u16, height: u16, pixel_bytes: &[u8], embed_palette: bool, palette: Option<&Palette>) -> Result<EncodedPIE, EncodeError> {
    let mut encoded = EncodedPIE {
        width, height,
        indices: Vec::new(),
        format: PixelFormat::RGBA,
        palette: None
    };


    let mut chunk_size = 4;
    if pixel_bytes.len() == (width as usize * height as usize * 3) {
        encoded.format = PixelFormat::RGB;
        chunk_size = 3;
    };

    // If palette is not included, it must be created on the fly.
    if palette.is_none() {
        let mut indices = Vec::new();
        let mut palette = Palette {
            format: encoded.format,
            colors: Vec::new()
        };
        let mut map = HashMap::new();
        let mut index: u8 = 0;
        for chunk in pixel_bytes.chunks(chunk_size) {
            if !map.contains_key(chunk) {
                map.insert(chunk, index);
                index += 1;
                palette.colors.append(&mut chunk.to_vec());
            }

            indices.push(*map.get(chunk).unwrap() as u8);
        }

        if embed_palette {
            encoded.palette = Some(palette);
        }
        encoded.indices = rle(&indices);
    } else if let Some(palette) = palette {
        let mut indices = Vec::new();
        let map = palette.colors.chunks(chunk_size).into_iter().enumerate().fold(HashMap::new(), |mut acc, (idx, x)| {
            acc.insert(x, idx);
            acc
        });
        for chunk in pixel_bytes.chunks(chunk_size) {
            if !map.contains_key(chunk) {
                return Err(EncodeError::SWR);
            }

            indices.push(*map.get(chunk).unwrap() as u8);

            if embed_palette {
                encoded.palette = Some(palette.to_owned());
            }
            encoded.indices = rle(&indices);
        }
    }

    Ok(encoded)
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
        format: PixelFormat::RGB, pixels: vec![]
    };

    let mut palette = Palette {
        format: PixelFormat::RGB,
        colors: Vec::new(),
    };

    assert!(bytes[0] == 'P' as u8 && bytes[1] == 'I' as u8 && bytes[2] == 'E' as u8);
    decoded.width = u16::from_be_bytes([bytes[4], bytes[5]]);
    decoded.height = u16::from_be_bytes([bytes[6], bytes[7]]);
    let flags = bytes[8];

    palette.format = PixelFormat::RGB;
    let mut step = 3;

    if flags & FLAG_TRANSPARENCY > 0 {
        palette.format = PixelFormat::RGBA;
        step = 4;
    }

    let data_length = u16::from_be_bytes([bytes[9], bytes[10]]);

    if flags & FLAG_PALETTE > 0 {
        for (index, _) in bytes.iter().skip(HEADER_SIZE + (data_length * 2) as usize).enumerate().step_by(step) {
            let absolute_index = HEADER_SIZE + (data_length * 2) as usize + index - 1;
            for i in 0..step {
                palette.colors.push(bytes[absolute_index + step - i]);
            }
        }
    } else if let Some(p) = maybe_palette {
        palette.format = p.format;
        palette.colors = p.colors.to_owned();
    } else {
        return Err(DecodeError::SWR);
    }

    for i in (HEADER_SIZE..(HEADER_SIZE + (data_length * 2) as usize)).step_by(2) {
        let run_length = bytes[i];
        let color_index = bytes[i + 1] as usize * step;

        for _ in 0..run_length {
            decoded.pixels.append(&mut vec![palette.colors[color_index + 2], palette.colors[color_index + 1], palette.colors[color_index]]);
        }
    }

    decoded.format = palette.format;

    Ok(decoded)
}

#[test]
fn test_decode() {
    let bytes = include_bytes!("../images/test_compressed_with_palette.sii");
    let decoded = decode(bytes, None).unwrap();
    let palette_bytes: [u8; 12] = [
        0x6A, 0xBE, 0x30,
        0xFF, 0xFF, 0xFF,
        0x00, 0x00, 0x00,
        0x5B, 0x6E, 0xE1,
    ];
    let start_pixel: [u8; 3] = [0x6A, 0xBE, 0x30];
    let end_pixel: [u8; 3] = [0x5B, 0x6E, 0xE1];
    let decoded_with_palette = decode(bytes, Some(&Palette {
        format: PixelFormat::RGB,
        colors: palette_bytes.to_vec(),
    })).unwrap();

    assert_eq!(start_pixel, decoded.pixels[0..3]);
    assert_eq!(end_pixel, decoded.pixels[decoded.pixels.len() - 3..]);
    assert_eq!(decoded.pixels, decoded_with_palette.pixels);
}

#[test]
fn test_encode() {
    let pixels: Vec<u8> = vec![
        0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 
        0xFF, 0x00, 0xCC, 0xFF, 0x00, 0xCC, 0xFF, 0x00, 0xCC, 0xFF, 0x00, 0xCC, 0xFF, 0x00, 0xCC, 
        0xBE, 0xEF, 0x00, 0xBE, 0xEF, 0x00, 0xBE, 0xEF, 0x00, 0xBE, 0xEF, 0x00, 0xFF, 0xFF, 0xFF, 
    ];

    let palette = Palette {
        format: PixelFormat::RGB,
        colors: vec![
            0xFF, 0xFF, 0xFF,
            0xFF, 0x00, 0x00,
            0xBE, 0xEF, 0x00,
            0xFF, 0x00, 0xCC,
        ],
    };

    let encoded = encode(5, 4, &pixels, true, Some(&palette)).unwrap();
    assert_eq!([5, 1] as [u8; 2], encoded.indices[0..2]);
    assert_eq!([5, 0] as [u8; 2], encoded.indices[2..4]);
    assert_eq!([5, 3] as [u8; 2], encoded.indices[4..6]);
    assert_eq!([4, 2] as [u8; 2], encoded.indices[6..8]);
    assert_eq!([1, 0] as [u8; 2], encoded.indices[8..10]);
    assert_eq!(palette.colors, encoded.palette.unwrap().colors);

    let encoded = encode(5, 4, &pixels, false, Some(&palette)).unwrap();
    assert_eq!([5, 1] as [u8; 2], encoded.indices[0..2]);
    assert_eq!([5, 0] as [u8; 2], encoded.indices[2..4]);
    assert_eq!([5, 3] as [u8; 2], encoded.indices[4..6]);
    assert_eq!([4, 2] as [u8; 2], encoded.indices[6..8]);
    assert_eq!([1, 0] as [u8; 2], encoded.indices[8..10]);
    assert!(encoded.palette.is_none());

    let encoded = encode(5, 4, &pixels, true, None).unwrap();
    assert_eq!([5, 0] as [u8; 2], encoded.indices[0..2]);
    assert_eq!([5, 1] as [u8; 2], encoded.indices[2..4]);
    assert_eq!([5, 2] as [u8; 2], encoded.indices[4..6]);
    assert_eq!([4, 3] as [u8; 2], encoded.indices[6..8]);
    assert_eq!([1, 1] as [u8; 2], encoded.indices[8..10]);
    assert_eq!([0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xCC, 0xBE, 0xEF, 0x00] as [u8; 12], encoded.palette.unwrap().colors.as_slice());

    let encoded = encode(5, 4, &pixels, false, None).unwrap();
    assert_eq!([5, 0] as [u8; 2], encoded.indices[0..2]);
    assert_eq!([5, 1] as [u8; 2], encoded.indices[2..4]);
    assert_eq!([5, 2] as [u8; 2], encoded.indices[4..6]);
    assert_eq!([4, 3] as [u8; 2], encoded.indices[6..8]);
    assert_eq!([1, 1] as [u8; 2], encoded.indices[8..10]);
    assert!(encoded.palette.is_none());
}

#[test]
fn test_read() {
    let decoded = read("images/test_compressed_with_palette.sii", None).unwrap();
    let palette_bytes: [u8; 12] = [
        0x6A, 0xBE, 0x30,
        0xFF, 0xFF, 0xFF,
        0x00, 0x00, 0x00,
        0x5B, 0x6E, 0xE1,
    ];
    let decoded_with_palette = read("images/test_compressed_with_palette.sii", Some(&Palette {
        format: PixelFormat::RGB,
        colors: palette_bytes.to_vec(),
    })).unwrap();

    let start_pixel: [u8; 3] = [0x6A, 0xBE, 0x30];
    let end_pixel: [u8; 3] = [0x5B, 0x6E, 0xE1];

    assert_eq!(start_pixel, decoded.pixels[0..3]);
    assert_eq!(end_pixel, decoded.pixels[decoded.pixels.len() - 3..]);
    assert_eq!(decoded.pixels, decoded_with_palette.pixels);
}

#[test]
fn test_write() {
    let pixels: Vec<u8> = vec![
        0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00, 0xFF, 0x00, 0x00,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 
        0xFF, 0x00, 0xCC, 0xFF, 0x00, 0xCC, 0xFF, 0x00, 0xCC, 0xFF, 0x00, 0xCC, 0xFF, 0x00, 0xCC, 
        0xBE, 0xEF, 0x00, 0xBE, 0xEF, 0x00, 0xBE, 0xEF, 0x00, 0xBE, 0xEF, 0x00, 0xFF, 0xFF, 0xFF, 
    ];

    let palette = Palette {
        format: PixelFormat::RGB,
        colors: vec![
            0xFF, 0xFF, 0xFF,
            0xFF, 0x00, 0x00,
            0xBE, 0xEF, 0x00,
            0xFF, 0x00, 0xCC,
        ],
    };

    assert!(write("tmp.pie", 5, 4, true, Some(&palette), pixels.to_owned()).is_ok());

    let decoded = read("tmp.pie", Some(&palette)).expect("Could not read");
    assert_eq!(pixels, decoded.pixels);
    assert!(fs::remove_file("tmp.pie").is_ok());
}
