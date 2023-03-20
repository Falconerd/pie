use std::fs::read_to_string;

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
        if i < data.len() && data[i] == data[i - 1] {
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
