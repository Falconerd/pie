mod pie;
use pie::PixelFormat;

use crate::pie::{decode, encode, Palette};

pub fn main() {
    // Do PIE stuff...
}

#[test]
fn test_decode() {
    let bytes = include_bytes!("../images/test_compressed_with_palette.sii");
    let decoded = decode(bytes, None).unwrap();
    decoded.pixels.iter().for_each(|x| print!("0x{:02X},", x));
}

#[test]
fn test_encode() {
    let pixels: Vec<u8> = vec![
        0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xCC, 0x90, 0xFF, 0xBE, 0xEF, 0x00,
        0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xCC, 0x90, 0xFF, 0xBE, 0xEF, 0x00,
        0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xCC, 0x90, 0xFF, 0xBE, 0xEF, 0x00,
        0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0xCC, 0x90, 0xFF, 0xBE, 0xEF, 0x00,
    ];

    let encoded = encode(5, 4, &pixels, None);
    println!("encoded: {:?}", encoded);
}
