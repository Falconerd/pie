use std::{fs::read_to_string, collections::HashMap};

/*
┌─ SII Image Format ──────┐
│         Header          │
│   Byte[0-2]: Magic      │
│   Byte[3]: Version      │
│   Byte[4]: Width (8px)  │
│   Byte[5]: Height (8px) │
│   Byte[6]: Flags        │
│     Bit 0: Compressed   │
│     Bit 1: Palette      │
│   Bytes[7+]: Data       │
│                         │
│  Palette (optional)     │
│   Bytes[0-3]: Color 0   │
│   Bytes[4-7]: Color 1   │
│    ...                  │
│   Bytes[n*4-3]: Color n │
└─────────────────────────┘
*/

#[derive(Debug)]
struct DecodedSII {
    width: u32,
    height: u32,
    pixels: Vec<u32>
}

#[derive(Debug)]
struct EncodedSII {
    width_divided_by_8: u8,
    height_divided_by_8: u8,
    is_compressed: u8,
    bytes: Vec<u8>,
}

#[derive(Debug)]
enum DecodeError {
    ColorNotFound,
    MissingPixels
}

#[derive(Debug)]
enum EncodeError {
    SomethingWentWrong
}

fn decode(palette: &Vec<u32>, data: &[u8]) -> Result<DecodedSII, DecodeError> {
    let mut pixels: Vec<u32> = vec![];

    let width = u32::from_be_bytes([0, 0, 0, data[4]]) * 8;
    let height = u32::from_be_bytes([0, 0, 0, data[5]]) * 8;

    for value in data.iter().skip(2) {
        if (*value) as usize > palette.len() {
            return Err(DecodeError::ColorNotFound);
        }

        pixels.push(palette[(*value) as usize]);
    }

    if pixels.len() < width as usize * height as usize {
        return Err(DecodeError::MissingPixels);
    }

    Ok(DecodedSII { width, height, pixels })
}

fn encode_rle(data: &[u8]) -> Vec<u8> {
    let mut encoded = vec![];
    let mut count = 1;

    for i in 1..=data.len() {
        if data[i] == data[i - 1] {
            count += 1;
        } else {
            encoded.push(data[i - 1]);
            encoded.push(count - 1);
            count = 1;
        }
    }

    encoded
}

fn encode(palette: &Vec<u32>, pixels: &Vec<u32>, width: u32, height: u32) -> Result<EncodedSII, EncodeError> {
    if palette.len() > 250 {
        return Err(EncodeError::SomethingWentWrong);
    }

    let mut bytes: Vec<u8> = vec![];
    // Store each index to a colour after lookup.
    let mut indices: HashMap<u32, u8> = HashMap::new();

    for pixel in pixels {
        // Find each value in the palette.
        if let Some(index) = indices.get(&pixel) {
            bytes.push(*index);
        } else {
            // Scan until colour found.
            let mut found = false;
            for (index, color) in palette.iter().enumerate() {
                if *color == *pixel {
                    indices.insert(*color, index as u8);
                    found = true;
                    bytes.push(index as u8);
                }
            }
            if !found {
                return Err(EncodeError::SomethingWentWrong);
            }
        }
    }

    let encoded = EncodedSII {
        width_divided_by_8: (width / 8) as u8,
        height_divided_by_8: (height / 8) as u8,
        is_compressed: 1,
        bytes,
    };

    return Ok(encoded)
}

fn main() {
    let contents = read_to_string("test.sii").expect("Could not read file");

    let palette: Vec<u32> = vec![0xFFFFFFFF, 0xFF0000FF, 0xFFFF00FF, 0xFF00FFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF];

    println!("Hello, world! {:?}\n{:?}", contents.as_bytes(), palette.len());

    let decoded = decode(&palette, contents.as_bytes()).expect("Could not decode");
    println!("Decoded! {:?}", decoded);

    let encoded = encode(&palette, &decoded.pixels, decoded.width, decoded.height);
    println!("Decoded! {:?}", encoded);
}
