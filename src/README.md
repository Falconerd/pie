```
/*
Memory Layout
-------------

header {
    magic: u8[3], // Magic bytes "SII".
    version: u8, // Version.
    width: u8, // Width divided by 8.
    height: u8, // Height divided by 8.
    flags: u8, // 0b00000001 is whether the file is compressed.
               // 0b00000010 is whether the palette is included.
               // Other bits are reserved for future updates.
    data: u8[], // Indices into the palette (external or internal).
    palette?: u32[], // Optional palette included in the image.
}

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

Description
-----------
This lossless image format only optionally contains colors in the file.
It is designed to be used in conjunction with a palette from which
colours can be sampled by the decoder.

Using an external palette reduces uncompressed image size by 75%
assuming a four channel format like RGBA, or 60% assuming a 3
channel format like RGB without alpha.

Using an internal palette will increase the size depending on the
palette.

Data Compression
----------------
Given this format is designed for pixel art images, some assumptions
are made.

1. Palettes will generally be 2-64 colours.
2. Horizontal repeating pixels will be common.

Therefore: 
- RLE is used for horizontal runs of pixels that have the same index.
- The vertical axis is not considered.

Palette Compression
-------------------
The palette is not compressed.
*/
```
