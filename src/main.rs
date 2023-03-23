use std::collections::HashMap;
use std::env;
use std::env::Args;
use std::fs::File;
use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

#[derive(Debug)]
struct DecodedSII {
    width: u16,
    height: u16,
    pixels: Vec<u32>
}

#[derive(Debug)]
enum DecodeError {
    ColorNotFound,
    MissingPixels
}

#[derive(Debug)]
enum EncodeError {
    InvalidDataLength
}

fn decode(palette: &Vec<u32>, data: &[u8]) -> Result<DecodedSII, DecodeError> {
    let mut pixels: Vec<u32> = vec![];

    assert!(data[0] == 0x53 && data[1] == 0x49 && data[2] == 0x49);

    let width = u16::from_be_bytes([data[4], data[5]]);
    let height = u16::from_be_bytes([data[6], data[7]]);
    let flags = data[8];

    // Compressed.
    if flags & 1 > 0 {
        for i in (9..data.len()).step_by(2) {
            let run_length = data[i];
            let color_index = data[i + 1] as usize;
            let color = palette[color_index];

            for _ in 0..run_length {
                pixels.push(color);
            }
        }
    } else {
        for value in data.iter().skip(9) {
            if (*value) as usize > palette.len() {
                return Err(DecodeError::ColorNotFound);
            }

            pixels.push(palette[(*value) as usize]);
        }

        if pixels.len() < width as usize * height as usize {
            return Err(DecodeError::MissingPixels);
        }
    }

    Ok(DecodedSII { width, height, pixels })
}

fn encode_rle(data: &[u8]) -> Vec<u8> {
    let mut encoded = vec![];
    let mut count = 1;

    for i in 1..=data.len() {
        if i < data.len() && data[i] == data[i - 1] && count < 255 {
            count += 1;
        } else {
            encoded.push(count);
            encoded.push(data[i - 1]);
            count = 1;
        }
    }

    encoded
}

fn encode(version: u8, width: u16, height: u16, compress: bool, has_alpha: bool, data: &[u8], palette: Option<Vec<u32>>) -> Result<Vec<u8>, EncodeError> {
    if data.len() % 2 != 0 {
        return Err(EncodeError::InvalidDataLength);
    }

    let w = width.to_be_bytes();
    let h = height.to_be_bytes();
    let mut flags: u8 = 0;
    if compress {
        flags |= 0b1;
    }
    if palette.is_some() {
        flags |= 0b10;
    }
    if has_alpha {
        flags |= 0b100;
    }
    let mut bytes: Vec<u8> = vec![0x53, 0x49, 0x49, version, w[0], w[1], h[0], h[1], flags];

    if compress {
        let mut compressed = encode_rle(data);
        bytes.append(&mut compressed);
    } else {
        let mut d = data.to_owned();
        bytes.append(&mut d);
    }

    if let Some(colors) = palette {
        bytes.push(0xFF);

        for color in colors {
            let color_bytes = color.to_le_bytes();
            let mut hex: Vec<u8> = vec![color_bytes[2], color_bytes[1], color_bytes[0]];
            if has_alpha {
                hex.insert(0, color_bytes[3]);
            }
            bytes.append(&mut hex);
        }
    }

    return Ok(bytes);
}

// For now assume RGBA.
fn palette_from_pixels(bytes: &[u8]) -> Vec<u32> {
    let mut exists = HashMap::<u32, ()>::new();
    let mut palette = Vec::<u32>::new();
    for (index, _) in bytes.iter().enumerate().step_by(4) {
        let r = bytes[index] as u32;
        let g = bytes[index + 1] as u32;
        let b = bytes[index + 2] as u32;
        let a = bytes[index + 3] as u32;
        let rgba = r << 24 | g << 16 | b << 8 | a;
        if exists.get(&rgba).is_none() {
            exists.insert(rgba, ());
            palette.push(rgba);
        }
    }
    return palette;
}

#[derive(PartialEq)]
enum ColorFormat {
    RGBA,
    BGRA,
    RGB,
    BGR,
}

