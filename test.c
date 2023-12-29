#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "pie.h"

pie_u8 bytes[] = {
    0x50, 0x49, 0x45, /* Magic "PIE" */ 0x02, /* Version */
    0x02, 0x00, 0x00, 0x00, /* Flags (2 - palette included) */
    0x08, 0x00, /* Width */ 0x08, 0x00, /* Height */
    0x2E, 0x00, 0x00, 0x00, /* Data Length */

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
    0x01, 0x02,

    // Optional Palette.
    0x6A, 0xBE, 0x30,
    0xFF, 0xFF, 0xFF,
    0x00, 0x00, 0x00,
    0x5B, 0x6E, 0xE1
};

int main(void) {
    pie_header h = pie_header_from_bytes(bytes);
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
    pie_pixels p2 = pie_pixels_from_bytes_and_palette(bytes, &bytes[62], buffer2);

    int diff = memcmp(p.data, p2.data, sizeof(buffer));
    assert(diff == 0);

    __debugbreak();

    exit(0);
}
