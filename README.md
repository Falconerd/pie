```
PIE - Pixel Indexed Encoding
Version 1.0

Description
-----------
This lossless image format only optionally stores colors in the file.
It is designed to be used in conjunction with a palette from which
colours can be sampled by the decoder.

Using an external palette reduces uncompressed image size by 75%
assuming a four channel format like RGBA, or 60% assuming a 3
channel format like RGB without alpha.

Using an internal palette will increase the size depending on the
palette, but still generally be smaller than other formats like PNG
for pixel art.

Comparison
----------
In the images/ folder you will find randomly selected .png pixel art
images from lospec.org as well as converted .pie files. If any of
these images are your and you want it removed, please create an issue.

a-strawberry-dude-509249.pie    77.00% the size of the png version
cubikism-023391.pie             81.00% ..
dune-portraits-787893.pie       74.00% ..
goblin-slayer-808592.pie        63.00% ..
khorne-berserker-509756.pie     50.00% ..
snowfighter-844418.pie          64.00% ..

Memory Layout
-------------
┌─ PIE Image Format ──────────────────────────────────────────────┐
│ magic    u8[3] -- Magic bytes "PIE"                             │
│ version  u8    -- Version                                       │
│ width    u16   -- Width in pixels (BE)                          │
│ height   u16   -- Height in pixels (BE)                         │
│ flags    u8    -- 0b00000001 is whether the palette is included │
│                -- 0b00000010 is whether there is transparency   │
│                -- Other bits are reserved for future updates    │
│ length   u16   -- Run count of the data section (BE)            │
│ data     u8[]  -- Indices into palette (external or internal)   │
│ palette? u8[]  -- Optional palette included in the image        │
│                -- Stride can be 3 or 4 depending on RGB/RGBA    │
└─────────────────────────────────────────────────────────────────┘

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

Runs can be no longer than 255 pixels and they wrap to the next row
as a byte array is 1-Dimensional and has no concept of rows.

Palette Compression
-------------------
The palette is not compressed.
```