fn encode_from_bytes(version: u8, width: u16, height: u16, bytes: &[u8], format: ColorFormat, compress: bool, embed_palette: bool) -> Result<Vec<u8>, EncodeError> {
    let mut exists: HashMap<u32, u8> = HashMap::new();
    let mut palette: Vec<u32> = vec![];

    let has_alpha = format == ColorFormat::RGBA || format == ColorFormat::BGRA;

    let w = width.to_be_bytes();
    let h = height.to_be_bytes();
    let mut flags: u8 = 0;
    if compress {
        flags |= 0b1;
    }
    if embed_palette {
        flags |= 0b10;
    }
    if has_alpha {
        flags |= 0b100;
    }
    let mut out: Vec<u8> = vec![0x53, 0x49, 0x49, version, w[0], w[1], h[0], h[1], flags];
    let mut out_bytes: Vec<u8> = vec![];

    let mut palette_index: u8 = 0;

    for (index, _) in bytes.iter().enumerate().step_by(4) {
        // Pixels always stored in RGBA anyway because palettes are.
        let pixel = match &format {
            ColorFormat::RGBA => (bytes[index + 0] as u32) << 24 |
                                 (bytes[index + 1] as u32) << 16 |
                                 (bytes[index + 2] as u32) <<  8 |
                                 (bytes[index + 3] as u32),
            ColorFormat::RGB =>  (bytes[index + 0] as u32) << 24 |
                                 (bytes[index + 1] as u32) << 16 |
                                 (bytes[index + 2] as u32) <<  8 | 0xFF,
            ColorFormat::BGRA => (bytes[index + 0] as u32) <<  8 |
                                 (bytes[index + 1] as u32) << 16 |
                                 (bytes[index + 2] as u32) << 24 |
                                 (bytes[index + 3] as u32),
            ColorFormat::BGR =>  (bytes[index + 0] as u32) <<  8 |
                                 (bytes[index + 1] as u32) << 16 |
                                 (bytes[index + 2] as u32) << 24 | 0xFF,
        };

        if exists.get(&pixel).is_none() {
            exists.insert(pixel, palette_index);
            palette.push(pixel);
            if palette_index == 255 {
                println!("Too many colours. Max is 256. Exiting.");
                exit(1);
            }
            palette_index += 1;
        }

        if let Some(i) = exists.get(&pixel) {
            out_bytes.push(*i);
        }
    }

    if compress {
        let mut compressed = encode_rle(&out_bytes);
        out.append(&mut compressed);
    } else {
        out.append(&mut out_bytes);
    }

    if embed_palette {
        out.push(0x00);

        for color in palette {
            let color_bytes = color.to_le_bytes();
            let mut hex: Vec<u8> = vec![color_bytes[2], color_bytes[1], color_bytes[0]];
            if has_alpha {
                hex.insert(0, color_bytes[3]);
            }
            out.append(&mut hex);
        }
    }

    Ok(out)
}

// Convert args[1] from PNG into SII with palette included
fn main() {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "-e" | "--encode" => do_encode(&args),
        "-d" | "--decode" => do_decode(&args),
        _ => usage(&args),
    }
}

fn do_encode(args: &Vec<String>) {
    let filename = PathBuf::from(&args[2]);
    let file_stem = filename.file_stem().unwrap().to_str().unwrap();
    let decoder = png::Decoder::new(File::open(&filename).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut buf: Vec<u8> = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];
    let encoded = encode_from_bytes(1, reader.info().width as u16, reader.info().height as u16, bytes, ColorFormat::RGBA, true, true);
    let mut file = File::create(filename.with_extension("sii")).expect("Could not create file");
    file.write_all(&encoded.unwrap());
}

fn do_decode(args: &Vec<String>) {
}

fn usage(args: &Vec<String>) {
    println!("USAGE:\n\t{} [OPTIONS] FILE", &args[0]);
    println!("\nOPTIONS:");
    println!("\t-e, --encode\tEncode the input file");
    println!("\t-d, --decode\tDecode the input file");
    println!("\nARGS:");
    println!("\tFILE\tInput file to be encoded or decoded:");
}

fn test(path: &str, palette: &Vec<u32>) -> DecodedSII {
    let contents = read_to_string(path).expect("Could not read file");
    return decode(&palette, contents.as_bytes()).unwrap();
}

#[test]
fn test_encode_all_but_data() {
    let indices: Vec<u8> = vec![];

    let encoded = encode(1, 8, 8, false, false, indices.as_slice(), None).unwrap();

    let magic = vec![encoded[0], encoded[1], encoded[2]];
    assert_eq!(magic, vec!['S' as u8, 'I' as u8, 'I' as u8]);

    let version = encoded[3];
    assert_eq!(version, 1);

    let width = u16::from_be_bytes([encoded[4], encoded[5]]);
    let height = u16::from_be_bytes([encoded[6], encoded[7]]);

    assert_eq!(width, 8);
    assert_eq!(height, 8);
    
    let flags = encoded[8];
    assert_eq!(flags, 0);
}

#[test]
fn test_encode_uncompressed_no_palette_no_alpha() {
    let indices: Vec<u8> = vec![
        0, 1, 1, 1, 1, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 1, 1, 2, 2,
        1, 1, 1, 1, 1, 1, 2, 1,
        3, 1, 1, 1, 1, 1, 1, 3,
    ];

    let encoded = encode(1, 8, 8, false, false, indices.as_slice(), None).unwrap();
    
    let flags = encoded[8];
    assert_eq!(flags, 0);

    let data = encoded[9..encoded.len()].to_vec();
    assert_eq!(data, indices);
}

