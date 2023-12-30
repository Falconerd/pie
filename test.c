#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "pie.h"

#define color_0 0x6A, 0xBE, 0x30
#define color_1 0xFF, 0xFF, 0xFF
#define color_2 0x00, 0x00, 0x00
#define color_3 0x5B, 0x6E, 0xE1

pie_u8 bytes[] = {
    0x50, 0x49, 0x45, /* Magic "PIE" */ 0x02, /* Version */
    0x02, 0x00, 0x00, 0x00, /* Flags (2 - palette included) */
    0x08, 0x00, /* Width */ 0x08, 0x00, /* Height */
    0x17, 0x00, 0x00, 0x00, /* Data Pair Count */

    // RLE encoded data [Index, Run Length]
    0x01, 0x01,
    0x00, 0x06,
    0x01, 0x06,
    0x02, 0x02,
    0x01, 0x01,
    0x02, 0x04,
    0x01, 0x01,
    0x02, 0x02,
    0x01, 0x01,
    0x02, 0x04,
    0x01, 0x01,
    0x02, 0x02,
    0x01, 0x06,
    0x02, 0x02,
    0x01, 0x01,
    0x02, 0x06,
    0x01, 0x02,
    0x02, 0x06,
    0x01, 0x01,
    0x02, 0x01,
    0x01, 0x01,
    0x03, 0x06,
    0x01, 0x01,

    // Optional Palette.
    color_0,
    color_1,
    color_2,
    color_3
};

pie_u8 expected_pixel_data[] = {
    color_1, color_0, color_0, color_0, color_0, color_0, color_0, color_1,
    color_1, color_1, color_1, color_1, color_1, color_2, color_2, color_1,
    color_2, color_2, color_2, color_2, color_1, color_2, color_2, color_1,
    color_2, color_2, color_2, color_2, color_1, color_2, color_2, color_1,
    color_1, color_1, color_1, color_1, color_1, color_2, color_2, color_1,
    color_2, color_2, color_2, color_2, color_2, color_2, color_1, color_1,
    color_2, color_2, color_2, color_2, color_2, color_2, color_1, color_2,
    color_1, color_3, color_3, color_3, color_3, color_3, color_3, color_1,
};

pie_u8 palette[] = {
    color_0,
    color_1,
    color_2,
    color_3
};

// Not including this in the library... Why?
pie_u32 pie_pixel_count(pie_u8 *b) {
    pie_header *h = (pie_header *)b;
    pie_u8 *data = (pie_u8 *)(h + 1);
    pie_size data_pairs = (pie_size)h->length;
    pie_u32 pixel_count = 0;
    for (pie_size i = 0; i < data_pairs; i += 1) {
        pixel_count += (pie_size)(data[i * 2 + 1]);
    }
    return pixel_count;
}

int main(void) {
    pie_header h = pie_header_from_bytes(bytes);

    pie_u32 pixel_count = pie_pixel_count(bytes);
    pie_u32 expected_pixel_count = (pie_u32)h.width * (pie_u32)h.height;
    assert(pixel_count == expected_pixel_count);

    pie_size s = pie_stride(&h);
    int ep = pie_has_embedded_palette(&h);

    assert(h.magic[0] == 'P');
    assert(h.magic[1] == 'I');
    assert(h.magic[2] == 'E');
    assert(h.version == 2);
    assert(h.flags == 0x2);
    assert(h.width == 8);
    assert(h.height == 8);
    assert(h.length == 23);
    assert(s == 3);
    assert(ep);

    pie_u8 buffer[64 * 3] = {0};
    pie_u8 buffer2[64 * 3] = {0};

    pie_pixels p = pie_pixels_from_bytes(bytes, buffer);
    pie_pixels p2 = pie_pixels_from_bytes_and_palette(bytes, palette, buffer2);

    int diff = memcmp(p.data, p2.data, sizeof(buffer));
    assert(diff == 0);

    diff = memcmp(p.data, expected_pixel_data, sizeof(buffer));
    assert(diff == 0);

    printf("All tests passed.\n");
    printf("Press Enter to exit.\n");
    getchar();

    exit(0);
}
