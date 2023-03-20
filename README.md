```
/*
Memory Layout
-------------

header {
    magic     u8[3] -- Magic bytes "SII".
    version   u8    -- Version.
    width     u16   -- Width in pixels (BE).
    height    u16   -- Height in pixels (BE).
    flags     u8    -- 0b00000001 is whether the file is compressed.
                    -- 0b00000010 is whether the palette is included.
                    -- 0b00000100 is whether there is transparency.
                    -- Other bits are reserved for future updates.
    data      u8[]  -- Indices into the palette (external or internal).
    palette?  u32[] -- Optional palette included in the image.
}

┌─ SII Image Format ─────┐
│ Header                 │
│  Byte[0-2]: Magic      │
│  Byte[3]: Version      │
│  Byte[4-5]: Width      │
│  Byte[6-7]: Height     │
│  Byte[8]: Flags        │
│    Bit 0: Compressed   │
│    Bit 1: Palette      │
│    Bit 2: Transparency │
│  Bytes[9+]: Data       │
│  Byte[n]: 0xFF         │
│                        │
│ Palette (optional)     │
│  Bytes[0-3]: Color 0   │
│  Bytes[4-7]: Color 1   │
│   ...                  │
│  Bytes[n*4-3]: Color n │
└────────────────────────┘

Description
-----------
This lossless image format only optionally stores colors in the file.
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

1. Palettes generally have 2-64 colours and very rarely exceed 256.
2. Horizontal repeating pixels will be common.

Therefore: 
- A Palette may contain up to 256 colours. Indices into the Palette may
  therfore be represented by a single byte.
- RLE is used for horizontal runs of pixels that have the same index.
- The vertical axis is not considered.

Runs cannot be no longer than 255 pixels - runs wrap to the next row;
as a byte array is 1-Dimensional and has no concept of rows.

Palette Compression
-------------------
The palette is not compressed.
*/
```