#[test]
fn test_encode_uncompressed_with_palette_no_alpha() {
    let indices: Vec<u8> = vec![
        0, 1, 1, 1, 1, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 1, 1, 2, 2,
        1, 1, 1, 1, 1, 1, 2, 1,
        3, 1, 1, 1, 1, 1, 1, 3,
    ];
    let palette: Vec<u32> = vec![0x6abe30, 0xffffff, 0x000000, 0x5b6ee1];

    let encoded = encode(1, 8, 8, false, false, indices.as_slice(), Some(palette)).unwrap();

    let flags = encoded[8];
    assert_eq!(flags, 0b10);

    let start = 9;
    if let Some(end) = encoded.iter().skip(9).position(|&x| x == 0xFF) {
        let data = encoded[start..end + 9].to_vec();
        assert_eq!(data, indices);
    }
}

#[test]
fn test_encode_compressed_no_palette_no_alpha() {
    let indices: Vec<u8> = vec![
        0, 1, 1, 1, 1, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 1, 1, 2, 2,
        1, 1, 1, 1, 1, 1, 2, 1,
        3, 1, 1, 1, 1, 1, 1, 3,
    ];
    let compressed_indices: Vec<u8> = vec![
        1, 0, 6, 1, 6, 2, 2, 1,
        1, 2, 4, 1, 1, 2, 2, 1,
        1, 2, 4, 1, 1, 2, 2, 1,
        6, 2, 2, 1, 1, 2, 6, 1,
        2, 2, 6, 1, 1, 2, 1, 1,
        1, 3, 6, 1, 1, 3
    ];

    let encoded = encode(1, 8, 8, true, false, indices.as_slice(), None).unwrap();

    let flags = encoded[8];
    assert_eq!(flags, 1);

    let data = encoded[9..encoded.len()].to_vec();
    assert_eq!(data, compressed_indices);
}

#[test]
fn test_encode_compressed_with_palette_no_alpha() {
    let indices: Vec<u8> = vec![
        0, 1, 1, 1, 1, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 1, 1, 2, 2,
        1, 1, 1, 1, 1, 1, 2, 1,
        3, 1, 1, 1, 1, 1, 1, 3,
    ];
    let compressed_indices: Vec<u8> = vec![
        1, 0, 6, 1, 6, 2, 2, 1,
        1, 2, 4, 1, 1, 2, 2, 1,
        1, 2, 4, 1, 1, 2, 2, 1,
        6, 2, 2, 1, 1, 2, 6, 1,
        2, 2, 6, 1, 1, 2, 1, 1,
        1, 3, 6, 1, 1, 3
    ];
    let palette: Vec<u32> = vec![0x6abe30, 0xffffff, 0x000000, 0x5b6ee1];

    let encoded = encode(1, 8, 8, true, false, indices.as_slice(), Some(palette)).unwrap();

    let flags = encoded[8];
    assert_eq!(flags, 0b11);

    let start = 9;
    if let Some(end) = encoded.iter().skip(9).position(|&x| x == 0xFF) {
        let data = encoded[start..end + 9].to_vec();
        assert_eq!(data, compressed_indices);
    }
}

#[test]
fn test_encode_compressed_with_palette_with_alpha() {
    let indices: Vec<u8> = vec![
        0, 1, 1, 1, 1, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 1, 1, 2, 2,
        1, 1, 1, 1, 1, 1, 2, 1,
        3, 1, 1, 1, 1, 1, 1, 3,
    ];
    let compressed_indices: Vec<u8> = vec![
        1, 0, 6, 1, 6, 2, 2, 1,
        1, 2, 4, 1, 1, 2, 2, 1,
        1, 2, 4, 1, 1, 2, 2, 1,
        6, 2, 2, 1, 1, 2, 6, 1,
        2, 2, 6, 1, 1, 2, 1, 1,
        1, 3, 6, 1, 1, 3
    ];
    let palette: Vec<u32> = vec![0x6abe30ff, 0xffffffff, 0x000000cc, 0x5b6ee1aa];

    let encoded = encode(1, 8, 8, true, true, indices.as_slice(), Some(palette.to_owned())).unwrap();

    let flags = encoded[8];
    assert_eq!(flags, 0b111);

    let start = 9;
    if let Some(end) = encoded.iter().skip(9).position(|&x| x == 0xFF) {
        let data = encoded[start..end + 9].to_vec();
        assert_eq!(data, compressed_indices);

        let colors_vec = encoded[end + 10..].to_vec();
        let colors: Vec<u32> = colors_vec.chunks_exact(4).map(|chunk| {
            ((chunk[0] as u32) << 24) | (chunk[1] as u32) << 16 | (chunk[2] as u32) << 8 | (chunk[3] as u32)
        }).collect();

        assert_eq!(palette, colors)
    }
}

