/*
────────────────────────────────────────────────────────────────────────────────
PIE - Pixel Indexed Encoding
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

NOTE: width, height, length are stored in Little Endian order to make direct
casting possible without swapping the order.

┌─ PIE Image Format ───────────────────────────────────────────────────────────┐
│ Name     Offset  Type          Description                                   │
├──────────────────────────────────────────────────────────────────────────────┤
│ magic    0       u8[3]         Magic bytes "PIE"                             │
│ version  3       u8            Version                                       │
│ width    4       u16           Width in pixels                               │
│ height   6       u16           Height in pixels                              │
│ flags    8       u8            0x1: Set if palette is included in image data │
│                                0x2: Set if RGBA, otherwise RGB asumed        │
│                                Other bits are reserved for future updates    │
│ length   9       u16           Length of the 1st array in data section       │
│ data     11      u8[length][2] Array of [count, color_index].                │
│ palette?         u8[]          Optional palette included in the image        │
│                                Stride can be 3 or 4 depending on RGB/RGBA    │
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
    - Header values width, height, length are now stored in Little Endian order.
      This makes the encoding and decoding process simpler on common systems.
    - Added a C header-only-library style reference parser.
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

typedef struct {
    pie_u32 magic;      // 4
    pie_u16 version;    // 2
    pie_u16 flags;      // 2
    pie_u32 length;     // 2
    pie_u16 width;      // 4
    pie_u16 height;     // 2
    void *data;         // 8
    
    // pie_u8 magic[3]; // 1 byte
    // pie_u8 version; // 1 byte
    // pie_u16 width; // 2 bytes
    // pie_u16 height; // 2 bytes
    // pie_u16 length; // 2 bytes
    // int flags; // 1 byte
    // pie_u8 *data; // 8 bytes
} pie_header;

typedef struct {
    int has_alpha;
    int width;
    int height;
    pie_u8 *pixels;
} pie_decoded;

typedef struct {
    int pie_u8_size;
    int pie_u16_size;
    int pie_u8_correct;
    int pie_u16_correct;
} pie_test_typesizes;

pie_test_typesizes pie_validate_types(void) {
    return (pie_test_typesizes){
        .pie_u8_size = sizeof(pie_u8),
        .pie_u16_size = sizeof(pie_u16),
        .pie_u8_correct = sizeof(pie_u8) == 1,
        .pie_u16_correct = sizeof(pie_u16) == 2,
    };
}

int pie_validate(pie_u8 *bytes) {
    pie_header h = *(pie_header *)bytes;
    // if (h.magic[0] != 'P' || h.magic[1] != 'I' || h.magic[2] != 'E') return 0;
    if (h.version != 1 && h.version != 2) return 0;
    if (!h.width || !h.height) return 0;
    if (h.flags > 3) return 0;
    if (!h.length) return 0;
    if (!h.data) return 0;
    return 1;
}

/*
    Assumed to be a valid v2 .pie file.
    Can check with pie_validate();
    params:
        bytes - raw bytes read from the image file.
*/
int pie_decode(pie_header h) {
    return 0;
}

// │ magic    0       u8[3]         Magic bytes "PIE"                             │
// │ version  3       u8            Version                                       │
// │ width    4       u16           Width in pixels                               │
// │ height   6       u16           Height in pixels                              │
// │ flags    8       u8            0x1: Set if palette is included in image data │
// │                                0x2: Set if RGBA, otherwise RGB asumed        │
// │                                Other bits are reserved for future updates    │
// │ length   9       u16           Length of the 1st array in data section       │
// │ data     11      u8[length][2] Array of [count, color_index].                │
// │ palette?         u8[]          Optional palette included in the image        │
// │                                Stride can be 3 or 4 depending on RGB/RGBA    │
