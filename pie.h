/*
────────────────────────────────────────────────────────────────────────────────
PIE - Palette Indexed Encoding
Version 2.0.0
────────────────────────────────────────────────────────────────────────────────

This lossless image format only optionally stores colors in the file. It is
designed to be used in conjunction with a palette from which colours can be
sampled by the decoder.

Using an internal palette will increase the size by the amount of colours in the
palette. Even with an embedded palette, the size has been shown to be smaller
than generalised formats like PNG when encoding pixel art images.

If your application has a set colour palette, define the palette once and then
store all image data without the colours. This is the ideal scenario.

NOTE: Width, Height and Length are stored in Little Endian order to make direct
casting possible without swapping the order.

┌─ PIE Image Format ───────────────────────────────────────────────────────────┐
│ Name     Offset  Type     Description                                        │
├──────────────────────────────────────────────────────────────────────────────┤
│ magic    0       u8[3]    PIE                                                │
│ version  3       u8       Version                                            │
│ flags    4       u32      0x1: Set if Palette is included in image data      │
│                           0x2: Set if RGBA, otherwise RGB asumed             │
│                           Other bits are reserved for future updates         │
│ width    8       u16      Width in pixels                                    │
│ height   10      u16      Height in pixels                                   │
│ length   12      u32      Length of the data segment in bytes                │
│ data     16      u8[][2]  Array of [Color Index, Count]                      │
│ palette?         u8[]     Optional palette included in the image             │
│                           Stride can be 3 or 4 depending on RGB/RGBA         │
└──────────────────────────────────────────────────────────────────────────────┘

In the images/ folder you will find randomly selected .png pixel art images from
lospec.org as well as converted .pie files. If any of these images are your and
you would like it removed, please create an issue.

┌─ PIE vs PNG Comparison ──────────────────────────────────────────────────────┐
│ File                                                     PNG Size Difference │
├──────────────────────────────────────────────────────────────────────────────┤
│ a-strawberry-dude-509249.pie ....................................... -23.00% │
│ cubikism-023391.pie ................................................ -19.00% │
│ dune-portraits-787893.pie .......................................... -26.00% │
│ goblin-slayer-808592.pie ........................................... -37.00% │
│ khorne-berserker-509756.pie ........................................ -50.00% │
│ snowfighter-844418.pie ............................................. -36.00% │
├──────────────────────────────────────────────────────────────────────────────┤
│ Average ............................................................ -31.83% │
└──────────────────────────────────────────────────────────────────────────────┘

Data Compression
────────────────────────────────────────────────────────────────────────────────

Given this format is designed for pixel art images, some assumptions are made.

1. Palettes generally have 2-64 colours and very rarely exceed 256.
2. Horizontal repeating pixels will be common.

Therefore: 
- A Palette may contain up to 256 colours. Indices into the Palette may therfore
    be represented by a single byte.
- RLE is used for horizontal runs of pixels that have the same index.
- The vertical axis is not considered.

Runs can be no longer than 255 pixels and they wrap to the next row as a byte
array is 1-Dimensional and has no concept of rows.

The palette is not compressed.

Changelog
────────────────────────────────────────────────────────────────────────────────

2023-12-29: Version 2.0.0.
    - Header values Width, Height, and Length are now stored in Little Endian
        order. This makes the encoding and decoding process simpler on common
        systems.
    - Added a C header-only-library style reference parser.
    - Changed the P from Pixel to Palette.
2023-03-29: Version 1.0.1.
    - This was a change to the Rust reference parser, not the spec.
    - Fix naming collision in Rust.
2023-03-29: Initial release.


Todo
────────────────────────────────────────────────────────────────────────────────
    - Add a separate code-path for v1 files.

*/

#ifndef pie_break
#ifdef _MSC_VER
#define pie_break() __debugbreak()
#else
#define pie_break() __builtin_trap()
#endif
#endif

#ifndef pie_assert
#define pie_assert(x) if (!(x)) pie_break();
#endif

#ifndef pie_u8
typedef unsigned char pie_u8;
#endif

#ifndef pie_u16
typedef unsigned short pie_u16;
#endif

#ifndef pie_u32
typedef unsigned int pie_u32;
#endif

#ifndef pie_size
typedef long long pie_size;
#endif

#define PIE_IMAGE_HAS_ALPHA   1
#define PIE_IMAGE_HAS_PALETTE 2

typedef struct {
    pie_u8 magic[3];       // 3
    pie_u8 version;        // 1
    pie_u32 flags;         // 4
    pie_u16 width;         // 2
    pie_u16 height;        // 2
    pie_u32 length;        // 4
} pie_header;              // 16

typedef struct {
    pie_size count;
    pie_size stride;
    pie_u8 *data;
} pie_pixels;

pie_header
pie_header_from_bytes(pie_u8 *b) {
    return *(pie_header *)b;
}

pie_size
pie_stride(pie_header *h) {
    return (pie_size)3 + (h->flags & PIE_IMAGE_HAS_ALPHA);
}

int
pie_has_embedded_palette(pie_header *h) {
    return h->flags & PIE_IMAGE_HAS_PALETTE;
}

pie_pixels
pie_pixels_from_bytes_and_palette(pie_u8 *b, pie_u8 *p, pie_u8 *dest) {
    pie_header *h = (pie_header *)b;
    pie_u8 *data = (pie_u8 *)(h + 1);
    pie_size stride = pie_stride(h);

    pie_size pixel_count = (pie_size)h->width * (pie_size)h->height;
    pie_size data_pairs = (pie_size)h->length / 2;
    pie_u8 *palette_bytes = p;

    pie_size pixel_index = 0;
    for (pie_size i = 0; i < data_pairs; i += 1) {
        pie_u8 palette_index = data[i * 2 + 0];
        pie_u8 run_length = data[i * 2 + 1];

        for (pie_u8 j = 0; j < run_length; j += 1) {
            pie_size local_index = pixel_index * stride;
            dest[local_index] = palette_bytes[palette_index * 3];
            dest[local_index + 1] = palette_bytes[palette_index * 3 + 1];
            dest[local_index + 2] = palette_bytes[palette_index * 3 + 2];
            pixel_index += 1;
        }
    }

    return (pie_pixels){pixel_count, stride, dest};
}

pie_pixels
pie_pixels_from_bytes(pie_u8 *b, pie_u8 *dest) {
    pie_header *h = (pie_header *)b;
    if (!pie_has_embedded_palette(h)) {
        return (pie_pixels){0};
    }
    pie_u8 *data = (pie_u8 *)(h + 1);
    pie_size data_length = (pie_size)h->length;
    pie_u8 *palette_bytes = data + data_length;

    return pie_pixels_from_bytes_and_palette(b, palette_bytes, dest);
}