#[test]
fn test_encode_compressed_no_palette_with_alpha() {
    let indices: Vec<u8> = vec![
        0, 1, 1, 1, 1, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 1, 1, 2, 2,
        1, 1, 1, 1, 1, 1, 2, 1,
        3, 1, 1, 1, 1, 1, 1, 3,
    ];
    let compressed_indices: Vec<u8> = vec![
        1, 0, 6, 1, 6, 2, 2, 1,
        1, 2, 4, 1, 1, 2, 2, 1,
        1, 2, 4, 1, 1, 2, 2, 1,
        6, 2, 2, 1, 1, 2, 6, 1,
        2, 2, 6, 1, 1, 2, 1, 1,
        1, 3, 6, 1, 1, 3
    ];

    let encoded = encode(1, 8, 8, true, true, indices.as_slice(), None).unwrap();

    let flags = encoded[8];
    assert_eq!(flags, 0b101);

    let start = 9;
    if let Some(end) = encoded.iter().skip(9).position(|&x| x == 0xFF) {
        let data = encoded[start..end + 9].to_vec();
        assert_eq!(data, compressed_indices);
    }
}

#[test]
fn test_decode_uncompressed_no_palette_no_alpha() {
    let indices: Vec<u8> = vec![
        0, 1, 1, 1, 1, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 1, 1, 2, 2,
        1, 1, 1, 1, 1, 1, 2, 1,
        3, 1, 1, 1, 1, 1, 1, 3,
    ];
    let encoded = encode(1, 8, 8, false, false, indices.as_slice(), None).unwrap();
    let palette: Vec<u32> = vec![0x6abe30, 0xffffff, 0x000000, 0x5b6ee1];
    let decoded = decode(&palette, encoded.as_slice()).unwrap();

    assert_eq!(decoded.height, 8);
    assert_eq!(decoded.width, 8);
    assert_eq!(decoded.pixels.len(), 64);
}

#[test]
fn test_decode_compressed_no_palette_no_alpha() {
    let indices: Vec<u8> = vec![
        0, 1, 1, 1, 1, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 1, 1, 2, 2,
        1, 1, 1, 1, 1, 1, 2, 1,
        3, 1, 1, 1, 1, 1, 1, 3,
    ];
    let encoded = encode(1, 8, 8, true, false, indices.as_slice(), None).unwrap();
    let palette: Vec<u32> = vec![0x6abe30, 0xffffff, 0x000000, 0x5b6ee1];
    let decoded = decode(&palette, encoded.as_slice()).unwrap();

    assert_eq!(decoded.height, 8);
    assert_eq!(decoded.width, 8);
    assert_eq!(decoded.pixels.len(), 64);
}

#[test]
fn test_decode_images() {
    let palette: Vec<u32> = vec![0x6abe30, 0xffffff, 0x000000, 0x5b6ee1];
    let expected_pixels = vec![
        0x6abe30, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x0,
        0x0,      0x0,      0x0,      0x0,      0x0,      0xffffff, 0xffffff, 0x0, 
        0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x0,      0xffffff, 0xffffff, 0x0,
        0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x0,      0xffffff, 0xffffff, 0x0,
        0x0,      0x0,      0x0,      0x0,      0x0,      0xffffff, 0xffffff, 0x0, 
        0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x0,      0x0,
        0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x0,      0xffffff,
        0x5b6ee1, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0x5b6ee1,
    ];

    let mut decoded = test("test_uncompressed.sii", &palette);
    assert!(decoded.width == 8);
    assert!(decoded.height == 8);
    assert!(decoded.pixels == expected_pixels);

    decoded = test("test_compressed.sii", &palette);
    assert!(decoded.width == 8);
    assert!(decoded.height == 8);
    assert!(decoded.pixels == expected_pixels);

    let indices: Vec<u8> = vec![
        0, 1, 1, 1, 1, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        1, 1, 1, 1, 2, 1, 1, 2,
        2, 2, 2, 2, 2, 1, 1, 2,
        1, 1, 1, 1, 1, 1, 2, 2,
        1, 1, 1, 1, 1, 1, 2, 1,
        3, 1, 1, 1, 1, 1, 1, 3,
    ];

    let mut encoded = encode(1, 8, 8, false, false, indices.as_slice(), None);
    decoded = decode(&palette, &encoded.unwrap()).unwrap();
    assert!(decoded.pixels == expected_pixels);

    encoded = encode(1, 8, 8, true, false, indices.as_slice(), None);
    decoded = decode(&palette, &encoded.unwrap()).unwrap();
    assert!(decoded.pixels == expected_pixels);
}
